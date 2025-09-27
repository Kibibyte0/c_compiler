use std::fmt;

pub enum Instruction {
    Unary { op: UnaryOP, dst: Value, src: Value },
    Ret(Value),
}

pub enum UnaryOP {
    BitwiseComplement,
    Negation,
}

impl fmt::Display for UnaryOP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOP::BitwiseComplement => write!(f, "BitwiseComplement"),
            UnaryOP::Negation => write!(f, "Negation")
        }
    }
}

pub enum Value {
    Constant(i32),
    Var(String),
}


impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Constant(val) => write!(f, "{}", val),
            Value::Var(name) => write!(f, "{}", name),
        }
    }
}

pub struct Program<'source> {
    function: FunctionDef<'source>,
}

pub struct FunctionDef<'source> {
    name: &'source str,
    instructions: Vec<Instruction>,
}

impl<'source> Program<'source> {
    pub fn new(function: FunctionDef<'source>) -> Self {
        Self { function }
    }

    pub fn print(&self) {
        self.print_with_indent(0);
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{}Program", " ".repeat(indent));
        self.function.print_with_indent(indent + 2);
    }
}

impl<'source> FunctionDef<'source> {
    pub fn new(name: &'source str, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{}FunctionDef", " ".repeat(indent));
        println!("{}name: {}", " ".repeat(indent + 2), self.name);
        self.print_instructions(indent + 2);
    }

    fn print_instructions(&self, indent: usize) {
        println!("{}Instructions:", " ".repeat(indent));
        for instruction in &self.instructions {
            match instruction {
                Instruction::Ret(val) => {
                    println!("{}Ret({})", " ".repeat(indent + 2), val);
                }
                Instruction::Unary { op, dst, src } => {
                    println!("{}{}(dst: {}, src: {})", " ".repeat(indent + 2), op, dst, src);
                }
            }
        }
    }
}
