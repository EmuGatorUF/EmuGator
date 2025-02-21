use super::{lexer::Token, Expression};


#[derive(Debug, Clone)]
pub struct AssemblerError {
    pub error_message: String,
    pub line_number: usize,
    pub column: usize,
    pub width: usize,
}

impl AssemblerError {
    pub fn new(error_message: String, line_number: usize, column: usize, width: usize) -> Self {
        Self {
            error_message,
            line_number,
            column,
            width,
        }
    }

    pub fn from_token(error_message: String, token: &Token) -> Self {
        Self {
            error_message,
            line_number: token.line,
            column: token.column,
            width: token.width,
        }
    }

    pub fn from_expression(error_message: String, expression: &Expression) -> Self {
        let first = &expression.first().expect("Expression is not empty.").token;
        let last = &expression.last().expect("Expression is not empty.").token;
        Self {
            error_message,
            line_number: first.line,
            column: first.column,
            width: last.column + last.width - first.column,
        }
    }
}
