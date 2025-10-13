use crate::DebuggingPrinter;
use crate::asm;

impl DebuggingPrinter {
    pub fn print(program: asm::Program) {
        println!("Program");

        let function = program.into_parts();
        Self::print_function(function);
    }

    fn print_function(function: asm::FunctionDef) {
        let (name, instructions) = function.into_parts();

        let indent = " ".repeat(2);
        println!("{}FunctionDef {}", indent, name.0);

        for instr in instructions {
            Self::print_instruction(instr);
        }
    }

    fn print_instruction(instr: asm::Instruction) {
        let indent = " ".repeat(4); // 4 spaces for indentation

        match instr {
            asm::Instruction::Mov { src, dst } => {
                println!("{}Mov(src: {:?}, dst: {:?})", indent, src, dst);
            }
            asm::Instruction::Unary { op, dst } => {
                println!("{}Unary(op: {:?}, dst: {:?})", indent, op, dst);
            }
            asm::Instruction::Binary { op, src, dst } => {
                println!(
                    "{}Binary(op: {:?}, src: {:?}, dst: {:?})",
                    indent, op, src, dst
                );
            }
            asm::Instruction::Cmp { src, dst } => {
                println!("{}Cmp(src: {:?}, dst: {:?})", indent, src, dst);
            }
            asm::Instruction::Idiv(src) => {
                println!("{}Idiv(src: {:?})", indent, src);
            }
            asm::Instruction::Cdq => {
                println!("{}Cdq", indent);
            }
            asm::Instruction::Jmp(label) => {
                println!("{}Jmp({:?})", indent, label);
            }
            asm::Instruction::JmpCC(cond, label) => {
                println!("{}JmpCC(cond: {:?}, label: {:?})", indent, cond, label);
            }
            asm::Instruction::SetCC(cond, dst) => {
                println!("{}SetCC(cond: {:?}, dst: {:?})", indent, cond, dst);
            }
            asm::Instruction::Label(label) => {
                println!("{}Label({:?})", indent, label);
            }
            asm::Instruction::AllocateStack(size) => {
                println!("{}AllocateStack({:?})", indent, size);
            }
            asm::Instruction::Ret => {
                println!("{}Ret", indent);
            }
        }
    }
}
