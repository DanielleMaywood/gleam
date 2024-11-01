use std::sync::Arc;

use camino::Utf8Path;
use ecow::EcoString;
use im::HashSet;
use itertools::Itertools;

use crate::{
    analyse::Inferred,
    ast::{
        ArgNames, AssignName, BinOp, ClauseGuard, Constant, CustomType, Definition, ModuleConstant,
        Pattern, Statement, TypedAssignment, TypedClause, TypedClauseGuard, TypedConstant,
        TypedExpr, TypedFunction, TypedModule, TypedPattern, TypedStatement,
    },
    type_::{ModuleValueConstructor, Type, ValueConstructorVariant},
    Error,
};

pub const PRELUDE: &'static str = include_str!("../templates/prelude.scm");
pub const COMPILE: &'static str = include_str!("../templates/compile.scm");
pub const RUNNER: &'static str = include_str!("../templates/run-chez.sh");

pub fn module(
    dir: &Utf8Path,
    module: &TypedModule,
    dependencies: &mut HashSet<String>,
) -> Result<String, Error> {
    let mut chez = Chez::new(dir, module);

    let body = chez.compile()?;
    let name = &chez.module.name;

    let exports = chez.exports.into_iter().join(" ");

    let imports = (chez.imports.iter())
        .map(|import| format!("({import})"))
        .join(" ");

    *dependencies = chez.imports;

    Ok(format!(
        r#"(library ({name})
  (export {exports})
  (import (chezscheme) (gleam/$prelude) {imports})
  {body})"#
    ))
}

struct Chez<'a> {
    imports: HashSet<String>,
    exports: HashSet<String>,
    module: &'a TypedModule,
    tmp_count: usize,
}

impl<'a> Chez<'a> {
    fn new(_dir: &'a Utf8Path, module: &'a TypedModule) -> Self {
        Self {
            module,
            imports: HashSet::new(),
            exports: HashSet::new(),
            tmp_count: 0,
        }
    }

    fn compile(&mut self) -> Result<String, Error> {
        let mut code = String::new();

        for definition in &self.module.definitions {
            match definition {
                Definition::Function(function) => {
                    code += &self.function_definition(function)?;
                    code += "\n";
                }
                Definition::TypeAlias(_) => {}
                Definition::CustomType(type_) => {
                    code += &self.custom_type(type_)?;
                    code += "\n";
                }
                Definition::Import(package) => {
                    let _ = self.imports.insert(package.module.to_string());
                }
                Definition::ModuleConstant(constant) => {
                    code += &self.module_constant(constant)?;
                    code += "\n";
                }
            }
        }

        Ok(code)
    }

    fn module_constant(
        &mut self,
        constant: &ModuleConstant<Arc<Type>, EcoString>,
    ) -> Result<String, Error> {
        let name = format!("{}.{}", self.module.name, constant.name);
        let value = self.constant(&constant.value)?;

        if constant.publicity.is_public() {
            let _ = self.exports.insert(name.clone());
        }

        Ok(format!("(define {name} {value})"))
    }

    fn custom_type(&mut self, type_: &CustomType<Arc<Type>>) -> Result<String, Error> {
        let mut code = String::new();

        for constructor in &type_.constructors {
            let name = format!("{}.{}", self.module.name, constructor.name);
            let tag = &constructor.name;

            let args = (0..constructor.arguments.len())
                .map(|idx| format!("_{idx}"))
                .join(" ");

            let variant = if args.is_empty() {
                format!("(define {name} (list '{tag}))")
            } else {
                format!("(define {name} (lambda ({args}) (list '{tag} (vector {args}))))")
            };

            code += &variant;
            code += "\n";

            let predicate =
                format!("(define {name}? (lambda (value) (equal? (car value) '{tag})))");

            code += &predicate;
            code += "\n";

            if type_.publicity.is_public() {
                let _ = self.exports.insert(format!("{name}?"));
                let _ = self.exports.insert(name);
            }
        }

        Ok(code)
    }

    fn function_definition(&mut self, function: &TypedFunction) -> Result<String, Error> {
        let parameters = (function.arguments.iter())
            .map(|arg| match &arg.names {
                ArgNames::Discard { name, .. }
                | ArgNames::LabelledDiscard { name, .. }
                | ArgNames::Named { name, .. }
                | ArgNames::NamedLabelled { name, .. } => format!("${name}"),
            })
            .join(" ");

        let name = function
            .name
            .as_ref()
            .map(|(_, name)| name)
            .expect("function to have name");

        let name = format!("{}.{name}", self.module.name);

        if function.publicity.is_public() {
            let _ = self.exports.insert(name.clone());
        }

        if let Some((module, external, _)) = &function.external_chez {
            let body = if !module.is_empty() {
                let _ = self.imports.insert(module.to_string());

                format!("({module}.{external} {parameters})")
            } else {
                format!("({external} {parameters})")
            };

            Ok(format!("(define {name} (lambda ({parameters}) {body}))"))
        } else {
            let vars = self.collect_statements_variables(&function.body);
            let body = self.statements(&function.body)?;

            Ok(format!(
                "(define {name} (lambda ({parameters}) {vars} {body}))"
            ))
        }
    }

    fn statements(&mut self, statements: &[TypedStatement]) -> Result<String, Error> {
        let statements: Vec<String> = statements
            .iter()
            .map(|statement| self.statement(statement))
            .try_collect()?;

        Ok(format!("(let () {})", statements.join(" ")))
    }

    fn statement(&mut self, statement: &TypedStatement) -> Result<String, Error> {
        match statement {
            Statement::Expression(expression) => self.expression(expression),
            Statement::Assignment(assignment) => self.assignment(assignment),
            Statement::Use(_) => panic!("unreachable"),
        }
    }

    fn assignments(&mut self, assignments: &[TypedAssignment]) -> Result<String, Error> {
        let assignments: Vec<String> = assignments
            .iter()
            .map(|assignment| self.assignment(assignment))
            .try_collect()?;

        Ok(assignments.join(" "))
    }

    fn assignment(&mut self, assignment: &TypedAssignment) -> Result<String, Error> {
        let subject = self.expression(&assignment.value)?;
        let pattern = self.pattern(&subject, &assignment.pattern)?;

        Ok(format!(
            "(if (not {pattern}) (error \"panic\" \"no match found\"))"
        ))
    }

    fn expression(&mut self, expr: &TypedExpr) -> Result<String, Error> {
        match expr {
            TypedExpr::Int { value, .. } => self.int(value),
            TypedExpr::Float { value, .. } => Ok(value.to_string()),
            TypedExpr::String { value, .. } => Ok(format!("\"{value}\"")),
            TypedExpr::Block { statements, .. } => {
                let variables = self.collect_statements_variables(statements);
                let statements = self.statements(statements)?;

                Ok(format!("(let () {variables} {statements})"))
            }
            TypedExpr::Pipeline {
                assignments,
                finally,
                ..
            } => {
                let variables = self.collect_assignments_variables(assignments);
                let assignments = self.assignments(assignments)?;
                let finally = self.expression(finally)?;

                Ok(format!("(let () {variables} {assignments} {finally})"))
            }
            TypedExpr::Var {
                name, constructor, ..
            } => match &constructor.variant {
                ValueConstructorVariant::LocalVariable { .. } => Ok(format!("${name}")),
                ValueConstructorVariant::ModuleConstant { literal, .. } => self.constant(literal),
                ValueConstructorVariant::LocalConstant { .. } => todo!(),
                ValueConstructorVariant::ModuleFn { module, name, .. } => {
                    Ok(format!("{module}.{name}"))
                }
                ValueConstructorVariant::Record { module, name, .. } => {
                    Ok(format!("{module}.{name}"))
                }
            },
            TypedExpr::Fn { args, body, .. } => {
                let parameters = (args.iter())
                    .map(|arg| match &arg.names {
                        ArgNames::Discard { name, .. }
                        | ArgNames::LabelledDiscard { name, .. }
                        | ArgNames::Named { name, .. }
                        | ArgNames::NamedLabelled { name, .. } => format!("${name}"),
                    })
                    .join(" ");

                let variables = self.collect_statements_variables(&body);
                let body = self.statements(&body)?;

                Ok(format!("(lambda ({parameters}) {variables} {body})"))
            }
            TypedExpr::List { elements, tail, .. } => {
                let tail = if let Some(tail) = tail {
                    self.expression(tail)?
                } else {
                    format!("(list)")
                };

                let list = elements.iter().try_rfold(tail, |acc, elem| {
                    self.expression(elem) //
                        .map(|elem| format!("(cons {elem} {acc})"))
                })?;

                Ok(list)
            }
            TypedExpr::Call { fun, args, .. } => {
                let fun = self.expression(fun)?;

                let args: Vec<String> = args
                    .iter()
                    .map(|arg| self.expression(&arg.value))
                    .try_collect()?;
                let args = args.join(" ");

                Ok(format!("({fun} {args})"))
            }
            TypedExpr::BinOp {
                left, right, name, ..
            } => match name {
                BinOp::And => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(and {lhs} {rhs})"))
                }
                BinOp::Or => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(or {lhs} {rhs})"))
                }
                BinOp::Eq => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(equal? {lhs} {rhs})"))
                }
                BinOp::NotEq => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(not (equal? {lhs} {rhs}))"))
                }
                BinOp::LtInt | BinOp::LtFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(< {lhs} {rhs})"))
                }
                BinOp::LtEqInt | BinOp::LtEqFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(<= {lhs} {rhs})"))
                }
                BinOp::GtEqInt | BinOp::GtEqFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(>= {lhs} {rhs})"))
                }
                BinOp::GtInt | BinOp::GtFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(> {lhs} {rhs})"))
                }
                BinOp::AddInt | BinOp::AddFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(+ {lhs} {rhs})"))
                }
                BinOp::SubInt | BinOp::SubFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(- {lhs} {rhs})"))
                }
                BinOp::MultInt | BinOp::MultFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(* {lhs} {rhs})"))
                }
                BinOp::DivInt => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(div {lhs} {rhs})"))
                }
                BinOp::DivFloat => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(/ {lhs} {rhs})"))
                }
                BinOp::RemainderInt => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(remainder {lhs} {rhs})"))
                }
                BinOp::Concatenate => {
                    let lhs = self.expression(left)?;
                    let rhs = self.expression(right)?;

                    Ok(format!("(string-append {lhs} {rhs})"))
                }
            },
            TypedExpr::Case {
                subjects, clauses, ..
            } => {
                let vars = self.collect_clause_variables(clauses);

                let clauses: Vec<String> = clauses
                    .iter()
                    .map(|cl| self.clause(subjects.len(), cl))
                    .try_collect()?;

                let clauses = clauses.join(" ");

                let subjects: Vec<String> = (subjects.iter().enumerate())
                    .map(|(index, subject)| {
                        self.expression(subject) //
                            .map(|subject| format!("(_{index} {subject})"))
                    })
                    .try_collect()?;

                let subjects = subjects.join(" ");

                let cond = format!("(cond {clauses} (else (error \"panic\" \"unreachable\")))");

                Ok(format!("(let () {vars} (let ({subjects}) {cond}))"))
            }
            TypedExpr::RecordAccess { record, index, .. } => {
                let record = self.expression(record)?;
                let value = format!("(cadr {record})");

                Ok(format!("(vector-ref {value} {index})"))
            }
            TypedExpr::ModuleSelect {
                constructor,
                module_name,
                label,
                ..
            } => match constructor {
                ModuleValueConstructor::Record { name, .. } => Ok(format!("{module_name}.{name}")),
                ModuleValueConstructor::Fn { module, name, .. } => Ok(format!("{module}.{name}")),
                ModuleValueConstructor::Constant { .. } => Ok(format!("{module_name}.{label}")),
            },
            TypedExpr::Tuple { elems, .. } => {
                let elems: Vec<String> = elems
                    .iter()
                    .map(|elem| self.expression(elem))
                    .try_collect()?;

                let elems = elems.join(" ");

                Ok(format!("(vector {elems})"))
            }
            TypedExpr::TupleIndex { tuple, index, .. } => {
                let tuple = self.expression(tuple)?;

                Ok(format!("(vector-ref {tuple} {index})"))
            }
            TypedExpr::Todo { message, .. } => {
                if let Some(message) = message {
                    let message = self.expression(message)?;

                    Ok(format!("(error \"todo\" {message})"))
                } else {
                    Ok(format!("(error \"todo\" \"unimplemented\")"))
                }
            }
            TypedExpr::Panic { message, .. } => {
                if let Some(message) = message {
                    let message = self.expression(message)?;

                    Ok(format!("(error \"panic\" {message})"))
                } else {
                    Ok(format!("(error \"panic\" \"an error has occurred\")"))
                }
            }
            TypedExpr::BitArray { .. } => todo!(),
            TypedExpr::RecordUpdate { spread, args, .. } => {
                let value = self.expression(spread)?;
                let value_var = self.tmp_var();

                let tag = format!("(car {value_var})");
                let tag_var = self.tmp_var();

                let vec = format!("(vector-copy (cadr {value_var}))");
                let vec_var = self.tmp_var();

                let setters: Vec<_> = (args.iter())
                    .map(|arg| {
                        self.expression(&arg.value).map(|value| {
                            format!("(vector-set! {vec_var} {index} {value})", index = arg.index)
                        })
                    })
                    .try_collect()?;

                let setters = setters.join(" ");

                Ok(format!(
                    "(let (({value_var} {value})) (let (({tag_var} {tag}) ({vec_var} {vec})) {setters} (list {tag_var} {vec_var})))"
                ))
            }
            TypedExpr::NegateBool { value, .. } => {
                let value = self.expression(value)?;

                Ok(format!("(not {value})"))
            }
            TypedExpr::NegateInt { value, .. } => {
                let value = self.expression(value)?;

                Ok(format!("(- {value})"))
            }
            TypedExpr::Invalid { .. } => panic!("unreachable"),
        }
    }

    fn clause(&mut self, subjects: usize, clause: &TypedClause) -> Result<String, Error> {
        let subjects = (0..subjects).map(|index| format!("_{index}")).collect_vec();

        let base = self.patterns(&subjects, &clause.pattern)?;

        let rest: Vec<String> = clause
            .alternative_patterns
            .iter()
            .map(|alt| self.patterns(&subjects, alt))
            .try_collect()?;

        let rest = rest.join(" ");

        let guard = (clause.guard.as_ref())
            .map(|guard| self.guard(guard))
            .transpose()?
            .unwrap_or_default();

        let then = self.expression(&clause.then)?;

        Ok(format!("((and (or {base} {rest}) {guard}) {then})"))
    }

    fn guard(&mut self, guard: &TypedClauseGuard) -> Result<String, Error> {
        match guard {
            ClauseGuard::Equals { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(equal? {lhs} {rhs})"))
            }
            ClauseGuard::NotEquals { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(not (equal? {lhs} {rhs}))"))
            }
            ClauseGuard::GtInt { left, right, .. } | ClauseGuard::GtFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(> {lhs} {rhs})"))
            }
            ClauseGuard::GtEqInt { left, right, .. }
            | ClauseGuard::GtEqFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(>= {lhs} {rhs})"))
            }
            ClauseGuard::LtInt { left, right, .. } | ClauseGuard::LtFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(< {lhs} {rhs})"))
            }
            ClauseGuard::LtEqInt { left, right, .. }
            | ClauseGuard::LtEqFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(<= {lhs} {rhs})"))
            }
            ClauseGuard::AddInt { left, right, .. } | ClauseGuard::AddFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(+ {lhs} {rhs})"))
            }
            ClauseGuard::SubInt { left, right, .. } | ClauseGuard::SubFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(- {lhs} {rhs})"))
            }
            ClauseGuard::MultInt { left, right, .. }
            | ClauseGuard::MultFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(* {lhs} {rhs})"))
            }
            ClauseGuard::DivInt { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(div {lhs} {rhs})"))
            }
            ClauseGuard::DivFloat { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(/ {lhs} {rhs})"))
            }
            ClauseGuard::RemainderInt { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(remainder {lhs} {rhs})"))
            }
            ClauseGuard::Or { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(or {lhs} {rhs})"))
            }
            ClauseGuard::And { left, right, .. } => {
                let lhs = self.guard(left)?;
                let rhs = self.guard(right)?;

                Ok(format!("(and {lhs} {rhs})"))
            }
            ClauseGuard::Not { expression, .. } => {
                let expression = self.guard(expression)?;

                Ok(format!("(not {expression})"))
            }
            ClauseGuard::Var { name, .. } => Ok(name.to_string()),
            ClauseGuard::TupleIndex { tuple, index, .. } => {
                let tuple = self.guard(tuple)?;

                Ok(format!("(vector-ref {tuple} {index})"))
            }
            ClauseGuard::FieldAccess { .. } => todo!(),
            ClauseGuard::ModuleSelect { .. } => todo!(),
            ClauseGuard::Constant(constant) => self.constant(constant),
        }
    }

    fn int(&self, value: &str) -> Result<String, Error> {
        Ok(value.replace("_", ""))
    }

    fn constant(&self, constant: &TypedConstant) -> Result<String, Error> {
        match constant {
            Constant::Int { value, .. } => self.int(value),
            Constant::Float { value, .. } => Ok(format!("{value}")),
            Constant::String { value, .. } => Ok(format!("\"{value}\"")),
            Constant::Tuple { elements, .. } => {
                let elements: Vec<String> = elements
                    .iter()
                    .map(|element| self.constant(element))
                    .try_collect()?;

                let elements = elements.join(" ");

                Ok(format!("(vector {elements})"))
            }
            Constant::List { elements, .. } => {
                let elements: Vec<String> = elements
                    .iter()
                    .map(|element| self.constant(element))
                    .try_collect()?;

                let elements = elements.join(" ");

                Ok(format!("(list {elements})"))
            }
            Constant::Record { .. } => todo!(),
            Constant::BitArray { .. } => todo!(),
            Constant::Var { .. } => todo!(),
            Constant::StringConcatenation { .. } => todo!(),
            Constant::Invalid { .. } => panic!("unreachable"),
        }
    }

    fn patterns(
        &mut self,
        subjects: &[String],
        patterns: &[TypedPattern],
    ) -> Result<String, Error> {
        let patterns: Vec<String> = patterns
            .iter()
            .zip(subjects.iter())
            .map(|(ptn, subject)| self.pattern(subject, ptn))
            .try_collect()?;

        let patterns = patterns.join(" ");

        Ok(format!("(and {patterns})"))
    }

    fn pattern(&mut self, subject: &str, pattern: &TypedPattern) -> Result<String, Error> {
        match pattern {
            Pattern::Int { value, .. } => Ok(format!("(equal? {subject} {})", self.int(value)?)),
            Pattern::Float { value, .. } => Ok(format!("(equal? {subject} {value})")),
            Pattern::String { value, .. } => Ok(format!("(equal? {subject} \"{value}\")")),
            Pattern::Variable { name, .. } => {
                Ok(format!("(begin (set! ${name} {subject}) gleam.True)"))
            }
            // TODO: BitArray
            Pattern::VarUsage { .. } => todo!(),
            Pattern::Assign { pattern, .. } => self.pattern(subject, pattern),
            Pattern::Discard { .. } => Ok(format!("gleam.True")),
            Pattern::List { elements, tail, .. } => self.list_pattern(subject, elements, tail),
            Pattern::Constructor {
                constructor,
                arguments,
                ..
            } => match constructor {
                Inferred::Known(constructor) => {
                    let check = format!("{}.{}?", constructor.module, constructor.name);

                    let arguments: Vec<String> = arguments
                        .iter()
                        .enumerate()
                        .map(|(index, argument)| {
                            let subject = format!("(vector-ref (cadr {subject}) {index})");

                            self.pattern(&subject, &argument.value)
                        })
                        .try_collect()?;

                    let arguments = arguments.join(" ");

                    Ok(format!("(and ({check} {subject}) {arguments})"))
                }
                Inferred::Unknown => panic!("unreachable"),
            },
            Pattern::Tuple { elems, .. } => {
                let elems: Vec<String> = elems
                    .iter()
                    .enumerate()
                    .map(|(index, elem)| {
                        let var = self.tmp_var();

                        self.pattern(&var, elem).map(|pattern| {
                            let elem = format!("(vector-ref {subject} {index})");

                            format!("(let (({var} {elem})) {pattern})")
                        })
                    })
                    .try_collect()?;

                let elems = elems.join(" ");

                Ok(format!("(and {elems})"))
            }
            Pattern::BitArray { .. } => todo!(),
            Pattern::StringPrefix {
                left_side_string,
                left_side_assignment,
                right_side_assignment,
                ..
            } => {
                let has_prefix = format!("(gleam.string-prefix? {subject} \"{left_side_string}\")");

                let left = left_side_assignment
                    .as_ref()
                    .map(|(name, _)| format!("(set! ${name} \"{left_side_string}\")"))
                    .unwrap_or_default();

                let right = right_side_assignment
                    .assigned_name()
                    .map(|name| {
                        let from = left_side_string.len();
                        let length = format!("(string-length {subject})");

                        format!("(set! ${name} (substring {subject} {from} {length}))")
                    })
                    .unwrap_or_default();

                Ok(format!(
                    "(if {has_prefix} (begin {left} {right} gleam.True) gleam.False)"
                ))
            }
            Pattern::Invalid { .. } => panic!("unreachable"),
        }
    }

    fn list_pattern(
        &mut self,
        subject: &str,
        elements: &[TypedPattern],
        tail: &Option<Box<TypedPattern>>,
    ) -> Result<String, Error> {
        match elements {
            [] => match tail {
                Some(tail) => self.pattern(subject, tail),
                None => Ok(format!("(equal? {subject} (list))")),
            },
            [element, elements @ ..] => {
                let item = format!("(car {subject})");
                let rest = format!("(cdr {subject})");

                let item_var = self.tmp_var();
                let rest_var = self.tmp_var();

                let cond = self.pattern(&item_var, element)?;
                let cond = format!("(if (not (equal? {subject} (list))) (let (({item_var} {item})) {cond}) gleam.False)");

                let next = self.list_pattern(&rest_var, elements, tail)?;
                let next = format!("(let (({rest_var} {rest})) {next})");

                Ok(format!("(if {cond} {next} gleam.False)"))
            }
        }
    }

    fn collect_statements_variables(&self, statements: &[TypedStatement]) -> String {
        let mut variables = HashSet::new();

        for statement in statements {
            self.do_collect_statement_variables(&mut variables, statement);
        }

        variables
            .into_iter()
            .map(|var| format!("(define ${var})"))
            .join(" ")
    }

    fn do_collect_statement_variables(
        &self,
        set: &mut HashSet<EcoString>,
        statement: &TypedStatement,
    ) {
        match statement {
            Statement::Expression(_) => {}
            Statement::Assignment(assignment) => {
                self.do_collect_pattern_variables(set, &assignment.pattern)
            }
            Statement::Use(_) => panic!("unreachable"),
        }
    }

    fn collect_assignments_variables(&self, assignments: &[TypedAssignment]) -> String {
        let mut variables = HashSet::new();

        for assignment in assignments {
            self.do_collect_pattern_variables(&mut variables, &assignment.pattern);
        }

        variables
            .into_iter()
            .map(|var| format!("(define ${var})"))
            .join(" ")
    }

    fn collect_clause_variables(&self, clauses: &[TypedClause]) -> String {
        let mut variables = HashSet::new();

        for clause in clauses {
            for pattern in &clause.pattern {
                self.do_collect_pattern_variables(&mut variables, pattern);
            }
        }

        variables
            .into_iter()
            .map(|var| format!("(define ${var})"))
            .join(" ")
    }

    fn do_collect_pattern_variables(&self, set: &mut HashSet<EcoString>, pattern: &TypedPattern) {
        match pattern {
            Pattern::Int { .. }
            | Pattern::Float { .. }
            | Pattern::String { .. }
            | Pattern::Discard { .. } => {}
            Pattern::Variable { name, .. } => {
                let _ = set.insert(name.clone());
            }
            Pattern::VarUsage { .. } => todo!(),
            Pattern::Assign { pattern, name, .. } => {
                let _ = set.insert(name.clone());

                self.do_collect_pattern_variables(set, pattern);
            }
            Pattern::List { elements, tail, .. } => {
                for element in elements {
                    self.do_collect_pattern_variables(set, element);
                }

                if let Some(tail) = tail {
                    self.do_collect_pattern_variables(set, tail);
                }
            }
            Pattern::Constructor { arguments, .. } => {
                for argument in arguments {
                    self.do_collect_pattern_variables(set, &argument.value);
                }
            }
            Pattern::Tuple { elems, .. } => {
                for elem in elems {
                    self.do_collect_pattern_variables(set, elem);
                }
            }
            Pattern::BitArray { .. } => todo!(),
            Pattern::StringPrefix {
                left_side_assignment,
                right_side_assignment,
                ..
            } => {
                if let Some((name, _)) = left_side_assignment {
                    let _ = set.insert(name.clone());
                }

                if let AssignName::Variable(name) = right_side_assignment {
                    let _ = set.insert(name.clone());
                }
            }
            Pattern::Invalid { .. } => panic!("unreachable!"),
        }
    }

    fn tmp_var(&mut self) -> String {
        self.tmp_count += 1;
        format!("$_{}", self.tmp_count)
    }
}
