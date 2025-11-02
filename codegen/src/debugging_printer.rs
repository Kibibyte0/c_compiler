use shared_context::Identifier;
use shared_context::{StaticVariable, interner::Interner};

use crate::DebuggingPrinter;
use crate::asm;

impl<'a> DebuggingPrinter<'a> {
    pub fn new(interner: &'a Interner<'a>) -> Self {
        Self { interner }
    }

    pub fn print(&self, program: asm::Program) {
        println!("Program");

        let items = program.into_parts();
        for item in items {
            match item {
                asm::TopLevel::F(fun_def) => self.print_function(fun_def),
                asm::TopLevel::S(var_def) => self.print_static_variable(var_def),
            }
        }
    }

    fn format_identifier(&self, identifier: Identifier) -> String {
        format!("{}", self.interner.lookup(identifier.get_symbol()))
    }

    fn format_lickage(external: bool) -> &'static str {
        if external { "external" } else { "internal" }
    }

    fn print_static_variable(&self, var_def: StaticVariable) {
        let (name, external, init) = var_def.into_parts();

        let indent = " ".repeat(2);
        println!(
            "{}StaticVariable(name: {}, linkage:{}, init: {})",
            indent,
            self.format_identifier(name),
            Self::format_lickage(external),
            init
        )
    }

    fn print_function(&self, function: asm::FunctionDef) {
        let (name, external, instructions) = function.into_parts();

        let indent = " ".repeat(2);
        println!(
            "{}FunctionDef(name: {}, linkage: {})",
            indent,
            self.format_identifier(name),
            Self::format_lickage(external)
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
            asm::Instruction::DeallocateStack(size) => {
                println!("{}DeallocateStack({:?})", indent, size);
            }
            asm::Instruction::Ret => {
                println!("{}Ret", indent);
            }
            asm::Instruction::Push(src) => {
                println!("{}Push({:?})", indent, src)
            }
            asm::Instruction::Call(label) => {
                println!("{}Call({})", indent, self.format_identifier(label))
            }
        }
    }
}
