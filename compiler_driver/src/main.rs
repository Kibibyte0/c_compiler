use clap::Parser;
use std::error::Error;

mod files;
mod stages;
use files::*;
use stages::*;

enum Stage {
    Lex,
    Parse,
    Tacky,
    Codegen,
    Validate,
    Asm,
    Obj,
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

    #[arg(long, group = "stage")]
    asm: bool,

    #[arg(short = 'c', group = "stage")]
    obj: bool,

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
        } else if self.asm {
            Stage::Asm
        } else if self.obj {
            Stage::Obj
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

    match arg.selected_stage() {
        Stage::Lex => lexer_stage(&file_path)?,
        Stage::Parse => parser_stage(&file_path, file_name)?,
        Stage::Validate => validate_stage(&file_path, file_name)?,
        Stage::Tacky => tacky_stage(&file_path, file_name)?,
        Stage::Codegen => codegen_stage(&file_path, file_name)?,
        Stage::Asm => {
            emit_assembly(&file_path, file_name)?;
            ()
        }

        // produce exe files
        Stage::None => {
            let output_file_path = emit_assembly(&file_path, file_name)?;
            compile_and_link_assembly_file(
                &output_file_path,
                remove_file_extension(&arg.file_path),
            );
            delete_file(&output_file_path);
        }

        // produce obj files
        Stage::Obj => {
            let output_file_path = emit_assembly(&file_path, file_name)?;
            let obj_file_path = format!("{}.o", remove_file_extension(&arg.file_path));
            compile_assembly_file(&output_file_path, &obj_file_path);
            delete_file(&output_file_path);
        }
    };
    delete_file(&file_path);
    Ok(())
}
