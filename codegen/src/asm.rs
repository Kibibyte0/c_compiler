// Re-export Identifier so it can be used directly by users of this module.
use shared_context::{Identifier, OperandSize, StaticVariable};

/// Represents an entire assembly-level program.
///
/// A Program consists of a collection of functions, each represented
/// by a FunctionDef. This is the final stage before emitting actual
/// assembly text or binary output.
pub struct Program {
    items: Vec<TopLevel>,
}

impl Program {
    /// Creates a new Program from a list of function definitions.
    pub fn new(items: Vec<TopLevel>) -> Self {
        Self { items }
    }

    /// Consumes the Program and returns the list of contained functions.
    pub fn into_parts(self) -> Vec<TopLevel> {
        self.items
    }

    /// Returns a mutable reference to the underlying vector of functions.
    pub fn get_mut_functions(&mut self) -> &mut Vec<TopLevel> {
        &mut self.items
    }
}

/// represent a global object in the assembly
///
/// can be a static variable definition or a function definition
pub enum TopLevel {
    S(StaticVariable),
    F(FunctionDef),
}

/// Represents a single function in the generated assembly program.
pub struct FunctionDef {
    name: Identifier,
    external: bool,
    instructions: Vec<Instruction>,
}

impl FunctionDef {
    /// Creates a new function definition.
    pub fn new(name: Identifier, external: bool, instructions: Vec<Instruction>) -> Self {
        Self {
            name,
            external,
            instructions,
        }
    }

    /// Consumes the `FunctionDef` and returns its name and instructions.
    pub fn into_parts(self) -> (Identifier, bool, Vec<Instruction>) {
        (self.name, self.external, self.instructions)
    }

    /// Returns a mutable reference to the function’s instruction list.
    pub fn get_mut_instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }
}

/// Represents a single assembly instruction in the program.
///
/// Each variant corresponds to a low-level x86-like operation,
#[derive(Clone, Copy)]
pub enum Instruction {
    /// Move data from `src` to `dst`
    Mov {
        size: OperandSize,
        src: Operand,
        dst: Operand,
    },

    // used to sign extend a longword to quadword
    Movsx {
        src: Operand,
        dst: Operand,
    },

    /// Unary operation (e.g., `neg`, `not`)
    Unary {
        op: UnaryOP,
        size: OperandSize,
        dst: Operand,
    },

    /// Binary operation (e.g., `add`, `sub`, `mul`)
    Binary {
        op: BinaryOP,
        size: OperandSize,
        src: Operand,
        dst: Operand,
    },

    /// Compare two operands (sets flags for conditional jumps)
    Cmp {
        size: OperandSize,
        src: Operand,
        dst: Operand,
    },

    /// Signed integer division
    Idiv(OperandSize, Operand),

    /// Sign-extend `EAX` into `EDX:EAX` before division (`cdq`)
    Cdq(OperandSize),

    /// Unconditional jump to label
    Jmp(Identifier),

    /// Conditional jump (based on flags)
    JmpCC(Cond, Identifier),

    /// Set destination byte based on condition flags
    SetCC(Cond, Operand),

    /// Marks a label within the instruction stream
    Label(Identifier),

    /// Push an operand onto the stack
    Push(Operand),

    /// Call a function
    Call(Identifier),

    /// Return from function
    Ret,
}

/// Represents possible jump or comparison conditions (for `JmpCC` / `SetCC`).
#[derive(Clone, Debug, Copy)]
pub enum Cond {
    E,  // Equal
    NE, // Not equal
    G,  // Greater
    GE, // Greater or equal
    L,  // Less
    LE, // Less or equal
}

/// Represents the types of operands that can appear in an instruction.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Operand {
    Reg(Register),      // Physical CPU register
    Pseudo(Identifier), // Compiler-generated pseudo-register (before allocation)
    Stack(i64),         // Stack slot (offset from base pointer)
    Immediate(i64),     // Immediate constant value
    Data(Identifier),   // For RIP relative addressing
}

/// Enumerates the general-purpose registers available for use.
///
/// These correspond to x86-64 registers typically used for temporaries or arguments.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Register {
    AX,
    CX,
    DX,
    DI,
    SI,
    R8,
    R9,
    R10,
    R11,
    SP,
}

/// Binary arithmetic operations supported in the assembly layer.
#[derive(Clone, Debug, Copy)]
pub enum BinaryOP {
    Add, // Addition
    Sub, // Subtraction
    Mul, // Multiplication
}

/// Unary operations supported in the assembly layer.
#[derive(Clone, Debug, Copy)]
pub enum UnaryOP {
    Not, // Bitwise NOT
    Neg, // Arithmetic negation
}
