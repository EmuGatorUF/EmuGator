use std::{collections::VecDeque, ops::Deref};

use ibig::IBig;

use super::{
    assembler_error::AssemblerError,
    lexer::{Token, TokenKind},
};

#[derive(PartialEq, Eq, Debug)]
enum Associativity {
    Left,
    Right,
}

#[derive(PartialEq, Eq, Debug)]
pub enum RPNKind {
    LParenthesis,
    RParenthesis,
    UnaryPlus,
    UnaryMinus,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    Integer(IBig),
    Variable(String),
}

impl RPNKind {
    fn properties(&self) -> (u32, Associativity) {
        match *self {
            Self::Multiply | Self::Divide => (3, Associativity::Left),
            Self::Add | Self::Subtract => (2, Associativity::Left),
            _ => (0, Associativity::Left),
        }
    }

    fn precedence(&self) -> u32 {
        self.properties().0
    }

    fn associativity(&self) -> Associativity {
        self.properties().1
    }
}

impl TryFrom<&Token<'_>> for RPNKind {
    type Error = AssemblerError;

    fn try_from(token: &Token<'_>) -> Result<Self, Self::Error> {
        match &token.kind {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Subtract),
            TokenKind::Asterisk => Ok(Self::Multiply),
            TokenKind::Slash => Ok(Self::Divide),
            TokenKind::LParenthesis => Ok(Self::LParenthesis),
            TokenKind::RParenthesis => Ok(Self::RParenthesis),
            TokenKind::IntLiteral(_, _, val) => Ok(Self::Integer(val.clone())),
            TokenKind::ChrLiteral(_, c) => Ok(Self::Integer((*c as u32).into())),
            TokenKind::Symbol(name) => Ok(Self::Variable((*name).into())),
            _ => Err(AssemblerError::from_token(
                "Invalid token encountered".into(),
                token,
            )),
        }
    }
}

pub struct RPN<'a> {
    pub kind: RPNKind,
    pub token: Token<'a>,
}

impl<'a> TryFrom<Token<'a>> for RPN<'a> {
    type Error = AssemblerError;

    fn try_from(token: Token<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: (&token).try_into()?,
            token,
        })
    }
}

pub struct Expression<'a>(Vec<RPN<'a>>);

impl<'a> Expression<'a> {
    pub fn evaluate<F: FnMut(&String) -> Result<&'a IBig, AssemblerError>>(
        &self,
        mut resolve: F,
    ) -> Result<IBig, AssemblerError> {
        let mut stack = Vec::new();
        for rpn in self.iter() {
            match &rpn.kind {
                RPNKind::Integer(val) => stack.push(val.clone()),
                RPNKind::Variable(name) => stack.push(resolve(&name)?.clone()),
                RPNKind::Add => {
                    let a = stack.pop().expect("Empty stack");
                    let b = stack.pop().expect("Empty stack");
                    stack.push(a + b);
                }
                RPNKind::Subtract => {
                    let a = stack.pop().expect("Empty stack");
                    let b = stack.pop().expect("Empty stack");
                    stack.push(b - a);
                }
                RPNKind::Multiply => {
                    let a = stack.pop().expect("Empty stack");
                    let b = stack.pop().expect("Empty stack");
                    stack.push(a * b);
                }
                RPNKind::Divide => {
                    let a = stack.pop().expect("Empty stack");
                    let b = stack.pop().expect("Empty stack");
                    stack.push(b / a);
                }
                _ => todo!(),
            }
        }

        Ok(stack.pop().expect("Empty stack"))
    }

    pub fn shunting_yard(
        tokens: &mut dyn Iterator<Item = Token<'a>>,
    ) -> Result<Expression<'a>, AssemblerError> {
        let mut output = VecDeque::new();
        let mut op_stack: VecDeque<RPN<'a>> = VecDeque::new();

        for token in tokens {
            match &token.kind {
                TokenKind::Symbol(_)
                | TokenKind::IntLiteral(_, _, _)
                | TokenKind::ChrLiteral(_, _) => {
                    output.push_back(token.try_into()?);
                }
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Asterisk
                | TokenKind::Slash
                | TokenKind::Ampersand
                | TokenKind::Pipe
                | TokenKind::Caret => {
                    let o1: RPN<'a> = token.try_into()?;
                    loop {
                        let o2 = op_stack.back();
                        if let Some(o2) = o2 {
                            if o2.kind != RPNKind::LParenthesis
                                && (o2.kind.precedence() > o1.kind.precedence()
                                    || (o2.kind.precedence() == o1.kind.precedence()
                                        && o1.kind.associativity() == Associativity::Left))
                            {
                                output.push_back(op_stack.pop_back().expect("Stack is not empty"));
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    op_stack.push_back(o1);
                }
                TokenKind::LParenthesis => {
                    op_stack.push_back(token.try_into()?);
                }
                TokenKind::RParenthesis => {
                    while op_stack
                        .back()
                        .ok_or(AssemblerError::from_token(
                            "Mismatched parenthesis".into(),
                            &token,
                        ))?
                        .kind
                        != RPNKind::LParenthesis
                    {
                        output.push_back(op_stack.pop_back().expect("Mismatched parenthesis"));
                    }
                    op_stack.pop_back();
                }
                _ => {
                    // Non-Expression Token encountered
                    break;
                }
            }
        }

        while let Some(back) = op_stack.pop_back() {
            if back.kind == RPNKind::LParenthesis || back.kind == RPNKind::RParenthesis {
                return Err(AssemblerError::from_token(
                    "Mismatched parenthesis".into(),
                    &back.token,
                ));
            }
            output.push_back(back);
        }

        Ok(output.into())
    }
}

impl<'a, T: Into<Vec<RPN<'a>>>> From<T> for Expression<'a> {
    fn from(value: T) -> Self {
        Expression(value.into())
    }
}

impl<'a> Deref for Expression<'a> {
    type Target = Vec<RPN<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
