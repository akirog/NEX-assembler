use std::mem;
use compiler::ast::{BlockNode, ExprKind, ExprNode, Field, StmtKind, StmtNode, ValueType, NONE_TYPE};
use compiler::errors::CompilerError;
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
    pub fn parse(&mut self) -> Result<Vec<StmtNode<'a>>, CompilerError<'_>> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(0).kind, TokenKind::EOF)
    }

    fn parse_statement(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let token = self.peek(0);

        let node = match token.kind {
            TokenKind::Let => {
                self.parse_declaration()?
            },
            TokenKind::Fn => {
                self.parse_function_declaration()?
            },
            TokenKind::Identifier(_) => {
                self.parse_assignment_or_func_call()?
            },

            _ => { panic!("Parsing of token kind: {:?} not implemented yet", token.kind); },
        };

        Ok(node)
    }


    fn parse_assignment_or_func_call(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let token = self.peek(1);

        match token.kind {
            TokenKind::Equals => { self.parse_assignment() },
            TokenKind::LParen => {
                let expr = self.parse_operand()?;
                self.expect(TokenKind::Semicolon, "Expected semicolon after function call")?;
                Ok(StmtNode::new(
                    expr.span,
                    StmtKind::Expression(Box::new(expr)),
                ))
            },
            _ => { Err(CompilerError::new(
                token.span,
                "Invalid statement."
            )) }
        }
    }

    fn parse_assignment(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        // Target
        let target = self.parse_target()?;

        // Equals
        self.expect(TokenKind::Equals, "Missing equals after variable name in variable assignment.")?;

        // Expression
        let expression = self.parse_expression()?;

        // Semicolon
        let end_span =self.expect(TokenKind::Semicolon, "Missing ';' after assignment.")?.span;

        // Return
        Ok(
            StmtNode::new(
                target.span.extend(end_span),
                StmtKind::Assignment {
                    target: target.into(),
                    value: expression.into(),
                }
            )
        )
    }


    fn parse_declaration(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        self.expect(TokenKind::Let, "Missing let keyword in variable declaration.")?;

        // Var name
        let name = self.consume();
        let start_span = name.span;
        let name = match &name.kind {
            TokenKind::Identifier(ident) => ident.clone(),
            _ => { return Err(CompilerError::new(
                name.span,
                "Invalid declaration name.",
            ))}
        };

        self.expect(TokenKind::Colon, "Missing ':' before variable type (type inference not implemented yet).")?;

        let var_type = self.parse_type()?;

        // Check for early exit
        let next = self.consume();
        match next.kind {
            TokenKind::Semicolon => {
                return Ok(StmtNode::new(
                    start_span.extend(next.span),
                    StmtKind::Declaration {
                        var_type,
                        name,
                        value: None,
                    }
                ));
            },
            TokenKind::Equals => {  },
            _ => { return Err(CompilerError::new(
                next.span,
                "Invalid token in variable declaration, missing semicolon?"
            ))}
        };


        // Expression
        let expression = self.parse_expression()?;

        // Semicolon
        let end_span = self.expect(TokenKind::Semicolon, "Missing ';' after declaration")?.span;

        // Return
        Ok(StmtNode::new(
            start_span.extend(end_span),
            StmtKind::Declaration {
                var_type,
                name,
                value: Some(expression.into()),
            }
        ))
    }


    fn parse_function_declaration(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        // Fn keyword
        let start_span = self.peek(0).span;
        self.expect(TokenKind::Fn, "Missing `fn` keyword in function declaration.")?;

        // Name
        let name = self.consume();
        let name = match &name.kind {
            TokenKind::Identifier(ident) => ident.clone(),
            _ => { return Err(CompilerError::new(
                name.span,
                "Invalid function name.",
            ))}
        };

        // Arguments
        self.expect(TokenKind::LParen, "Missing function args.")?;

        let arguments = self.parse_fields()?;

        // Return type
        let ret_type = if self.accept(TokenKind::Arrow) {
            self.parse_type()?
        } else {
            ValueType::Primitive(NONE_TYPE.to_string())
        };


        // Body
        let body = self.parse_body()?;


        Ok(StmtNode::new(
            start_span,
            StmtKind::FunctionDeclaration {
                name,
                ret_type,
                arguments,
                body
            }
        ))
    }

    fn parse_target(&mut self) -> Result<ExprNode<'a>, CompilerError<'a>> {
        let target = self.consume();
        match &target.kind {
            TokenKind::Identifier(name) => Ok( ExprNode::new(
                target.span,
                ExprKind::Identifier(name.clone()),
                None
            ) ),

            _ => { Err(
                CompilerError::new(
                    target.span,
                    "Invalid target.",
                ) )
            }
        }
    }


    fn parse_expression(&mut self) -> Result<ExprNode<'a>, CompilerError<'a>> {
        self.parse_operand()
    }


    fn parse_operand(&mut self) -> Result<ExprNode<'a>, CompilerError<'a>> {
        let token = self.consume();
        let span = token.span;
        let mut atom = match &token.kind {
            // Primitive types
            TokenKind::Int(val) => {
                Ok(ExprNode::new(
                    span,
                    ExprKind::Number(*val as u32),
                    None
                ))
            }

            // Identifier
            TokenKind::Identifier(name) => {
                Ok(ExprNode::new(
                    span,
                    ExprKind::Identifier(name.clone()),
                    None
                ))
            }

            // Prefixes like &
            TokenKind::Ampersand => {
                let var = self.parse_operand()?;
                let var_name = match var.kind {
                    ExprKind::Identifier(name) => name.clone(),
                    _ => { return Err(
                        CompilerError::new(
                            span,
                            "Invalid target type for `address of`.",
                        )
                    )}
                };

                Ok(ExprNode::new(
                    span,
                    ExprKind::AddressOf(var_name),
                    None
                ))
            }
            TokenKind::Star => {
                let expression = self.parse_operand()?;
                Ok(ExprNode::new(
                    span,
                    ExprKind::Dereference(expression.into()),
                    None
                ))
            }
            TokenKind::LParen => {
                self.consume();
                let expr = self.parse_expression();
                self.expect(TokenKind::RParen, "Missing ')' after expression.")?;
                expr
            }

            _ => { Err(
                CompilerError::new(
                    span,
                    "Invalid operand atom.",
                )
            )}
        }?;

        if self.accept(TokenKind::LParen) {

            let end_span = self.expect(TokenKind::RParen, "Missing ')' after expression.")?.span;
            atom = ExprNode::new(
                atom.span.extend(end_span),
                ExprKind::FunctionCall {
                    function: Box::new(atom),
                    arguments: Vec::new(),
                },
                None
            )
        }

        Ok(atom)
    }


    fn parse_body(&mut self) -> Result<BlockNode<'a>, CompilerError<'a>> {
        let mut statements = Vec::new();

        let start_span = self.peek(0).span;
        self.expect(TokenKind::LBrace, "Missing starting '{' for block.")?;
        while !matches!(self.peek(0).kind, TokenKind::RBrace) {
            statements.push(self.parse_statement()?);
        }
        let end_span = self.peek(0).span;
        self.expect(TokenKind::RBrace, "Missing ending '}'.")?;

        Ok(BlockNode {
            statements,
            span: start_span.extend(end_span),
        })
    }


    fn parse_fields(&mut self) -> Result<Vec<Field>, CompilerError<'a>> {
        let mut fields = Vec::new();
        if !self.accept(TokenKind::RParen) {
            loop {
                let name = self.consume();
                let name = match &name.kind {
                    TokenKind::Identifier(ident) => ident.clone(),
                    _ => { return Err(CompilerError::new(
                        name.span,
                        "Invalid field name.",
                    ))}
                };

                self.expect(TokenKind::Colon, "Missing ':' separator in field declaration.")?;

                let field_type = self.parse_type()?;

                fields.push(Field {
                    name,
                    field_type,
                });

                let next = self.consume();
                match next.kind {
                    TokenKind::RParen => { break; },
                    TokenKind::Comma => { },
                    _ => { return Err(CompilerError::new(
                        next.span,
                        "Invalid token in fields, missing comma?"
                    ))}
                }
            }
        };
        Ok(fields)
    }


    /// Parses a type like int `*(*ptr)[2]`, returns a function to wrap a variable name in this type, and returns the inner name if found
    fn parse_type(&mut self) -> Result<ValueType, CompilerError<'a>> {
        // With the new type system, we simply match the token and dispatch work
        let next = self.consume();
        match &next.kind {
            TokenKind::Identifier(name) => {
                Ok(ValueType::Primitive(name.clone()))
            },
            TokenKind::Star => {
                let inner = self.parse_type()?;
                Ok(ValueType::Pointer(Box::new(inner)))
            },
            TokenKind::LBracket => {
                let inner = self.parse_type()?;
                self.expect(TokenKind::Semicolon, "Missing ';' between array type and length")?;

                let next = self.consume();
                let length = match next.kind {
                    TokenKind::Int(num) => num as usize,
                    _ => { return Err(
                        CompilerError::new(
                            next.span,
                            "Invalid array length.",
                        )
                    )}
                };
                self.expect(TokenKind::RBracket, "Missing ']' after array type.")?;

                Ok(ValueType::Array {
                    length: Some(length),
                    inner_type: Box::new(inner),
                })
            }
            TokenKind::Fn => {
                self.expect(TokenKind::LParen, "Missing '(' in function type.")?;
                let args = self.parse_fields()?;
                let mut params = Vec::with_capacity(args.len());
                for arg in args {
                    params.push(arg.field_type);
                };

                let ret_type = ValueType::Primitive(NONE_TYPE.to_string());

                Ok(ValueType::Function {
                    params,
                    ret_type
                })
            }
            _ => {
                Err(
                    CompilerError::new(
                        next.span,
                        "Invalid type.",
                    )
                )
            }
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
        let next = &self.peek(0);
        if mem::discriminant(&next.kind) == mem::discriminant(&kind) {
            self.consume();
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


