// Intermediate Representation (IR) for the compiler.
//
// This module defines a simple, instruction-based intermediate representation
// called TAC (Three Adress Code) used by the compiler backend. It represents the lowered form of the source
// program after semantic analysis, suitable for optimization and code generation.

mod instructions;
pub use instructions::{BinaryOP, Instruction, UnaryOP, Value};
use shared_context::Identifier;

/// Represents a compiled program at the IR level.
///
/// A program is made out of a vector of function defintions
/// each definition repressent a block of code in the IR
pub struct Program {
    functions: Vec<FunctionDef>,
}

/// A single function definition in the intermediate representation.
///
/// Each FunctionDef contains the function's name, its parameters,
/// and the list of IR Instructions that make up its body.
pub struct FunctionDef {
    name: Identifier,
    params: Vec<Identifier>,
    instructions: Vec<Instruction>,
}

impl Program {
    /// Creates a new Program from a vector of FunctionDef.
    pub fn new(functions: Vec<FunctionDef>) -> Self {
        Self { functions }
    }

    /// Consumes the Program and returns its inner vector of FunctionDef.
    pub fn into_parts(self) -> Vec<FunctionDef> {
        self.functions
    }
}

impl FunctionDef {
    /// Creates a new FunctionDef.
    pub fn new(name: Identifier, params: Vec<Identifier>, instructions: Vec<Instruction>) -> Self {
        Self {
            name,
            params,
            instructions,
        }
    }

    /// Consumes the FunctionDef and returns its components.
    ///
    /// Returns a tuple of `(name, params, instructions)`.
    pub fn into_parts(self) -> (Identifier, Vec<Identifier>, Vec<Instruction>) {
        (self.name, self.params, self.instructions)
    }
}
