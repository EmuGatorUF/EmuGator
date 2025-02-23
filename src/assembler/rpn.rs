use std::{collections::VecDeque, fmt::Display, ops::Deref};

use dioxus::html::b;
use ibig::IBig;

use super::{
    Section,
    assembled_program::Address,
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
    UnaryMinus,
    BitwiseNot,
    Multiply,
    Divide,
    Modulo,
    ShiftLeft,
    ShiftRight,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOrNot,
    Add,
    Subtract,
    Integer(IBig),
    Variable(String),
}

impl RPNKind {
    fn properties(&self) -> (u32, Associativity) {
        match *self {
            Self::UnaryMinus | Self::BitwiseNot => (2, Associativity::Right),
            Self::Multiply | Self::Divide | Self::Modulo | Self::ShiftLeft | Self::ShiftRight => {
                (3, Associativity::Left)
            }
            Self::BitwiseOr | Self::BitwiseAnd | Self::BitwiseXor | Self::BitwiseOrNot => {
                (4, Associativity::Left)
            }
            Self::Add | Self::Subtract => (5, Associativity::Left),
            Self::LParenthesis | Self::RParenthesis | Self::Integer(_) | Self::Variable(_) => {
                panic!("Invalid token encountered")
            }
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
            TokenKind::LParenthesis => Ok(Self::LParenthesis),
            TokenKind::RParenthesis => Ok(Self::RParenthesis),

            TokenKind::Tilde => Ok(Self::BitwiseNot),
            TokenKind::Asterisk => Ok(Self::Multiply),
            TokenKind::Slash => Ok(Self::Divide),
            TokenKind::Percent => Ok(Self::Modulo),
            TokenKind::ShiftLeft => Ok(Self::ShiftLeft),
            TokenKind::ShiftRight => Ok(Self::ShiftRight),
            TokenKind::Pipe => Ok(Self::BitwiseOr),
            TokenKind::Ampersand => Ok(Self::BitwiseAnd),
            TokenKind::Caret => Ok(Self::BitwiseXor),
            TokenKind::Exclamation => Ok(Self::BitwiseOrNot),
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Subtract),

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

impl<'a> Display for RPN<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            RPNKind::LParenthesis => write!(f, "("),
            RPNKind::RParenthesis => write!(f, ")"),
            RPNKind::UnaryMinus => write!(f, "u-"),
            RPNKind::BitwiseNot => write!(f, "~"),
            RPNKind::Multiply => write!(f, "*"),
            RPNKind::Divide => write!(f, "/"),
            RPNKind::Modulo => write!(f, "%"),
            RPNKind::ShiftLeft => write!(f, "{}", "<".repeat(self.token.width)),
            RPNKind::ShiftRight => write!(f, "{}", ">".repeat(self.token.width)),
            RPNKind::BitwiseOr => write!(f, "|"),
            RPNKind::BitwiseAnd => write!(f, "&"),
            RPNKind::BitwiseXor => write!(f, "^"),
            RPNKind::BitwiseOrNot => write!(f, "!"),
            RPNKind::Add => write!(f, "+"),
            RPNKind::Subtract => write!(f, "-"),
            RPNKind::Integer(val) => write!(f, "{}", val),
            RPNKind::Variable(name) => write!(f, "{}", name),
        }
    }
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
    pub fn evaluate<F: FnMut(&String) -> Result<Address, AssemblerError>>(
        self,
        mut resolve: F,
    ) -> Result<Address, AssemblerError> {
        if self.is_empty() {
            return Ok(Address(Section::Absolute, 0.into()));
        }

        let invalid_err = AssemblerError::from_expression("Invalid expression".into(), &self);

        let mut stack = Vec::new();

        for rpn in self.0 {
            let result = match rpn.kind {
                RPNKind::Integer(val) => Ok(Address(Section::Absolute, val.clone())),
                RPNKind::Variable(name) => Ok(resolve(&name)?),
                RPNKind::LParenthesis | RPNKind::RParenthesis => {
                    Err("Mismatched parenthesis".into())
                }
                // Unary operators
                RPNKind::UnaryMinus | RPNKind::BitwiseNot => {
                    let a: Address = stack.pop().ok_or(AssemblerError::from_token(
                        format!("Not enough arguments for operator {}.", rpn),
                        &rpn.token,
                    ))?;

                    match &rpn.kind {
                        RPNKind::UnaryMinus => -a,
                        RPNKind::BitwiseNot => !a,
                        _ => unreachable!(),
                    }
                }
                // Binary operators
                RPNKind::Multiply
                | RPNKind::Divide
                | RPNKind::Modulo
                | RPNKind::ShiftLeft
                | RPNKind::ShiftRight
                | RPNKind::BitwiseOr
                | RPNKind::BitwiseAnd
                | RPNKind::BitwiseXor
                | RPNKind::BitwiseOrNot
                | RPNKind::Add
                | RPNKind::Subtract => {
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        format!("Not enough arguments for operator {}.", rpn),
                        &rpn.token,
                    ))?;
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        format!("Not enough arguments for operator {}.", rpn),
                        &rpn.token,
                    ))?;

                    match &rpn.kind {
                        RPNKind::Multiply => a * b,
                        RPNKind::Divide => a / b,
                        RPNKind::Modulo => a % b,
                        RPNKind::ShiftLeft => a << b,
                        RPNKind::ShiftRight => a >> b,
                        RPNKind::BitwiseOr => a | b,
                        RPNKind::BitwiseAnd => a & b,
                        RPNKind::BitwiseXor => a ^ b,
                        RPNKind::BitwiseOrNot => {
                            let b = !b;
                            match b {
                                Ok(b) => a | b,
                                Err(e) => Err(e),
                            }
                        }
                        RPNKind::Add => a + b,
                        RPNKind::Subtract => a - b,
                        _ => unreachable!(),
                    }
                }
            }
            .map_err(|e| AssemblerError::from_token(e, &rpn.token))?;

            stack.push(result);
        }

        let result = Ok(stack.pop().ok_or(invalid_err.clone())?);
        if !stack.is_empty() {
            return Err(invalid_err.clone());
        }
        result
    }

    pub fn shunting_yard(
        tokens: &mut dyn Iterator<Item = Token<'a>>,
    ) -> Result<Expression<'a>, AssemblerError> {
        let mut output = VecDeque::new();
        let mut op_stack: VecDeque<RPN<'a>> = VecDeque::new();
        let mut infix: bool = false;

        for token in tokens {
            match &token.kind {
                TokenKind::Symbol(_)
                | TokenKind::IntLiteral(_, _, _)
                | TokenKind::ChrLiteral(_, _) => {
                    output.push_back(token.try_into()?);
                    infix = true;
                }
                TokenKind::Tilde
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::ShiftLeft
                | TokenKind::ShiftRight
                | TokenKind::Pipe
                | TokenKind::Ampersand
                | TokenKind::Caret
                | TokenKind::Exclamation
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Asterisk => {
                    let o1: RPN<'a> = if !infix {
                        if token.kind == TokenKind::Minus {
                            RPN {
                                kind: RPNKind::UnaryMinus,
                                token,
                            }
                        } else if token.kind == TokenKind::Tilde {
                            token.try_into()?
                        } else {
                            return Err(AssemblerError::from_token(
                                "Invalid token encountered. Expected prefix operator, found infix operator.".into(),
                                &token,
                            ));
                        }
                    } else {
                        token.try_into()?
                    };

                    loop {
                        let o2 = op_stack.back();
                        if let Some(o2) = o2 {
                            if o2.kind != RPNKind::LParenthesis
                                && (o2.kind.precedence() < o1.kind.precedence()
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
                    infix = false;
                }
                TokenKind::LParenthesis => {
                    op_stack.push_back(token.try_into()?);
                    infix = false;
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
                    infix = true;
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
