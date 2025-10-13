use super::Token;

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
            | Token::GreaterThanOrEq
            | Token::Assignment
            | Token::QuestionMark => true,
            _ => false,
        }
    }

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
