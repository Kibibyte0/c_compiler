use crate::asm::Identifier;
// use std::fmt;

#[derive(Clone)]
pub enum Instruction {
    Mov {
        src: Operand,
        dst: Operand,
    },
    Unary {
        op: UnaryOP,
        dst: Operand,
    },
    Binary {
        op: BinaryOP,
        src: Operand,
        dst: Operand,
    },
    Cmp(Operand, Operand),
    Idiv(Operand),
    Cdq,
    Jmp(Identifier),
    JmpCC(Cond, Identifier),
    SetCC(Cond, Operand),
    Label(Identifier),
    AllocateStack(i32),
    Ret,
}

#[derive(Clone, Debug)]
pub enum Cond {
    E,
    NE,
    G,
    GE,
    L,
    LE,
}

#[derive(Clone, Debug)]
pub enum Operand {
    Reg(Register),
    Pseudo(Identifier),
    Stack(i32),
    Immediate(i32),
}

#[derive(Clone, Debug)]
pub enum Register {
    AX,
    DX,
    R10,
    R11,
}

// assembly binary operator
#[derive(Clone, Debug)]
pub enum BinaryOP {
    Add,
    Sub,
    Mul,
}

// assmebly unary operator
#[derive(Clone, Debug)]
pub enum UnaryOP {
    Not,
    Neg,
}
