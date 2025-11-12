use crate::files::*;
use codegen::{DebuggingPrinter, codegen};
use emitter::Emitter;
use ir_gen::{lower_to_tacky, print_ir};
use parser::parse;
use semantic_analysis::analize;
use std::{error::Error, fs};

use shared_context::{Bump, Interner, SymbolRegistery, SymbolTable, source_map::SourceMap};

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
    let mut interner = Interner::new(&arena);
    let smap = SourceMap::new(file_name, &input_string);
    let program_ast = parse(lexer, &mut interner.ty, &mut interner.sy, &smap)?;

    parser::print_ast::DebugTreePrinter::new(&interner.ty, &interner.sy).print(program_ast);

    Ok(())
}

pub fn validate_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut interner = Interner::new(&arena);
    let smap = SourceMap::new(file_name, &input_string);
    let mut sytab = SymbolTable::new();

    let program_ast = parse(lexer, &mut interner.ty, &mut interner.sy, &smap)?;

    let (analized_program, _) = analize(
        &interner.ty,
        &mut interner.sy,
        &mut sytab,
        &smap,
        program_ast,
    )?;

    parser::print_ast::DebugTreePrinter::new(&interner.ty, &interner.sy).print(analized_program);

    Ok(())
}

pub fn tacky_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut interner = Interner::new(&arena);
    let smap = SourceMap::new(file_name, &input_string);
    let mut sytab = SymbolTable::new();

    let program_ast = parse(lexer, &mut interner.ty, &mut interner.sy, &smap)?;

    let (analized_program, counter) = analize(
        &interner.ty,
        &mut interner.sy,
        &mut sytab,
        &smap,
        program_ast,
    )?;

    let program_tacky = lower_to_tacky(analized_program, &mut interner.sy, &mut sytab, counter);
    print_ir::DebuggingPrinter::new(&interner.sy).print(program_tacky);

    Ok(())
}

pub fn codegen_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut interner = Interner::new(&arena);
    let smap = SourceMap::new(file_name, &input_string);
    let mut sytab = SymbolTable::new();

    let program_ast = parse(lexer, &mut interner.ty, &mut interner.sy, &smap)?;

    let (analized_program, counter) = analize(
        &interner.ty,
        &mut interner.sy,
        &mut sytab,
        &smap,
        program_ast,
    )?;

    let program_tacky = lower_to_tacky(analized_program, &mut interner.sy, &mut sytab, counter);

    let syreg = SymbolRegistery::build(sytab);

    let program_asm = codegen(program_tacky, &interner.ty, &syreg);

    let asm_printer = DebuggingPrinter::new(&interner.sy);
    asm_printer.print(program_asm);

    Ok(())
}

pub fn emit_assembly(file_path: &str, file_name: &str) -> Result<String, Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string);
    let arena = Bump::new();
    let mut interner = Interner::new(&arena);
    let smap = SourceMap::new(file_name, &input_string);
    let mut sytab = SymbolTable::new();

    let program_ast = parse(lexer, &mut interner.ty, &mut interner.sy, &smap)?;

    let (analized_program, counter) = analize(
        &interner.ty,
        &mut interner.sy,
        &mut sytab,
        &smap,
        program_ast,
    )?;

    let program_tacky = lower_to_tacky(analized_program, &mut interner.sy, &mut sytab, counter);

    let syreg = SymbolRegistery::build(sytab);

    let program_asm = codegen(program_tacky, &interner.ty, &syreg);

    let asm_file_name = format!("{}.s", remove_file_extension(file_name));
    let output_path = set_file_name(file_path, &asm_file_name);
    Emitter::new(&interner.sy, &syreg).write_program(program_asm, &output_path)?;

    Ok(output_path)
}
