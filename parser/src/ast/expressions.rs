use crate::ast::{Identifier, Spanned};

#[derive(Debug)]
pub enum Expression {
    Constant(i32),
    Unary {
        operator: UnaryOP,
        operand: Box<Spanned<Expression>>,
    },
    Binary {
        operator: BinaryOP,
        operand1: Box<Spanned<Expression>>,
        operand2: Box<Spanned<Expression>>,
    },
    Conditional {
        cond: Box<Spanned<Expression>>,
        cons: Box<Spanned<Expression>>,
        alt: Box<Spanned<Expression>>,
    },
    Var(Spanned<Identifier>),
    Assignment {
        lvalue: Box<Spanned<Expression>>,
        rvalue: Box<Spanned<Expression>>,
    },
}

#[derive(Debug)]
pub enum UnaryOP {
    // arithmatic
    Neg,
    // logical
    LogicalNot,
    // bitwise
    Not,
}

#[derive(Debug)]
pub enum BinaryOP {
    // arithmatic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // logical
    LogicalAnd,
    LogicalOr,

    // comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEq,
    GreaterThanOrEq,
}
