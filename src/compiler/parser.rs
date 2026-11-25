use std::mem;

use crate::compiler::errors::*;
use crate::compiler::ir::ast::*;
use crate::compiler::ir::operations::Operation;
use crate::compiler::ir::types::*;
use crate::compiler::lexer::{Token, TokenKind};


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
    pub fn parse(&mut self) -> Result<Vec<AstStmt<'a>>, CompilerError<'a>> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            nodes.push(self.parse_statement()?);
        }

        Ok(nodes)
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(0).kind, TokenKind::EOF)
    }


    fn parse_statement(&mut self) -> Result<AstStmt<'a>, CompilerError<'a>> {
        let token = self.peek(0);

        let node = match token.kind {
            TokenKind::Let => {
                self.parse_variable_declaration()?
            },

            // Items
            TokenKind::Type | TokenKind::Struct | TokenKind::Enum | TokenKind::Fn => {
                let item = self.parse_item()?;
                AstStmt::new(
                    item.span,
                    AstStmtKind::Item(Box::new(item)),
                )
            },

            TokenKind::Return => {
                let span = token.span;
                self.consume();
                if self.accept(TokenKind::Semicolon) {
                    return Ok(AstStmt::new(
                        span,
                        AstStmtKind::Return { value: None },
                    ))
                };

                let expr = self.parse_expression(0)?;
                self.expect(TokenKind::Semicolon, "Expected ';' after return statement")?;
                AstStmt::new(
                    span.extend(self.peek(-1).span),
                    AstStmtKind::Return { value: Some(Box::new(expr)) }
                )
            },

            TokenKind::Asm => {
                self.parse_asm_block()?
            }

            _ => {
                let stmt = self.parse_expression_statement()?;
                self.accept(TokenKind::Semicolon);
                stmt
            },
        };

        Ok(node)
    }


    fn parse_item(&mut self) -> Result<AstItem<'a>, CompilerError<'a>> {
        let token = self.peek(0);

        let node = match token.kind {
            TokenKind::Type => {
                self.parse_type_declaration()?
            }
            TokenKind::Struct => {
                self.parse_struct_declaration()?
            },
            TokenKind::Fn => {
                self.parse_function_declaration()?
            },
            _ => {
                return Err(CompilerError::new(
                    token.span,
                    "Invalid global declaration, expected `struct`, `fn`, `enum`, or `type. If what you wrote is in this list update parse declaration as it is not implemented yet."
                ));
            }
        };

        Ok(node)
    }



    fn parse_type_declaration(&mut self) -> Result<AstItem<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::Type, "Missing `type` keyword at start of type declaration")?.span;

        let name = self.parse_identifier()?;

        self.expect(TokenKind::Equals, "Missing equals after type declaration name.")?;

        let new_type = self.parse_type()?;

        self.expect(TokenKind::Semicolon, "Missing semicolon after type declaration")?;


        Ok(AstItem::new(
            start_span.extend(self.peek(-1).span),
            AstItemKind::TypeDeclaration {
                name,
                new_type
            }
        ))
    }

    fn parse_struct_declaration(&mut self) -> Result<AstItem<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::Struct, "Expected struct definition")?.span;

        let name = self.parse_identifier()?;

        self.expect(TokenKind::LBrace, "Missing struct definition body.")?;

        let mut fields = Vec::new();
        while !matches!(self.peek(0).kind, TokenKind::RBrace) && !self.is_eof() {
            let start_field_span = self.peek(0).span;
            let field_name = self.parse_identifier()?;
            self.expect(TokenKind::Colon, "Missing field type.")?;

            let field_type = self.parse_type()?;
            fields.push(WrittenField {
                name: field_name,
                ty: field_type,
                span: start_field_span.extend(self.peek(-1).span),
            });

            self.accept(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace, "Missing right brace at end of struct definition.")?;


        Ok(AstItem::new(
            start_span.extend(self.peek(-1).span),
            AstItemKind::StructDefinition {
                name,
                fields,
            }
        ))

    }

    fn parse_if_expression(&mut self) -> Result<AstExpr<'a>, CompilerError<'a>> {
        let start_span = self.peek(-1).span;

        let condition = self.parse_expression(0)?;

        let body = self.parse_body()?;

        let mut else_statements = Vec::new();
        while self.accept(TokenKind::Else) {
            let else_condition = if self.peek(0).kind == TokenKind::If {
                self.consume();
                Some(Box::new(self.parse_expression(0)?))
            } else {
                None
            };

            let else_body = Box::new(self.parse_body()?);
            else_statements.push((else_condition, else_body));
        }

        Ok(AstExpr::new(
            start_span.extend(self.peek(-1).span),
            AstExprKind::IfExpression {
                condition: Box::new(condition),
                body: Box::new(body),
                else_statements
            }
        ))
    }

    fn parse_while_loop(&mut self) -> Result<AstExpr<'a>, CompilerError<'a>> {
        let start_span = self.peek(-1).span;

        let condition = self.parse_expression(0)?;
        let body = self.parse_body()?;
        Ok(AstExpr::new(
            start_span.extend(self.peek(-1).span),
            AstExprKind::WhileLoop {
                condition: Box::new(condition),
                body: Box::new(body),
            }
        ))
    }

    fn parse_for_loop(&mut self) -> Result<AstExpr<'a>, CompilerError<'a>> {
        let start_span = self.peek(-1).span;

        let init = self.parse_statement()?;
        let condition = self.parse_expression(0)?;
        self.accept(TokenKind::Semicolon);
        let update = self.parse_expression_statement()?; // Parse expression statement skips ; requirement

        let body = self.parse_body()?;

        Ok(AstExpr::new(
            start_span.extend(self.peek(-1).span),
            AstExprKind::ForLoop {
                init: Box::new(init),
                condition: Box::new(condition),
                update: Box::new(update),
                body: Box::new(body),
            }
        ))
    }

    fn parse_unconditional_loop(&mut self) -> Result<AstExpr<'a>, CompilerError<'a>> {
        let start_span = self.peek(-1).span;

        let body = self.parse_body()?;

        Ok(AstExpr::new(
            start_span.extend(self.peek(-1).span),
            AstExprKind::Loop {
                body: Box::new(body)
            }
        ))
    }

    // Parse expression statement will parse an expression, then check for an equals sign, if found it returns the correct assignment statement, else an expression statement
    fn parse_expression_statement(&mut self) -> Result<AstStmt<'a>, CompilerError<'a>> {
        let expr = self.parse_expression(0)?;

        if matches!(self.peek(0).kind, TokenKind::Semicolon) {
            return Ok(AstStmt::new(
                expr.span.extend(self.peek(-1).span),
                AstStmtKind::Semi(Box::new(expr)),
            ));
        };

        let mut assign_op = None;
        let next = self.peek(0);
        match &next.kind {
            TokenKind::Equals => {  },
            TokenKind::PlusAssign => {
                assign_op = Some(Operation::Add)
            },
            TokenKind::MinusAssign => {
                assign_op = Some(Operation::Subtract)
            },
            TokenKind::StarAssign => {
                assign_op = Some(Operation::Multiply)
            },
            TokenKind::SlashAssign => {
                assign_op = Some(Operation::Divide)
            },
            TokenKind::PercentAssign => {
                assign_op = Some(Operation::Modulo)
            },
            TokenKind::CaretAssign => {
                assign_op = Some(Operation::Xor);
            },

            _ => {
                return Ok(AstStmt::new(
                    expr.span.extend(self.peek(-1).span),
                    AstStmtKind::Expression(Box::new(expr)),
                ));
            }
        }
        self.consume();

        let mut rhs = self.parse_expression(0)?;

        if let Some(op) = assign_op {
            rhs = AstExpr::new(
                rhs.span,
                AstExprKind::BinaryOp {
                    left: Box::new(expr.clone()),
                    operation: op,
                    right: Box::new(rhs),
                }
            )
        }


        Ok(AstStmt::new(
            expr.span.extend(self.peek(0).span),
            AstStmtKind::Assignment {
                target: Box::new(expr),
                value: Box::new(rhs),
            }
        ))
    }


    fn parse_variable_declaration(&mut self) -> Result<AstStmt<'a>, CompilerError<'a>> {
        let start_span = self.expect(TokenKind::Let, "Missing let keyword in variable declaration.")?.span;

        // Var name
        let name = self.parse_identifier()?;

        let var_type = if self.accept(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };



        // Check for early exit
        let next = self.consume();
        match next.kind {
            TokenKind::Semicolon => {
                return Ok(AstStmt::new(
                    start_span.extend(next.span),
                    AstStmtKind::VariableDeclaration {
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
        let expression = self.parse_expression(0)?;

        // Semicolon
        let end_span = self.expect(TokenKind::Semicolon, "Missing ';' after declaration")?.span;

        // Return
        Ok(AstStmt::new(
            start_span.extend(end_span),
            AstStmtKind::VariableDeclaration {
                var_type,
                name,
                value: Some(expression.into()),
            }
        ))
    }


    fn parse_function_declaration(&mut self) -> Result<AstItem<'a>, CompilerError<'a>> {
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
            WrittenType {
                span: Span::none(),
                kind: WrittenTypeKind::Name(NONE_TYPE.to_string())
            }
        };

        let end_span = self.peek(-1).span;

        // Body
        let body = self.parse_body()?;


        Ok(AstItem::new(
            start_span.extend(end_span),
            AstItemKind::FunctionDeclaration {
                name,
                ret_type,
                arguments,
                body
            }
        ))
    }


    fn parse_expression(&mut self, min_precedence: usize) -> Result<AstExpr<'a>, CompilerError<'a>> {
        let mut left = self.parse_operand()?;


        loop {
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
                TokenKind::Percent => {
                    Operation::Modulo
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
                TokenKind::Ampersand => {
                    Operation::And
                },
                TokenKind::LogicalAnd => {
                    Operation::LogicalAnd
                },
                TokenKind::Pipe => {
                    Operation::Or
                },
                TokenKind::LogicalOr => {
                    Operation::LogicalOr
                }
                _ => { break; }
            };

            if operation.precedence() < min_precedence { break; }
            self.consume();

            let right = self.parse_expression(operation.precedence())?;
            let span = left.span.extend(right.span);

            left = AstExpr::new(
                span,
                AstExprKind::BinaryOp {
                    left: Box::new(left),
                    right: Box::new(right),
                    operation
                },
            )
        }

        Ok(left)
    }


    fn parse_operand(&mut self) -> Result<AstExpr<'a>, CompilerError<'a>> {
        let token = self.consume();
        let span = token.span;
        let mut atom = match &token.kind {
            // Primitive types
            TokenKind::Number(val) => {
                Ok(AstExpr::new(
                    span,
                    AstExprKind::Number(*val),
                ))
            },
            TokenKind::String(val) => {
                Ok(AstExpr::new(
                    span,
                    AstExprKind::String(val.clone()),
                ))
            },
            TokenKind::Char(val) => {
                Ok(AstExpr::new(
                    span,
                    AstExprKind::Char(*val),
                ))
            },
            TokenKind::Bool(val) => {
                Ok(AstExpr::new(
                    span,
                    AstExprKind::Bool(*val),
                ))
            },

            // Array literal
            TokenKind::LBracket => {
                let mut values = Vec::new();
                while !matches!(self.peek(0).kind, TokenKind::RBracket) && !self.is_eof() {
                    values.push(self.parse_expression(0)?);
                    match self.peek(0).kind {
                        TokenKind::RBracket => {
                            self.consume();
                            break;
                        },
                        TokenKind::Comma => {
                            self.consume();
                        }
                        _ => { return Err(CompilerError::new(
                            self.peek(0).span,
                            "invalid token in array literal"
                        )) }
                    }
                }
                Ok(AstExpr::new(
                    span.extend(self.peek(-1).span),
                    AstExprKind::ArrayLiteral(values),
                ))
            }

            // Identifier
            TokenKind::Identifier(name) => {
                let name = name.clone();

                // Check for struct literal
                if matches!(self.peek(0).kind, TokenKind::LBrace) {
                    self.consume();
                    let mut fields = Vec::new();

                    while self.peek(0).kind != TokenKind::RBrace {
                        let start_span = self.peek(0).span;
                        let name = self.parse_identifier()?;
                        self.expect(TokenKind::Colon, "Missing colon in struct literal")?;
                        let value = self.parse_expression(0)?;
                        fields.push(AstFieldExpr {
                            name,
                            value: Box::new(value),
                            span: start_span.extend(self.peek(-1).span),
                        });

                        match self.peek(0).kind {
                            TokenKind::RBrace => { break; },
                            TokenKind::Comma => { self.consume(); },
                            _ => { return Err(CompilerError::new(
                                self.peek(0).span,
                                "invalid token in struct literal"
                            ))}
                        }
                    }

                    self.expect(TokenKind::RBrace, "End of struct literal missing")?;

                    Ok(AstExpr::new(
                        span.extend(self.peek(-1).span),
                        AstExprKind::StructLiteral {
                            name,
                            fields,
                        }
                    ))

                } else {
                    Ok(AstExpr::new(
                        span,
                        AstExprKind::Identifier(name),
                    ))
                }
            },

            // Continue or Break
            TokenKind::Continue => {
                Ok(AstExpr::new(span, AstExprKind::Continue { value: None }))
            },
            TokenKind::Break => {
                Ok(AstExpr::new(span, AstExprKind::Break { value: None }))
            }

            // Prefixes like &
            TokenKind::Ampersand => {
                let expr = self.parse_operand()?;

                Ok(AstExpr::new(
                    span.extend(expr.span),
                    AstExprKind::AddressOf(Box::new(expr)),
                ))
            },
            TokenKind::Star => {
                let expression = self.parse_operand()?;
                Ok(AstExpr::new(
                    span,
                    AstExprKind::Dereference(expression.into()),
                ))
            },
            TokenKind::LParen => {
                let expr = self.parse_expression(0);
                self.expect(TokenKind::RParen, "Missing ')' after expression.")?;
                expr
            },

            // Statement expressions
            TokenKind::If => {
                self.parse_if_expression()
            },
            TokenKind::For => {
                self.parse_for_loop()
            },
            TokenKind::While => {
                self.parse_while_loop()
            },
            TokenKind::Loop => {
                self.parse_unconditional_loop()
            },
            TokenKind::LBrace => {
                // Quick fix
                self.position -= 1;
                let block = self.parse_body()?;
                Ok(AstExpr::new(
                    block.span,
                    AstExprKind::BlockExpression(Box::new(block))
                ))
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
                            args.push(self.parse_expression(0)?);

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

                    atom = AstExpr::new(
                        atom.span.extend(self.peek(-1).span),
                        AstExprKind::FunctionCall {
                            function: Box::new(atom),
                            arguments: args,
                        },
                    )
                },

                TokenKind::LBracket => {
                    self.consume();
                    let expr = self.parse_expression(0)?;
                    self.expect(TokenKind::RBracket, "Missing ']' after index expression.")?;

                    atom = AstExpr::new(
                        atom.span.extend(self.peek(-1).span),
                        AstExprKind::IndexAccess {
                            left: Box::new(atom),
                            index: Box::new(expr),
                        },
                    )
                },

                TokenKind::As => {
                    self.consume();
                    let new_type = self.parse_type()?;

                    atom = AstExpr::new(
                        atom.span.extend(self.peek(-1).span),
                        AstExprKind::TypeCast {
                            new_type,
                            expr: Box::new(atom),
                        },
                    )
                }

                TokenKind::Dot => {
                    // Member access
                    self.consume();
                    let atom_span = atom.span.clone();
                    let left = atom;

                    let member = match &self.consume().kind {
                        TokenKind::Identifier(name) => { name.clone() },
                        _ => { return Err(CompilerError::new(
                            self.peek(-1).span,
                            "Missing member in member access.",
                        ))}
                    };

                    atom = AstExpr::new(
                        atom_span.extend(self.peek(-1).span),
                        AstExprKind::MemberAccess {
                            left: Box::new(left),
                            member
                        }
                    )
                }

                TokenKind::Arrow => {
                    self.consume();
                    let member = self.consume();
                    let member_span = member.span.clone();

                    let member = match &member.kind {
                        TokenKind::Identifier(name) => { name.clone() },
                        _ => { return Err(
                            CompilerError::new(
                                member_span,
                                "Invalid member access.",
                            )
                        )}
                    };

                    atom = AstExpr::new(
                        atom.span.extend(member_span),
                        AstExprKind::DerefMemberAccess {
                            left: Box::new(atom),
                            member
                        }
                    )
                }

                _ => { break }
            }

        }

        Ok(atom)
    }


    fn parse_body(&mut self) -> Result<AstBlock<'a>, CompilerError<'a>> {
        let mut statements = Vec::new();

        let start_span = self.peek(0).span;
        self.expect(TokenKind::LBrace, "Missing starting '{' for block.")?;
        while !matches!(self.peek(0).kind, TokenKind::RBrace) && !self.is_eof() {
            statements.push(self.parse_statement()?);
        }
        let end_span = self.peek(0).span;
        self.expect(TokenKind::RBrace, "Missing ending '}'.")?;

        Ok(AstBlock::new(
            statements,
            start_span.extend(end_span),
        ))
    }


    fn parse_fields(&mut self) -> Result<Vec<WrittenField<'a>>, CompilerError<'a>> {
        let mut fields = Vec::new();
        if !self.accept(TokenKind::RParen) {
            loop {
                let name = self.consume();
                let start_field_span = name.span;
                let name = match &name.kind {
                    TokenKind::Identifier(ident) => ident.clone(),
                    _ => { return Err(CompilerError::new(
                        name.span,
                        "Invalid field name.",
                    ))}
                };

                self.expect(TokenKind::Colon, "Missing ':' separator in field declaration.")?;

                let field_type = self.parse_type()?;

                fields.push(WrittenField {
                    name,
                    ty: field_type,
                    span: start_field_span.extend(self.peek(-1).span),
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
    fn parse_type(&mut self) -> Result<WrittenType<'a>, CompilerError<'a>> {
        // With the new type system, we simply match the token and dispatch work
        let next = self.consume();
        let start_span = next.span;
        let kind = match &next.kind {
            TokenKind::Identifier(name) => {
                WrittenTypeKind::Name(name.clone())
            },
            TokenKind::Star => {
                let inner = self.parse_type()?;
                WrittenTypeKind::Pointer(Box::new(inner))
            },
            TokenKind::LBracket => {
                let inner = self.parse_type()?;

                if self.accept(TokenKind::RBracket) {
                    WrittenTypeKind::Slice(Box::new(inner))
                } else {
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

                    WrittenTypeKind::Array {
                        length: Some(length),
                        ty: Box::new(inner),
                    }
                }
            }
            TokenKind::Fn => {
                self.expect(TokenKind::LParen, "Missing '(' in function type.")?;
                let args = self.parse_fields()?;
                let mut params = Vec::with_capacity(args.len());
                for arg in args {
                    params.push(arg.ty);
                };

                let ret_type = if self.accept(TokenKind::Arrow) {
                    self.parse_type()?
                } else {
                    WrittenType {
                        span: Span::none(),
                        kind: WrittenTypeKind::Name(NONE_TYPE.to_string())
                    }
                };

                WrittenTypeKind::FuncPtr {
                    params,
                    ret_type: Box::new(ret_type)
                }
            }
            _ => {
                return Err(
                    CompilerError::new(
                        next.span,
                        "Invalid type.",
                    )
                )
            }
        };

        Ok(WrittenType {
            span: start_span.extend(self.peek(-1).span),
            kind,
        })
    }

    fn parse_identifier(&mut self) -> Result<String, CompilerError<'a>> {
        let name = self.consume();
        match &name.kind {
            TokenKind::Identifier(name) => Ok(name.clone()),
            _ => { Err(CompilerError::new(
                name.span,
                "Invalid identifier.",
            ))}
        }
    }

    fn parse_asm_block(&mut self) -> Result<AstStmt<'a>, CompilerError<'a>> {
        let start_span = self.peek(0).span;
        self.expect(TokenKind::Asm, "Uhh asm block")?;

        self.expect(TokenKind::LParen, "Missing '(' in asm block.")?;

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut lines = Vec::new();

        while !self.accept(TokenKind::RParen) {
            let next = self.consume();
            match &next.kind {
                TokenKind::Identifier(name) => {
                    let target_vec = match name.as_str() {
                        "in" => &mut inputs,
                        "out" => &mut outputs,
                        _ => { return Err(
                            CompilerError::new(
                                next.span,
                                "Invalid identifier.",
                            )
                        )}
                    };

                    self.expect(TokenKind::LParen, "Missing '(' in asm input/output.")?;
                    let value = self.parse_expression(0)?;
                    self.expect(TokenKind::Comma, "Missing ',' between value/dest.")?;
                    let reg_token = self.consume();
                    let reg_num = match reg_token.kind {
                        TokenKind::Number(num) => num,
                        _ => { return Err(
                            CompilerError::new(
                                reg_token.span,
                                "Invalid reg_num.",
                            )
                        )}
                    } as usize;
                    self.expect(TokenKind::RParen, "Missing ')' after asm input/output.")?;

                    target_vec.push((value, crate::compiler::assembly_generation::register_manager::Register::from_num(reg_num)));
                },
                TokenKind::String(text) => {
                    lines.push(text.clone());
                },
                _ => {
                    return Err(
                        CompilerError::new(
                            next.span,
                            "Invalid asm block input.",
                        )
                    )
                }
            }

            if !self.accept(TokenKind::Comma) {
                self.expect(TokenKind::RParen, "Missing ')' after asm block.")?;
                break;
            }
        }
        self.expect(TokenKind::Semicolon, "Missing ';' after asm block.")?;

        Ok(AstStmt::new(
            start_span.extend(self.peek(-1).span),
            AstStmtKind::AsmBlock {
                inputs,
                outputs,
                lines
            }
        ))
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


