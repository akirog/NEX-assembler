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
            TokenKind::Identifier(_) => {
                // Declaration
                self.parse_declaration()
            },
            _ => {
                Err(CompilerError::new(
                    self.peek(2).span,
                    "Unable to parse expression, invalid statement.",
                ))
            }
        }

    }

    fn parse_assignment(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        // Target
        let target = self.parse_target()?;

        // Equals
        self.expect(TokenKind::Equals, "Expected equals after variable name in variable assignment.")?;

        // Expression
        let expression = self.parse_expression()?;

        // Semicolon
        let end_span =self.expect(TokenKind::Semicolon, "Expected ';' after assignment.")?.span;

        // Return
        Ok(AstNode::new(target.span.extend(end_span), AstNodeKind::Assignment { target: target.into(), value: expression.into() }))
    }


    fn parse_declaration(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        // Var type
        let var_type = self.consume();
        let start_span = var_type.span;
        let var_type = match &var_type.kind {
            TokenKind::Identifier(ident) => ident.clone(),
            _ => { return Err(CompilerError::new(
                var_type.span,
                "Invalid target type.",
            ))}
        };

        // Var name
        let var_name = self.consume();
        let var_name = match &var_name.kind {
            TokenKind::Identifier(ident) => ident.clone(),
            _ => { return Err(CompilerError::new(
                var_name.span,
                "Invalid target type.",
            ))}
        };

        // Equals
        self.expect(TokenKind::Equals, "Expected = in variable declaration.")?;

        // Expression
        let expression = self.parse_expression()?;

        // Semicolon
        let end_span = self.expect(TokenKind::Semicolon, "Expected ';' after declaration")?.span;

        // Return
        Ok(AstNode::new(
            start_span.extend(end_span),
            AstNodeKind::Declaration {
                var_type,
                name: var_name,
                value: expression.into(),
            }
        ))
    }



    fn parse_target(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        let target = self.consume();
        match &target.kind {
            TokenKind::Identifier(name) => Ok( AstNode::new( target.span, AstNodeKind::Identifier(name.clone()) ) ),

            _ => { Err(
                CompilerError::new(
                    target.span,
                    "Invalid target.",
                ) )
            }
        }
    }


    fn parse_expression(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {
        self.parse_operand()
    }


    fn parse_operand(&mut self) -> Result<AstNode<'a>, CompilerError<'a>> {



        let token = self.consume();
        let span = token.span;
        match &token.kind {
            // Primitive types
            TokenKind::Int(val) => {
                Ok(AstNode::new(
                    span,
                    AstNodeKind::Number(*val as u32)
                ))
            }

            // Identifier
            TokenKind::Identifier(name) => {
                Ok(AstNode::new(
                    span,
                    AstNodeKind::Identifier(name.clone())
                ))
            }

            // Prefixes like &
            TokenKind::Ampersand => {
                let var = self.parse_operand()?;
                let var_name = match var.kind {
                    AstNodeKind::Identifier(name) => name.clone(),
                    _ => { return Err(
                        CompilerError::new(
                            span,
                            "Invalid target type for `address of`.",
                        )
                    )}
                };

                Ok(AstNode::new(
                    span,
                    AstNodeKind::AddressOf(var_name)
                ))
            }
            TokenKind::Star => {
                let expression = self.parse_operand()?;
                Ok(AstNode::new(
                    span,
                    AstNodeKind::Dereference(expression.into())
                ))
            }

            _ => { Err(
                CompilerError::new(
                    span,
                    "Invalid operand atom.",
                )
            )}
        }
    }


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

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<&Token<'a>, CompilerError<'a>> {
        let next = self.consume();
        if mem::discriminant(&next.kind) == mem::discriminant(&kind) {
            Ok(next)
        } else {
            Err(CompilerError::new(
                next.span,
                message,
            ))
        }
    }
}


