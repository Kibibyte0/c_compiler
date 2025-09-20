use clap::Parser;
use std::{error::Error, fs};

#[derive(Parser)]
struct Cli {
    #[arg(long, group = "stage")]
    lex: bool,

    #[arg(long, group = "stage")]
    parse: bool,

    #[arg(long, group = "stage")]
    codegen: bool,

    file_path: std::path::PathBuf,
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
        println!("using the code generator");
    } else {
        println!("going through the entire pip line");
    }

    Ok(())
}
