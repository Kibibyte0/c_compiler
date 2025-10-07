use std::fmt;
mod instructions;

pub use instructions::{BinaryOP, Cond, Instruction, Operand, Register, UnaryOP};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

    pub fn into_parts(self) -> FunctionDef {
        self.function
    }

    pub fn get_mut_function(&mut self) -> &mut FunctionDef {
        &mut self.function
    }

    // pub fn print(&self) {
    //     self.print_with_indent(0);
    // }

    // fn print_with_indent(&self, indent: usize) {
    //     println!("{}Program", " ".repeat(indent));
    //     self.function.print_with_indent(indent + 2);
    // }
}

pub struct FunctionDef {
    name: Identifier,
    instructions: Vec<Instruction>,
}

impl FunctionDef {
    pub fn new(name: Identifier, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }

    pub fn into_parts(self) -> (Identifier, Vec<Instruction>) {
        (self.name, self.instructions)
    }

    pub fn get_mut_instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }

    // pub fn print_with_indent(&self, indent: usize) {
    //     println!("{}FunctionDef", " ".repeat(indent));
    //     println!("{}name: {}", " ".repeat(indent + 2), self.name.0);
    //     self.print_instructions(indent + 2);
    // }

    // fn print_instructions(&self, indent: usize) {
    //     println!("{}Instructions:", " ".repeat(indent));
    //     for instruction in &self.instructions {
    //         println!("{}{}", " ".repeat(indent + 2), instruction);
    //     }
    // }
}
