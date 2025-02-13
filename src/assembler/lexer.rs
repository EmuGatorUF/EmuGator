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
    IntLiteral(&'a str, u32, usize),
    ChrLiteral(&'a str, char),
    StrLiteral(&'a str, String),

    // Keywords
    Dot,
    Comma,
    Colon,
    Comment(&'a str),
    Symbol(&'a str),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: usize,
    pub col: usize,
}

impl<'a> Token<'a> {
    pub fn width(&self) -> usize {
        match &self.kind {
            TokenKind::Newline
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Ampersand
            | TokenKind::Pipe
            | TokenKind::Caret
            | TokenKind::LParenthesis
            | TokenKind::RParenthesis
            | TokenKind::Dot
            | TokenKind::Comma
            | TokenKind::Colon => 1,
            TokenKind::IntLiteral(s, _, _) => s.len(),
            TokenKind::ChrLiteral(s, _) => s.len(),
            TokenKind::StrLiteral(s, String) => s.len(),
            TokenKind::Comment(s) => s.len(),
            TokenKind::Symbol(s) => s.len(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    char_iter: std::iter::Peekable<Enumerate<std::str::Chars<'a>>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            char_iter: source.chars().enumerate().peekable(),
            line: 1,
            col: 0,
        }
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        self.col += 1;
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
    type Item = Result<Token<'a>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        self.next_char().and_then(|(i, c)| {
            Some((|| {
                Ok(match c {
                    '\r' | '\n' | ';' => {
                        let line = self.line;
                        let col = self.col;

                        // Handle CRLF line endings
                        if let Some((_, '\n')) = self.char_iter.peek() {
                            self.next_char();
                        }

                        self.line += 1;
                        self.col = 0;

                        Token {
                            kind: TokenKind::Newline,
                            line,
                            col: col,
                        }
                    }
                    '+' => Token {
                        kind: TokenKind::Plus,
                        line: self.line,
                        col: self.col,
                    },
                    '-' => Token {
                        kind: TokenKind::Minus,
                        line: self.line,
                        col: self.col,
                    },
                    '*' => Token {
                        kind: TokenKind::Asterisk,
                        line: self.line,
                        col: self.col,
                    },
                    '/' => Token {
                        kind: TokenKind::Slash,
                        line: self.line,
                        col: self.col,
                    },
                    '&' => Token {
                        kind: TokenKind::Ampersand,
                        line: self.line,
                        col: self.col,
                    },
                    '|' => Token {
                        kind: TokenKind::Pipe,
                        line: self.line,
                        col: self.col,
                    },
                    '^' => Token {
                        kind: TokenKind::Caret,
                        line: self.line,
                        col: self.col,
                    },
                    '(' => Token {
                        kind: TokenKind::LParenthesis,
                        line: self.line,
                        col: self.col,
                    },
                    ')' => Token {
                        kind: TokenKind::RParenthesis,
                        line: self.line,
                        col: self.col,
                    },
                    '.' => Token {
                        kind: TokenKind::Dot,
                        line: self.line,
                        col: self.col,
                    },
                    ',' => Token {
                        kind: TokenKind::Comma,
                        line: self.line,
                        col: self.col,
                    },
                    ':' => Token {
                        kind: TokenKind::Colon,
                        line: self.line,
                        col: self.col,
                    },
                    '#' => {
                        // Comment token
                        let mut end = i;
                        let token_col = self.col;

                        while let Some((j, c)) = self.char_iter.peek() {
                            end = *j;
                            if *c == '\n' {
                                break;
                            }
                            self.next_char();
                        }

                        Token {
                            kind: TokenKind::Comment(&self.source[i..end]),
                            line: self.line,
                            col: token_col,
                        }
                    }
                    '0'..='9' => {
                        // Int token
                        let mut end = i;
                        let token_col = self.col;

                        // Detect base
                        let base = if c == '0' {
                            let (j, b) = self
                                .char_iter
                                .peek()
                                .ok_or("Unexpected EOF while parsing int".to_string())?;
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
                            if !c.is_digit(base) {
                                break;
                            }
                            self.next_char();
                        }

                        let literal = &self.source[i..end];
                        let value = usize::from_str_radix(&self.source[start..end], base)
                            .map_err(|e| e.to_string())?;
                        Token {
                            kind: TokenKind::IntLiteral(literal, base, value),
                            line: self.line,
                            col: token_col,
                        }
                    }
                    '\'' => {
                        // Char token
                        let mut end = i;
                        let token_col = self.col;

                        loop {
                            if let Some((j, c)) = self.next_char() {
                                end = j;
                                if c == '\\' {
                                    self.next_char();
                                } else if c == '\'' {
                                    break;
                                }
                            } else {
                                return Err("Unexpected EOF while parsing char".to_string());
                            }
                        }

                        let literal = &self.source[i..end + 1];

                        Token {
                            kind: TokenKind::ChrLiteral(
                                literal,
                                Self::parse_char(&literal[1..literal.len() - 1]),
                            ),
                            line: self.line,
                            col: token_col,
                        }
                    }
                    '"' => {
                        // String token
                        let mut end = i;
                        let token_col = self.col;

                        loop {
                            if let Some((j, c)) = self.next_char() {
                                end = j;
                                if c == '\\' {
                                    self.next_char();
                                } else if c == '"' {
                                    break;
                                }
                            } else {
                                return Err("Unexpected EOF while parsing string".to_string());
                            }
                        }

                        let literal = &self.source[i..end + 1];

                        Token {
                            kind: TokenKind::StrLiteral(
                                literal,
                                Self::parse_string(&literal[1..literal.len() - 1]),
                            ),
                            line: self.line,
                            col: token_col,
                        }
                    }
                    _ => {
                        // Symbol token
                        let mut end = i;
                        let token_col = self.col;

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
                            col: token_col,
                        }
                    }
                })
            })())
        })
    }
}
