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

        while !self.is_eof() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(0).kind, TokenKind::EOF)
    }

    fn parse_statement(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        let token = self.peek(0);

        let node = match token.kind {
            TokenKind::Identifier(_) => {
                self.parse_declaration_or_assignment()?
            }

            _ => { panic!("Parsing of token kind: {:?} not implemented yet", token.kind); },
        };

        Ok(node)
    }

    fn parse_declaration_or_assignment(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        match self.peek(1).kind {
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
                    self.peek(2).span,
                    "Unable to parse expression, invalid statement.".to_string(),
                    "".to_string(),
                ))
            }
        }

    }

    fn parse_assignment(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        let target = self.consume();
        let start_span = target.span;

        let target = match &target.kind {
            TokenKind::Identifier(name) => AstNode::new( target.span, AstNodeKind::Ident(name.clone()) ),
            _ => { return Err(
                CompilerError::new(
                    start_span,
                    "Unable to parse expression, invalid assignment.".to_string(),
                    "".to_string(),
                ))
            }
        };


        self.expect(TokenKind::Equals, "Expected equals after variable name in variable assignment.".to_string())?;


        let expression = self.consume();
        let end_span = expression.span;

        let expression = match expression.kind {
            TokenKind::Int(int) => { AstNode::new( expression.span, AstNodeKind::Value(int as u32)) }
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

    fn peek(&self, offset: i32) -> &Token<'a> {
        let pos = self.position as i32 + offset;
        if pos < 0 {
            panic!("Attempted to peek an invalid position! peeked: {}", pos);
        } else if pos >= self.tokens.len() as i32 {
            // Last token is always EOF token
            return self.tokens.last().unwrap();
        }

        &self.tokens[pos as usize]
    }

    fn consume(&mut self) -> &Token<'a> {
        self.position += 1;
        self.peek(-1)
    }

    fn accept(&mut self, kind: TokenKind) -> bool {
        let next = &self.consume();
        if mem::discriminant(&next.kind) == mem::discriminant(&kind) {
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, message: String) -> Result<(), CompilerError<'a>> {
        let next = self.consume();
        if mem::discriminant(&next.kind) == mem::discriminant(&kind) {
            Ok(())
        } else {
            Err(CompilerError::new(
                next.span,
                message,
                "Unexpected token".to_string(),
            ))
        }
    }
}


