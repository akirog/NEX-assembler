use std::mem;
use clap::builder::Resettable::Value;
use compiler;
use compiler::ast::nodes::*;
use compiler::errors::{CompilerError, Span, SpanPosition};
use compiler::lexer::{Token, TokenKind};
use compiler::typing::types::ValueType;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

type ParserNode<'a> = AstNode<'a, Option<compiler::typing::types::ValueType>>;


impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
    pub fn parse(&mut self) -> Result<Vec<ParserNode<'a>>, CompilerError> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(0).kind, TokenKind::EOF)
    }

    fn parse_statement(&mut self) -> Result<ParserNode<'a>, CompilerError<'a>> {
        let token = self.peek(0);

        let node = match token.kind {
            TokenKind::Identifier(_) => {
                self.parse_declaration_or_assignment()?
            }

            _ => { panic!("Parsing of token kind: {:?} not implemented yet", token.kind); },
        };

        Ok(node)
    }

    fn parse_declaration_or_assignment(&mut self) -> Result<ParserNode<'a>, CompilerError<'a>> {
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

    fn parse_assignment(&mut self) -> Result<ParserNode<'a>, CompilerError<'a>> {
        // Target
        let target = self.parse_target()?;

        // Equals
        self.expect(TokenKind::Equals, "Expected equals after variable name in variable assignment.")?;

        // Expression
        let expression = self.parse_expression()?;

        // Semicolon
        let end_span =self.expect(TokenKind::Semicolon, "Expected ';' after assignment.")?.span;

        // Return
        Ok(AstNode::new(target.span.extend(end_span), AstNodeKind::Assignment { target: target.into(), value: expression.into(), target_type: None }))
    }


    fn parse_declaration(&mut self) -> Result<ParserNode<'a>, CompilerError<'a>> {
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
                var_type: None,
                name: var_name,
                value: expression.into(),
            }
        ))
    }



    fn parse_target(&mut self) -> Result<ParserNode<'a>, CompilerError<'a>> {
        let target = self.consume();
        match &target.kind {
            TokenKind::Identifier(name) => Ok( ParserNode::new( target.span, AstNodeKind::Identifier(name.clone(), None) ) ),

            _ => { Err(
                CompilerError::new(
                    target.span,
                    "Invalid target.",
                ) )
            }
        }
    }


    fn parse_expression(&mut self) -> Result<ParserNode<'a>, CompilerError<'a>> {
        self.parse_operand()
    }


    fn parse_operand(&mut self) -> Result<ParserNode<'a>, CompilerError<'a>> {



        let token = self.consume();
        let span = token.span;
        match &token.kind {
            // Primitive types
            TokenKind::Int(val) => {
                Ok(AstNode::new(
                    span,
                    AstNodeKind::Number(*val as u32, None)
                ))
            }

            // Identifier
            TokenKind::Identifier(name) => {
                Ok(AstNode::new(
                    span,
                    AstNodeKind::Identifier(name.clone(), None)
                ))
            }

            // Prefixes like &
            TokenKind::Ampersand => {
                let var = self.parse_operand()?;
                let var_name = match var.kind {
                    AstNodeKind::Identifier(name, None) => name.clone(),
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
                    AstNodeKind::Dereference(expression.into(), None)
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


    /// Parses a type like int `*(*ptr)[2]`, returns a function to wrap a variable name in this type, and returns the inner name if found
    fn parse_type(&mut self) -> Result<(ValueType, Option<String>), CompilerError<'a>> {
        // Get pointer count
        let mut ptr_count = 0;

        while matches!(self.peek(0).kind, TokenKind::Star) {
            ptr_count += 1;
        };

        // Check for inner name or left paren
        // lparen -> parse_type
        // inner name -> take name
        // none -> continue with name as None
        let mut inner_type = None;
        let mut name = None;
        match &self.peek(0).kind {
            TokenKind::Identifier(ident_name) => { name = Some(ident_name.clone()); },
            TokenKind::LParen => {
                self.consume();
                let result = self.parse_type()?;
                inner_type = Some(result.0);
                name = result.1;
            }
            _ => { /* if no inner name or inner type then name and wrap can stay none */},
        };


        // Get postfixes
        let mut arr_lengths: Vec<Option<usize>> = Vec::new();
        while matches!(self.peek(0).kind, TokenKind::LBracket) {
            self.consume();
            let mut arr_length = None;
            let next = self.consume();
            match next.kind {
                TokenKind::RBracket => {},
                TokenKind::Int(num) => { arr_length = Some(num as usize) }
                _ => { return Err(
                    CompilerError::new(
                        next.span,
                        "Expected right bracket or array length",
                    )
                )}
            }

            arr_lengths.push(arr_length);
        }

        // Construct type
        let mut this_type = inner_type.unwrap_or(ValueType::Primitive("".to_string()));

        for _ in 0..ptr_count {
            this_type = ValueType::Pointer(Box::new(this_type));
        }

        for arr_length in arr_lengths.into_iter().rev() {
            this_type = ValueType::Array {
                length: arr_length,
                inner_type: Box::new(this_type),
            }
        }

        // return type and optional name
        Ok((this_type, name))
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


