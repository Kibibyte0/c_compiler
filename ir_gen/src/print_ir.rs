use shared_context::Identifier;

use crate::{IRgen, tacky};

// a printer for the IR, for debugging

impl<'a, 'b> IRgen<'a, 'b> {
    pub fn print(&self, program: tacky::Program) {
        println!("Program");

        let function = program.into_parts();
        self.print_function(function);
    }

    fn format_identifier(&self, identifier: Identifier) -> String {
        let (symbol, id, _) = identifier.into_parts();
        format!("{}.{}", self.interner.lookup(symbol), id)
    }

    fn format_value(&self, val: tacky::Value) -> String {
        match val {
            tacky::Value::Constant(int) => format!("{}", int),
            tacky::Value::Var(id) => format!("{}", self.format_identifier(id)),
        }
    }

    fn print_function(&self, function: tacky::FunctionDef) {
        let (name, instructions) = function.into_parts();

        let indent = " ".repeat(2);
        println!("{}FunctionDef {}", indent, self.format_identifier(name));

        for instr in instructions {
            self.print_instruction(instr);
        }
    }

    fn print_instruction(&self, instr: tacky::Instruction) {
        let indent = " ".repeat(4);
        match instr {
            tacky::Instruction::Ret(val) => {
                println!("{}Ret({})", indent, self.format_value(val));
            }

            tacky::Instruction::Unary { .. } | tacky::Instruction::Binary { .. } => {
                self.print_operator(instr, indent);
            }

            tacky::Instruction::Label(_)
            | tacky::Instruction::Copy { .. }
            | tacky::Instruction::Jump(_)
            | tacky::Instruction::JumpIfZero(_, _)
            | tacky::Instruction::JumpIfNotZero(_, _) => {
                self.print_control_flow(instr, indent);
            }
        }
    }

    fn print_operator(&self, instr: tacky::Instruction, indent: String) {
        match instr {
            tacky::Instruction::Unary { op, src, dst } => {
                println!(
                    "{}Unary({:?} ,src: {}, dst: {})",
                    indent,
                    op,
                    self.format_value(src),
                    self.format_value(dst),
                )
            }

            tacky::Instruction::Binary {
                op,
                src1,
                src2,
                dst,
            } => {
                println!(
                    "{}Binary({:?} ,src1: {}, src2: {}, dst: {})",
                    indent,
                    op,
                    self.format_value(src1),
                    self.format_value(src2),
                    self.format_value(dst)
                );
            }
            _ => unreachable!("Only Binary and Unary expr will reach print_operator"),
        }
    }

    fn print_control_flow(&self, instr: tacky::Instruction, indent: String) {
        match instr {
            tacky::Instruction::Label(label) => {
                println!("{}Label({})", indent, self.format_identifier(label));
            }

            tacky::Instruction::Copy { src, dst } => {
                println!(
                    "{}Copy(src: {}, dst: {})",
                    indent,
                    self.format_value(src),
                    self.format_value(dst)
                );
            }

            tacky::Instruction::Jump(target) => {
                println!("{}Jump(tar: {})", indent, self.format_identifier(target));
            }

            tacky::Instruction::JumpIfZero(condition, target) => {
                println!(
                    "{}JumpIfZero(cond: {}, tar: {})",
                    indent,
                    self.format_value(condition),
                    self.format_identifier(target)
                );
            }

            tacky::Instruction::JumpIfNotZero(condition, target) => {
                println!(
                    "{}JumpIfNotZero(cond: {}, tar: {})",
                    indent,
                    self.format_value(condition),
                    self.format_identifier(target)
                );
            }

            _ => {
                println!("{}Invalid control-flow instruction", indent);
            }
        }
    }
}
