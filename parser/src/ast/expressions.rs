use shared_context::{Const, Type};

use crate::ast::{Span, SpannedIdentifier};

/// Represents a single expression node in the AST.
///
/// Each Expression contains its type variant (`InnerExpression`)
/// and a Span indicating its position in the source.
#[derive(Debug)]
pub struct Expression {
    inner: InnerExpression,
    expr_type: Type,
    span: Span,
}

/// Enumerates all supported expression variants.
///
/// Expressions can represent literals, operations, variable references,
/// assignments, function calls, and conditional expressions.
#[derive(Debug)]
pub enum InnerExpression {
    /// A constant integer literal, e.g. `42`.
    Constant(Const),

    /// A unary operation such as negation or logical NOT.
    Unary {
        operator: UnaryOP,
        operand: Box<Expression>,
    },

    /// A binary operation (e.g., `a + b`, `x == y`).
    Binary {
        operator: BinaryOP,
        operand1: Box<Expression>,
        operand2: Box<Expression>,
    },

    /// A conditional expression (`cond ? cons : alt`).
    Conditional {
        cond: Box<Expression>,
        cons: Box<Expression>,
        alt: Box<Expression>,
    },

    /// a type casting (e.g., `(long) 12 * 6`)
    Cast {
        target_type: Type,
        expr: Box<Expression>,
    },

    /// A variable reference.
    Var(SpannedIdentifier),

    /// An assignment expression (`a = b`).
    Assignment {
        lvalue: Box<Expression>,
        rvalue: Box<Expression>,
    },

    /// A function call expression, e.g. `foo(x, y)`.
    FunctionCall {
        name: SpannedIdentifier,
        args: Vec<Box<Expression>>,
    },
}

impl Expression {
    /// Creates a new Expression with the given type and span.
    pub fn new(inner: InnerExpression, expr_type: Type, span: Span) -> Self {
        Self {
            inner,
            expr_type,
            span,
        }
    }

    pub fn set_type(&mut self, new_type: Type) {
        self.expr_type = new_type
    }

    pub fn get_type(&self) -> Type {
        self.expr_type
    }

    /// Returns a shared reference to the underlying InnerExpression.
    pub fn get_inner_ref(&self) -> &InnerExpression {
        &self.inner
    }

    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Deconstructs the expression into its variant and span.
    pub fn into_parts(self) -> (InnerExpression, Type, Span) {
        (self.inner, self.expr_type, self.span)
    }
}

/// Represents all supported unary operators.
///
/// These are operators that operate on a single operand.
#[derive(Debug, Clone, Copy)]
pub enum UnaryOP {
    /// Arithmetic negation (`-x`).
    Neg,
    /// Logical NOT (`!x`).
    LogicalNot,
    /// Bitwise NOT (`~x`).
    BitwiseNot,
}

/// Represents all supported binary operators.
///
/// These are operators that combine two operands.
#[derive(Debug, Clone, Copy)]
pub enum BinaryOP {
    // Arithmetic operators
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Logical operators
    LogicalAnd,
    LogicalOr,

    // Comparison operators
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEq,
    GreaterThanOrEq,
}

impl BinaryOP {
    pub fn is_logical(&self) -> bool {
        matches!(self, Self::LogicalAnd | Self::LogicalOr)
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod
        )
    }
}
