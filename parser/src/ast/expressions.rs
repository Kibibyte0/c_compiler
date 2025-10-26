use crate::ast::{Span, SpannedIdentifier};

/// Represents a single expression node in the AST.
///
/// Each [`Expression`] contains its type variant (`ExpressionType`)
/// and a [`Span`] indicating its position in the source.
#[derive(Debug)]
pub struct Expression {
    expr: ExpressionType,
    span: Span,
}

/// Enumerates all supported expression variants.
///
/// Expressions can represent literals, operations, variable references,
/// assignments, function calls, and conditional expressions.
#[derive(Debug)]
pub enum ExpressionType {
    /// A constant integer literal, e.g. `42`.
    Constant(i32),

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
    /// Creates a new [`Expression`] with the given type and span.
    pub fn new(expr: ExpressionType, span: Span) -> Self {
        Self { expr, span }
    }

    /// Returns a shared reference to the underlying [`ExpressionType`].
    pub fn get_expr_type_ref(&self) -> &ExpressionType {
        &self.expr
    }

    /// Deconstructs the expression into its variant and span.
    pub fn into_parts(self) -> (ExpressionType, Span) {
        (self.expr, self.span)
    }
}

/// Represents all supported unary operators.
///
/// These are operators that operate on a single operand.
#[derive(Debug)]
pub enum UnaryOP {
    /// Arithmetic negation (`-x`).
    Neg,
    /// Logical NOT (`!x`).
    LogicalNot,
    /// Bitwise NOT (`~x`).
    Not,
}

/// Represents all supported binary operators.
///
/// These are operators that combine two operands.
#[derive(Debug)]
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
