use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    error,
    f32::consts::E,
    io::Write,
    iter::Peekable,
    mem::replace,
    ops::Deref,
    str::FromStr,
};

use bimap::BiBTreeMap;
use dioxus::html::{g::direction, geometry::ElementSpace, symbol};
use ibig::{IBig, error::OutOfBoundsError};
use peeking_take_while::PeekableExt;

use crate::assembler::lexer::{Lexer, Token, TokenKind};
use crate::isa::{ISA, Instruction, InstructionFormat, Operands};

use super::{AssembledProgram, AssemblerError, Section};
use crate::bits;

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

pub struct Expression<'a>(Vec<RPN<'a>>);

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

impl<'a> Expression<'a> {
    fn evaluate<F: FnMut(&String) -> Result<&'a IBig, AssemblerError>>(
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
}

pub struct RPN<'a> {
    pub kind: RPNKind,
    pub token: Token<'a>,
}

struct IBigLittleEndianIterator<'a> {
    value: &'a IBig,
    index: usize,
}

impl<'a> From<&'a IBig> for IBigLittleEndianIterator<'a> {
    fn from(value: &'a IBig) -> Self {
        Self { value, index: 0 }
    }
}

impl Iterator for IBigLittleEndianIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        // Check for negative numbers
        let shifted = self.value >> (self.index * 8);

        if (shifted == 0.into() || shifted == (-1).into()) && self.index != 0 {
            return None;
        }

        let byte: u8 = (shifted & 0xFFu8);

        self.index += 1;
        Some(byte)
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
        match token.kind {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Subtract),
            TokenKind::Asterisk => Ok(Self::Multiply),
            TokenKind::Slash => Ok(Self::Divide),
            TokenKind::LParenthesis => Ok(Self::LParenthesis),
            TokenKind::RParenthesis => Ok(Self::RParenthesis),
            TokenKind::IntLiteral(_, _, val) => Ok(Self::Integer(val.try_into().map_err(|e| {
                AssemblerError::from_token(format!("Invalid integer value: {}", e), token)
            })?)),
            TokenKind::ChrLiteral(_, c) => Ok(Self::Integer((c as u32).into())),
            TokenKind::Symbol(name) => Ok(Self::Variable(name.into())),
            _ => Err(AssemblerError::from_token(
                "Invalid token encountered".into(),
                token,
            )),
        }
    }
}

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

fn shunting_yard<'a>(
    tokens: &mut dyn Iterator<Item = Token<'a>>,
) -> Result<Expression<'a>, AssemblerError> {
    let mut output = VecDeque::new();
    let mut op_stack: VecDeque<RPN<'a>> = VecDeque::new();

    for token in tokens {
        match &token.kind {
            TokenKind::Symbol(_) | TokenKind::IntLiteral(_, _, _) | TokenKind::ChrLiteral(_, _) => {
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

enum Directive<'a> {
    Data(Vec<u8>, u32), // Data, Length, Alignment
    Alignment(u32),
    Symbol(String, (Expression<'a>, Token<'a>)),
    Section(Section, (Expression<'a>, Token<'a>)),
}

fn parse_directive<'a>(
    token: &mut Token<'a>,
    lexer: &mut Peekable<Lexer<'a>>,
    symbol_table: Option<&HashMap<String, IBig>>,
) -> Result<Option<Directive<'a>>, AssemblerError> {
    if token.kind == TokenKind::Dot {
        *token = lexer.next().ok_or(AssemblerError::from_token(
            "Unexpected EOF after '.'".into(),
            token,
        ))??;
        if let TokenKind::Symbol(directive_str) = token.kind {
            let output = match directive_str {
                "data" | "text" => {
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

                    let section = if directive_str == "text" {
                        Section::Text
                    } else {
                        Section::Data
                    };

                    Directive::Section(section, (expression, token.clone()))
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
                                (
                                    expression.evaluate(|name| {
                                        symbol_table.get(name).ok_or(AssemblerError::from_token(
                                            format!("Symbol {} not defined.", name),
                                            &expression[0].token,
                                        ))
                                    }),
                                    expression,
                                )
                            })
                            .collect();

                        let mut data = vec![];
                        for (value, expression) in ibig.into_iter() {
                            let value = value?;

                            let mut bytes: Vec<_> =
                                IBigLittleEndianIterator::from(&value).collect();

                            if bytes.len() > width {
                                return Err(AssemblerError::from_expression(
                                    format!("Value {} is too large for {} bytes.", value, width),
                                    &expression,
                                ));
                            } else {
                                let pad = if (*bytes.last().expect("Bytes is not empty.") as i8) < 0 { 0xFF } else { 0x00 };
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
    symbol_table: &HashMap<String, IBig>,
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

        match &parts.as_slice() {
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
                imm @ ..,
            ] => {
                let rest = &parts[4..];

                let rd =
                    parse_register(rd).map_err(|e| AssemblerError::from_token(e, &rd_token))?;
                let rs1 =
                    parse_register(rs1).map_err(|e| AssemblerError::from_token(e, &rs1_token))?;
                let imm = shunting_yard(&mut imm.iter().cloned())?;

                let operands = Operands {
                    rd,
                    rs1,
                    imm: imm
                        .evaluate(|name| {
                            symbol_table.get(name).ok_or(AssemblerError::from_token(
                                format!("Symbol {} not defined.", name),
                                &imm[0].token,
                            ))
                        })?
                        .try_into()
                        .map_err(|e| {
                            AssemblerError::from_expression(
                                format!("Invalid immediate value."),
                                &imm,
                            )
                        })?,
                    rs2: 0u32,
                };

                Ok(Some((Instruction::from_def_operands(def, operands), instruction_token)))
            }
            _ => todo!(),
        }
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

pub fn assemble<'a>(source: &'a str) -> Result<AssembledProgram, Vec<AssemblerError>> {
    let mut errors = Vec::new();

    let mut symbol_table: HashMap<String, (Expression, Token)> = std::collections::HashMap::new();
    symbol_table.insert(
        "!org(0,0)".into(),
        (
            vec![RPN {
                kind: RPNKind::Integer(0.into()),
                token: Token {
                    kind: TokenKind::IntLiteral("0", 10, 0),
                    line: 1,
                    column: 1,
                    width: 0,
                },
            }]
            .into(),
            Token {
                kind: TokenKind::Symbol("!org(0,0)"),
                line: 1,
                column: 1,
                width: 0,
            },
        ),
    );

    // First Pass
    let _ = {
        let mut lexer = Lexer::new(source).peekable();
        let mut current_section = Section::Text;
        let mut current_org: String = "!org(0,0)".into();
        let mut offset: u32 = 0;

        errors.append(&mut run_pass(&mut lexer, |token, lexer| {
            // Check for a label
            let label = parse_label(token, lexer)?;

            // Check for other directives
            let directive = parse_directive(token, lexer, None)?;

            // Handle section directive and label (must be handled together)
            if let Some(Directive::Section(section, (expression, token))) = directive {
                offset = 0;
                current_section = section;
                let (org, entry) = if let Some((label, token)) = label {
                    (label.into(), (expression, token))
                } else {
                    (
                        format!("!org({},{})", token.line, token.column),
                        (expression, token),
                    )
                };
                current_org = org;
                symbol_table.insert(current_org.clone(), entry);
            } else {
                if let Some((label, token)) = label {
                    symbol_table.insert(
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
                            ]
                            .into(),
                            token,
                        ),
                    );
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
                            symbol_table.insert(symbol, entry);
                        }
                        Directive::Section(section, _) => unreachable!(),
                    }
                }
            }

            // Check for instruction
            if let TokenKind::Symbol(instr) = token.kind {
                // Parse instruction
                ISA::from_str(instr).map_err(|e| {
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
        for (symbol, _) in symbol_table.iter() {
            if let Err(err) =
                resolve_label(symbol, &symbol_table, &mut resolved_symbols, &mut visited).map_err(
                    |e| match e {
                        Ok(e) => e,
                        Err((msg, label)) => AssemblerError::from_token(
                            msg,
                            &symbol_table.get(&label).expect("Symbol not found").1,
                        ),
                    },
                )
            {
                errors.push(err);
            }
        }
    };

    let symbol_table = resolved_symbols;

    for (symbol, value) in symbol_table.iter() {
        println!("{}: {}", symbol, value);
    }

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
                    format!("!org({},{})", token.line, token.column)
                };

                current_section = section;
                memory = match current_section {
                    Section::Text => &mut instruction_memory,
                    Section::Data => &mut data_memory,
                };

                address = symbol_table.get(&org).ok_or(AssemblerError::from_token(
                    format!("Symbol {} not defined.", org),
                    &token,
                ))?.clone().try_into().map_err(|e| {
                            AssemblerError::from_token(format!("Invalid address."), &token)
                        })?;
            } else {
                if let Some((label, label_token)) = label {
                    if IBig::from(address) != symbol_table[label] {
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
                                memory.insert(address + u32::try_from(i).expect("Data too large to fit in memory."), *data);
                            }

                            address += data.len() as u32;
                        }
                        Directive::Symbol(symbol, entry) => {
                        }
                        Directive::Section(section, _) => unreachable!(),
                    }
                }
            }

            // Check for instruction
            let instruction = parse_instruction(token, lexer, &symbol_table)?;

            if let Some((instruction, instruction_token)) = instruction {
                
                let data = instruction.raw().to_le_bytes();
                for (i, data) in data.iter().enumerate() {
                    memory.insert(address + u32::try_from(i).unwrap(), *data);
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
            labels: symbol_table
                .iter()
                .map(|(k, v)| (k.clone(), v.try_into().expect("")))
                .collect(),
            source_map,
            data_labels: symbol_table
                .iter()
                .map(|(k, v)| (k.clone(), v.try_into().expect("")))
                .collect(),
        })
    }
}

fn resolve_label(
    label: &String,
    labels: &HashMap<String, (Expression, Token)>,
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

        for rpn in expression.iter() {
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

        let value = stack.pop().ok_or(Ok(AssemblerError::from_expression(
            "Invalid expression.".into(),
            expression,
        )))?;
        resolved_labels.insert(label.clone(), value.clone());
        value
    })
}
