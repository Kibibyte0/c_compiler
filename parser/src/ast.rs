pub struct Program<'source> {
    function: FunctionDef<'source>,
}

pub struct FunctionDef<'source> {
    name: &'source str,
    body: Statement,
}

pub enum Statement {
    Return(Expression),
}

pub enum Expression {
    Constant(i32),
    Unary(UnaryOP, Box<Expression>),
}

#[derive(Debug)]
pub enum UnaryOP {
    Negation,
    BitwiseComplement,
}

//
// Program impl
//

impl<'source> Program<'source> {
    pub fn new(function: FunctionDef<'source>) -> Self {
        Self { function }
    }

    pub fn get_function(&self) -> &FunctionDef<'source> {
        &self.function
    }

    pub fn print(&self) {
        self.print_with_indent(0);
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{0:1$}Program", "", indent);
        self.function.print_with_indent(indent + 2);
    }
}

//
// Function impl
//

impl<'source> FunctionDef<'source> {
    pub fn new(name: &'source str, body: Statement) -> Self {
        Self { name, body }
    }

    pub fn get_name(&self) -> &'source str {
        self.name
    }

    pub fn get_body(&self) -> &Statement {
        &self.body
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{0:1$}FunctionDef", "", indent);
        println!("{0:1$}name: {2}", "", indent + 2, self.name);
        self.body.print_with_indent(indent + 2);
    }
}

impl Statement {
    fn print_with_indent(&self, indent: usize) {
        match self {
            Statement::Return(expr) => {
                println!("{0:1$}Return", "", indent);
                expr.print_with_indent(indent + 2);
            }
        }
    }
}

impl Expression {
    fn print_with_indent(&self, indent: usize) {
        match self {
            Expression::Constant(val) => {
                println!("{0:1$}Constant({2})", "", indent, val);
            }
            Expression::Unary(op, expr) => {
                println!("{0:1$}Unary({2:?})", "", indent, op);
                expr.print_with_indent(indent + 2);
            }
        }
    }
}
