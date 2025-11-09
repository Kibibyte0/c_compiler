// Intermediate Representation (IR) for the compiler.
//
// This module defines a simple, instruction-based intermediate representation
// called TAC (Three Adress Code) used by the compiler backend. It represents the lowered form of the source
// program after semantic analysis, suitable for optimization and code generation.

use shared_context::{Const, Identifier, StaticVariable};

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

/// A single instruction in the intermediate representation.
pub enum Instruction {
    /// A binary operation: `dst = src1 op src2`.
    Binary {
        op: BinaryOP,
        src1: Value,
        src2: Value,
        dst: Value,
    },

    /// A unary operation: `dst = op src`.
    Unary { op: UnaryOP, src: Value, dst: Value },

    /// Copies a value from one variable to another: `dst = src`.
    Copy { src: Value, dst: Value },

    /// Function call: `dst = name(args...)`.
    FunCall {
        name: Identifier,
        args: Vec<Value>,
        dst: Value,
    },

    /// Unconditional jump to the given label.
    Jump(Identifier),

    /// Conditional jump if the given value is zero.
    JumpIfZero(Value, Identifier),

    /// Conditional jump if the given value is non-zero.
    JumpIfNotZero(Value, Identifier),

    /// A label marking a jump target.
    Label(Identifier),

    /// used to cast an int to long
    SignExtend { src: Value, dst: Value },

    /// used to cast a long to int
    Truncate { src: Value, dst: Value },

    /// Function return with the given value.
    Ret(Value),
}

/// Represents a source or destination operand in an instruction.
///
/// A Value can either be a constant integer or a variable (identified by name).
#[derive(Clone, Copy)]
pub enum Value {
    /// Immediate integer constant.
    Constant(Const),

    /// Named variable, represented by an Identifier.
    Var(Identifier),
}

/// Supported binary operations in the IR.
///
/// These include arithmetic, logical, and comparison operators.
/// The semantics are defined according to the source language’s specification.
#[derive(Debug)]
pub enum BinaryOP {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Logical
    LogicalAnd,
    LogicalOr,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEq,
    GreaterThanOrEq,
}

/// Supported unary operations in the IR.s
///
/// These represent single-operand transformations such as negation
/// or logical inversion.
#[derive(Debug)]
pub enum UnaryOP {
    /// Bitwise or logical NOT.
    Not,

    /// Arithmetic negation.
    Neg,

    /// Logical NOT (e.g., `!x`).
    LogicalNot,
}
