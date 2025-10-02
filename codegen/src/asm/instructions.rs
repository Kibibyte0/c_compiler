use crate::asm::Identifier;
use std::fmt;

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
    Idiv(Operand),
    Cdq,
    AllocateStack(i32),
    Ret,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Mov { src, dst } => write!(f, "Mov(src: {}, dst: {})", src, dst),
            Instruction::Unary { op, dst } => {
                write!(f, "Unary(op: {:?}, dst: {})", op, dst)
            }
            Instruction::AllocateStack(size) => write!(f, "AllocateStack({})", size),
            Instruction::Ret => write!(f, "Ret"),
            Instruction::Binary { op, src, dst } => {
                write!(f, "Binary(op: {:?}, src: {}, dst: {})", op, src, dst)
            }
            Instruction::Idiv(operand) => write!(f, "Idv({})", operand),
            Instruction::Cdq => write!(f, "Cdq"),
        }
    }
}

#[derive(Clone)]
pub enum Operand {
    Reg(Register),
    Pseudo(Identifier),
    Stack(i32),
    Immediate(i32),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Reg(reg) => write!(f, "{:?}", reg),
            Operand::Pseudo(id) => write!(f, "Pseudo({})", id.0),
            Operand::Stack(offset) => write!(f, "Stack({})", offset),
            Operand::Immediate(val) => write!(f, "Immediate({})", val),
        }
    }
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
