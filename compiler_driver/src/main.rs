use clap::Parser;

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

fn main() {
    let arg = Cli::parse();

    if arg.lex == true {
        println!("using the lexer");
    } else if arg.parse == true {
        println!("using the parser");
    } else if arg.codegen == true {
        println!("using the code generator");
    } else {
        println!("going through the entire pip line");
    }
}
