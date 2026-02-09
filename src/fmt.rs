use crate::*;
use std::fmt::{Debug, Formatter, Result};

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::If(cond, then, els) => {
                if let Some(els) = els {
                    write!(f, "if {cond:?} then {then:?} else {els:?}")
                } else {
                    write!(f, "if {cond:?} then {then:?}")
                }
            }
            Expr::While(cond, body) => {
                write!(f, "while {cond:?} do {body:?}")
            }
            Expr::Break(expr) => {
                write!(f, "break {expr:?}")
            }
            Expr::Return(expr) => write!(f, "return {expr:?}"),
            Expr::Block(lines) => {
                let lines = lines
                    .iter()
                    .map(|line| format!("\t{line:?}"))
                    .collect::<Vec<String>>()
                    .join("\n");
                write!(f, "{{\n{lines}\n}}")
            }
            Expr::Call(callee, args) => {
                let args = args
                    .iter()
                    .map(|arg| format!("{arg:?}"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{callee:?}({args})")
            }
            Expr::Variable(name) => write!(f, "{name}"),
            Expr::Pointer(var) => write!(f, "&{var}"),
            Expr::Derefer(expr) => write!(f, "*{expr:?}"),
            Expr::Let(name, value) => write!(f, "let {name:?} = {value:?}"),
            Expr::Integer(value) => write!(f, "{value}"),
            Expr::String(value) => write!(f, "{value}"),
            Expr::Add(lhs, rhs) => write!(f, "({lhs:?} + {rhs:?})"),
            Expr::Sub(lhs, rhs) => write!(f, "({lhs:?} - {rhs:?})"),
            Expr::Mul(lhs, rhs) => write!(f, "({lhs:?} * {rhs:?})"),
            Expr::Eql(lhs, rhs) => write!(f, "({lhs:?} == {rhs:?})"),
            Expr::NotEq(lhs, rhs) => write!(f, "({lhs:?} != {rhs:?})"),
            Expr::Gt(lhs, rhs) => write!(f, "({lhs:?} > {rhs:?})"),
            Expr::Lt(lhs, rhs) => write!(f, "({lhs:?} < {rhs:?})"),
            Expr::GtEq(lhs, rhs) => write!(f, "({lhs:?} >= {rhs:?})"),
            Expr::LtEq(lhs, rhs) => write!(f, "({lhs:?} <= {rhs:?})"),
            Expr::And(lhs, rhs) => write!(f, "({lhs:?} & {rhs:?})"),
            Expr::Or(lhs, rhs) => write!(f, "({lhs:?} | {rhs:?})"),
            Expr::Xor(lhs, rhs) => write!(f, "({lhs:?} ^ {rhs:?})"),
            Expr::Div(lhs, rhs) => write!(f, "({lhs:?} / {rhs:?})"),
            Expr::Mod(lhs, rhs) => write!(f, "({lhs:?} % {rhs:?})"),
        }
    }
}
