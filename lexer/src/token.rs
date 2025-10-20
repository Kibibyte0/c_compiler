use logos::Logos;
use logos::Skip;
mod token_impl;

#[derive(Default, Debug)]
pub struct LexerExtras {
    pub line: usize,
}

fn logos_newline(lexer: &mut logos::Lexer<Token>) -> Skip {
    lexer.extras.line += 1;
    Skip
}

// sync the pre_processed file line number with line number of the orginal source file
// using line directives
fn logos_line_directive(lexer: &mut logos::Lexer<Token>) -> Skip {
    let slice = lexer.slice();
    let parts: Vec<&str> = slice.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(line) = parts[1].parse::<usize>() {
            lexer.extras.line = line.saturating_sub(1);
        }
    }
    Skip
}

#[derive(Debug, PartialEq, Logos, Clone, Copy, Eq)]
#[logos(extras = LexerExtras)]
pub enum Token {
    // Identifiers: starts with a letter or underscore, followed by letters, digits, or underscores
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 0)]
    Identifier,

    //
    // literals
    //

    // Integer constants
    #[regex(r"\d+")]
    ConstantInt,

    //
    // Keywords
    //
    #[token("return")]
    Return,

    #[token("int")]
    Int,

    #[token("void")]
    Void,

    #[token("else")]
    Else,

    #[token("if")]
    If,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("do")]
    Do,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    //
    // Operators
    //
    #[token("=")]
    Assignment,

    // Arithmatic operators
    #[token("-")]
    Neg,

    #[token("--")]
    Dec,

    #[token("+")]
    Add,

    #[token("*")]
    Mul,

    #[token("/")]
    Div,

    #[token("%")]
    Mod,

    // logical operators
    #[token("&&")]
    LogicalAnd,

    #[token("||")]
    LogicalOr,

    #[token("!")]
    LogicalNot,

    // comparison operators
    #[token("==")]
    Equal,

    #[token("!=")]
    NotEqual,

    #[token("<")]
    LessThan,

    #[token(">")]
    GreaterThan,

    #[token("<=")]
    LessThanOrEq,

    #[token(">=")]
    GreaterThanOrEq,

    // bitwise operators
    #[token("~")]
    Not,

    //
    // Symbols
    //
    #[token("(")]
    LeftParenthesis,

    #[token(")")]
    RightParenthesis,

    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token("?")]
    QuestionMark,

    // skipped patterns
    #[regex(r"\n", logos_newline)]
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*[^*]*\*+([^/*][^*]*\*+)*/", logos::skip)]
    Skip,

    // line directive left by the pre_processor to preserve original lines position
    #[regex(r"# [^\n]*", logos_line_directive)]
    LineDirective,

    // invalid patterns
    #[regex(r"\d+[a-zA-Z_][a-zA-Z0-9_]*")]
    Error,
}
