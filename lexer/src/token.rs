use logos::Logos;
use logos::Skip;
mod token_impl;

/// Additional state information maintained by the `logos` lexer.
///
/// Currently, this tracks the current line number in the source file.
#[derive(Default, Debug)]
pub struct LexerExtras {
    pub line: usize,
}

/// Called when a newline is encountered.
///
/// Increments the line counter and skips the token.
fn logos_newline(lexer: &mut logos::Lexer<Token>) -> Skip {
    lexer.extras.line += 1;
    Skip
}

/// Handles `#line`-style preprocessor directives that preserve the original
/// source line numbering in preprocessed files.
///
/// Example directive: `# 42 "source.c"`
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

/// All possible token kinds recognized by the lexer.
///
/// Each variant corresponds to a language construct, keyword, operator, or symbol.
/// The `logos` crate automatically generates the pattern matching code.
#[derive(Debug, PartialEq, Logos, Clone, Copy, Eq)]
#[logos(extras = LexerExtras)]
pub enum Token {
    //
    // Identifiers and literals
    //
    /// Identifiers: start with a letter or underscore, followed by alphanumeric characters or underscores.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 0)]
    Identifier,

    /// Integer constatns
    #[regex(r"[0-9]+", priority = 0)]
    ConstantInt,

    /// Long integer constanst
    #[regex(r"[0-9]+[lL]", priority = 1)]
    ConstantLong,
    //
    // Keywords
    //
    #[token("return")]
    Return,
    #[token("int")]
    Int,
    #[token("long")]
    Long,
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
    #[token("static")]
    Static,
    #[token("extern")]
    Extern,

    //
    // Operators
    //

    // Assignment and arithmetic
    #[token("=")]
    Assignment,
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

    // Logical operators
    #[token("&&")]
    LogicalAnd,
    #[token("||")]
    LogicalOr,
    #[token("!")]
    LogicalNot,

    // Comparison operators
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

    // Bitwise operator
    #[token("~")]
    BitwiseNot,

    //
    // Symbols and punctuation
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
    #[token(",")]
    Comma,

    //
    // Skipped patterns (whitespace, comments, etc.)
    //
    /// Preprocessor line directive (`# ...`).
    #[regex(r"# [^\n]*", logos_line_directive)]
    LineDirective,

    /// Newlines increment the line counter.
    #[regex(r"\n", logos_newline)]

    /// Whitespace and comments are skipped entirely.
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*[^*]*\*+([^/*][^*]*\*+)*/", logos::skip)]
    Skip,

    //
    // Errors
    //
    /// Invalid token pattern (e.g. `123abc`).
    Error,
}
