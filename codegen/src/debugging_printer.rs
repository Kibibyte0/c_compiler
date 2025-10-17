use shared_context::Identifier;
use shared_context::interner::Interner;

use crate::DebuggingPrinter;
use crate::asm;

impl<'a> DebuggingPrinter<'a> {
    pub fn new(interner: &'a Interner<'a>) -> Self {
        Self { interner }
    }

    pub fn print(&self, program: asm::Program) {
        println!("Program");

        let function = program.into_parts();
        self.print_function(function);
    }

    fn format_identifier(&self, identifier: Identifier) -> String {
        format!("{}", self.interner.lookup(identifier.get_symbol()))
    }

    fn print_function(&self, function: asm::FunctionDef) {
        let (name, instructions) = function.into_parts();

        let indent = " ".repeat(2);
        println!(
            "{}FunctionDef {}",
            indent,
            self.interner.lookup(name.get_symbol())
        );

        for instr in instructions {
            self.print_instruction(instr);
        }
    }

    fn print_instruction(&self, instr: asm::Instruction) {
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
                println!("{}Jmp({:?})", indent, self.format_identifier(label));
            }
            asm::Instruction::JmpCC(cond, label) => {
                println!(
                    "{}JmpCC(cond: {:?}, label: {:?})",
                    indent,
                    cond,
                    self.format_identifier(label)
                );
            }
            asm::Instruction::SetCC(cond, dst) => {
                println!("{}SetCC(cond: {:?}, dst: {:?})", indent, cond, dst);
            }
            asm::Instruction::Label(label) => {
                println!("{}Label({:?})", indent, self.format_identifier(label));
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
