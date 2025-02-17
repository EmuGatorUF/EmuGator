use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    error,
    f32::consts::E,
    iter::Peekable,
    mem::replace,
    str::FromStr,
};

use bimap::BiBTreeMap;
use dioxus::html::{g::direction, geometry::ElementSpace, symbol};
use ibig::{error::OutOfBoundsError, IBig};
use peeking_take_while::PeekableExt;

use crate::isa::{Instruction, InstructionFormat, Operands, ISA};

use super::{AssembledProgram, AssemblerError, Lexer, Section, Token, TokenKind};

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

#[derive(PartialEq, Eq, Debug)]
enum Associativity {
    Left,
    Right,
}

struct RPN<'a> {
    kind: RPNKind,
    token: Token<'a>,
}

impl<'a> RPN<'a> {
    fn from_token(token: Token<'a>) -> Result<Self, AssemblerError> {
        Ok(Self {
            kind: RPNKind::from_token(&token)?,
            token,
        })
    }
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

    fn from_token(token: &Token) -> Result<Self, AssemblerError> {
        match token.kind {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Subtract),
            TokenKind::Asterisk => Ok(Self::Multiply),
            TokenKind::Slash => Ok(Self::Divide),
            TokenKind::LParenthesis => Ok(Self::LParenthesis),
            TokenKind::RParenthesis => Ok(Self::RParenthesis),
            TokenKind::IntLiteral(_, _, val) => Ok(Self::Integer(val.try_into().unwrap())),
            TokenKind::ChrLiteral(_, c) => Ok(Self::Integer((c as u32).into())),
            TokenKind::Symbol(name) => Ok(Self::Variable(name.into())),
            _ => Err(AssemblerError::from_token(
                "Invalid token encountered".into(),
                token,
            )),
        }
    }
}

fn consume_line<'a>(lexer: &mut Peekable<Lexer<'a>>) -> Result<Vec<Token<'a>>, AssemblerError> {
    let parts = lexer
        .peeking_take_while(|token_result| {
            token_result
                .as_ref()
                .is_ok_and(|token| token.kind != TokenKind::Newline)
        })
        .map(|token_result| token_result.unwrap())
        .collect();

    // lexer.next() is either None, Newline, or Err(_) at this point
    match lexer.next() {
        None
        | Some(Ok(Token {
            kind: TokenKind::Newline,
            ..
        })) => {}
        Some(Err(e)) => return Err(e),
        _ => unreachable!(),
    }

    Ok(parts)
}

fn shunting_yard<'a>(
    tokens: &mut dyn Iterator<Item = Token<'a>>,
) -> Result<Vec<RPN<'a>>, AssemblerError> {
    let mut output = VecDeque::new();
    let mut op_stack: VecDeque<RPN<'a>> = VecDeque::new();

    for token in tokens {
        match &token.kind {
            TokenKind::Symbol(_) | TokenKind::IntLiteral(_, _, _) | TokenKind::ChrLiteral(_, _) => {
                output.push_back(RPN::from_token(token)?);
            }
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Ampersand
            | TokenKind::Pipe
            | TokenKind::Caret => {
                let o1 = RPN::from_token(token)?;
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
                op_stack.push_back(RPN::from_token(token)?);
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

fn parse_label<'a>(
    token: &mut Token<'a>,
    lexer: &mut Peekable<Lexer<'a>>,
) -> Result<Option<(&'a str, Token<'a>)>, AssemblerError> {
    Ok(if is_kind(lexer.peek(), TokenKind::Colon) {
        // Parse label
        lexer.next(); // Skip colon

        if let TokenKind::Symbol(name) = token.kind {
            let label = replace(
                token,
                lexer.next().ok_or(AssemblerError::from_token(
                    "Unexpected EOF after label".into(),
                    token,
                ))??,
            );
            Some((name, label))
        } else {
            // Error: Missing label
            return Err(AssemblerError::from_token(
                "Expected label before ':'".into(),
                token,
            ));
        }
    } else {
        None
    })
}

fn parse_section<'a>(
    token: &mut Token<'a>,
    lexer: &mut Peekable<Lexer<'a>>,
) -> Result<Option<(Section, Vec<RPN<'a>>, Token<'a>)>, AssemblerError> {
    if token.kind == TokenKind::Dot {
        // Parse directive
        let directive = lexer
            .peek()
            .ok_or(AssemblerError::from_token(
                "Unexpected EOF after '.'".into(),
                token,
            ))?
            .as_ref()
            .map_err(|e| e.clone())?;

        if let TokenKind::Symbol(directive_str) = directive.kind {
            Ok(if directive_str == "text" || directive_str == "data" {
                let directive_token = lexer.next().unwrap()?; // Skip directive
                let expression = parse_expression(lexer)?;

                *token = lexer.next().ok_or(AssemblerError::from_token(
                    "Unexpected EOF after section directive, expected one or more instructions or data.".into(),
                    &directive_token,
                ))??;

                let expression = if expression.len() > 0 {
                    expression
                } else {
                    vec![RPN {
                        kind: RPNKind::Integer(0.into()),
                        token: directive_token.clone(),
                    }]
                };

                let section = if directive_str == "text" {
                    Section::Text
                } else {
                    Section::Data
                };
                Some((section, expression, directive_token))
            } else {
                None
            })
        } else {
            // Error: Invalid token after '.'
            Err(AssemblerError::from_token(
                "Invalid token, expected directive after '.'".into(),
                directive,
            ))
        }
    } else {
        Ok(None)
    }
}

fn parse_expression<'a>(lexer: &mut Peekable<Lexer<'a>>) -> Result<Vec<RPN<'a>>, AssemblerError> {
    shunting_yard(
        &mut lexer
            .peeking_take_while(|token_result| {
                token_result.as_ref().is_ok_and(|token| {
                    token.kind != TokenKind::Newline && token.kind != TokenKind::Comma
                })
            })
            .map(|token_result| token_result.unwrap()),
    )
}

fn is_kind(token: Option<&Result<Token, AssemblerError>>, token_kind: TokenKind) -> bool {
    if let Some(Ok(Token { kind, .. })) = token {
        *kind == token_kind
    } else {
        false
    }
}

pub fn assemble<'a>(source: &'a str) -> Result<AssembledProgram, Vec<AssemblerError>> {
    let mut errors = Vec::new();

    let mut symbol_table: HashMap<String, (Vec<RPN>, Token)> = std::collections::HashMap::new();
    symbol_table.insert(
        "!org0".into(),
        (
            vec![RPN {
                kind: RPNKind::Integer(0.into()),
                token: Token {
                    kind: TokenKind::IntLiteral("0", 10, 0),
                    line: 1,
                    column: 1,
                    width: 0,
                },
            }],
            Token {
                kind: TokenKind::Symbol("!org0"),
                line: 1,
                column: 1,
                width: 0,
            },
        ),
    );

    // First Pass
    let mut lexer = Lexer::new(source).peekable();
    let mut current_section = Section::Text;
    let mut current_org: String = "!org0".into();
    let mut offset: u32 = 0;

    while let Some(mut token) = lexer.next() {
        match token {
            Ok(mut token) => {
                let err: Result<_, _> = (|token: &mut Token<'a>| {
                    // Check for a label
                    let label = parse_label(token, &mut lexer)?;

                    // Check for section directive
                    let directive = parse_section(token, &mut lexer)?;

                    // Handle label
                    if let Some((section, label, expression)) = match (directive, label) {
                        (Some((section, expression, _)), Some((label, token))) => {
                            offset = 0;
                            Some((section, label.into(), (expression, token)))
                        }
                        (None, Some((label, token))) => Some((
                            current_section,
                            label.into(),
                            (
                                vec![
                                    RPN {
                                        kind: RPNKind::Variable(current_org.clone()),
                                        token: token.clone(),
                                    },
                                    RPN {
                                        kind: RPNKind::Integer(offset.into()),
                                        token: token.clone(),
                                    },
                                    RPN {
                                        kind: RPNKind::Add,
                                        token: token.clone(),
                                    },
                                ],
                                token,
                            ),
                        )),
                        (Some((section, expression, directive_token)), None) => {
                            offset = 0;
                            Some((
                                section,
                                format!("!org{}", directive_token.line),
                                (expression, directive_token),
                            ))
                        }
                        (None, None) => None,
                    } {
                        symbol_table.insert(label, expression);
                    }

                    // Check for other directives
                    if token.kind == TokenKind::Dot {
                        let directive = lexer.next().ok_or(AssemblerError::from_token(
                            "Unexpected EOF after '.'".into(),
                            token,
                        ))??;

                        let data_length = if let TokenKind::Symbol(directive_str) = directive.kind {
                            match directive_str {
                                "data" | "text" => Ok(0),
                                "byte" => {
                                    parse_expression(&mut lexer)?;
                                    let mut length = 1;

                                    // lexer.peek() is either None, Newline, or Comma
                                    while is_kind(lexer.peek(), TokenKind::Comma) {
                                        lexer.next(); // Skip comma
                                        parse_expression(&mut lexer)?;
                                        length += 1;
                                    }

                                    // Return number of bytes
                                    Ok(length)
                                }
                                "word" => {
                                    parse_expression(&mut lexer)?;
                                    let mut length = 4;

                                    // lexer.peek() is either None, Newline, or Comma
                                    while is_kind(lexer.peek(), TokenKind::Comma) {
                                        lexer.next(); // Skip comma
                                        parse_expression(&mut lexer)?;
                                        length += 4;
                                    }

                                    // Return number of bytes
                                    Ok(length)
                                }
                                "dword" => todo!(),
                                "string" => {
                                    let string =
                                        lexer.next().ok_or(AssemblerError::from_token(
                                            "Unexpected EOF after '.string'".into(),
                                            &directive,
                                        ))??;

                                    if let TokenKind::StrLiteral(_, c) = string.kind {
                                        // Return string length
                                        Ok(c.as_bytes().len() as u32 + 1) // Include null terminator
                                    } else {
                                        // Error: Invalid string
                                        return Err(AssemblerError::from_token(
                                            "Expected string literal after .string directive."
                                                .into(),
                                            &string,
                                        ));
                                    }
                                }
                                "ascii" => {
                                    let string =
                                        lexer.next().ok_or(AssemblerError::from_token(
                                            "Unexpected EOF after '.ascii'".into(),
                                            &directive,
                                        ))??;

                                    if let TokenKind::StrLiteral(_, c) = string.kind {
                                        // Return string length
                                        Ok(c.as_bytes().len() as u32)
                                    } else {
                                        // Error: Invalid string
                                        return Err(AssemblerError::from_token(
                                            "Expected string literal after .ascii directive."
                                                .into(),
                                            &string,
                                        ));
                                    }
                                }
                                _ => Err(AssemblerError::from_token(
                                    format!("Unknown directive '{}'", directive_str),
                                    &directive,
                                )),
                            }
                        } else {
                            Err(AssemblerError::from_token(
                                "Invalid token, expected directive after '.'".into(),
                                &directive,
                            ))
                        }?;

                        offset += data_length;
                    }

                    // Check for instruction
                    if let TokenKind::Symbol(instr) = token.kind {
                        let instruction = token;

                        // Parse instruction
                        ISA::from_str(instr).map_err(|e| {
                            AssemblerError::from_token(
                                format!("Invalid instruction {}", instr),
                                instruction,
                            )
                        })?;

                        // Instructions are 4 bytes
                        offset += 4;
                    }

                    Ok(())
                })(&mut token);

                if let Err(err) = err {
                    errors.push(err);
                }

                // Consume rest of the line
                if token.kind != TokenKind::Newline {
                    match consume_line(&mut lexer) {
                        Ok(_) => {}
                        Err(err) => errors.push(err),
                    };
                }
            }
            Err(err) => {
                errors.push(err);
                continue;
            }
        }
    }

    // Resolve text labels
    let mut resolved_symbols = HashMap::new();

    let mut visited = HashSet::new();
    for (symbol, _) in symbol_table.iter() {
        if let Err(err) = resolve_label(symbol, &symbol_table, &mut resolved_symbols, &mut visited)
            .map_err(|e| match e {
                Ok(e) => e,
                Err((msg, label)) => AssemblerError::from_token(
                    msg,
                    &symbol_table.get(&label).expect("Label not found").1,
                ),
            })
        {
            errors.push(err);
        }
    }

    let symbol_table = resolved_symbols;

    // Second Pass
    let mut lexer = Lexer::new(source).peekable();
    let mut current_section = Section::Text;
    let mut address: u32 = 0;

    while let Some(mut token) = lexer.next() {
        // Resolve lexer errors
        match token {
            Ok(mut token) => {
                let err: Result<(), AssemblerError> = (|token: &mut Token<'a>| {
                    // Check for a label
                    let label = parse_label(token, &mut lexer)?;

                    // Check for directive
                    let directive = parse_section(token, &mut lexer)?;

                    // Handle label
                    if let Some((section, label)) = match (directive, label) {
                        (Some((section, _, _)), Some((label))) => {
                            Some((section, (label.0.into(), label.1)))
                        }
                        (Some((section, _, directive_token)), None) => Some((
                            section,
                            (format!("!org{}", directive_token.line), directive_token),
                        )),
                        (None, _) => None,
                    } {
                        current_section = section;
                        address = symbol_table
                            .get(&label.0)
                            .ok_or(AssemblerError::from_token(
                                "Label not resolved".into(),
                                &label.1,
                            ))?
                            .try_into()
                            .map_err(|e: OutOfBoundsError| {
                                AssemblerError::from_token(
                                    format!("Label \"{}\" is too large (>32-bits)", label.0),
                                    &label.1,
                                )
                            })?;
                    }

                    // Check for other directives
                    if token.kind == TokenKind::Dot {
                        let directive = lexer.next().ok_or(AssemblerError::from_token(
                            "Unexpected EOF after '.'".into(),
                            token,
                        ))??;

                        // let data: &[u8] = if let TokenKind::Symbol(directive_str) = directive.kind {
                        //     match directive_str {
                        //         "data" | "text" => Ok(&[]),
                        //         "byte" => {
                        //             todo!();
                        //         }
                        //         "word" => {
                        //             todo!();
                        //         }
                        //         "dword" => {
                        //             todo!();
                        //         }
                        //         "string" => {
                        //             todo!()
                        //         }
                        //         "ascii" => {
                        //             todo!()
                        //         }
                        //         _ => Err(AssemblerError::from_token(
                        //             format!("Unknown directive '{}'", directive_str),
                        //             &directive,
                        //         )),
                        //     }
                        // } else {
                        //     Err(AssemblerError::from_token(
                        //         "Invalid token, expected directive after '.'".into(),
                        //         &directive,
                        //     ))
                        // }?;
                    }

                    // Check for instruction
                    if let TokenKind::Symbol(instr) = token.kind {
                        let instruction = token;

                        // Parse instruction
                        ISA::from_str(instr).map_err(|e| {
                            AssemblerError::from_token(
                                format!("Invalid instruction {}", instr),
                                &instruction,
                            )
                        })?;

                        // Instructions are 4 bytes
                        offset += 4;
                    }
                    Ok(())
                })(&mut token);

                // Consume rest of the line
                if token.kind != TokenKind::Newline {
                    match consume_line(&mut lexer) {
                        Ok(_) => {}
                        Err(err) => errors.push(err),
                    };
                }
            }
            Err(err) => {
                errors.push(err);
                continue;
            }
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(AssembledProgram {
            instruction_memory: BTreeMap::new(),
            data_memory: BTreeMap::new(),
            labels: symbol_table
                .iter()
                .map(|(k, v)| (k.clone(), v.try_into().expect("")))
                .collect(),
            source_map: BiBTreeMap::new(),
            data_labels: symbol_table
                .iter()
                .map(|(k, v)| (k.clone(), v.try_into().expect("")))
                .collect(),
        })
    }
}

fn resolve_label(
    label: &String,
    labels: &HashMap<String, (Vec<RPN>, Token)>,
    resolved_labels: &mut HashMap<String, IBig>,
    visited: &mut HashSet<String>,
) -> Result<IBig, Result<AssemblerError, (String, String)>> {
    Ok(if let Some(val) = resolved_labels.get(label) {
        val.clone()
    } else {
        if visited.contains(label) {
            return Err(Ok(AssemblerError::from_token(
                format!("Recursive loop found while resolving {}", label),
                &labels[label].1,
            )));
        }

        visited.insert(label.clone());
        let mut stack = Vec::new();
        let (expression, token) = labels.get(label).ok_or(Err((
            format!("Symbol {} not defined.", label),
            label.clone(),
        )))?;
        for rpn in expression {
            match &rpn.kind {
                RPNKind::Integer(val) => stack.push(val.clone()),
                RPNKind::Variable(name) => {
                    stack.push(resolve_label(&name, labels, resolved_labels, visited)?)
                }
                RPNKind::Add => {
                    let a = stack.pop().ok_or(Ok(AssemblerError::from_token(
                        "Empty stack".into(),
                        &rpn.token,
                    )))?;
                    let b = stack.pop().ok_or(Ok(AssemblerError::from_token(
                        "Empty stack".into(),
                        &rpn.token,
                    )))?;
                    stack.push(a + b);
                }
                _ => todo!(),
            }
        }

        let first = &expression.first().expect("Expression is not empty").token;
        let last = &expression.last().expect("Expression is not empty").token;
        let value = stack.pop().ok_or(Ok(AssemblerError::new(
            "Empty stack".into(),
            first.line,
            first.column,
            last.column + last.width - first.column,
        )))?;
        resolved_labels.insert(label.clone(), value.clone());
        value
    })
}
