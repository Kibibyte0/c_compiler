#[derive(Debug)]
pub struct AsmProgram<'source> {
    pub function: AsmFunctionDefinition<'source>,
}

impl<'source> AsmProgram<'source> {
    pub fn dump(&self, indent: usize) {
        let prefix = " ".repeat(indent);
        println!("{}AsmProgram:", prefix);
        self.function.dump(indent + 2);
    }
}

#[derive(Debug)]
pub struct AsmFunctionDefinition<'source> {
    pub name: &'source str,
    pub instructions: Vec<Instruction>,
}

impl<'source> AsmFunctionDefinition<'source> {
    pub fn dump(&self, indent: usize) {
        let prefix = " ".repeat(indent);
        println!("{}Function: {}", prefix, self.name);
        println!("{}Instructions:", prefix);
        for instr in &self.instructions {
            instr.dump(indent + 2);
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand),
    Ret,
}

impl Instruction {
    pub fn dump(&self, indent: usize) {
        let prefix = " ".repeat(indent);
        match self {
            Instruction::Mov(dst, src) => {
                println!("{}Mov:", prefix);
                println!("{}  Destination:", prefix);
                dst.dump(indent + 4);
                println!("{}  Source:", prefix);
                src.dump(indent + 4);
            }
            Instruction::Ret => {
                println!("{}Ret", prefix);
            }
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Register,
    Immediate(i32),
}

impl Operand {
    pub fn dump(&self, indent: usize) {
        let prefix = " ".repeat(indent);
        match self {
            Operand::Register => println!("{}Register", prefix),
            Operand::Immediate(val) => println!("{}Immediate({})", prefix, val),
        }
    }
}
