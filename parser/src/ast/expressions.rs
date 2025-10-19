use crate::ast::{Span, SpannedIdentifier};

#[derive(Debug)]
pub struct Expression {
    expr: ExpressionType,
    span: Span,
}

#[derive(Debug)]
pub enum ExpressionType {
    Constant(i32),
    Unary {
        operator: UnaryOP,
        operand: Box<Expression>,
    },
    Binary {
        operator: BinaryOP,
        operand1: Box<Expression>,
        operand2: Box<Expression>,
    },
    Conditional {
        cond: Box<Expression>,
        cons: Box<Expression>,
        alt: Box<Expression>,
    },
    Var(SpannedIdentifier),
    Assignment {
        lvalue: Box<Expression>,
        rvalue: Box<Expression>,
    },
}

impl Expression {
    pub fn new(expr: ExpressionType, span: Span) -> Self {
        Self { expr, span }
    }

    pub fn get_expr_type_ref(&self) -> &ExpressionType {
        &self.expr
    }

    pub fn into_parts(self) -> (ExpressionType, Span) {
        (self.expr, self.span)
    }
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
