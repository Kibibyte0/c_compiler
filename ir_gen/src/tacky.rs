mod instructions;
pub use instructions::{BinaryOP, Instruction, UnaryOP, Value};

pub struct Program {
    function: FunctionDef,
}

pub struct FunctionDef {
    name: Identifier,
    instructions: Vec<Instruction>,
}

#[derive(Clone)]
pub struct Identifier(pub String);

impl Program {
    pub fn new(function: FunctionDef) -> Self {
        Self { function }
    }

    pub fn into_parts(self) -> FunctionDef {
        self.function
    }
}

impl FunctionDef {
    pub fn new(name: Identifier, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }

    pub fn into_parts(self) -> (Identifier, Vec<Instruction>) {
        (self.name, self.instructions)
    }
}
