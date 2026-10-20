use std::mem;
use compiler::ast::nodes::*;
use compiler::errors::{CompilerError, Span, SpanPosition};
use compiler::lexer::{Token, TokenKind};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
    pub fn parse(&mut self) -> Result<Vec<AstNode<'a>>, CompilerError> {
        let mut nodes = Vec::new();

        while self.peek(0).is_some() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    fn parse_statement(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        let token = self.peek_or_err(0)?;

        let node = match token.kind {
            TokenKind::Identifier(_) => {
                self.parse_declaration_or_assignment()?
            }

            _ => { panic!("Parsing of token kind: {:?} not implemented yet", token.kind); },
        };

        Ok(node)
    }

    fn parse_declaration_or_assignment(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        match self.peek_or_err(1)?.kind {
            TokenKind::Equals => {
                // Assignment
                self.parse_assignment()
            },
            // TokenKind::Identifier(ident) => {
            //     // Declaration
            //     //self.parse_declaration()
            // },
            _ => {
                Err(CompilerError::new(
                    self.peek_or_err(1)?.span,
                    "Unable to parse expression, invalid statement.".to_string(),
                    "".to_string(),
                ))
            }
        }

    }

    fn parse_assignment(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        let target = self.consume().unwrap();
        let start_span = target.span;

        let target = match &target.kind {
            TokenKind::Identifier(name) => AstNodeKind::Ident(name.clone()),
            _ => { return Err(
                CompilerError::new(
                    start_span,
                    "Unable to parse expression, invalid assignment.".to_string(),
                    "".to_string(),
                ))
            }
        };


        self.expect(TokenKind::Equals, "Expected equals after variable name in variable assignment.".to_string())?;


        let expression = self.consume().unwrap();
        let end_span = expression.span;

        let expression = match expression.kind {
            TokenKind::Int(int) => { AstNodeKind::Value(int as u32) }
            _ => { return Err(
                CompilerError::new(
                    expression.span,
                    "Unable to parse expression, invalid assignment.".to_string(),
                    "".to_string(),
                )
            )}
        };

        self.expect(TokenKind::Semicolon, "Expected ';' after assignment.".to_string())?;

        Ok(AstNode::new(start_span, AstNodeKind::Assignment { target: target.into(), value: expression.into() }))
    }

    // fn parse_declaration(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
    //
    // }

    fn peek(&self, offset: i32) -> Option<&Token<'a>> {
        let pos = self.position as i32 + offset;
        if pos < 0 {
            panic!("Attempted to peek an invalid position! peeked: {}", pos);
        } else if pos >= self.tokens.len() as i32 {
            return None;
        }

        Some(&self.tokens[pos as usize])
    }

    fn peek_or_err(&self, offset: i32) -> Result<&Token<'a>, CompilerError<'a>> {
        let tok = self.peek(offset);
        if tok.is_none() {
            return Err(CompilerError::new(
                self.peek(offset-1).unwrap().span,
                "Could not peek token, eof reached".to_string(),
                "".to_string(),
            ))
        }
        Ok(tok.unwrap())
    }

    fn consume(&mut self) -> Option<&Token<'a>> {
        self.position += 1;
        self.peek(-1)
    }

    fn accept(&mut self, kind: TokenKind) -> bool {
        let next = &self.consume();
        if next.is_none() {
            false
        } else if mem::discriminant(&next.unwrap().kind) == mem::discriminant(&kind) {
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, message: String) -> Result<(), CompilerError<'a>> {
        let next = self.consume();
        if next.is_none() { return Err(CompilerError::new(
            self.peek(-2).unwrap().span,
            message,
            "Unexpected token".to_string(),
        )); };

        if mem::discriminant(&next.unwrap().kind) == mem::discriminant(&kind) {
            Ok(())
        } else {
            Err(CompilerError::new(
                next.unwrap().span,
                message,
                "Unexpected token".to_string(),
            ))
        }
    }
}


