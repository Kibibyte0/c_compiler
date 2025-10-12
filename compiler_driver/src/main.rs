use clap::Parser;
use codegen::{self, AsmGen, InstructionFix, RegisterAllocation};
use emitter::Emitter;
use parser;
use semantic_analysis::analize;
use std::{error::Error, fs};

mod files;
use files::*;

enum Stage {
    Lex,
    Parse,
    Tacky,
    Codegen,
    Validate,
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

    #[arg(long, group = "stage")]
    validate: bool,

    file_path: String,
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
        } else if self.validate {
            Stage::Validate
        } else {
            Stage::None
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arg = Cli::parse();

    let file_path = pre_process_file(&arg.file_path);
    let file_name = get_file_name(&arg.file_path);

    let result = match arg.selected_stage() {
        Stage::Lex => lexer_stage(&file_path, file_name),

        Stage::Parse | Stage::Validate => parser_stage(&file_path, file_name),

        Stage::Tacky => tacky_stage(&file_path, file_name),

        Stage::Codegen => codegen_stage(&file_path, file_name),

        Stage::None => emit_assembly(&file_path, file_name),
    };
    delete_file(&file_path);
    result
}

// lex the program then exit without starting the other stages
fn lexer_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let mut lexer = lexer::Lexer::new(&input_string, &file_name);

    while let Some(tok) = lexer.next() {
        println!(
            "matched string: {}, token type: {:?}",
            tok.get_lexeme(),
            tok.get_token()
        );
    }

    Ok(())
}

fn parser_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string, &file_name);
    let program_ast = parser::Parser::new(lexer)?.parse()?;
    let (analized_program, _) = analize(file_name, &input_string, program_ast)?;

    parser::print_ast::DebuggingPrinter::print(analized_program);

    Ok(())
}

fn tacky_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string, &file_name);
    let program_ast = parser::Parser::new(lexer)?.parse()?;
    let (analized_program, counter) = analize(file_name, &input_string, program_ast)?;

    let program_ir = ir_gen::IRgen::new(counter).gen_tacky(analized_program);
    ir_gen::IRgen::print(program_ir);

    Ok(())
}

fn codegen_stage(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string, &file_name);
    let program_ast = parser::Parser::new(lexer)?.parse()?;
    let (analized_program, counter) = analize(file_name, &input_string, program_ast)?;

    let program_ir = ir_gen::IRgen::new(counter).gen_tacky(analized_program);

    let mut program_asm = AsmGen::gen_asm(program_ir);

    let mut codegen = RegisterAllocation::new();
    codegen.allocate_registers(&mut program_asm);

    InstructionFix::fix_instructions(&mut program_asm);
    codegen::DebuggingPrinter::print(program_asm);

    Ok(())
}

fn emit_assembly(file_path: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let input_string = fs::read_to_string(&file_path)?;

    let lexer = lexer::Lexer::new(&input_string, &file_name);
    let program_ast = parser::Parser::new(lexer)?.parse()?;
    let (analized_program, counter) = analize(file_name, &input_string, program_ast)?;

    let program_ir = ir_gen::IRgen::new(counter).gen_tacky(analized_program);

    let mut program_asm = AsmGen::gen_asm(program_ir);

    let mut codegen = RegisterAllocation::new();
    codegen.allocate_registers(&mut program_asm);

    InstructionFix::fix_instructions(&mut program_asm);

    let output_path = set_file_name(file_path, "out.s");
    Emitter::new(12, 16, 2).write_program(program_asm, output_path)?;

    Ok(())
}
