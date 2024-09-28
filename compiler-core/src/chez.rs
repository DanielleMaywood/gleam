use std::sync::Arc;

use camino::Utf8Path;
use itertools::Itertools;

use crate::{
    ast::{
        ArgNames, BinOp, CustomType, Definition, Pattern, Statement, TypedAssignment, TypedClause,
        TypedExpr, TypedFunction, TypedModule, TypedPattern, TypedStatement,
    },
    type_::{Type, ValueConstructorVariant},
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
                code += &function_definition(module, function)?;
                code += "\n";
            }
            Definition::TypeAlias(_) => todo!(),
            Definition::CustomType(type_) => {
                code += &custom_type(module, type_)?;
                code += "\n";
            }
            Definition::Import(_) => todo!(),
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

fn function_definition(module: &TypedModule, function: &TypedFunction) -> Result<String, Error> {
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

    let body = statements(module, &function.body)?;
    let body = format!("(begin {body})");

    Ok(format!("(define {name} (lambda ({parameters}) {body}))"))
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
        Pattern::Invalid { .. } => todo!(),
    }
}

fn expression(module: &TypedModule, expr: &TypedExpr) -> Result<String, Error> {
    match expr {
        TypedExpr::Int { value, .. } => Ok(value.to_string()),
        TypedExpr::Float { value, .. } => Ok(value.to_string()),
        TypedExpr::String { .. } => todo!(),
        TypedExpr::Block { .. } => todo!(),
        TypedExpr::Pipeline { .. } => todo!(),
        TypedExpr::Var {
            name, constructor, ..
        } => match &constructor.variant {
            ValueConstructorVariant::LocalVariable { .. } => Ok(name.to_string()),
            ValueConstructorVariant::ModuleConstant { .. } => todo!(),
            ValueConstructorVariant::LocalConstant { .. } => todo!(),
            ValueConstructorVariant::ModuleFn {
                module,
                name,
                external_chez,
                ..
            } => {
                if let Some((_module, name)) = external_chez {
                    Ok(format!("{name}"))
                } else {
                    Ok(format!("{module}.{name}"))
                }
            }
            ValueConstructorVariant::Record { module, name, .. } => Ok(format!("{module}.{name}")),
        },
        TypedExpr::Fn { .. } => todo!(),
        TypedExpr::List { .. } => todo!(),
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
            BinOp::Or => todo!(),
            BinOp::Eq => todo!(),
            BinOp::NotEq => todo!(),
            BinOp::LtInt => todo!(),
            BinOp::LtEqInt => todo!(),
            BinOp::LtFloat => todo!(),
            BinOp::LtEqFloat => todo!(),
            BinOp::GtEqInt => todo!(),
            BinOp::GtInt => todo!(),
            BinOp::GtEqFloat => todo!(),
            BinOp::GtFloat => todo!(),
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
            BinOp::MultInt => todo!(),
            BinOp::MultFloat => todo!(),
            BinOp::DivInt => todo!(),
            BinOp::DivFloat => todo!(),
            BinOp::RemainderInt => todo!(),
            BinOp::Concatenate => todo!(),
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
        TypedExpr::ModuleSelect { .. } => todo!(),
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
        TypedExpr::Todo { .. } => todo!(),
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
        Pattern::Int { value, .. } => Ok(format!("(eq? {subject} {value})")),
        Pattern::Float { .. } => todo!(),
        Pattern::String { .. } => todo!(),
        Pattern::Variable { .. } => Ok(format!("gleam.True")),
        Pattern::VarUsage { .. } => todo!(),
        Pattern::Assign { .. } => todo!(),
        Pattern::Discard { .. } => Ok(format!("gleam.True")),
        Pattern::List { .. } => todo!(),
        Pattern::Constructor { .. } => todo!(),
        Pattern::Tuple { .. } => todo!(),
        Pattern::BitArray { .. } => todo!(),
        Pattern::StringPrefix { .. } => todo!(),
        Pattern::Invalid { .. } => panic!("unreachable"),
    }
}
