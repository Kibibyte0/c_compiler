use shared_context::Identifier;

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

    /// Function return with the given value.
    Ret(Value),
}

/// Represents a source or destination operand in an instruction.
///
/// A Value can either be a constant integer or a variable (identified by name).
#[derive(Clone, Copy)]
pub enum Value {
    /// Immediate integer constant.
    Constant(i32),

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

/// Supported unary operations in the IR.
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
