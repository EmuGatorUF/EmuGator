use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::Write,
    iter::Peekable,
    mem::replace,
    str::FromStr,
};

use assembled_program::Address;
use bimap::BiBTreeMap;
use dioxus::html::symbol;
use ibig::IBig;
use lexer::{Lexer, Token, TokenKind};
use peeking_take_while::PeekableExt;
use rpn::{Expression, RPN, RPNKind};

use crate::{
    bits,
    isa::{
        ISA, Instruction, InstructionBuildError, InstructionBuildErrorType, InstructionDefinition,
        InstructionFormat, Operands,
    },
    utils::IBigLittleEndianIterator,
};

pub use assembled_program::{AssembledProgram, Section};
pub use assembler_error::AssemblerError;

#[cfg(test)]
mod tests;

mod assembled_program;
mod assembler_error;
mod lexer;
mod rpn;

fn consume_line<'a>(
    token: &mut Token<'a>,
    lexer: &mut Peekable<Lexer<'a>>,
) -> Result<Vec<Token<'a>>, AssemblerError> {
    let parts = lexer
        .peeking_take_while(|token_result| {
            token_result
                .as_ref()
                .is_ok_and(|token| token.kind != TokenKind::Newline)
        })
        // Safe to unwrap because we know the tokens are Ok
        .map(|token_result| token_result.unwrap())
        .collect();

    // lexer.next() is either None, Newline, or Err(_) at this point
    if let Some(Err(e)) = lexer.next_if(|token_result| token_result.is_err()) {
        return Err(e);
    } else if let Some(Ok(next_token)) = lexer.next_if(|token_result| token_result.is_ok()) {
        *token = next_token;
    }

    Ok(parts)
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

fn parse_expression<'a>(lexer: &mut Peekable<Lexer<'a>>) -> Result<Expression<'a>, AssemblerError> {
    Expression::shunting_yard(
        &mut lexer
            .peeking_take_while(|token_result| {
                token_result.as_ref().is_ok_and(|token| {
                    token.kind != TokenKind::Newline && token.kind != TokenKind::Comma
                })
            })
            // Safe to unwrap because we know the tokens are Ok
            .map(|token_result| token_result.unwrap()),
    )
}

enum Directive<'a> {
    Data(Vec<u8>, u32), // Data, Length, Alignment
    Alignment(u32),
    Symbol(String, (Expression<'a>, Token<'a>)),
    Section(Section, (Expression<'a>, Token<'a>)),
}

fn parse_directive<'a>(
    token: &mut Token<'a>,
    lexer: &mut Peekable<Lexer<'a>>,
    symbol_table: Option<&HashMap<String, Address>>,
) -> Result<Option<Directive<'a>>, AssemblerError> {
    if token.kind == TokenKind::Dot {
        *token = lexer.next().ok_or(AssemblerError::from_token(
            "Unexpected EOF after '.'".into(),
            token,
        ))??;
        if let TokenKind::Symbol(directive_str) = token.kind {
            let output = match directive_str {
                "equ" => {
                    if let Some(Ok(
                        symbol @ Token {
                            kind: TokenKind::Symbol(name),
                            ..
                        },
                    )) = lexer.next_if(|next| {
                        matches!(
                            next,
                            Ok(Token {
                                kind: TokenKind::Symbol(_),
                                ..
                            })
                        )
                    }) {
                        if is_kind(lexer.peek(), TokenKind::Comma) {
                            lexer.next() // Skip comma
                        } else {
                            return Err(AssemblerError::from_token(
                                "Expected comma after symbol in '.equ' directive.".into(),
                                token,
                            ));
                        };

                        let expression = parse_expression(lexer)?;

                        Directive::Symbol(name.into(), (expression, symbol))
                    } else {
                        return Err(AssemblerError::from_token(
                            "Expected symbol after '.equ' directive.".into(),
                            &token,
                        ));
                    }
                }
                "data" | "text" | "section" => {
                    let section_str = if directive_str == "section" {
                        if let Some(Ok(Token {
                            kind: TokenKind::Symbol(section_str),
                            ..
                        })) = lexer.next_if(|token_result| {
                            matches!(
                                token_result,
                                Ok(Token {
                                    kind: TokenKind::Symbol(_),
                                    ..
                                })
                            )
                        }) {
                            section_str
                        } else {
                            return Err(AssemblerError::from_token(
                                "Expected section name after '.section' directive.".into(),
                                &token,
                            ));
                        }
                    } else {
                        directive_str
                    };

                    let expression = parse_expression(lexer)?;

                    let expression = if expression.len() > 0 {
                        expression
                    } else {
                        vec![RPN {
                            kind: RPNKind::Integer(0.into()),
                            token: token.clone(),
                        }]
                        .into()
                    };

                    Directive::Section(section_str.into(), (expression, token.clone()))
                }
                "byte" | "2byte" | "4byte" | "8byte" | "half" | "word" | "dword" => {
                    let (width, alignment) = match directive_str {
                        "byte" => (1, 1),
                        "2byte" => (2, 1),
                        "4byte" => (4, 1),
                        "8byte" => (8, 1),
                        "half" => (2, 2),
                        "word" => (4, 4),
                        "dword" => (8, 8),
                        _ => unreachable!(),
                    };
                    let mut data = vec![];

                    loop {
                        data.push(parse_expression(lexer)?);
                        if is_kind(lexer.peek(), TokenKind::Comma) {
                            lexer.next(); // Skip comma
                        } else {
                            break;
                        }
                    }

                    // lexer.peek() is either None, Newline(#comment), or Comma
                    while is_kind(lexer.peek(), TokenKind::Comma) {
                        lexer.next(); // Skip comma
                        data.push(parse_expression(lexer)?);
                    }

                    let data = if let Some(symbol_table) = symbol_table {
                        let ibig: Vec<_> = data
                            .into_iter()
                            .map(|expression| {
                                let expression_err =
                                    AssemblerError::from_expression("".into(), &expression);
                                (
                                    expression.evaluate(|name| {
                                        symbol_table
                                            .get(name)
                                            .ok_or(AssemblerError {
                                                error_message: format!(
                                                    "Symbol {} not defined.",
                                                    name
                                                ),
                                                ..expression_err.clone()
                                            })
                                            .cloned()
                                    }),
                                    expression_err,
                                )
                            })
                            .collect();

                        let mut data = vec![];
                        for (value, expression_err) in ibig.into_iter() {
                            let value = value?;

                            let mut bytes: Vec<_> =
                                IBigLittleEndianIterator::from(&value.1).collect();

                            if bytes.len() > width {
                                return Err(AssemblerError {
                                    error_message: format!(
                                        "Value {} is too large for {} bytes.",
                                        value, width
                                    ),
                                    ..expression_err
                                });
                            } else {
                                let pad = if value.1 >= 0.into() { 0x00 } else { 0xFF };
                                bytes.resize(width, pad);
                                data.extend(bytes);
                            }
                        }

                        data
                    } else {
                        vec![0u8; data.len() * width]
                    };

                    // Return bytes
                    Directive::Data(data, 1)
                }
                "ascii" | "asciz" | "string" => {
                    let mut data = vec![];

                    loop {
                        let string = lexer.next().ok_or(AssemblerError::from_token(
                            format!("Unexpected EOF after '.{}' directive.", directive_str),
                            token,
                        ))??;

                        if let TokenKind::StrLiteral(_, c) = string.kind {
                            // Safe to unwrap because Vec::write() is infallible
                            data.write(c.as_bytes()).unwrap();
                            if matches!(directive_str, "asciz" | "string") {
                                data.push(0u8);
                            }
                        } else {
                            // Error: Invalid string
                            return Err(AssemblerError::from_token(
                                format!(
                                    "Expected string [, string ...] literal after .{} directive.",
                                    directive_str
                                ),
                                &string,
                            ));
                        }

                        if is_kind(lexer.peek(), TokenKind::Comma) {
                            lexer.next(); // Skip comma
                        } else {
                            break;
                        }
                    }

                    // Return string length
                    Directive::Data(data, 1)
                }
                _ => {
                    return Err(AssemblerError::from_token(
                        format!("Unknown directive '{}'", directive_str),
                        token,
                    ));
                }
            };

            *token = lexer.next().ok_or(AssemblerError::from_token(
                "Unexpected EOF after directive, expected newline.".into(),
                &token,
            ))??;

            Ok(Some(output))
        } else {
            Err(AssemblerError::from_token(
                "Invalid token, expected directive after '.'".into(),
                &token,
            ))
        }
    } else {
        Ok(None)
    }
}

fn parse_instruction<'a>(
    token: &mut Token<'a>,
    lexer: &mut Peekable<Lexer<'a>>,
    symbol_table: &HashMap<String, Address>,
    current_address: u32,
) -> Result<Option<(Instruction, Token<'a>)>, AssemblerError> {
    if let TokenKind::Symbol(instr) = token.kind {
        let instruction_token = token.clone();
        let parts = consume_line(token, lexer)?;

        // Parse instruction
        let def = ISA::from_str(&instr.to_uppercase())
            .map_err(|e| {
                AssemblerError::from_token(format!("Invalid instruction {}", instr), &token)
            })?
            .definition();

        let (operands, (rd_token, rs1_token, rs2_token, imm_expression)) =
            match (def.format, &parts.as_slice()) {
                // Special syntax for ECALL and EBREAK
                // ECALL
                (InstructionFormat::I, &[]) if def.opcode == ISA::ECALL.definition().opcode => (
                    Operands {
                        imm: match instr.to_uppercase().as_str() {
                            "ECALL" => 0,
                            "EBREAK" => 1,
                            _ => unreachable!(),
                        },
                        ..Default::default()
                    },
                    (None, None, None, None),
                ),
                // Special syntax for FENCE
                // FENCE
                (InstructionFormat::I, &[]) if def.opcode == ISA::FENCE.definition().opcode => (
                    Operands {
                        ..Default::default()
                    },
                    (None, None, None, None),
                ),
                // Register-relative Load and Store instructions
                // LB rd, imm(rs1)
                (
                    InstructionFormat::I | InstructionFormat::S,
                    &[
                        other_token @ Token {
                            kind: TokenKind::Symbol(other),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        imm_expression @ ..,
                        Token {
                            kind: TokenKind::LParenthesis,
                            ..
                        },
                        rs1_token @ Token {
                            kind: TokenKind::Symbol(rs1),
                            ..
                        },
                        Token {
                            kind: TokenKind::RParenthesis,
                            ..
                        },
                    ],
                ) if def.opcode == ISA::LB.definition().opcode
                    || def.opcode == ISA::SB.definition().opcode =>
                {
                    let other = parse_register(other)
                        .map_err(|e| AssemblerError::from_token(e, &other_token))?;
                    let rs1 = parse_register(rs1)
                        .map_err(|e| AssemblerError::from_token(e, &rs1_token))?;
                    let imm = parse_immediate(imm_expression, &def, symbol_table, current_address)?;

                    if def.format == InstructionFormat::S {
                        // S-type
                        (
                            Operands {
                                rs1,
                                rs2: other,
                                imm,
                                ..Default::default()
                            },
                            (
                                None,
                                Some(rs1_token),
                                Some(other_token),
                                Some(imm_expression),
                            ),
                        )
                    } else {
                        // I-type
                        (
                            Operands {
                                rd: other,
                                rs1,
                                imm,
                                ..Default::default()
                            },
                            (
                                Some(other_token),
                                Some(rs1_token),
                                None,
                                Some(imm_expression),
                            ),
                        )
                    }
                }
                // Register-register arithmetic instructions
                // ADD rd, rs1, rs2
                (
                    InstructionFormat::R,
                    &[
                        rd_token @ Token {
                            kind: TokenKind::Symbol(rd),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        rs1_token @ Token {
                            kind: TokenKind::Symbol(rs1),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        rs2_token @ Token {
                            kind: TokenKind::Symbol(rs2),
                            ..
                        },
                    ],
                ) => {
                    let rd =
                        parse_register(rd).map_err(|e| AssemblerError::from_token(e, &rd_token))?;
                    let rs1 = parse_register(rs1)
                        .map_err(|e| AssemblerError::from_token(e, &rs1_token))?;
                    let rs2 = parse_register(rs2)
                        .map_err(|e| AssemblerError::from_token(e, &rs2_token))?;

                    (
                        Operands {
                            rd,
                            rs1,
                            rs2,
                            ..Default::default()
                        },
                        (Some(rd_token), Some(rs1_token), Some(rs2_token), None),
                    )
                }
                // Branch instructions
                // BEQ rs1, rs2, address
                (
                    InstructionFormat::B,
                    &[
                        rs1_token @ Token {
                            kind: TokenKind::Symbol(rs1),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        rs2_token @ Token {
                            kind: TokenKind::Symbol(rs2),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        imm_expression @ ..,
                    ],
                ) => {
                    let rs1 = parse_register(rs1)
                        .map_err(|e| AssemblerError::from_token(e, &rs1_token))?;
                    let rs2 = parse_register(rs2)
                        .map_err(|e| AssemblerError::from_token(e, &rs2_token))?;
                    let imm = parse_immediate(imm_expression, &def, symbol_table, current_address)?;

                    (
                        Operands {
                            rs1,
                            rs2,
                            imm,
                            ..Default::default()
                        },
                        (None, Some(rs1_token), Some(rs2_token), Some(imm_expression)),
                    )
                }
                // Register-immediate arithmetic instructions
                // ADDI rd, rs1, imm
                (
                    InstructionFormat::I,
                    &[
                        rd_token @ Token {
                            kind: TokenKind::Symbol(rd),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        rs1_token @ Token {
                            kind: TokenKind::Symbol(rs1),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        imm_expression @ ..,
                    ],
                ) => {
                    let rd =
                        parse_register(rd).map_err(|e| AssemblerError::from_token(e, &rd_token))?;
                    let rs1 = parse_register(rs1)
                        .map_err(|e| AssemblerError::from_token(e, &rs1_token))?;
                    let imm = parse_immediate(imm_expression, &def, symbol_table, current_address)?;

                    (
                        Operands {
                            rd,
                            rs1,
                            imm,
                            ..Default::default()
                        },
                        (Some(rd_token), Some(rs1_token), None, Some(imm_expression)),
                    )
                }
                // Absolute Load and Store instructions
                // LB rd, imm
                (
                    InstructionFormat::I | InstructionFormat::S,
                    &[
                        other_token @ Token {
                            kind: TokenKind::Symbol(other),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        imm_expression @ ..,
                    ],
                ) if def.opcode == ISA::LB.definition().opcode
                    || def.opcode == ISA::SB.definition().opcode =>
                {
                    let other = parse_register(other)
                        .map_err(|e| AssemblerError::from_token(e, &other_token))?;
                    let rs1 = 0;
                    let imm = parse_immediate(imm_expression, &def, symbol_table, current_address)?;

                    if def.format == InstructionFormat::S {
                        // S-type store instructions
                        (
                            Operands {
                                rs1,
                                rs2: other,
                                imm,
                                ..Default::default()
                            },
                            (None, None, Some(other_token), Some(imm_expression)),
                        )
                    } else {
                        // I-type load instructions
                        (
                            Operands {
                                rd: other,
                                rs1,
                                imm,
                                ..Default::default()
                            },
                            (Some(other_token), None, None, Some(imm_expression)),
                        )
                    }
                }
                // Load Upper Immediate and Jump instructions
                // LUI rd, imm
                // JAL rd, address
                (
                    InstructionFormat::U | InstructionFormat::J,
                    &[
                        rd_token @ Token {
                            kind: TokenKind::Symbol(rd),
                            ..
                        },
                        Token {
                            kind: TokenKind::Comma,
                            ..
                        },
                        imm_expression @ ..,
                    ],
                ) => {
                    let rd =
                        parse_register(rd).map_err(|e| AssemblerError::from_token(e, &rd_token))?;
                    let imm = parse_immediate(imm_expression, &def, symbol_table, current_address)?;

                    (
                        Operands {
                            rd,
                            imm,
                            ..Default::default()
                        },
                        (Some(rd_token), None, None, Some(imm_expression)),
                    )
                }
                _ => {
                    return Err(AssemblerError::from_token(
                        format!("Invalid operands for instruction {}", instr),
                        &token,
                    ));
                }
            };

        Ok(Some((
            Instruction::try_from_def_operands(def, operands).map_err(|e| match e.error_type {
                InstructionBuildErrorType::InvalidOpcode
                | InstructionBuildErrorType::InvalidFunct3
                | InstructionBuildErrorType::InvalidFunct7 => {
                    AssemblerError::from_token(e.error_message, &instruction_token)
                }
                InstructionBuildErrorType::InvalidRd => AssemblerError::from_token(
                    e.error_message,
                    rd_token.unwrap_or(&instruction_token),
                ),
                InstructionBuildErrorType::InvalidRs1 => AssemblerError::from_token(
                    e.error_message,
                    rs1_token.unwrap_or(&instruction_token),
                ),
                InstructionBuildErrorType::InvalidRs2 => AssemblerError::from_token(
                    e.error_message,
                    rs2_token.unwrap_or(&instruction_token),
                ),
                InstructionBuildErrorType::InvalidImm => match imm_expression {
                    Some([first, rest @ ..]) => AssemblerError {
                        error_message: e.error_message,
                        line_number: first.line,
                        column: first.column,
                        width: rest
                            .iter()
                            .fold(first.width, |acc, token| acc + token.width),
                    },
                    _ => AssemblerError::from_token(e.error_message, &instruction_token),
                },
            })?,
            instruction_token,
        )))
    } else {
        Ok(None)
    }
}

fn parse_register(reg: &str) -> Result<u32, String> {
    let reg = reg.to_lowercase();
    if !reg.starts_with('x') {
        return Err(format!("Invalid register (must start with 'x'): {}", reg));
    }

    match reg[1..].parse::<u32>() {
        Ok(num) if num < 32 => Ok(num),
        _ => Err(format!("Invalid register number (must be 0-31): {}", reg)),
    }
}

fn parse_immediate(
    imm: &[Token],
    def: &InstructionDefinition,
    symbol_table: &HashMap<String, Address>,
    current_address: u32,
) -> Result<i32, AssemblerError> {
    let expression = Expression::shunting_yard(&mut imm.iter().cloned())?;
    let expression_err = AssemblerError::from_expression("".into(), &expression);
    let imm = expression
        .evaluate(|name| {
            symbol_table
                .get(name)
                .ok_or(AssemblerError {
                    error_message: format!("Symbol {} not defined.", name),
                    ..expression_err.clone()
                })
                .cloned()
        })?
        .1
        .try_into()
        .map_err(|e| AssemblerError {
            error_message: format!("Invalid immediate value."),
            ..expression_err
        })?;

    match def.format {
        InstructionFormat::I | InstructionFormat::S => {
            if imm > 2047 || imm < -2048 {
                Err(AssemblerError {
                    error_message: format!(
                        "Immediate value {} is out of range (-2048 to 2047)",
                        imm
                    ),
                    ..expression_err
                })
            } else {
                Ok(imm)
            }
        }
        InstructionFormat::U => {
            let imm = imm as u32;
            if imm > 0xFFFFF {
                Err(AssemblerError {
                    error_message: format!(
                        "Immediate value {} is out of range (0 to 0xFFFFF)",
                        imm
                    ),
                    ..expression_err
                })
            } else {
                Ok((imm << 12) as i32)
            }
        }
        InstructionFormat::J => {
            let offset = imm - current_address as i32;

            if bits!(offset, 0) != 0 {
                Err(AssemblerError {
                    error_message: format!("Jump offset {} must be 2-byte aligned.", offset),
                    ..expression_err
                })
            } else if offset > 0xFFFFF || offset < -0x100000 {
                Err(AssemblerError {
                    error_message: format!(
                        "Jump target is too far ({} bytes), must be within -1048576 to +1048575 bytes",
                        offset
                    ),
                    ..expression_err
                })
            } else {
                Ok(offset)
            }
        }
        InstructionFormat::B => {
            let offset = imm - current_address as i32;

            if bits!(offset, 0) != 0 {
                Err(AssemblerError {
                    error_message: format!("Branch offset {} must be 2-byte aligned.", offset),
                    ..expression_err
                })
            } else if offset > 0xFFF || offset < -0x1000 {
                Err(AssemblerError {
                    error_message: format!(
                        "Branch target is too far ({} bytes), must be within -4096 to +4095 bytes",
                        offset
                    ),
                    ..expression_err
                })
            } else {
                Ok(offset)
            }
        }
        InstructionFormat::R => unreachable!(), // R-type instructions should not have immediates
    }
}

fn run_pass<
    'a,
    T: FnMut(&mut Token<'a>, &mut Peekable<Lexer<'a>>) -> Result<(), AssemblerError>,
>(
    lexer: &mut Peekable<Lexer<'a>>,
    mut pass: T,
) -> Vec<AssemblerError> {
    let mut errors = Vec::new();
    while let Some(mut token) = lexer.next() {
        match token {
            Ok(mut token) => {
                let err = pass(&mut token, lexer);

                if let Err(err) = err {
                    errors.push(err);

                    // Consume the rest of the line
                    while let Err(err) = consume_line(&mut token, lexer) {
                        errors.push(err);
                    }
                }

                // Line should be consumed
                if token.kind != TokenKind::Newline {
                    let err = AssemblerError::from_token(
                        "Expected newline at end of line".into(),
                        &token,
                    );
                    errors.push(err);
                };
            }
            Err(err) => {
                errors.push(err);
            }
        }
    }

    errors
}

fn is_kind(token: Option<&Result<Token, AssemblerError>>, token_kind: TokenKind) -> bool {
    if let Some(Ok(Token { kind, .. })) = token {
        *kind == token_kind
    } else {
        false
    }
}

fn insert<'a>(
    symbol_table: &mut HashMap<String, (Option<Section>, Expression<'a>, Token<'a>)>,
    symbol: String,
    entry: (Option<Section>, Expression<'a>, Token<'a>),
) -> Result<(), AssemblerError> {
    let line = entry.2.line;
    match symbol_table.insert(symbol.clone(), entry) {
        Some((_, _, token)) => Err(AssemblerError::from_token(
            format!("Symbol {} redefined at line {}.", symbol, line),
            &token,
        )),
        None => Ok(()),
    }
}

pub fn assemble<'a>(source: &'a str) -> Result<AssembledProgram, Vec<AssemblerError>> {
    let mut errors = Vec::new();

    let mut symbol_table: HashMap<String, (Option<Section>, Expression<'a>, Token<'a>)> =
        std::collections::HashMap::new();
    let first_org: &str = ".section(0,0)".into();
    insert(
        &mut symbol_table,
        first_org.into(),
        (
            Some(Section::Text),
            vec![RPN {
                kind: RPNKind::Integer(0.into()),
                token: Token {
                    kind: TokenKind::IntLiteral("0", 10, 0.into()),
                    line: 1,
                    column: 1,
                    width: 0,
                },
            }]
            .into(),
            Token {
                kind: TokenKind::Symbol(first_org),
                line: 1,
                column: 1,
                width: 0,
            },
        ),
    )
    .expect("There should be no errors inserting the initial section.");

    // First Pass
    let _ = {
        let mut lexer = Lexer::new(source).peekable();
        let mut current_section = Section::Text;
        let mut current_org: String = first_org.into();
        let mut offset: u32 = 0;

        errors.append(&mut run_pass(&mut lexer, |token, lexer| {
            // Check for a label
            let label = parse_label(token, lexer)?;

            // Check for other directives
            let directive = parse_directive(token, lexer, None)?;

            // Handle section directive and label (must be handled together)
            if let Some(Directive::Section(section, (mut expression, token))) = directive {
                offset = 0;
                current_section = section.clone();

                let (org, entry) = if let Some((label, token)) = label {
                    (label.into(), (Some(section), expression, token))
                } else {
                    (
                        format!(".section({},{})", token.line, token.column),
                        (Some(section), expression, token),
                    )
                };
                current_org = org;
                insert(&mut symbol_table, current_org.clone(), entry)?;
            } else {
                if let Some((label, token)) = label {
                    insert(
                        &mut symbol_table,
                        label.into(),
                        (
                            Some(current_section.clone()),
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
                            ]
                            .into(),
                            token,
                        ),
                    )?;
                }

                // Handle the rest of the directives
                if let Some(directive) = directive {
                    match directive {
                        Directive::Alignment(alignment) => {
                            offset = (offset + alignment - 1) & !(alignment - 1);
                        }
                        Directive::Data(data, alignment) => {
                            offset = (offset + alignment - 1) & !(alignment - 1);
                            offset += data.len() as u32;
                        }
                        Directive::Symbol(symbol, entry) => {
                            insert(&mut symbol_table, symbol, (None, entry.0, entry.1))?;
                        }
                        Directive::Section(section, _) => unreachable!(), // Section directives are handled above
                    }
                }
            }

            // Check for instruction
            if let TokenKind::Symbol(instr) = token.kind {
                // Parse instruction
                ISA::from_str(&instr.to_uppercase()).map_err(|e| {
                    AssemblerError::from_token(format!("Invalid instruction {}", instr), token)
                })?;

                consume_line(token, lexer)?;

                // Instructions are 4 bytes
                offset += 4;
            }

            Ok(())
        }));
    };

    // Resolve text labels
    let mut resolved_symbols = HashMap::new();

    let _ = {
        let mut visited = HashSet::new();

        while !symbol_table.is_empty() {
            let (symbol, label_err) = {
                let (symbol, (_, _, label_token)) = symbol_table.iter().next().unwrap();
                (
                    symbol.clone(),
                    AssemblerError::from_token("".into(), label_token),
                )
            };

            if let Err(err) = resolve_label(
                &symbol,
                &mut symbol_table,
                &mut resolved_symbols,
                &mut visited,
            )
            .map_err(|e| {
                if e.line_number == 0 {
                    AssemblerError {
                        error_message: e.error_message,
                        ..label_err
                    }
                } else {
                    e
                }
            }) {
                errors.push(err);
            }
        }
    };

    let symbol_table = resolved_symbols;

    let mut instruction_memory = BTreeMap::new();
    let mut data_memory = BTreeMap::new();
    let mut source_map = BiBTreeMap::new();

    // Second Pass
    let _ = {
        let mut lexer = Lexer::new(source).peekable();
        let mut current_section = Section::Text;
        let mut address: u32 = 0;

        errors.append(&mut run_pass(&mut lexer, |token, lexer| {
            let mut memory = match current_section {
                Section::Text => &mut instruction_memory,
                Section::Data => &mut data_memory,
                _ => todo!(), // TODO: Add support for other sections and user-defined sections
            };

            // Check for a label
            let label = parse_label(token, lexer)?;

            // Check for section directive
            let directive = parse_directive(token, lexer, Some(&symbol_table))?;

            // Handle section directive and label (must be handled together)
            if let Some(Directive::Section(section, (_, token))) = directive {
                let org = if let Some((label, _)) = label {
                    label.into()
                } else {
                    format!(".section({},{})", token.line, token.column)
                };

                current_section = section;
                memory = match current_section {
                    Section::Text => &mut instruction_memory,
                    Section::Data => &mut data_memory,
                    _ => todo!(), // TODO: Add support for other sections and user-defined sections
                };

                address = symbol_table.get(&org).ok_or(AssemblerError::from_token(
                    format!("Symbol {} not defined.", org),
                    &token,
                ))?.1.clone().try_into().map_err(|e| {
                            AssemblerError::from_token(format!("Invalid address."), &token)
                        })?;
            } else {
                if let Some((label, label_token)) = label {
                    if IBig::from(address) != symbol_table[label].1 {
                        return Err(AssemblerError::from_token(
                            format!("Invalid address. Expected {}, got {}. This is probably due to a misaligned .data directive.", symbol_table[label], address),
                            &label_token,
                        ));
                    }
                }

                // Handle the rest of the directives
                if let Some(directive) = directive {
                    match directive {
                        Directive::Alignment(alignment) => {
                            address = (address + alignment - 1) & !(alignment - 1);
                        }
                        Directive::Data(data, alignment) => {
                            address = (address + alignment - 1) & !(alignment - 1);

                            for (i, data) in data.iter().enumerate() {
                                match memory.insert(address + u32::try_from(i).map_err(|_| AssemblerError::from_token("Data too large to fit in memory.".into(), token))?, *data) {
                                    Some(_) => Err(AssemblerError::from_token("Memory collision.".into(), token)),
                                    None => Ok(())
                                }?
                            }

                            address += data.len() as u32;
                        }
                        Directive::Symbol(symbol, entry) => {} // Symbols are already resolved
                        Directive::Section(section, _) => unreachable!(), // Section directives are handled above
                    }
                }
            }

            // Check for instruction
            let instruction = parse_instruction(token, lexer, &symbol_table, address)?;

            if let Some((instruction, instruction_token)) = instruction {
                
                let data = instruction.raw().to_le_bytes();
                
                for (i, data) in data.iter().enumerate() {
                    // Safe to unwrap because we know i < 4 
                    match memory.insert(address + u32::try_from(i).unwrap(), *data) {
                        Some(_) => Err(AssemblerError::from_token("Memory collision.".into(), &instruction_token)),
                        None => Ok(())
                    }?
                }
                source_map.insert(address, instruction_token.line);
                
                // Instructions are 4 bytes
                address += 4;
            }

            Ok(())
        }));
    };

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(AssembledProgram {
            instruction_memory,
            data_memory,
            source_map,
            symbol_table,
        })
    }
}

fn resolve_label(
    label: &String,
    labels: &mut HashMap<String, (Option<Section>, Expression, Token)>,
    resolved_labels: &mut HashMap<String, Address>,
    visited: &mut HashSet<String>,
) -> Result<Address, AssemblerError> {
    Ok(if let Some(val) = resolved_labels.get(label) {
        val.clone()
    } else {
        if visited.contains(label) {
            return Err(AssemblerError {
                error_message: format!("Recursive loop found while resolving {}", label),
                line_number: 0,
                column: 0,
                width: 0,
            });
        }

        visited.insert(label.clone());

        let (section, expression, token) = labels.remove(label).ok_or(AssemblerError {
            error_message: format!("Label {} not found.", label),
            line_number: 0,
            column: 0,
            width: 0,
        })?;

        let value =
            expression.evaluate(|name| resolve_label(name, labels, resolved_labels, visited))?;
        resolved_labels.insert(
            label.clone(),
            if let Some(section) = section {
                Address(section, value.1.clone())
            } else {
                value.clone()
            },
        );

        value
    })
}
