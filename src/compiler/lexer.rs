use std::fmt::Display;
use std::str::Chars;
use compiler::errors::{CompilerError, Span, SpanPosition};

#[derive(Debug)]
pub enum TokenKind {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    Ampersand,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equals,     // =
    EqualTo,    // ==
    NotEqual,   // !=
    Exclamation,// !
    Struct,
    If,
    Else,
    Return,
    Break,
    Continue,
    Asm,
    Dot,
    Comma,
    Arrow,      // ->
    Int(i32),
    String(String),
    Char(char),
    Bool(bool),
    Identifier(String),
    Semicolon,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub span: Span<'a>,
    pub kind: TokenKind,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self.kind)
    }
}



pub struct Lexer<'a> {
    input: &'a str,
    chars: Chars<'a>,
    position: usize,
    line: usize,
    column: usize,
}


impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input,
            chars: input.chars(),
            position: 0,
            line: 0,
            column: 0,
        }
    }


    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, CompilerError<'a>> {
        let mut tokens: Vec<Token> = Vec::new();

        while self.chars.as_str().len() > 0 {
            let start = SpanPosition::new( self.line, self.column );
            let token = self.next_token();
            let end = SpanPosition::new( self.line, self.column );
            if let Err(e) = token {
                return Err(CompilerError::new(
                    Span::new( start, end, self.input, "idk what file" ),
                    "Unknown token.".to_string(),
                    "".to_string(),
                ));
            }
            let token = token.unwrap();

            if token.is_none() {
                continue;
            }


            tokens.push(token.unwrap());
        }

        Ok(tokens)
    }

    fn next_char(&mut self) -> Result<char, String> {
        self.position += 1;
        self.column += 1;
        match self.chars.next() {
            Some(c) => {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                Ok(c)
            },
            None => Err("Unexpected EOF".to_owned()),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token<'a>>, String> {
        let start_span = Span::new(
            SpanPosition::new(self.line, self.column),
            SpanPosition::new(self.line, self.column),
            self.input,
            "idk what file"
        );
        let next_char = self.next_char()?;


        match next_char {
            '(' => Ok(Some(Token { span: start_span, kind: TokenKind::LParen })),
            ')' => Ok(Some(Token { span: start_span, kind: TokenKind::RParen })),
            '{' => Ok(Some(Token { span: start_span, kind: TokenKind::LBrace })),
            '}' => Ok(Some(Token { span: start_span, kind: TokenKind::RBrace })),
            '[' => Ok(Some(Token { span: start_span, kind: TokenKind::LBracket })),
            ']' => Ok(Some(Token { span: start_span, kind: TokenKind::RBracket })),
            '+' => Ok(Some(Token { span: start_span, kind: TokenKind::Plus })),
            '-' => {
                if self.peek(0)? == '>' {
                    self.next_char()?;
                    Ok(Some(Token { span: start_span, kind: TokenKind::Arrow }))
                } else{
                    Ok(Some(Token { span: start_span, kind: TokenKind::Minus }))
                }
            }
            '*' => Ok(Some(Token { span: start_span, kind: TokenKind::Star })),
            '/' => {
                if self.peek(0)? == '/' {
                    // Line comment
                    while self.next_char()? != '\n' {
                        continue;
                    }
                    Ok(None)
                } else if self.peek(0)? == '*' {
                    self.next_char()?;
                    // Multiline comment
                    loop {
                        if self.next_char()? == '*' {
                            if self.next_char()? == '/' {
                                break;
                            }
                        }
                    }
                    Ok(None)
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Slash }))
                }
            },
            '%' => Ok(Some(Token { span: start_span, kind: TokenKind::Percent })),
            '<' => {
                if self.peek(0)? == '=' {
                    self.next_char()?;
                    Ok(Some(Token { span: start_span, kind: TokenKind::LessThanOrEqual }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::LessThan }))
                }
            },
            '>' => {
                if self.peek(0)? == '=' {
                    self.next_char()?;
                    Ok(Some(Token { span: start_span, kind: TokenKind::GreaterThanOrEqual }))
                } else{
                    Ok(Some(Token { span: start_span, kind: TokenKind::GreaterThan }))
                }
            },
            '!' => {
                if self.peek(0)? == '=' {
                    self.next_char()?;
                    Ok(Some(Token { span: start_span, kind: TokenKind::NotEqual }))
                } else{
                    Ok(Some(Token { span: start_span, kind: TokenKind::Exclamation }))
                }
            }
            '^' => Ok(Some(Token { span: start_span, kind: TokenKind::Caret })),
            '=' => {
                if self.peek(0)? == '=' {
                    self.next_char()?;
                    Ok(Some(Token { span: start_span, kind: TokenKind::EqualTo }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Equals }))
                }
            },
            '0'..='9' => {
                let mut number = next_char.to_digit(10).unwrap();

                let mut base = 10;
                let next_char = self.peek(0)?.to_lowercase().next().unwrap();
                if number == 0 && (next_char == 'x' || next_char == 'b') {
                    // Something like 0b00 or 0x00
                    let _ =self.next_char();

                    base = match next_char {
                        'x' => Ok::<u32, String>(16),
                        'b' => Ok(2),
                        'o' => Ok(8),
                        _ => return Err(format!("Invalid number format prefix: {}", next_char)),
                    }?;

                }

                while let Some(next_char) = self.chars.clone().next() {
                    if next_char.is_numeric() {
                        number *= base;
                        number += self.next_char()?.to_digit(base).unwrap();
                    } else {
                        break;
                    }
                }

                Ok(Some(Token { span: start_span, kind: TokenKind::Int(number as i32) }))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut name = next_char.to_string();
                while let Some(next_char) = self.chars.clone().next() {
                    if next_char.is_alphanumeric() {
                        name.push(next_char);
                        self.next_char()?;
                    } else {
                        break;
                    }
                }

                let span = start_span.extend(Span::new(
                    SpanPosition::new(self.line, self.column-1),
                    SpanPosition::new(self.line, self.column-1),
                    self.input,
                    "idk what file"
                ));

                match name.as_str() {
                    "true" =>       Ok(Some(Token { span, kind: TokenKind::Bool(true) })),
                    "false" =>      Ok(Some(Token { span, kind: TokenKind::Bool(false) })),
                    "struct" =>     Ok(Some(Token { span, kind: TokenKind::Struct })),
                    "if" =>         Ok(Some(Token { span, kind: TokenKind::If })),
                    "else" =>       Ok(Some(Token { span, kind: TokenKind::Else })),
                    "return" =>     Ok(Some(Token { span, kind: TokenKind::Return })),
                    "break" =>      Ok(Some(Token { span, kind: TokenKind::Break })),
                    "continue" =>   Ok(Some(Token { span, kind: TokenKind::Continue })),
                    "asm" =>        Ok(Some(Token { span, kind: TokenKind::Asm })),

                    _ => Ok(Some(Token { span, kind: TokenKind::Identifier(name) }))
                }
            }
            '"' => {
                let mut string = String::new();
                while let Some(char) = self.chars.next() {
                    if char == '"' {
                        break
                    }
                    string.push(char);
                }
                Ok(Some(Token { span: start_span, kind: TokenKind::String(string) }))
            },
            '.' => Ok(Some(Token { span: start_span, kind: TokenKind::Dot})),
            ',' => Ok(Some(Token { span: start_span, kind: TokenKind::Comma })),
            '&' => Ok(Some(Token { span: start_span, kind: TokenKind::Ampersand })),
            '\'' => {
                let char = self.next_char()?;
                if self.next_char()? != '\'' {
                    Err(format!("Unexpected character '{}'", char))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Char(char) }))
                }
            }
            ';' => { Ok(Some(Token { span: start_span, kind: TokenKind::Semicolon })) }
            ' ' | '\t' | '\n' => {Ok(None)},
            _ => {Err(format!("Unexpected character '{}'", next_char))},
        }
    }

    fn peek(&self, offset: i32) -> Result<char, String> {
         match self.chars.clone().nth(offset as usize) {
             Some(c) => Ok(c),
             None => Err("Unexpected EOF".to_owned()),
         }
    }
}