use shared_context::Span;

use logos::Logos;
use token::Token;

pub mod token;

/// A token along with its source text slice and location (span).
///
/// This structure is used to carry both the `Token` type (e.g. `Identifier`, `If`, etc.)
/// and its lexeme (the exact substring from the source code), as well as the `Span`
/// information (start/end indices and line number).
#[derive(Clone, Copy)]
pub struct SpannedToken<'a> {
    token: Token,
    lexeme: &'a str,
    span: Span,
}

impl<'a> Default for SpannedToken<'a> {
    fn default() -> Self {
        Self {
            token: Token::Skip,
            lexeme: "",
            span: Span {
                start: 0,
                end: 0,
                line: 0,
            },
        }
    }
}

impl<'a> SpannedToken<'a> {
    /// Creates a new SpannedToken.
    pub fn new(token: Token, lexeme: &'a str, span: Span) -> Self {
        Self {
            token,
            lexeme,
            span,
        }
    }

    /// Returns the Token kind.
    pub fn get_token(&self) -> Token {
        self.token
    }

    /// Returns the Span (location info) for this token.
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Returns the original lexeme string for this token.
    pub fn get_lexeme(&self) -> &'a str {
        self.lexeme
    }
}

/// The lexer structure that wraps the `logos` lexer.
///
/// It provides iteration over tokens and constructs SpannedToken values
/// that include span and line information.
pub struct Lexer<'a> {
    lex: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    /// Creates a new [`Lexer`] for the given source code.
    pub fn new(source_code: &'a str) -> Self {
        Self {
            lex: Token::lexer(source_code),
        }
    }

    /// Retrieves the next token from the input stream, if available.
    ///
    /// This wraps `logos`'s `next()` function, mapping any lexing errors
    /// to the Token::Error variant.
    pub fn next(&mut self) -> Option<SpannedToken<'a>> {
        let token = match self.lex.next()? {
            Ok(tok) => tok,
            Err(_) => Token::Error,
        };

        // Get the current line number from the lexer extras.
        // this line number is in sync with the source file line number after pre-processing
        let line = self.lex.extras.line;

        Some(SpannedToken::new(
            token,
            self.lex.slice(),
            Span::new(self.lex.span().start, self.lex.span().end, line),
        ))
    }
}
