// Re-export Identifier so it can be used directly by users of this module.
use shared_context::Identifier;

/// Represents an entire assembly-level program.
///
/// A Program consists of a collection of functions, each represented
/// by a FunctionDef. This is the final stage before emitting actual
/// assembly text or binary output.
pub struct Program {
    functions: Vec<FunctionDef>,
}

impl Program {
    /// Creates a new Program from a list of function definitions.
    pub fn new(functions: Vec<FunctionDef>) -> Self {
        Self { functions }
    }

    /// Consumes the Program and returns the list of contained functions.
    pub fn into_parts(self) -> Vec<FunctionDef> {
        self.functions
    }

    /// Returns a mutable reference to the underlying vector of functions.
    pub fn get_mut_functions(&mut self) -> &mut Vec<FunctionDef> {
        &mut self.functions
    }
}

/// Represents a single function in the generated assembly program.
///
/// Each function has:
/// - a `name` (identifier)
/// - a list of assembly `instructions`
pub struct FunctionDef {
    name: Identifier,
    instructions: Vec<Instruction>,
}

impl FunctionDef {
    /// Creates a new function definition.
    pub fn new(name: Identifier, instructions: Vec<Instruction>) -> Self {
        Self { name, instructions }
    }

    /// Consumes the `FunctionDef` and returns its name and instructions.
    pub fn into_parts(self) -> (Identifier, Vec<Instruction>) {
        (self.name, self.instructions)
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
    Mov { src: Operand, dst: Operand },

    /// Unary operation (e.g., `neg`, `not`)
    Unary { op: UnaryOP, dst: Operand },

    /// Binary operation (e.g., `add`, `sub`, `mul`)
    Binary {
        op: BinaryOP,
        src: Operand,
        dst: Operand,
    },

    /// Compare two operands (sets flags for conditional jumps)
    Cmp { src: Operand, dst: Operand },

    /// Signed integer division
    Idiv(Operand),

    /// Sign-extend `EAX` into `EDX:EAX` before division (`cdq`)
    Cdq,

    /// Unconditional jump to label
    Jmp(Identifier),

    /// Conditional jump (based on flags)
    JmpCC(Cond, Identifier),

    /// Set destination byte based on condition flags
    SetCC(Cond, Operand),

    /// Marks a label within the instruction stream
    Label(Identifier),

    /// Reserve stack space (e.g., `sub rsp, n`)
    AllocateStack(i32),

    /// Free stack space (e.g., `add rsp, n`)
    DeallocateStack(i32),

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
#[derive(Clone, Debug, Copy)]
pub enum Operand {
    Reg(Register),      // Physical CPU register
    Pseudo(Identifier), // Compiler-generated pseudo-register (before allocation)
    Stack(i32),         // Stack slot (offset from base pointer)
    Immediate(i32),     // Immediate constant value
}

/// Enumerates the general-purpose registers available for use.
///
/// These correspond to x86-64 registers typically used for temporaries or arguments.
#[derive(Clone, Debug, Copy)]
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
