// Intermediate Representation (IR) for the compiler.
//
// This module defines a simple, instruction-based intermediate representation
// called TAC (Three Adress Code) used by the compiler backend. It represents the lowered form of the source
// program after semantic analysis, suitable for optimization and code generation.

mod instructions;
pub use instructions::{BinaryOP, Instruction, UnaryOP, Value};
use shared_context::{Identifier, StaticVariable};

/// Represents a compiled program at the IR level.
///
/// A program is made out of a vector of function defintions
/// each definition repressent a block of code in the IR
pub struct Program {
    items: Vec<TopLevel>,
}

impl Program {
    /// Creates a new Program from a vector of FunctionDef.
    pub fn new(items: Vec<TopLevel>) -> Self {
        Self { items }
    }

    /// Consumes the Program and returns its inner vector of FunctionDef.
    pub fn into_parts(self) -> Vec<TopLevel> {
        self.items
    }
}

/// represent a global object in the IR
///
/// can be a static variable definition or a function definition
pub enum TopLevel {
    S(StaticVariable),
    F(FunctionDef),
}

/// A single function definition in the intermediate representation.
///
/// Each FunctionDef contains the function's name, its parameters,
/// and the list of IR Instructions that make up its body.
pub struct FunctionDef {
    name: Identifier,
    external: bool,
    params: Vec<Identifier>,
    instructions: Vec<Instruction>,
}

impl FunctionDef {
    /// Creates a new FunctionDef.
    pub fn new(
        name: Identifier,
        external: bool,
        params: Vec<Identifier>,
        instructions: Vec<Instruction>,
    ) -> Self {
        Self {
            name,
            external,
            params,
            instructions,
        }
    }

    /// Consumes the FunctionDef and returns its components.
    ///
    /// Returns a tuple of `(name, params, instructions)`.
    pub fn into_parts(self) -> (Identifier, bool, Vec<Identifier>, Vec<Instruction>) {
        (self.name, self.external, self.params, self.instructions)
    }
}
