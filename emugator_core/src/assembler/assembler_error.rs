use super::{Expression, lexer::Token, rpn::RPN};

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
        if let Some(RPN { token: first, .. }) = expression.first() {
            // Safe to unwrap because we know the expression is not empty
            let last = &expression.last().unwrap().token;
            Self {
                error_message,
                line_number: first.line,
                column: first.column,
                width: last.column + last.width - first.column,
            }
        } else {
            Self {
                error_message: error_message + " (empty expression somewhere)",
                line_number: 0,
                column: 0,
                width: 0,
            }
        }
    }
}
