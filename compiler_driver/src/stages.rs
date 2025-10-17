use crate::files::*;
use codegen::DebuggingPrinter;
use codegen::{self, AsmGen, InstructionFix, RegisterAllocation};
use emitter::Emitter;
use parser::{self, parse};
use semantic_analysis::analize;
use shared_context::Bump;
use shared_context::CompilerContext;
use std::{error::Error, fs};

// lex the program then exit without starting the other stages
pub fn lexer_stage(file_path: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let mut lexer = lexer::Lexer::new(&input_string);

    while let Some(tok) = lexer.next() {
        println!(
            "matched string: {}, token type: {:?}",
            tok.get_lexeme(),
            tok.get_token()
        );
    }

    Ok(())
}

pub fn parser_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);
    let program_ast = parse(lexer, &mut ctx)?;
    let (analized_program, _) = analize(&ctx, program_ast)?;

    parser::print_ast::DebuggingPrinter::new(&ctx.interner).print(analized_program);

    Ok(())
}

pub fn tacky_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);
    let program_ast = parse(lexer, &mut ctx)?;
    let (analized_program, counter) = analize(&ctx, program_ast)?;

    let mut ir_gen = ir_gen::IRgen::new(counter, &mut ctx.interner);
    let program_tacky = ir_gen.gen_tacky(analized_program);
    ir_gen.print(program_tacky);

    Ok(())
}

pub fn codegen_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);
    let program_ast = parse(lexer, &mut ctx)?;
    let (analized_program, counter) = analize(&ctx, program_ast)?;

    let mut ir_gen = ir_gen::IRgen::new(counter, &mut ctx.interner);
    let program_tacky = ir_gen.gen_tacky(analized_program);

    let mut program_asm = AsmGen::gen_asm(program_tacky);

    let mut codegen = RegisterAllocation::new();
    codegen.allocate_registers(&mut program_asm);

    InstructionFix::fix_instructions(&mut program_asm);
    let asm_printer = DebuggingPrinter::new(&ctx.interner);
    asm_printer.print(program_asm);

    Ok(())
}

pub fn emit_assembly(file_path: &str, file_name: &str) -> Result<String, Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);
    let program_ast = parse(lexer, &mut ctx)?;
    let (analized_program, counter) = analize(&ctx, program_ast)?;

    let mut ir_gen = ir_gen::IRgen::new(counter, &mut ctx.interner);
    let program_tacky = ir_gen.gen_tacky(analized_program);

    let mut program_asm = AsmGen::gen_asm(program_tacky);

    let mut codegen = RegisterAllocation::new();
    codegen.allocate_registers(&mut program_asm);

    InstructionFix::fix_instructions(&mut program_asm);

    let asm_file_name = format!("{}.s", remove_file_extension(file_name));
    let output_path = set_file_name(file_path, &asm_file_name);
    Emitter::new(12, 16, 2, &ctx.interner).write_program(program_asm, &output_path)?;

    Ok(output_path)
}
