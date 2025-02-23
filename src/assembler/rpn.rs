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

                RPNKind::UnaryMinus => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for -.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(-a);
                }
                RPNKind::BitwiseNot => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for ~.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(!a);
                }
                RPNKind::Multiply => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for *.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for *.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(a * b);
                }
                RPNKind::Divide => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for /.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for /.".into(),
                        &rpn.token,
                    ))?;
                    if a == 0.into() {
                        return Err(AssemblerError::from_token(
                            "Division by zero encountered.".into(),
                            &rpn.token,
                        ));
                    }
                    stack.push(b / a);
                }
                RPNKind::Modulo => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for %.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for %.".into(),
                        &rpn.token,
                    ))?;
                    if a == 0.into() {
                        return Err(AssemblerError::from_token(
                            "Modulo by zero encountered.".into(),
                            &rpn.token,
                        ));
                    }
                    stack.push(b % a);
                }
                RPNKind::ShiftLeft => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        format!("Not enough operands for {}.", "<".repeat(rpn.token.width)),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        format!("Not enough operands for {}.", "<".repeat(rpn.token.width)),
                        &rpn.token,
                    ))?;
                    // represented as b << a
                    // Will panic if a is too large
                    stack.push(
                        b << usize::try_from(&a).map_err(|e| {
                            AssemblerError::from_token(
                                "Shift amount is too large.".into(),
                                &rpn.token,
                            )
                        })?,
                    );
                }
                RPNKind::ShiftRight => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        format!("Not enough operands for {}.", ">".repeat(rpn.token.width)),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        format!("Not enough operands for {}.", ">".repeat(rpn.token.width)),
                        &rpn.token,
                    ))?;
                    // represented as b >> a
                    // Will panic if a is too large
                    stack.push(
                        b >> usize::try_from(&a).map_err(|e| {
                            AssemblerError::from_token(
                                "Shift amount is too large.".into(),
                                &rpn.token,
                            )
                        })?,
                    );
                }
                RPNKind::BitwiseOr => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for |.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for |.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(b | a);
                }
                RPNKind::BitwiseAnd => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for &.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for &.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(b & a);
                }
                RPNKind::BitwiseXor => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for ^.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for ^.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(b ^ a);
                }
                RPNKind::BitwiseOrNot => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for !.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for !.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(b | !a);
                }
                RPNKind::Add => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for +.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for +.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(a + b);
                }
                RPNKind::Subtract => {
                    let a = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for -.".into(),
                        &rpn.token,
                    ))?;
                    let b = stack.pop().ok_or(AssemblerError::from_token(
                        "Not enough operands for -.".into(),
                        &rpn.token,
                    ))?;
                    stack.push(b - a);
                }
                _ => todo!(),
            }
        }

        Ok(stack.pop().ok_or(AssemblerError::from_expression(
            "Invalid expression".into(),
            self,
        ))?)
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
