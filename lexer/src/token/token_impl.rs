use super::Token;
use std::fmt;

impl Token {
    /// Returns `true` if this token represents a unary operator.
    pub fn is_unary(&self) -> bool {
        matches!(self, Token::Neg | Token::Not | Token::LogicalNot)
    }

    /// Returns `true` if this token represents a binary operator.
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            // Arithmetic
            Token::Add
                | Token::Neg
                | Token::Mul
                | Token::Div
                | Token::Mod
                // Logical and comparison
                | Token::LogicalAnd
                | Token::LogicalOr
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan
                | Token::LessThanOrEq
                | Token::GreaterThanOrEq
                // Assignment and ternary
                | Token::Assignment
                | Token::QuestionMark
        )
    }

    /// Returns the operator precedence value (higher = binds more tightly).
    pub fn precedence(&self) -> usize {
        match self {
            Token::Mul | Token::Div | Token::Mod => 50,
            Token::Add | Token::Neg => 45,
            Token::LessThan | Token::LessThanOrEq | Token::GreaterThan | Token::GreaterThanOrEq => {
                35
            }
            Token::Equal | Token::NotEqual => 30,
            Token::LogicalAnd => 10,
            Token::LogicalOr => 5,
            Token::QuestionMark => 3,
            Token::Assignment => 1,
            _ => 0,
        }
    }
}

/// Pretty-print each token in human-readable form.
///
/// This is used for debugging and displaying tokens in error messages.
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Identifiers and literals
            Token::Identifier => write!(f, "identifier"),
            Token::ConstantInt => write!(f, "int constant"),

            // Keywords
            Token::Return => write!(f, "return"),
            Token::Int => write!(f, "int"),
            Token::Void => write!(f, "void"),
            Token::Else => write!(f, "else"),
            Token::If => write!(f, "if"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::Do => write!(f, "do"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),

            // Operators
            Token::Assignment => write!(f, "="),
            Token::Neg => write!(f, "-"),
            Token::Dec => write!(f, "--"),
            Token::Add => write!(f, "+"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Mod => write!(f, "%"),

            // Logical operators
            Token::LogicalAnd => write!(f, "&&"),
            Token::LogicalOr => write!(f, "||"),
            Token::LogicalNot => write!(f, "!"),

            // Comparison operators
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::LessThan => write!(f, "<"),
            Token::GreaterThan => write!(f, ">"),
            Token::LessThanOrEq => write!(f, "<="),
            Token::GreaterThanOrEq => write!(f, ">="),

            // Bitwise
            Token::Not => write!(f, "~"),

            // Symbols
            Token::LeftParenthesis => write!(f, "("),
            Token::RightParenthesis => write!(f, ")"),
            Token::LeftCurlyBracket => write!(f, "{{"),
            Token::RightCurlyBracket => write!(f, "}}"),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::QuestionMark => write!(f, "?"),
            Token::Comma => write!(f, ","),

            // Skipped, line directive, or invalid
            Token::Skip => write!(f, "<skip>"),
            Token::Error => write!(f, "<error>"),
            Token::LineDirective => write!(f, "<#line>"),
        }
    }
}
