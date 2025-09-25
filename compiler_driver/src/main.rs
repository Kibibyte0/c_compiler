use clap::Parser;
use codegen::Codegen;
use emitter::Emitter;
use std::{error::Error, fs};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(long, group = "stage")]
    lex: bool,

    #[arg(long, group = "stage")]
    parse: bool,

    #[arg(long, group = "stage")]
    codegen: bool,

    file_path: PathBuf,
}

fn replace_last_component(mut path: PathBuf, new: &str) -> PathBuf {
    path.set_file_name(new);
    path
}

fn main() -> Result<(), Box<dyn Error>> {
    let arg = Cli::parse();

    if arg.lex == true {
        let input_string = fs::read_to_string(&arg.file_path)?;

        let mut lexer = lexer::Lexer::new(&input_string);
        while let Some(tok) = lexer.next() {
            println!(
                "mathced string: {}, token Type :{:?}, line: {}, column: {}",
                tok.lexeme, tok.token_type, tok.span.line_num, tok.span.col_start
            );
        }
    } else if arg.parse == true {
        let input_string = fs::read_to_string(&arg.file_path)?;
        let lexer = lexer::Lexer::new(&input_string);
        let mut parser = parser::Parser::new(lexer);
        parser.parse_program().dump(0);

    } else if arg.codegen == true {
        let input_string = fs::read_to_string(&arg.file_path)?;
        let lexer = lexer::Lexer::new(&input_string);
        let mut parser = parser::Parser::new(lexer);
        let codegen = Codegen::new(parser.parse_program());
        codegen.gen_program().dump(0);

    } else {
        let input_string = fs::read_to_string(&arg.file_path)?;
        let lexer = lexer::Lexer::new(&input_string);
        let mut parser = parser::Parser::new(lexer);
        let codegen = Codegen::new(parser.parse_program());

        let output_path = replace_last_component(arg.file_path, "out.s");
        let mut emitter = Emitter::new(codegen.gen_program(), output_path);
        emitter.emit_asm();

    }

    Ok(())
}
