use std::fmt::Display;
use std::str::Chars;

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
pub struct Token {
    kind: TokenKind,
}

impl Display for Token {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self.kind)
    }
}



pub struct Lexer<'a> {
    chars: Chars<'a>,
}


impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            chars: input.chars(),
        }
    }


    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = Vec::new();

        while self.chars.as_str().len() > 0 {
            let token = self.next_token()?;
            if token.is_none() {
                continue;
            }

            tokens.push(token.unwrap());
        }

        Ok(tokens)
    }

    fn next_char(&mut self) -> Result<char, String> {
        match self.chars.next() {
            Some(c) => Ok(c),
            None => Err("Unexpected EOF".to_owned()),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        let next_char = self.next_char()?;
        match next_char {
            '(' => Ok(Some(Token { kind: TokenKind::LParen })),
            ')' => Ok(Some(Token { kind: TokenKind::RParen })),
            '{' => Ok(Some(Token { kind: TokenKind::LBrace })),
            '}' => Ok(Some(Token { kind: TokenKind::RBrace })),
            '[' => Ok(Some(Token { kind: TokenKind::LBracket })),
            ']' => Ok(Some(Token { kind: TokenKind::RBracket })),
            '+' => Ok(Some(Token { kind: TokenKind::Plus })),
            '-' => {
                if self.peek(0)? == '>' {
                    self.chars.next();
                    Ok(Some(Token { kind: TokenKind::Arrow }))
                } else{
                    Ok(Some(Token { kind: TokenKind::Minus }))
                }
            }
            '*' => Ok(Some(Token { kind: TokenKind::Star })),
            '/' => {
                if self.peek(0)? == '/' {
                    // Line comment
                    while self.next_char()? != '\n' {
                        continue;
                    }
                    Ok(None)
                } else if self.peek(0)? == '*' {
                    self.chars.next();
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
                    Ok(Some(Token { kind: TokenKind::Slash }))
                }
            },
            '%' => Ok(Some(Token { kind: TokenKind::Percent })),
            '<' => {
                if self.peek(0)? == '=' {
                    self.chars.next();
                    Ok(Some(Token { kind: TokenKind::LessThanOrEqual }))
                } else {
                    Ok(Some(Token { kind: TokenKind::LessThan }))
                }
            },
            '>' => {
                if self.peek(0)? == '=' {
                    self.chars.next();
                    Ok(Some(Token { kind: TokenKind::GreaterThanOrEqual }))
                } else{
                    Ok(Some(Token { kind: TokenKind::GreaterThan }))
                }
            },
            '!' => {
                if self.peek(0)? == '=' {
                    self.chars.next();
                    Ok(Some(Token { kind: TokenKind::NotEqual }))
                } else{
                    Ok(Some(Token { kind: TokenKind::Exclamation }))
                }
            }
            '^' => Ok(Some(Token { kind: TokenKind::Caret })),
            '=' => {
                if self.next_char()? == '=' {
                    self.chars.next();
                    Ok(Some(Token { kind: TokenKind::EqualTo }))
                } else {
                    Ok(Some(Token { kind: TokenKind::Equals }))
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
                        self.chars.next();
                    } else {
                        break;
                    }
                }



                Ok(Some(Token { kind: TokenKind::Int(number as i32) }))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut name = next_char.to_string();
                while let Some(next_char) = self.chars.clone().next() {
                    if next_char.is_alphanumeric() {
                        name.push(next_char);
                        self.chars.next();
                    } else {
                        break;
                    }
                }

                match name.as_str() {
                    "true" => Ok(Some(Token { kind: TokenKind::Bool(true) })),
                    "false" => Ok(Some(Token { kind: TokenKind::Bool(false) })),
                    "struct" => Ok(Some(Token { kind: TokenKind::Struct })),
                    "if" => Ok(Some(Token { kind: TokenKind::If })),
                    "else" => Ok(Some(Token { kind: TokenKind::Else })),
                    "return" => Ok(Some(Token { kind: TokenKind::Return })),
                    "break" => Ok(Some(Token { kind: TokenKind::Break })),
                    "continue" => Ok(Some(Token { kind: TokenKind::Continue })),
                    "asm" => Ok(Some(Token { kind: TokenKind::Asm })),

                    _ => Ok(Some(Token { kind: TokenKind::Identifier(name) }))
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
                Ok(Some(Token { kind: TokenKind::String(string) }))
            },
            '.' => Ok(Some(Token { kind: TokenKind::Dot})),
            ',' => Ok(Some(Token { kind: TokenKind::Comma })),
            '&' => Ok(Some(Token { kind: TokenKind::Ampersand })),
            '\'' => {
                let char = self.next_char()?;
                if self.next_char()? != '\'' {
                    Err(format!("Unexpected character '{}'", char))
                } else {
                    Ok(Some(Token { kind: TokenKind::Char(char) }))
                }
            }
            ';' => { Ok(Some(Token { kind: TokenKind::Semicolon })) }
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