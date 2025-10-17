use shared_context::Identifier;

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
    Copy {
        src: Value,
        dst: Value,
    },
    Jump(Identifier),
    JumpIfZero(Value, Identifier),
    JumpIfNotZero(Value, Identifier),
    Label(Identifier),
    Ret(Value),
}

#[derive(Clone, Copy)]
pub enum Value {
    Constant(i32),
    Var(Identifier),
}

// impl fmt::Display for Value {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Value::Constant(val) => write!(f, "{}", val),
//             Value::Var(name) => write!(f, "{}", name.0),
//         }
//     }
// }

// tacky binary operations
#[derive(Debug)]
pub enum BinaryOP {
    // arithmatic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // logical
    LogicalAnd,
    LogicalOr,
    // comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEq,
    GreaterThanOrEq,
}

// tacky unary operations
#[derive(Debug)]
pub enum UnaryOP {
    Not,
    Neg,
    LogicalNot,
}
