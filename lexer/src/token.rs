use logos::Logos;

// Extras for logos to keep track of line number
#[derive(Debug, Default)]
pub struct LinePosition {
    pub line_num: usize,
    pub col_num: usize,
}

// callback function to be used whenever logos match a new line char
// always return None to skip the newline
fn update_line_num(lex: &mut logos::Lexer<Token>) {
    lex.extras.line_num += 1;
    // reset the col_num to 0 not to one
    // because when col number gets updated, it includes the column taken by the new line char
    lex.extras.col_num = 0;
}

#[derive(Debug, PartialEq, Logos, Clone, Copy)]
#[logos(extras = LinePosition)]
pub enum Token {
    // Identifiers: starts with a letter or underscore, followed by letters, digits, or underscores
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 0)]
    Identifier,

    // Integer constants (decimal only)
    #[regex(r"\d+")]
    ConstantInt,

    // Keywords
    #[token("return")]
    Return,

    #[token("int")]
    Int,

    #[token("void")]
    Void,

    // unary operators
    #[token("~")]
    BitwiseComplement,

    #[token("-")]
    Negation,

    #[token("--")]
    Decrement,

    // binary operators
    #[token("+")]
    Addition,

    #[token("*")]
    Multiplication,

    #[token("/")]
    Divison,

    #[token("%")]
    Mod,

    // Symbols
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

    // skipped patterns
    #[regex(r"\n", callback = update_line_num)]
    #[regex(r"[ \t\f]+")]
    #[regex(r"//[^\n]*")]
    #[regex(r"/\*[^*]*\*+([^/*][^*]*\*+)*/")]
    Skip,

    // invalid patterns
    #[regex(r"\d+[a-zA-Z_][a-zA-Z0-9_]*")]
    Error,
}

impl Token {
    pub fn is_unary(&self) -> bool {
        match self {
            Token::Negation | Token::BitwiseComplement => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Token::Addition
            | Token::Negation
            | Token::Multiplication
            | Token::Divison
            | Token::Mod => true,
            _ => false,
        }
    }

    pub fn precednece(&self) -> usize {
        match self {
            Token::Multiplication | Token::Divison | Token::Mod => 50,
            Token::Addition | Token::Negation => 45,
            _ => 0,
        }
    }
}
