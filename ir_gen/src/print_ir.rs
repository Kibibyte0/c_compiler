use crate::{IRgen, tacky};

// a printer for the IR, for debugging

impl IRgen {
    pub fn print(program: tacky::Program) {
        println!("Program");

        let function = program.into_parts();
        IRgen::print_function(function);
    }

    fn print_function(function: tacky::FunctionDef) {
        let (name, instructions) = function.into_parts();

        let indent = " ".repeat(2);
        println!("{}FunctionDef {}", indent, name.0);

        for instr in instructions {
            IRgen::print_instruction(instr);
        }
    }

    fn print_instruction(instr: tacky::Instruction) {
        let indent = " ".repeat(4);
        match instr {
            tacky::Instruction::Ret(val) => {
                println!("{}Ret({})", indent, val);
            }

            tacky::Instruction::Unary { .. } | tacky::Instruction::Binary { .. } => {
                IRgen::print_operator(instr, indent);
            }

            tacky::Instruction::Label(_)
            | tacky::Instruction::Copy { .. }
            | tacky::Instruction::Jump(_)
            | tacky::Instruction::JumpIfZero(_, _)
            | tacky::Instruction::JumpIfNotZero(_, _) => {
                IRgen::print_control_flow(instr, indent);
            }
        }
    }

    fn print_operator(instr: tacky::Instruction, indent: String) {
        match instr {
            tacky::Instruction::Unary { op, src, dst } => {
                println!("{}Unary({:?} ,src: {}, dst: {})", indent, op, src, dst);
            }

            tacky::Instruction::Binary {
                op,
                src1,
                src2,
                dst,
            } => {
                println!(
                    "{}Binary({:?} ,src1: {}, src2: {}, dst: {})",
                    indent, op, src1, src2, dst
                );
            }
            _ => unreachable!("Only Binary and Unary expr will reach print_operator"),
        }
    }

    fn print_control_flow(instr: tacky::Instruction, indent: String) {
        match instr {
            tacky::Instruction::Label(label) => {
                println!("{}Label({})", indent, label.0);
            }

            tacky::Instruction::Copy { src, dst } => {
                println!("{}Copy(src: {}, dst: {})", indent, src, dst);
            }

            tacky::Instruction::Jump(target) => {
                println!("{}Jump(tar: {})", indent, target.0);
            }

            tacky::Instruction::JumpIfZero(condition, target) => {
                println!(
                    "{}JumpIfZero(cond: {}, tar: {})",
                    indent, condition, target.0
                );
            }

            tacky::Instruction::JumpIfNotZero(condition, target) => {
                println!(
                    "{}JumpIfNotZero(cond: {}, tar: {})",
                    indent, condition, target.0
                );
            }

            _ => {
                println!("{}Invalid control-flow instruction", indent);
            }
        }
    }
}
