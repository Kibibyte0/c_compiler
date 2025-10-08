use clap::Parser;
//use emitter::Emitter;
use codegen::{self, AsmGen, DebuggingPrinter, InstructionFix, RegisterAllocation};
use emitter::Emitter;
use parser;
use std::path::PathBuf;
use std::{error::Error, fs};

enum Stage {
    Lex,
    Parse,
    Tacky,
    Codegen,
    None,
}

#[derive(Parser)]
struct Cli {
    #[arg(long, group = "stage")]
    lex: bool,

    #[arg(long, group = "stage")]
    parse: bool,

    #[arg(long, group = "stage")]
    tacky: bool,

    #[arg(long, group = "stage")]
    codegen: bool,

    file_path: PathBuf,
}

impl Cli {
    fn selected_stage(&self) -> Stage {
        if self.lex {
            Stage::Lex
        } else if self.parse {
            Stage::Parse
        } else if self.tacky {
            Stage::Tacky
        } else if self.codegen {
            Stage::Codegen
        } else {
            Stage::None
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);

        // shows lower causes of error
        // put of potential error hierarchy in the future
        let mut source = e.source();
        while let Some(s) = source {
            eprintln!("  caused by: {}", s);
            source = s.source();
        }

        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arg = Cli::parse();

    match arg.selected_stage() {
        Stage::Lex => lexer_stage(arg.file_path),

        Stage::Parse => parser_stage(arg.file_path),

        Stage::Tacky => tacky_stage(arg.file_path),

        Stage::Codegen => codegen_stage(arg.file_path),

        Stage::None => emit_assembly(arg.file_path),
    }
}

// lex the program then exit without starting the other stages
fn lexer_stage(file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;
    let file_name = file_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| "Failed to extract file name as valid UTF-8")?;

    let mut lexer = lexer::Lexer::new(&input_string, file_name);

    while let Some(tok) = lexer.next() {
        println!(
            "matched string: {}, token type: {:?}",
            tok.get_lexeme(),
            tok.get_token()
        );
    }

    Ok(())
}

fn parser_stage(file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;
    let file_name = file_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| "Failed to extract file name as valid UTF-8")?;

    let lexer = lexer::Lexer::new(&input_string, file_name);
    let program_ast = parser::Parser::new(lexer)?.parse_program()?;
    parser::Parser::print(program_ast);

    Ok(())
}

fn tacky_stage(file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;
    let file_name = file_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| "Failed to extract file name as valid UTF-8")?;

    let lexer = lexer::Lexer::new(&input_string, file_name);
    let program_ast = parser::Parser::new(lexer)?.parse_program()?;
    let program_ir = ir_gen::IRgen::new().gen_tacky(program_ast);
    ir_gen::IRgen::print(program_ir);

    Ok(())
}

fn codegen_stage(file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;
    let file_name = file_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| "Failed to extract file name as valid UTF-8")?;

    let lexer = lexer::Lexer::new(&input_string, file_name);
    let program_ast = parser::Parser::new(lexer)?.parse_program()?;
    let program_ir = ir_gen::IRgen::new().gen_tacky(program_ast);

    let mut program_asm = AsmGen::gen_asm(program_ir);

    let mut codegen = RegisterAllocation::new();
    codegen.allocate_registers(&mut program_asm);

    InstructionFix::fix_instructions(&mut program_asm);
    DebuggingPrinter::print(program_asm);

    Ok(())
}

fn emit_assembly(mut file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;
    let file_name = file_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| "Failed to extract file name as valid UTF-8")?;

    let lexer = lexer::Lexer::new(&input_string, file_name);
    let program_ast = parser::Parser::new(lexer)?.parse_program()?;
    let program_ir = ir_gen::IRgen::new().gen_tacky(program_ast);

    let mut program_asm = AsmGen::gen_asm(program_ir);

    let mut codegen = RegisterAllocation::new();
    codegen.allocate_registers(&mut program_asm);

    InstructionFix::fix_instructions(&mut program_asm);

    file_path.set_file_name("out.s");
    Emitter::new(12, 16, 2).write_program(program_asm, file_path)?;

    Ok(())
}
