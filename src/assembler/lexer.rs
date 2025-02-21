use super::AssemblerError;
use std::iter::Enumerate;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenKind<'a> {
    Newline,

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Ampersand,
    Pipe,
    Caret,
    LParenthesis,
    RParenthesis,

    // Literals
    IntLiteral(&'a str, u32, i32),
    ChrLiteral(&'a str, char),
    StrLiteral(&'a str, String),

    // Keywords
    Dot,
    Comma,
    Colon,
    // Comment(&'a str),
    Symbol(&'a str),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: usize,
    pub column: usize,
    pub width: usize,
}

#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    char_iter: std::iter::Peekable<Enumerate<std::str::Chars<'a>>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            char_iter: source.chars().enumerate().peekable(),
            line: 1,
            column: 0,
        }
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        self.column += 1;
        self.char_iter.next()
    }

    fn skip_whitespace(&mut self) {
        loop {
            let next_char = self.char_iter.peek();
            match next_char {
                Some((_, c)) => match c {
                    ' ' | '\t' => {
                        self.next_char();
                    }
                    _ => break,
                },
                None => break,
            }
        }
    }

    fn parse_string(literal: &'a str) -> String {
        let mut out = String::new();
        let mut str_iter = literal.chars();

        while let Some(c) = str_iter.next() {
            out.push(match c {
                '\\' => match str_iter.next() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some(escaped_c) => escaped_c,
                    None => unreachable!(),
                },
                _ => c,
            });
        }

        out
    }

    fn parse_char(literal: &'a str) -> char {
        let parsed_str = Self::parse_string(literal);

        let mut iter = parsed_str.chars();
        let c = iter.next().expect("String was empty!");
        assert!(iter.next().is_none(), "String was too long!");

        c
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, AssemblerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let output =
            self.next_char().and_then(|(i, c)| {
                Some((|| {
                    Ok(match c {
                        '\r' | '\n' | ';' | '#' => {
                            // Comment/Newline tokens
                            let mut end = i;
                            let line = self.line;
                            let token_col = self.column;

                            let mut c = c;
                            loop {
                                if c == '\n' {
                                    break;
                                }

                                if let Some((j, next_c)) = self.char_iter.peek() {
                                    end = *j;
                                    c = *next_c;
                                    self.next_char();
                                } else {
                                    break;
                                }
                            }
                            self.line += 1;
                            self.column = 0;

                            Token {
                                kind: TokenKind::Newline,
                                line: line,
                                column: token_col,
                                width: end - i,
                            }
                        }
                        '+' => Token {
                            kind: TokenKind::Plus,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '-' => Token {
                            kind: TokenKind::Minus,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '*' => Token {
                            kind: TokenKind::Asterisk,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '/' => Token {
                            kind: TokenKind::Slash,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '&' => Token {
                            kind: TokenKind::Ampersand,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '|' => Token {
                            kind: TokenKind::Pipe,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '^' => Token {
                            kind: TokenKind::Caret,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '(' => Token {
                            kind: TokenKind::LParenthesis,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        ')' => Token {
                            kind: TokenKind::RParenthesis,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '.' => Token {
                            kind: TokenKind::Dot,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        ',' => Token {
                            kind: TokenKind::Comma,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        ':' => Token {
                            kind: TokenKind::Colon,
                            line: self.line,
                            column: self.column,
                            width: 1,
                        },
                        '0'..='9' => {
                            // Int token
                            let mut end = i;
                            let token_col = self.column;

                            // Detect base
                            let base = if c == '0' {
                                let (j, b) = self.char_iter.peek().ok_or(AssemblerError::new(
                                    "Unexpected EOF while parsing string".to_string(),
                                    self.line,
                                    token_col,
                                    1,
                                ))?;
                                match b {
                                    'x' | 'X' => {
                                        self.next_char();
                                        self.next_char();
                                        16
                                    }
                                    'b' | 'B' => {
                                        self.next_char();
                                        self.next_char();
                                        2
                                    }
                                    '0'..='9' => {
                                        self.next_char();
                                        8
                                    }
                                    _ => 10, // This is a single digit 0
                                }
                            } else {
                                10
                            };

                            let start = if let Some((j, c)) = self.char_iter.peek() {
                                *j - 1
                            } else {
                                i
                            };

                            while let Some((j, c)) = self.char_iter.peek() {
                                end = *j;
                                if *c == '_' {
                                } else if c.is_whitespace() || !c.is_alphanumeric() {
                                    break;
                                } else if !c.is_digit(base) {
                                    return Err(AssemblerError::new(
                                        format!(
                                            "Invalid character '{}' in int literal of base {}",
                                            c, base
                                        ),
                                        self.line,
                                        token_col + (end - i),
                                        1,
                                    ));
                                    break;
                                }
                                self.next_char();
                            }

                            let literal = &self.source[i..end];
                            let value = i32::from_str_radix(
                                &*self.source[start..end].replace("_", ""),
                                base,
                            )
                            .map_err(|e| {
                                AssemblerError::new(e.to_string(), self.line, token_col, end - i)
                            })?;
                            Token {
                                kind: TokenKind::IntLiteral(literal, base, value),
                                line: self.line,
                                column: token_col,
                                width: literal.len(),
                            }
                        }
                        '\'' => {
                            // Char token
                            let mut end = i;
                            let token_col = self.column;

                            loop {
                                if let Some((j, c)) = self.next_char() {
                                    end = j;
                                    if c == '\\' {
                                        self.next_char();
                                    } else if c == '\'' {
                                        break;
                                    }
                                } else {
                                    return Err(AssemblerError::new(
                                        "Unexpected EOF while parsing string".to_string(),
                                        self.line,
                                        token_col,
                                        end - i,
                                    ));
                                }
                            }

                            let literal = &self.source[i..end + 1];

                            Token {
                                kind: TokenKind::ChrLiteral(
                                    literal,
                                    Self::parse_char(&literal[1..literal.len() - 1]),
                                ),
                                line: self.line,
                                column: token_col,
                                width: literal.len(),
                            }
                        }
                        '"' => {
                            // String token
                            let mut end = i;
                            let token_col = self.column;

                            loop {
                                if let Some((j, c)) = self.next_char() {
                                    end = j;
                                    if c == '\\' {
                                        self.next_char();
                                    } else if c == '"' {
                                        break;
                                    }
                                } else {
                                    return Err(AssemblerError::new(
                                        "Unexpected EOF while parsing string".to_string(),
                                        self.line,
                                        token_col,
                                        end - i,
                                    ));
                                }
                            }

                            let literal = &self.source[i..end + 1];

                            Token {
                                kind: TokenKind::StrLiteral(
                                    literal,
                                    Self::parse_string(&literal[1..literal.len() - 1]),
                                ),
                                line: self.line,
                                column: token_col,
                                width: literal.len(),
                            }
                        }
                        _ => {
                            // Symbol token
                            let mut end = i;
                            let token_col = self.column;

                            while let Some((j, c)) = self.char_iter.peek() {
                                end = *j;
                                if !c.is_ascii_alphanumeric() && *c != '_' && *c != '.' {
                                    break;
                                }
                                self.next_char();
                            }

                            Token {
                                kind: TokenKind::Symbol(&self.source[i..end]),
                                line: self.line,
                                column: token_col,
                                width: end - i,
                            }
                        }
                    })
                })())
            });

        if let Some(Err(error)) = &output {
            // Consume the rest of the line if there was an error
            while let Some((_, c)) = self.next_char() {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                    break;
                } else if c == ';' {
                    break;
                }
            }
        };

        output
    }
}
