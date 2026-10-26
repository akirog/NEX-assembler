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
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    CaretAssign,
    PercentAssign,
    Ampersand,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equals,     // =
    EqualTo,    // ==
    NotEqual,   // !=
    Exclamation,// !
    Type,
    Struct,
    Enum,
    If,
    Else,
    For,
    While,
    Loop,
    Return,
    Break,
    Continue,
    Asm,
    As,
    Dot,
    Comma,
    Arrow,      // ->
    Let,
    Fn,
    Number(i32),
    String(String),
    Char(char),
    Bool(bool),
    Identifier(String),
    Semicolon,
    Colon,

    /// End of file token
    EOF,
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
    input_path: &'a str,
    input: &'a str,
    chars: Chars<'a>,
    position: usize,
    line: usize,
    column: usize,
}


impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, input_path: &'a str) -> Lexer<'a> {
        Lexer {
            input_path,
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
                    Span::new( start, end, self.input, self.input_path ),
                    e.as_str(),
                ));
            }
            let token = token.unwrap();

            if token.is_none() {
                continue;
            }


            tokens.push(token.unwrap());
        }
        tokens.push(
            Token {
                span: Span::new(
                    SpanPosition::new( self.line, self.column ),
                    SpanPosition::new( self.line, self.column ),
                    self.input,
                    self.input_path
                ),
                kind: TokenKind::EOF,
            }
        );

        Ok(tokens)
    }

    fn next_char(&mut self) -> char {
        self.position += 1;
        self.column += 1;
        match self.chars.next() {
            Some(c) => {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                c
            },
            None => '\0',
        }
    }

    fn next_token(&mut self) -> Result<Option<Token<'a>>, String> {
        let start_span = Span::new(
            SpanPosition::new(self.line, self.column),
            SpanPosition::new(self.line, self.column),
            self.input,
            self.input_path
        );
        let next_char = self.next_char();


        match next_char {
            '(' => Ok(Some(Token { span: start_span, kind: TokenKind::LParen })),
            ')' => Ok(Some(Token { span: start_span, kind: TokenKind::RParen })),
            '{' => Ok(Some(Token { span: start_span, kind: TokenKind::LBrace })),
            '}' => Ok(Some(Token { span: start_span, kind: TokenKind::RBrace })),
            '[' => Ok(Some(Token { span: start_span, kind: TokenKind::LBracket })),
            ']' => Ok(Some(Token { span: start_span, kind: TokenKind::RBracket })),
            '+' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::PlusAssign }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Plus }))
                }
            },
            '-' => {
                if self.peek(0) == '>' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::Arrow }))
                } else if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::MinusAssign }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Minus }))
                }
            }
            '*' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::StarAssign }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Star }))
                }
            },
            '/' => {
                if self.peek(0) == '/' {
                    // Line comment
                    while self.peek(0) != '\0' && self.next_char() != '\n' {
                        continue;
                    }
                    Ok(None)
                } else if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::SlashAssign }))
                } else if self.peek(0) == '*' {
                    self.next_char();
                    // Multiline comment
                    loop {
                        if self.next_char() == '*' {
                            if self.next_char() == '/' {
                                break;
                            }
                        }
                    }
                    Ok(None)
                } else if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::EqualTo }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Slash }))
                }
            },
            '%' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::PercentAssign }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Percent }))
                }
            },
            '<' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::LessThanOrEqual }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::LessThan }))
                }
            },
            '>' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::GreaterThanOrEqual }))
                } else{
                    Ok(Some(Token { span: start_span, kind: TokenKind::GreaterThan }))
                }
            },
            '!' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::NotEqual }))
                } else{
                    Ok(Some(Token { span: start_span, kind: TokenKind::Exclamation }))
                }
            }
            '^' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::CaretAssign }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Caret }))
                }
            },
            '=' => {
                if self.peek(0) == '=' {
                    self.next_char();
                    Ok(Some(Token { span: start_span, kind: TokenKind::EqualTo }))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Equals }))
                }
            },
            '0'..='9' => {
                let mut number = next_char.to_digit(10).unwrap();

                let mut base = 10;
                let next_char = self.peek(0).to_lowercase().next().unwrap();
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
                        number += self.next_char().to_digit(base).unwrap();
                    } else {
                        break;
                    }
                }

                Ok(Some(Token { span: start_span, kind: TokenKind::Number(number as i32) }))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut name = next_char.to_string();
                while let Some(next_char) = self.chars.clone().next() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        name.push(next_char);
                        self.next_char();
                    } else {
                        break;
                    }
                }

                let span = start_span.extend(Span::new(
                    SpanPosition::new(self.line, self.column-1),
                    SpanPosition::new(self.line, self.column-1),
                    self.input,
                    self.input_path
                ));

                match name.to_lowercase().as_str() {
                    "let" =>        Ok(Some(Token { span, kind: TokenKind::Let })),
                    "fn" =>         Ok(Some(Token { span, kind: TokenKind::Fn })),
                    "true" =>       Ok(Some(Token { span, kind: TokenKind::Bool(true) })),
                    "false" =>      Ok(Some(Token { span, kind: TokenKind::Bool(false) })),
                    "type" =>       Ok(Some(Token { span, kind: TokenKind::Type })),
                    "struct" =>     Ok(Some(Token { span, kind: TokenKind::Struct })),
                    "enum" =>       Ok(Some(Token { span, kind: TokenKind::Enum })),
                    "if" =>         Ok(Some(Token { span, kind: TokenKind::If })),
                    "else" =>       Ok(Some(Token { span, kind: TokenKind::Else })),
                    "for" =>        Ok(Some(Token { span, kind: TokenKind::For })),
                    "while" =>      Ok(Some(Token { span, kind: TokenKind::While })),
                    "loop" =>       Ok(Some(Token { span, kind: TokenKind::Loop })),
                    "return" =>     Ok(Some(Token { span, kind: TokenKind::Return })),
                    "break" =>      Ok(Some(Token { span, kind: TokenKind::Break })),
                    "continue" =>   Ok(Some(Token { span, kind: TokenKind::Continue })),
                    "as" =>         Ok(Some(Token { span, kind: TokenKind::As })),

                    _ => Ok(Some(Token { span, kind: TokenKind::Identifier(name) }))
                }
            }
            '"' => {
                let mut string = String::new();
                let mut end_span = Span::new(
                    SpanPosition::new(self.line, self.column),
                    SpanPosition::new(self.line, self.column),
                    self.input,
                    self.input_path
                );

                loop {
                    let char = self.next_char();
                    if char == '"' {
                        break
                    }
                    string.push(char);
                    end_span = Span::new(
                        SpanPosition::new(self.line, self.column),
                        SpanPosition::new(self.line, self.column),
                        self.input,
                        self.input_path
                    );
                }


                Ok(Some(Token { span: start_span.extend(end_span), kind: TokenKind::String(string) }))
            },
            '.' => Ok(Some(Token { span: start_span, kind: TokenKind::Dot})),
            ',' => Ok(Some(Token { span: start_span, kind: TokenKind::Comma })),
            '&' => Ok(Some(Token { span: start_span, kind: TokenKind::Ampersand })),
            '\'' => {
                let mut char = self.next_char();
                if char == '\\' {
                    char = match self.next_char() {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'b' => '\x08',
                        'f' => '\x0c',
                        '0' => '\0',
                        '\\' => '\\',
                        _ => { return Err(format!("Invalid escape character: {}", char)) }
                    }
                }

                if self.next_char() != '\'' {
                    Err(format!("Expected `'` at the end of char literal '{}'", char))
                } else {
                    Ok(Some(Token { span: start_span, kind: TokenKind::Char(char) }))
                }
            }
            ';' => { Ok(Some(Token { span: start_span, kind: TokenKind::Semicolon })) }
            ':' => { Ok(Some(Token { span: start_span, kind: TokenKind::Colon })) }
            ' ' | '\t' | '\n' => {Ok(None)},
            _ => {Err(format!("Unexpected character '{}'", next_char))},
        }
    }

    fn peek(&self, offset: i32) -> char {
        self.chars.clone().nth(offset as usize).unwrap_or_else(|| '\0')
    }
}