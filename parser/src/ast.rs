pub struct Identifier<'source>(pub &'source str);

pub struct Program<'source> {
    pub function: FunctionDefinition<'source>,
}

pub struct FunctionDefinition<'source> {
    pub name: Identifier<'source>,
    pub body: Statement,
}

pub enum Statement {
    Return(Expression),
}

pub enum Expression {
    Constant(i32),
}


// implement pretty printing for the AST
impl<'source> Program<'source> {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        println!("{pad}Program(");
        self.function.dump(indent + 4);
        println!("{pad})");
    }
}

impl<'source> FunctionDefinition<'source> {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        println!("{pad}Function(");
        println!("{pad}    name=\"{}\",", self.name.0);
        println!("{pad}    body=");
        self.body.dump(indent + 8);
        println!("{pad})");
    }
}

impl Statement {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        match self {
            Statement::Return(expr) => {
                println!("{pad}Return(");
                expr.dump(indent + 4);
                println!("{pad})");
            }
        }
    }
}

impl Expression {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        match self {
            Expression::Constant(val) => {
                println!("{pad}Constant({})", val);
            }
        }
    }
}

