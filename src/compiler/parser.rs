use std::mem;
use compiler::ast::{BlockNode, ExprKind, ExprNode, Field, Operation, StmtKind, StmtNode, TypeDefinition, TypeKind, ValueType, NONE_TYPE};
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
    pub fn parse(&mut self) -> Result<BlockNode<'a>, CompilerError<'_>> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            nodes.push(self.parse_statement()?);
        }

        // Should never panic as eof is always added.
        let span = nodes.first().unwrap().span.extend(nodes.last().unwrap().span);

        let body = BlockNode {
            statements: nodes,
            span,
        };

        Ok(body)
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
            TokenKind::If => {
                self.parse_if_statement()?
            },
            TokenKind::For => {
                self.parse_for_loop()?
            },
            TokenKind::While => {
                self.parse_while_loop()?
            },
            TokenKind::Loop => {
                self.parse_unconditional_loop()?
            },
            TokenKind::Struct => {
                self.parse_struct_definition()?
            },

            _ => {
                let stmt = self.parse_expression_statement()?;
                self.expect(TokenKind::Semicolon, "Missing semicolon after assignment")?;
                stmt
            },
        };

        Ok(node)
    }


    fn parse_struct_definition(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::Struct, "Expected struct definition")?.span;

        let name = self.parse_identifier()?;

        self.expect(TokenKind::LBrace, "Missing struct definition body.")?;

        let mut fields = Vec::new();
        while !matches!(self.peek(0).kind, TokenKind::RBrace) {
            let field_name = self.parse_identifier()?;
            self.expect(TokenKind::Colon, "Missing field type.")?;

            let field_type = self.parse_type()?;
            fields.push(Field {
                name: field_name,
                field_type,
            });

            self.accept(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace, "Missing right brace at end of struct definition.")?;


        Ok(StmtNode::new(
            start_span.extend(self.peek(-1).span),
            StmtKind::StructDefinition {
                name,
                fields,
            }
        ))

    }

    fn parse_if_statement(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let start_span = self.peek(0).span;
        self.expect(TokenKind::If, "Missing `if` at start of if statement")?;

        let condition = self.parse_expression()?;

        let body = self.parse_body()?;

        self.accept(TokenKind::Semicolon);
        Ok(StmtNode::new(
            start_span.extend(self.peek(-1).span),
            StmtKind::IfStatement {
                condition,
                body
            }
        ))
    }

    fn parse_while_loop(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::While, "Missing `while` at start of while loop")?.span;
        let condition = self.parse_expression()?;
        let body = self.parse_body()?;
        self.accept(TokenKind::Semicolon);
        Ok(StmtNode::new(
            start_span.extend(self.peek(-1).span),
            StmtKind::WhileLoop {
                condition, body
            }
        ))
    }

    fn parse_for_loop(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::For, "Missing `for` at start of for loop")?.span;

        let init = self.parse_statement()?;
        let condition = self.parse_expression()?;
        self.accept(TokenKind::Semicolon);
        let update = self.parse_expression_statement()?; // Parse expression statement skips ; requirement

        let body = self.parse_body()?;

        self.accept(TokenKind::Semicolon);

        Ok(StmtNode::new(
            start_span.extend(self.peek(-1).span),
            StmtKind::ForLoop {
                init: Box::new(init), condition, update: Box::new(update), body
            }
        ))
    }

    fn parse_unconditional_loop(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::Loop, "Missing `loop` at start of unconditional loop")?.span;

        let body = self.parse_body()?;

        self.accept(TokenKind::Semicolon);

        Ok(StmtNode::new(
            start_span.extend(self.peek(-1).span),
            StmtKind::Loop {
                body
            }
        ))
    }

    // Parse expression statement will parse an expression, then check for an equals sign, if found it returns the correct assignment statement, else an expression statement
    fn parse_expression_statement(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let mut expr = self.parse_expression()?;

        if matches!(self.peek(0).kind, TokenKind::Semicolon) {
            return Ok(StmtNode::new(
                expr.span.extend(self.peek(-1).span),
                StmtKind::Expression(Box::new(expr)),
            ));
        };

        let next = self.consume();
        match &next.kind {
            TokenKind::Equals => {  },
            TokenKind::PlusAssign => {
                expr = ExprNode::new(
                    expr.span,
                    ExprKind::BinaryOp {
                        left: Box::new(expr.clone()),
                        operation: Operation::Add,
                        right: Box::new(expr),
                    },
                    None
                )
            }

            _ => { return Err(CompilerError::new(
                self.peek(-2).span,
                "Missing semicolon after statement"
            ))}
        }

        let rhs = self.parse_expression()?;


        Ok(StmtNode::new(
            expr.span.extend(self.peek(0).span),
            StmtKind::Assignment {
                target: Box::new(expr),
                value: Box::new(rhs),
            }
        ))
    }


    fn parse_declaration(&mut self) -> Result<StmtNode<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::Let, "Missing let keyword in variable declaration.")?.span;

        // Var name
        let name = self.parse_identifier()?;

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
        let start_span = self.expect(TokenKind::Fn, "Missing `fn` keyword in function declaration.")?.span;

        // Name
        let name = self.parse_identifier()?;

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


    fn parse_expression(&mut self) -> Result<ExprNode<'a>, CompilerError<'a>> {
        let left = self.parse_operand()?;

        // Check for an operation
        let next = self.peek(0);
        let operation = match next.kind {
            TokenKind::Plus => {
                Operation::Add
            },
            TokenKind::Minus => {
                Operation::Subtract
            },
            TokenKind::Star => {
                Operation::Multiply
            },
            TokenKind::Slash => {
                Operation::Divide
            },
            TokenKind::EqualTo => {
                Operation::Equal
            },
            TokenKind::NotEqual => {
                Operation::NotEqual
            },
            TokenKind::GreaterThan => {
                Operation::GreaterThan
            },
            TokenKind::LessThan => {
                Operation::LessThan
            },
            TokenKind::GreaterThanOrEqual => {
                Operation::GreaterThanOrEqual
            },
            TokenKind::LessThanOrEqual => {
                Operation::LessThanOrEqual
            },
            _ => { return Ok(left); }
        };
        self.consume();

        let check_precedence = match self.peek(0).kind {
            TokenKind::LParen => { false },
            _ => { true }
        };
        let right = self.parse_expression()?;
        match &right.kind {
            ExprKind::BinaryOp { left: other_left, operation: other_op, right: other_right } => {
                if !check_precedence || other_op.precedence() > operation.precedence() {
                    return Ok(ExprNode::new(
                        left.span,
                        ExprKind::BinaryOp {
                            left: Box::new(left),
                            operation,
                            right: Box::new(right),
                        },
                        None
                    ))
                };
                /* For a binary operation we want to change:
                 Example: 1 * 2 + 3
                     *
                    / \
                   1   +
                      / \
                     2   3

                To:
                    +
                   / \
                  *   3
                 / \
                1   2

                so if rhs op.precedence < our precedence:
                new_lhs = binaryop( this.lhs, this.op, rhs.lhs )
                this = binaryop( new_lhs, rhs.op, rhs.rhs  )
                */
                let full_span = left.span.extend(other_right.span);

                let new_lhs = ExprNode::new(
                    left.span.extend(other_left.span),
                    ExprKind::BinaryOp {
                        left: Box::new(left),
                        operation,
                        right: other_left.clone(),
                    },
                    None
                );

                Ok(ExprNode::new(
                    full_span,
                    ExprKind::BinaryOp {
                        left: Box::new(new_lhs),
                        operation: other_op.clone(),
                        right: other_right.clone(),
                    },
                    None
                ))
            }
            _ => {
                Ok(ExprNode::new(
                    left.span,
                    ExprKind::BinaryOp {
                        left: Box::new(left),
                        operation,
                        right: Box::new(right),
                    },
                    None
                ))
            },
        }
    }


    fn parse_operand(&mut self) -> Result<ExprNode<'a>, CompilerError<'a>> {
        let token = self.consume();
        let span = token.span;
        let mut atom = match &token.kind {
            // Primitive types
            TokenKind::Number(val) => {
                Ok(ExprNode::new(
                    span,
                    ExprKind::Number(*val as u32),
                    None
                ))
            },
            TokenKind::String(val) => {
                Ok(ExprNode::new(
                    span,
                    ExprKind::String(val.clone()),
                    None
                ))
            },
            TokenKind::Char(val) => {
                Ok(ExprNode::new(
                    span,
                    ExprKind::Char(*val),
                    None
                ))
            },
            TokenKind::Bool(val) => {
                Ok(ExprNode::new(
                    span,
                    ExprKind::Bool(*val),
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
            },

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
            },
            TokenKind::Star => {
                let expression = self.parse_operand()?;
                Ok(ExprNode::new(
                    span,
                    ExprKind::Dereference(expression.into()),
                    None
                ))
            },
            TokenKind::LParen => {
                let expr = self.parse_expression();
                self.expect(TokenKind::RParen, "Missing ')' after expression.")?;
                expr
            },

            // Invalid token
            _ => { Err(
                CompilerError::new(
                    span,
                    "Invalid expression.",
                )
            )}
        }?;

        loop {
            let next = self.peek(0);

            match next.kind {
                TokenKind::LParen => {
                    self.consume();

                    let mut args = Vec::new();
                    if !self.accept(TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression()?);

                            let next = self.consume();
                            match next.kind {
                                TokenKind::Comma => { continue; },
                                TokenKind::RParen => { break; },
                                _ => { return Err(
                                    CompilerError::new(
                                        next.span,
                                        "Missing ')' after function args.",
                                    )
                                )}
                            }
                        }

                    }

                    atom = ExprNode::new(
                        atom.span.extend(self.peek(-1).span),
                        ExprKind::FunctionCall {
                            function: Box::new(atom),
                            arguments: args,
                        },
                        None
                    )
                },

                TokenKind::LBracket => {
                    self.consume();
                    let expr = self.parse_expression()?;
                    self.expect(TokenKind::RBracket, "Missing ']' after index expression.")?;

                    atom = ExprNode::new(
                        atom.span.extend(self.peek(-1).span),
                        ExprKind::IndexAccess {
                            left: Box::new(atom),
                            index: Box::new(expr),
                        },
                        None
                    )
                },

                TokenKind::As => {
                    self.consume();
                    let new_type = self.parse_type()?;

                    atom = ExprNode::new(
                        atom.span.extend(self.peek(-1).span),
                        ExprKind::TypeCast {
                            new_type,
                            expr: Box::new(atom),
                        },
                        None
                    )
                }

                _ => { break }
            }

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

                if self.accept(TokenKind::RBracket) {
                    return Ok(ValueType::Array {
                        length: None,
                        inner_type: Box::new(inner),
                    });
                };

                self.expect(TokenKind::Semicolon, "Missing `;` between array type and length")?;

                let next = self.consume();
                let length = match next.kind {
                    TokenKind::Number(num) => num as usize,
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

                let ret_type = if self.accept(TokenKind::Arrow) {
                    self.parse_type()?
                } else {
                    ValueType::Primitive(NONE_TYPE.to_string())
                };


                Ok(ValueType::Function {
                    params,
                    ret_type: Box::new(ret_type)
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

    fn parse_identifier(&mut self) -> Result<String, CompilerError<'a>> {
        let name = self.consume();
        match &name.kind {
            TokenKind::Identifier(name) => Ok(name.clone()),
            _ => { return Err(CompilerError::new(
                name.span,
                "Invalid identifier.",
            ))}
        }
    }

    fn peek(&self, offset: i32) -> &Token<'a> {
        let pos = self.position as i32 + offset;
        if pos < 0 || pos >= self.tokens.len() as i32 {
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
        let prev = self.peek(-1).span;
        let next = self.consume();
        if mem::discriminant(&next.kind) == mem::discriminant(&kind) {
            Ok(next)
        } else {
            Err(CompilerError::new(
                if prev.start.line == next.span.start.line {
                    next.span
                } else {
                    prev
                },
                message,
            ))
        }
    }
}


