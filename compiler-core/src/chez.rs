use std::sync::Arc;

use camino::Utf8Path;
use itertools::Itertools;

use crate::{
    analyse::Inferred,
    ast::{
        ArgNames, BinOp, CustomType, Definition, Pattern, Statement, TypedAssignment, TypedClause,
        TypedExpr, TypedFunction, TypedModule, TypedPattern, TypedStatement,
    },
    type_::{ModuleValueConstructor, Type, ValueConstructorVariant},
    Error,
};

pub const PRELUDE: &'static str = include_str!("../templates/prelude.scm");

pub fn module(dir: &Utf8Path, module: &TypedModule) -> Result<String, Error> {
    let mut code = String::new();

    let prelude = dir
        .parent()
        .expect("expecting a parent directory")
        .join("prelude.scm");

    code += &format!(r#"(load "{prelude}")"#);
    code += "\n";

    for definition in &module.definitions {
        match definition {
            Definition::Function(function) => {
                code += &function_definition(dir, module, function)?;
                code += "\n";
            }
            Definition::TypeAlias(_) => todo!(),
            Definition::CustomType(type_) => {
                code += &custom_type(module, type_)?;
                code += "\n";
            }
            Definition::Import(package) => {
                let path = dir.join(package.module.as_str());

                code += &format!("(load \"{path}.scm\")");
                code += "\n";
            }
            Definition::ModuleConstant(_) => todo!(),
        }
    }

    Ok(code)
}

fn custom_type(module: &TypedModule, type_: &CustomType<Arc<Type>>) -> Result<String, Error> {
    let mut code = String::new();

    for constructor in &type_.constructors {
        let name = format!("{}.{}", module.name, constructor.name);
        let tag = &constructor.name;

        let args = (0..constructor.arguments.len())
            .map(|idx| format!("_{idx}"))
            .join(" ");

        let variant = if args.is_empty() {
            format!("(define ({name}) (list '{tag}))")
        } else {
            format!("(define ({name} {args}) (list '{tag} (vector {args})))")
        };

        code += &variant;
        code += "\n";

        let predicate = format!("(define ({name}? value) (eq? (car value) '{tag}))");

        code += &predicate;
        code += "\n";
    }

    Ok(code)
}

fn function_definition(
    dir: &Utf8Path,
    module: &TypedModule,
    function: &TypedFunction,
) -> Result<String, Error> {
    let parameters = (function.arguments.iter())
        .map(|arg| match &arg.names {
            ArgNames::Discard { name, .. }
            | ArgNames::LabelledDiscard { name, .. }
            | ArgNames::Named { name, .. }
            | ArgNames::NamedLabelled { name, .. } => name,
        })
        .join(" ");

    let name = function
        .name
        .as_ref()
        .map(|(_, name)| name)
        .expect("function to have name");

    let name = format!("{}.{name}", module.name);

    if let Some((module, external, _)) = &function.external_chez {
        let (import, external) = if !module.is_empty() {
            let path = dir.join(module.as_str());
            let import = format!("(load \"{path}.scm\")\n");

            (import, format!("{module}.{external}"))
        } else {
            (String::new(), external.to_string())
        };

        let body = format!("({external} {parameters})");

        Ok(format!(
            "{import}(define {name} (lambda ({parameters}) {body}))"
        ))
    } else {
        let body = statements(module, &function.body)?;
        let body = format!("(begin {body})");

        Ok(format!("(define {name} (lambda ({parameters}) {body}))"))
    }
}

fn statements(module: &TypedModule, stmts: &[TypedStatement]) -> Result<String, Error> {
    match stmts {
        [] => Ok(String::new()),
        [statement, stmts @ ..] => match statement {
            Statement::Expression(expr) => {
                let expr = expression(module, expr)?;
                let rest = statements(module, stmts)?;

                Ok(format!("{expr} {rest}"))
            }
            Statement::Assignment(assign) => assignment(module, assign, stmts),
            Statement::Use(_) => panic!("unreachable"),
        },
    }
}

fn assignment(
    module: &TypedModule,
    assign: &TypedAssignment,
    stmts: &[TypedStatement],
) -> Result<String, Error> {
    let value = expression(module, &assign.value)?;
    let rest = statements(module, stmts)?;

    match &assign.pattern {
        Pattern::Int { .. } => todo!(),
        Pattern::Float { .. } => todo!(),
        Pattern::String { .. } => todo!(),
        Pattern::Variable { name, .. } => Ok(format!("(let (({name} {value})) (begin {rest}))")),
        Pattern::VarUsage { .. } => todo!(),
        Pattern::Assign { .. } => todo!(),
        Pattern::Discard { .. } => Ok(format!("(let ((_ {value})) (begin {rest}))")),
        Pattern::List { .. } => todo!(),
        Pattern::Constructor { .. } => todo!(),
        Pattern::Tuple { .. } => todo!(),
        Pattern::BitArray { .. } => todo!(),
        Pattern::StringPrefix { .. } => todo!(),
        Pattern::Invalid { .. } => panic!("unreachable"),
    }
}

fn expression(module: &TypedModule, expr: &TypedExpr) -> Result<String, Error> {
    match expr {
        TypedExpr::Int { value, .. } => Ok(value.to_string()),
        TypedExpr::Float { value, .. } => Ok(value.to_string()),
        TypedExpr::String { value, .. } => Ok(format!("\"{value}\"")),
        TypedExpr::Block { .. } => todo!(),
        TypedExpr::Pipeline { .. } => todo!(),
        TypedExpr::Var {
            name, constructor, ..
        } => match &constructor.variant {
            ValueConstructorVariant::LocalVariable { .. } => Ok(name.to_string()),
            ValueConstructorVariant::ModuleConstant { .. } => todo!(),
            ValueConstructorVariant::LocalConstant { .. } => todo!(),
            ValueConstructorVariant::ModuleFn { module, name, .. } => {
                Ok(format!("{module}.{name}"))
            }
            ValueConstructorVariant::Record { module, name, .. } => Ok(format!("{module}.{name}")),
        },
        TypedExpr::Fn { args, body, .. } => {
            let parameters = (args.iter())
                .map(|arg| match &arg.names {
                    ArgNames::Discard { name, .. }
                    | ArgNames::LabelledDiscard { name, .. }
                    | ArgNames::Named { name, .. }
                    | ArgNames::NamedLabelled { name, .. } => name,
                })
                .join(" ");

            let body = statements(module, &body)?;
            let body = format!("(begin {body})");

            Ok(format!("(lambda ({parameters}) {body})"))
        }
        TypedExpr::List { elements, tail, .. } => {
            let tail = if let Some(tail) = tail {
                expression(module, tail)?
            } else {
                format!("(list)")
            };

            let list = elements.iter().try_rfold(tail, |acc, elem| {
                expression(module, elem) //
                    .map(|elem| format!("(cons {elem} {acc})"))
            })?;

            Ok(list)
        }
        TypedExpr::Call { fun, args, .. } => {
            let fun = expression(module, fun)?;

            let args: Vec<String> = args
                .iter()
                .map(|arg| expression(module, &arg.value))
                .try_collect()?;
            let args = args.join(" ");

            Ok(format!("({fun} {args})"))
        }
        TypedExpr::BinOp {
            left, right, name, ..
        } => match name {
            BinOp::And => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(and {lhs} {rhs})"))
            }
            BinOp::Or => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(or {lhs} {rhs})"))
            }
            BinOp::Eq => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(eq? {lhs} {rhs})"))
            }
            BinOp::NotEq => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(not (eq? {lhs} {rhs}))"))
            }
            BinOp::LtInt | BinOp::LtFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(< {lhs} {rhs})"))
            }
            BinOp::LtEqInt | BinOp::LtEqFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(<= {lhs} {rhs})"))
            }
            BinOp::GtEqInt | BinOp::GtEqFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(>= {lhs} {rhs})"))
            }
            BinOp::GtInt | BinOp::GtFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(> {lhs} {rhs})"))
            }
            BinOp::AddInt | BinOp::AddFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(+ {lhs} {rhs})"))
            }
            BinOp::SubInt | BinOp::SubFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(- {lhs} {rhs})"))
            }
            BinOp::MultInt | BinOp::MultFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(* {lhs} {rhs})"))
            }
            BinOp::DivInt => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(div {lhs} {rhs})"))
            }
            BinOp::DivFloat => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(/ {lhs} {rhs})"))
            }
            BinOp::RemainderInt => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(% {lhs} {rhs})"))
            }
            BinOp::Concatenate => {
                let lhs = expression(module, left)?;
                let rhs = expression(module, right)?;

                Ok(format!("(string-append {lhs} {rhs})"))
            }
        },
        TypedExpr::Case {
            subjects, clauses, ..
        } => {
            let clauses: Vec<String> = clauses
                .iter()
                .map(|cl| clause(module, subjects.len(), cl))
                .try_collect()?;

            let clauses = clauses.join(" ");

            let subjects: Vec<String> = (subjects.iter().enumerate())
                .map(|(index, subject)| {
                    expression(module, subject) //
                        .map(|subject| format!("(_{index} {subject})"))
                })
                .try_collect()?;

            let subjects = subjects.join(" ");

            let cond = format!("(cond {clauses} (else (error \"panic\" \"unreachable\")))");

            Ok(format!("(let ({subjects}) {cond})"))
        }
        TypedExpr::RecordAccess { record, index, .. } => {
            let record = expression(module, record)?;
            let value = format!("(cadr {record})");

            Ok(format!("(vector-ref {value} {index})"))
        }
        TypedExpr::ModuleSelect {
            constructor,
            module_name,
            ..
        } => match constructor {
            ModuleValueConstructor::Record { name, .. } => Ok(format!("{module_name}.{name}")),
            ModuleValueConstructor::Fn { module, name, .. } => Ok(format!("{module}.{name}")),
            ModuleValueConstructor::Constant { .. } => todo!(),
        },
        TypedExpr::Tuple { elems, .. } => {
            let elems: Vec<String> = elems
                .iter()
                .map(|elem| expression(module, elem))
                .try_collect()?;

            let elems = elems.join(" ");

            Ok(format!("(vector {elems})"))
        }
        TypedExpr::TupleIndex { tuple, index, .. } => {
            let tuple = expression(module, tuple)?;

            Ok(format!("(vector-ref {tuple} {index})"))
        }
        TypedExpr::Todo { message, .. } => {
            if let Some(message) = message {
                let message = expression(module, message)?;

                Ok(format!("(error \"todo\" {message})"))
            } else {
                Ok(format!("(error \"todo\" \"unimplemented\")"))
            }
        }
        TypedExpr::Panic { message, .. } => {
            if let Some(message) = message {
                let message = expression(module, message)?;

                Ok(format!("(error \"panic\" {message})"))
            } else {
                Ok(format!("(error \"panic\" \"an error has occurred\")"))
            }
        }
        TypedExpr::BitArray { .. } => todo!(),
        TypedExpr::RecordUpdate { .. } => todo!(),
        TypedExpr::NegateBool { value, .. } => {
            let value = expression(module, value)?;

            Ok(format!("(not {value})"))
        }
        TypedExpr::NegateInt { value, .. } => {
            let value = expression(module, value)?;

            Ok(format!("(- {value})"))
        }
        TypedExpr::Invalid { .. } => panic!("unreachable"),
    }
}

fn clause(module: &TypedModule, subjects: usize, clause: &TypedClause) -> Result<String, Error> {
    let subjects = (0..subjects).map(|index| format!("_{index}")).collect_vec();

    let base = patterns(module, &subjects, &clause.pattern)?;

    let rest: Vec<String> = clause
        .alternative_patterns
        .iter()
        .map(|alt| patterns(module, &subjects, alt))
        .try_collect()?;

    let rest = rest.join(" ");

    let then = expression(module, &clause.then)?;

    Ok(format!("((or {base} {rest}) {then})"))
}

fn patterns(
    module: &TypedModule,
    subjects: &[String],
    patterns: &[TypedPattern],
) -> Result<String, Error> {
    let patterns: Vec<String> = patterns
        .iter()
        .zip(subjects.iter())
        .map(|(ptn, subject)| pattern(module, subject, ptn))
        .try_collect()?;

    let patterns = patterns.join(" ");

    Ok(format!("(and {patterns})"))
}

fn pattern(_module: &TypedModule, subject: &str, ptn: &TypedPattern) -> Result<String, Error> {
    match ptn {
        Pattern::Int { value, .. } | Pattern::Float { value, .. } => {
            Ok(format!("(eq? {subject} {value})"))
        }
        Pattern::String { value, .. } => Ok(format!("(eq? {subject} \"{value}\")")),
        Pattern::Variable { .. } => Ok(format!("gleam.True")),
        // TODO: BitArray
        Pattern::VarUsage { .. } => todo!(),
        Pattern::Assign { .. } => todo!(),
        Pattern::Discard { .. } => Ok(format!("gleam.True")),
        Pattern::List { .. } => todo!(),
        Pattern::Constructor { constructor, .. } => match constructor {
            Inferred::Known(constructor) => {
                let check = format!("{}.{}?", constructor.module, constructor.name);

                Ok(format!("({check} {subject})"))
            }
            Inferred::Unknown => panic!("unreachable"),
        },
        Pattern::Tuple { .. } => todo!(),
        Pattern::BitArray { .. } => todo!(),
        Pattern::StringPrefix { .. } => todo!(),
        Pattern::Invalid { .. } => panic!("unreachable"),
    }
}
