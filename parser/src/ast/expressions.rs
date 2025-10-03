pub enum Expression {
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
