use crate::tacky::Identifier;
use std::fmt;

pub enum Instruction {
    Binary {
        op: BinaryOP,
        src1: Value,
        src2: Value,
        dst: Value,
    },
    Unary {
        op: UnaryOP,
        src: Value,
        dst: Value,
    },
    Ret(Value),
}

pub enum Value {
    Constant(i32),
    Var(Identifier),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Constant(val) => write!(f, "{}", val),
            Value::Var(name) => write!(f, "{}", name.0),
        }
    }
}

// tacky binary operations
#[derive(Debug)]
pub enum BinaryOP {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

// tacky unary operations
#[derive(Debug)]
pub enum UnaryOP {
    Not,
    Neg,
}

// pretty print the instruction
impl Instruction {
    pub fn print(&self, indent: usize) {
        match self {
            Instruction::Ret(val) => {
                println!("{}Ret({})", " ".repeat(indent), val);
            }

            Instruction::Unary { op, src, dst } => {
                println!("{}{:?}(src: {}, dst: {})", " ".repeat(indent), op, src, dst);
            }

            Instruction::Binary {
                op,
                src1,
                src2,
                dst,
            } => {
                println!(
                    "{}{:?}(src1: {}, src2: {}, dst: {})",
                    " ".repeat(indent),
                    op,
                    src1,
                    src2,
                    dst
                );
            }
        }
    }
}
