use camino::Utf8Path;
use itertools::Itertools;

use crate::{
    ast::{
        ArgNames, BinOp, Definition, Pattern, Statement, TypedAssignment, TypedExpr, TypedFunction,
        TypedModule, TypedStatement,
    },
    type_::ValueConstructorVariant,
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

    for definition in &module.definitions {
        match definition {
            Definition::Function(function) => {
                code += &function_definition(module, function)?;
                code += "\n";
            }
            Definition::TypeAlias(_) => todo!(),
            Definition::CustomType(_) => todo!(),
            Definition::Import(_) => todo!(),
            Definition::ModuleConstant(_) => todo!(),
        }
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

    Ok(format!(
        r#"(define {name}
  (lambda ({parameters})
    {body}))"#
    ))
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
            ValueConstructorVariant::ModuleFn { module, name, .. } => {
                Ok(format!("{module}.{name}"))
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
            BinOp::And => todo!(),
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
        TypedExpr::Case { .. } => todo!(),
        TypedExpr::RecordAccess { .. } => todo!(),
        TypedExpr::ModuleSelect { .. } => todo!(),
        TypedExpr::Tuple { .. } => todo!(),
        TypedExpr::TupleIndex { .. } => todo!(),
        TypedExpr::Todo { .. } => todo!(),
        TypedExpr::Panic { .. } => todo!(),
        TypedExpr::BitArray { .. } => todo!(),
        TypedExpr::RecordUpdate { .. } => todo!(),
        TypedExpr::NegateBool { .. } => todo!(),
        TypedExpr::NegateInt { .. } => todo!(),
        TypedExpr::Invalid { .. } => panic!("unreachable"),
    }
}
