use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"[0-9]+")]
    ConstantInt,

    // keywords
    #[token("return")]
    Return,

    #[token("void")]
    Void,

    #[token("int")]
    Int,

    // symbols
    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token("(")]
    LeftParenthesis,

    #[token(")")]
    RightParenthesis,

    #[token(";")]
    Semicolon,

    // Skip single-line comments starting with //
    #[regex(r"//[^\n]*", logos::skip)]
    Comment,

    // Skip whitespace
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
}

pub fn tokenize(input_string: &str) -> impl Iterator {
    Token::lexer(input_string)
}

// testing the lexer
#[cfg(test)]
mod tests {
    use super::*; // brings in `Token` and `tokenize`

    fn lex(input: &str) -> Result<Vec<Token>, ()> {
        Token::lexer(input).collect()
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex("foo bar _baz");
        assert_eq!(
            tokens.unwrap(),
            vec![Token::Identifier, Token::Identifier, Token::Identifier,]
        );
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("int void return");
        assert_eq!(
            tokens.unwrap(),
            vec![Token::Int, Token::Void, Token::Return,]
        );
    }

    #[test]
    fn test_constants() {
        let tokens = lex("123 0 45678");
        assert_eq!(
            tokens.unwrap(),
            vec![Token::ConstantInt, Token::ConstantInt, Token::ConstantInt,]
        );
    }

    #[test]
    fn test_symbols() {
        let tokens = lex("{ } ( ) ;");
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::LeftCurlyBracket,
                Token::RightCurlyBracket,
                Token::LeftParenthesis,
                Token::RightParenthesis,
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn test_ignore_whitespace_and_comments() {
        let tokens = lex("int // this is a comment\n 123");
        assert_eq!(tokens.unwrap(), vec![Token::Int, Token::ConstantInt,]);
    }

    #[test]
    fn test_mixed_input() {
        let tokens = lex("int main() { return 0; }");
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::Int,
                Token::Identifier,
                Token::LeftParenthesis,
                Token::RightParenthesis,
                Token::LeftCurlyBracket,
                Token::Return,
                Token::ConstantInt,
                Token::Semicolon,
                Token::RightCurlyBracket,
            ]
        );
    }
}
