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

    //
    // Operators
    //

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
            Token::Neg | Token::Not | Token::LogicalNot => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            // arithmatic
            Token::Add
            | Token::Neg
            | Token::Mul
            | Token::Div
            | Token::Mod
            // logical
            | Token::LogicalAnd
            | Token::LogicalOr
            | Token::Equal
            | Token::NotEqual
            | Token::LessThan
            | Token::GreaterThan
            | Token::LessThanOrEq
            | Token::GreaterThanOrEq => true,
            _ => false,
        }
    }

    pub fn precednece(&self) -> usize {
        match self {
            Token::Mul | Token::Div | Token::Mod => 50,
            Token::Add | Token::Neg => 45,
            Token::LessThan | Token::LessThanOrEq | Token::GreaterThan | Token::GreaterThanOrEq => {
                35
            }
            Token::Equal | Token::NotEqual => 30,
            Token::LogicalAnd => 10,
            Token::LogicalOr => 5,
            _ => 0,
        }
    }
}
