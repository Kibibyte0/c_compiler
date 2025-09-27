// use std::{fs::File, path::PathBuf};
// use std::fmt::{self, Write as FmtWrite};
// use std::io::Write as IoWrite;
// use codegen::asm_ast::Operand;

// use codegen::asm_ast::{AsmFunctionDefinition, AsmProgram, Instruction};

// pub struct Emitter<'source> {
//     program: AsmProgram<'source>,
//     output_code: String,
//     file_path: PathBuf,
// }

// impl<'source> Emitter<'source> {
//     pub fn new(program: AsmProgram<'source>, file_path: PathBuf) -> Self {
//         Self {
//             program,
//             output_code: String::new(),
//             file_path,
//         }
//     }

//     // write to the output string, and panic if the operation fails
//     fn emit(buffer: &mut impl FmtWrite, s: impl fmt::Display) {
//         writeln!(buffer, "{}", s)
//             .expect("error: failed to write output code");
//     }

//     pub fn emit_asm(&mut self) {
//         self.write_program();
//         let mut file = File::create(&self.file_path)
//             .expect("error: failed to create the output file");
//         file.write_all(self.output_code.as_bytes()).unwrap();
//     }

//     fn write_program(&mut self) {
//         Emitter::write_function_def(&mut self.output_code, &self.program.function);
//         Emitter::emit(
//             &mut self.output_code,
//             format!("   .section .note.GNU-stack,\"\",@progbits")
//         );
//     }

//     fn write_function_def(output_code: &mut String, function: &AsmFunctionDefinition) {
//         Emitter::emit(output_code, format!("   .globl {}", function.name));
//         Emitter::emit(output_code, format!("{}:", function.name));
//         Emitter::write_instructions(output_code, &function.instructions);
//     }

//     fn write_instructions(output_code: &mut String, instructions: &Vec<Instruction>) {
//         for instruction in instructions {
//             Emitter::write_instruction(output_code, instruction);
//         }
//     }

//     fn write_instruction(output_code: &mut String, instruction: &Instruction) {
//         match instruction {
//             Instruction::Mov(src, dst) => Emitter::write_mov(output_code, src, dst),
//             Instruction::Ret => Emitter::write_ret(output_code),
//         }
//     }

//     fn write_mov(output_code: &mut String, src: &Operand, dst: &Operand) {
//         Emitter::emit(output_code, format!("    movl    {}, {}",
//             Emitter::fetch_operand(src),
//             Emitter::fetch_operand(dst),
//         ));
//     }

//     fn fetch_operand(op: &Operand) -> String {
//         match op {
//             Operand::Immediate(int) => format!("${}", int),
//             Operand::Register => "%eax".to_string(),
//         }
//     }

//     fn write_ret(output_code: &mut String) {
//         Emitter::emit(output_code, format!("    ret"));
//     }
// }
