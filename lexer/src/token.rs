use logos::Logos;

mod token_impl;

#[derive(Debug, PartialEq, Logos, Clone, Copy, Eq)]
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
    #[regex(r"\n")]
    #[regex(r"[ \t\f]+")]
    #[regex(r"//[^\n]*")]
    #[regex(r"/\*[^*]*\*+([^/*][^*]*\*+)*/")]
    Skip,

    // invalid patterns
    #[regex(r"\d+[a-zA-Z_][a-zA-Z0-9_]*")]
    Error,
}
