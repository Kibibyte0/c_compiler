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

fn main() -> Result<(), Box<dyn Error>> {
    let arg = Cli::parse();

    match arg.selected_stage() {
        Stage::Lex => {
            let input_string = fs::read_to_string(&arg.file_path)?;
            let mut lexer = lexer::Lexer::new(&input_string);
            while let Some(tok) = lexer.next() {
                println!(
                    "mathced string: {}, token Type :{:?}, line: {}, column: {}",
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
            ir_gen::IRgen::new(program).emit_tacky().print();
        }

        Stage::Codegen => {}

        Stage::None => {}
    }

    Ok(())
}
