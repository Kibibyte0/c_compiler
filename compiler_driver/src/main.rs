use clap::Parser;
use std::path::PathBuf;
use std::{error::Error, fs};

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

fn _replace_last_component(mut path: PathBuf, new: &str) -> PathBuf {
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
                tok.lexeme, tok.token_type, tok.line_num, tok.col_start
            );
        }
    } else if arg.parse == true {
        let input_string = fs::read_to_string(&arg.file_path)?;
        let lexer = lexer::Lexer::new(&input_string);
        let mut parser = parser::Parser::build(lexer)?;
        let program = parser.parse_program()?;

        let tacky = ir_gen::IRgen::new(program).emit_tacky();
        tacky.print();

    } else if arg.codegen == true {
    } else {
    }

    Ok(())
}
