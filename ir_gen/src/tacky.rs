mod instructions;
pub use instructions::{BinaryOP, Instruction, UnaryOP, Value};

pub struct Program {
    function: FunctionDef,
}

pub struct FunctionDef {
    name: Identifier,
    instructions: Vec<Instruction>,
}

pub struct Identifier(pub String);

impl Program {
    pub fn new(function: FunctionDef) -> Self {
        Self { function }
    }

    pub fn into_parts(self) -> FunctionDef {
        self.function
    }

    pub fn print(&self) {
        self.print_with_indent(0);
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{}Program", " ".repeat(indent));
        self.function.print_with_indent(indent + 2);
    }
}

impl FunctionDef {
    pub fn new(name: Identifier, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }

    pub fn into_parts(self) -> (Identifier, Vec<Instruction>) {
        (self.name, self.instructions)
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{}FunctionDef", " ".repeat(indent));
        println!("{}name: {}", " ".repeat(indent + 2), self.name.0);
        self.print_instructions(indent + 2);
    }

    fn print_instructions(&self, indent: usize) {
        println!("{}Instructions:", " ".repeat(indent));
        for instruction in &self.instructions {
            instruction.print(indent + 2);
        }
    }
}
