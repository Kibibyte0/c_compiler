use clap::Parser;
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
        Stage::Lex => {
            let input_string = fs::read_to_string(&arg.file_path)?;
            let mut lexer = lexer::Lexer::new(&input_string);
            while let Some(tok) = lexer.next() {
                println!(
                    "matched string: {}, token type: {:?}, line: {}, column: {}",
                    tok.lexeme, tok.token_type, tok.line_num, tok.col_start
                );
            }
        }

        Stage::Parse => {
            let input_string = fs::read_to_string(&arg.file_path)?;
            let lexer = lexer::Lexer::new(&input_string);
            let mut parser = parser::Parser::build(lexer)?;
            let program = parser.parse_program()?;
            program.print();
        }

        Stage::Tacky => {
            let input_string = fs::read_to_string(&arg.file_path)?;
            let lexer = lexer::Lexer::new(&input_string);
            let mut parser = parser::Parser::build(lexer)?;
            let program = parser.parse_program()?;
            ir_gen::IRgen::new().emit_tacky(program).print();
        }

        Stage::Codegen => {
            // TODO: Implement code generation stage
        }

        Stage::None => {
            // TODO: Chain the whole compilation pipeline
        }
    }

    Ok(())
}
