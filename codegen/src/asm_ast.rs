use std::fmt;

pub enum Instruction {
    Mov { dst: Operand, src: Operand },
    Unary { operator: UnaryOP, operand: Operand },
    AllocateStack(usize),
    Ret,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Mov { dst, src } => write!(f, "Mov(dst: {}, src: {})", dst, src),
            Instruction::Unary { operator, operand } => {
                write!(f, "Unary(operator: {}, operand: {})", operator, operand)
            }
            Instruction::AllocateStack(size) => write!(f, "AllocateStack({})", size),
            Instruction::Ret => write!(f, "Ret"),
        }
    }
}

pub enum Operand {
    Reg(Register),
    Pseudo(Identifier),
    Stack(i32),
    Immediate(i32),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Reg(reg) => write!(f, "{}", reg),
            Operand::Pseudo(id) => write!(f, "Pseudo({})", id.0),
            Operand::Stack(offset) => write!(f, "Stack({})", offset),
            Operand::Immediate(val) => write!(f, "Immediate({})", val),
        }
    }
}

pub enum Register {
    AX,
    R10,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::AX => write!(f, "AX"),
            Register::R10 => write!(f, "R10"),
        }
    }
}

pub enum UnaryOP {
    Not,
    Neg,
}

impl fmt::Display for UnaryOP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOP::Not => write!(f, "Not"),
            UnaryOP::Neg => write!(f, "Neg"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Program {
    function: FunctionDef,
}

impl Program {
    pub fn new(function: FunctionDef) -> Self {
        Self { function }
    }

    pub fn get_function(&mut self) -> &mut FunctionDef {
        &mut self.function
    }

    pub fn print(&self) {
        self.print_with_indent(0);
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{}Program", " ".repeat(indent));
        self.function.print_with_indent(indent + 2);
    }
}

pub struct FunctionDef {
    name: Identifier,
    instructions: Vec<Instruction>,
}

impl FunctionDef {
    pub fn new(name: Identifier, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }

    pub fn get_instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }

    pub fn print_with_indent(&self, indent: usize) {
        println!("{}FunctionDef", " ".repeat(indent));
        println!("{}name: {}", " ".repeat(indent + 2), self.name.0);
        self.print_instructions(indent + 2);
    }

    fn print_instructions(&self, indent: usize) {
        println!("{}Instructions:", " ".repeat(indent));
        for instruction in &self.instructions {
            println!("{}{}", " ".repeat(indent + 2), instruction);
        }
    }
}
