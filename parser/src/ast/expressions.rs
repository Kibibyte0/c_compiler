pub enum Expression {
    Constant(i32),
    Unary(UnaryOP, Box<Expression>),
    Binary {
        op: BinaryOP,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum UnaryOP {
    Negation,
    BitwiseComplement,
}

#[derive(Debug)]
pub enum BinaryOP {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Mod,
}

impl Expression {
    pub fn print_with_indent(&self, indent: usize) {
        match self {
            Expression::Constant(val) => {
                println!("{}Constant({})", " ".repeat(indent), val);
            }
            Expression::Unary(op, expr) => {
                println!("{}Unary({:?})", " ".repeat(indent), op);
                expr.print_with_indent(indent + 2);
            }
            Expression::Binary { op, left, right } => {
                println!("{}Binary({:?})", " ".repeat(indent), op);
                left.print_with_indent(indent + 2);
                right.print_with_indent(indent + 2);
            }
        }
    }
}
