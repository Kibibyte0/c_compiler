use crate::files::*;
use codegen::{DebuggingPrinter, codegen};
use emitter::Emitter;
use ir_gen::{lower_to_tacky, print_ir};
use parser::parse;
use semantic_analysis::analize;
use shared_context::{Bump, CompilerContext, asm_symbol_table::AsmSymbolTable};
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
    let program_ast = parse(
        lexer,
        &mut ctx.ty_interner,
        &mut ctx.sy_interner,
        &ctx.source_map,
    )?;

    parser::print_ast::DebugTreePrinter::new(&ctx.ty_interner, &ctx.sy_interner).print(program_ast);

    Ok(())
}

pub fn validate_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);
    let program_ast = parse(
        lexer,
        &mut ctx.ty_interner,
        &mut ctx.sy_interner,
        &ctx.source_map,
    )?;
    let (analized_program, _) = analize(
        &ctx.ty_interner,
        &mut ctx.sy_interner,
        &mut ctx.symbol_table,
        &ctx.source_map,
        program_ast,
    )?;

    parser::print_ast::DebugTreePrinter::new(&ctx.ty_interner, &ctx.sy_interner)
        .print(analized_program);

    Ok(())
}

pub fn tacky_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);
    let program_ast = parse(
        lexer,
        &mut ctx.ty_interner,
        &mut ctx.sy_interner,
        &ctx.source_map,
    )?;
    let (analized_program, counter) = analize(
        &ctx.ty_interner,
        &mut ctx.sy_interner,
        &mut ctx.symbol_table,
        &ctx.source_map,
        program_ast,
    )?;

    let program_tacky = lower_to_tacky(
        analized_program,
        &mut ctx.sy_interner,
        &mut ctx.symbol_table,
        counter,
    );
    print_ir::DebuggingPrinter::new(&ctx.sy_interner).print(program_tacky);

    Ok(())
}

pub fn codegen_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);

    let program_ast = parse(
        lexer,
        &mut ctx.ty_interner,
        &mut ctx.sy_interner,
        &ctx.source_map,
    )?;
    let (analized_program, counter) = analize(
        &ctx.ty_interner,
        &mut ctx.sy_interner,
        &mut ctx.symbol_table,
        &ctx.source_map,
        program_ast,
    )?;
    let program_tacky = lower_to_tacky(
        analized_program,
        &mut ctx.sy_interner,
        &mut ctx.symbol_table,
        counter,
    );

    let backend_table = AsmSymbolTable::new(&ctx.symbol_table);

    let program_asm = codegen(
        program_tacky,
        &backend_table,
        &ctx.ty_interner,
        &ctx.symbol_table,
    );
    let asm_printer = DebuggingPrinter::new(&ctx.sy_interner);
    asm_printer.print(program_asm);

    Ok(())
}

pub fn emit_assembly(file_path: &str, file_name: &str) -> Result<String, Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;
    let lexer = lexer::Lexer::new(&input_string);

    let arena = Bump::new();
    let mut ctx = CompilerContext::new(&arena, file_name, &input_string);

    let program_ast = parse(
        lexer,
        &mut ctx.ty_interner,
        &mut ctx.sy_interner,
        &ctx.source_map,
    )?;
    let (analized_program, counter) = analize(
        &ctx.ty_interner,
        &mut ctx.sy_interner,
        &mut ctx.symbol_table,
        &ctx.source_map,
        program_ast,
    )?;
    let program_tacky = lower_to_tacky(
        analized_program,
        &mut ctx.sy_interner,
        &mut ctx.symbol_table,
        counter,
    );
    let backend_table = AsmSymbolTable::new(&ctx.symbol_table);
    let program_asm = codegen(
        program_tacky,
        &backend_table,
        &ctx.ty_interner,
        &ctx.symbol_table,
    );

    let asm_file_name = format!("{}.s", remove_file_extension(file_name));
    let output_path = set_file_name(file_path, &asm_file_name);
    Emitter::new(&ctx.sy_interner, &ctx.symbol_table).write_program(program_asm, &output_path)?;

    Ok(output_path)
}
