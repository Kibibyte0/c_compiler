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

impl Expression {
    pub fn print_with_indent(&self, indent: usize) {
        match self {
            Expression::Constant(val) => {
                println!("{}Constant({})", " ".repeat(indent), val);
            }
            Expression::Unary { operator, operand } => {
                println!("{}Unary({:?})", " ".repeat(indent), operator);
                operand.print_with_indent(indent + 2);
            }
            Expression::Binary {
                operator,
                operand1,
                operand2,
            } => {
                println!("{}Binary({:?})", " ".repeat(indent), operator);
                operand1.print_with_indent(indent + 2);
                operand2.print_with_indent(indent + 2);
            }
        }
    }
}
