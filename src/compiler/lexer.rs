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
    LessThan,
    GreaterThan,
    Equal,
    Int(i32),
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
    position: usize,
}


impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            chars: input.chars(),
            position: 0,
        }
    }


    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = Vec::new();

        while self.position < self.chars.as_str().len() {
            let token = self.next_token();
            if token.is_none() {
                continue;
            }
            tokens.push(token.unwrap());
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Option<Token> {
        let next_char = self.chars.next()?;
        match next_char {
            '(' => Some(Token {kind: TokenKind::LParen}),
            ')' => Some(Token {kind: TokenKind::RParen}),
            '{' => Some(Token {kind: TokenKind::LBrace}),
            '}' => Some(Token {kind: TokenKind::RBrace}),
            '[' => Some(Token {kind: TokenKind::LBracket}),
            ']' => Some(Token {kind: TokenKind::RBracket}),
            '+' => Some(Token {kind: TokenKind::Plus}),
            '-' => Some(Token {kind: TokenKind::Minus}),
            '*' => Some(Token {kind: TokenKind::Star}),
            '/' => Some(Token {kind: TokenKind::Slash}),
            '%' => Some(Token {kind: TokenKind::Percent}),
            '<' => Some(Token {kind: TokenKind::LessThan}),
            '>' => Some(Token {kind: TokenKind::GreaterThan}),
            '^' => Some(Token {kind: TokenKind::Caret}),
            '=' => Some(Token {kind: TokenKind::Equal}),
            '0'..='9' => {
                let mut number = next_char.to_digit(10).unwrap();
                while let Some(next_char) = self.chars.clone().next() {
                    if next_char.is_numeric() {
                        number *= 10;
                        number += self.chars.next()?.to_digit(10).unwrap();
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                Some(Token {kind: TokenKind::Int(number as i32)})
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
                Some(Token {kind: TokenKind::Identifier(name)})
            }
            ' ' | '\t' => {None},
            _ => {None}
        }
    }

    fn peek(&self, offset: i32) -> Option<char> {
         None
    }
}