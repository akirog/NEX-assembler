use std::collections::HashMap;
use crate::compiler::*;


pub struct ScopeBuilder<'a> {
    type_table: HashMap<String, TypeDefinition<'a>>,
    declarations: &'a Vec<DeclNode<'a>>
}

impl<'a> ScopeBuilder<'a> {
    pub fn new(type_table: HashMap<String, TypeDefinition<'a>>, declarations: &'a Vec<DeclNode<'a>>) -> Self {
        Self {
            type_table,
            declarations,
        }
    }

    pub fn build(&mut self) -> Result<ScopeHandler, CompilerError<'a>> {
        let mut scope_handler = ScopeHandler::new();

        for node in self.declarations {
            match &node.kind {
                DeclKind::FunctionDeclaration { name, body, ret_type, arguments } => {
                    let mut resolved_params= Vec::new();

                    for arg in arguments {
                        let resolved_arg = self.resolve_type(&arg.field_type, arg.span)?;
                        resolved_params.push(resolved_arg);
                    };

                    scope_handler.register_global(name.clone(),
                        ResolvedType::Function {
                            ret_type: Box::new(self.resolve_type(ret_type, node.span)?),
                            params: resolved_params.clone(),
                        },
                        SymbolAddress::FunctionLabel(name.clone())
                    );

                    self.build_body_scope(&mut scope_handler, body, name.clone(), Some(arguments))?;

                }
                _ => { continue; }
            }
        };

        Ok(scope_handler)
    }

    fn build_body_scope(
        &self,
        scope_handler: &mut ScopeHandler,
        body: &'a BlockNode<'a>,
        name: String,
        extras: Option<&Vec<UnresolvedField<'a>>>
    ) -> Result<(), CompilerError<'a>> {

        let parent_scope: &Scope = scope_handler.get_curr_scope();
        let parent_base = parent_scope.base_offset;
        let parent_size = parent_scope.size;

        scope_handler.make_new_scope(body.id);
        scope_handler.get_curr_scope_mut().debug_name = name.clone();

        let mut curr_offset = parent_base + parent_size;

        if let Some(extras) = extras {
            for extra in extras {
                let resolved_type = self.resolve_type(&extra.field_type, extra.span)?;

                let size = resolved_type.get_size();
                scope_handler.register_symbol(extra.name.clone(), resolved_type, SymbolAddress::StackOffset(curr_offset));
                curr_offset += size;

            }
        }


        // Variable declarations
        for node in &body.statements {
            match &node.kind {
                StmtKind::VariableDeclaration { name, value: _, var_type } => {
                    let resolved_type = self.resolve_type(&var_type, node.span)?;

                    let var_address = SymbolAddress::StackOffset(curr_offset);
                    curr_offset += resolved_type.get_size();

                    scope_handler.register_symbol(name.clone(), resolved_type, var_address);
                },
                _ => { continue; }
            }
        }

        let this_scope = scope_handler.get_curr_scope_mut();
        this_scope.base_offset = parent_base + parent_size;
        this_scope.size = curr_offset - this_scope.base_offset;

        // Inner scopes
        for node in &body.statements {
            match &node.kind {
                StmtKind::IfStatement { condition: _, body } => {
                    self.build_body_scope(scope_handler, body, "If statement".to_string(), None)?;
                },
                StmtKind::WhileLoop { condition: _, body } => {
                    self.build_body_scope(scope_handler, body, "While loop".to_string(), None)?;
                },
                StmtKind::ForLoop { init: _, condition: _, update: _, body } => {
                    self.build_body_scope(scope_handler, body, "for loop".to_string(), None)?;
                },
                StmtKind::Loop { body } => {
                    self.build_body_scope(scope_handler, body, "Loop".to_string(), None)?;
                },
                _ => { continue; }
            }
        }


        scope_handler.pop_scope();

        Ok(())
    }

    fn resolve_type(&self, unresolved: &UnresolvedType, span: Span<'a>) -> Result<ResolvedType, CompilerError<'a>> {
        match unresolved {
            UnresolvedType::Primitive(name) => {
                // Check for primitive
                if let Some(primitive) = ResolvedPrimitive::from_str(name) {
                    return Ok(ResolvedType::Primitive(primitive));
                };

                let new_type = self.type_table.get(name);
                if new_type.is_none() {
                    return Err(CompilerError::new(
                        span,
                        format!("Could not resolve type {}.", name).as_str(),
                    ));
                }
                let new_type = new_type.unwrap();

                let resolved_type = match &new_type.kind {
                    TypeKind::Alias(ty) => {
                        self.resolve_type(&ty, span)?
                    },
                    TypeKind::Struct { fields } => {
                        let mut resolved_fields = Vec::new();
                        for field in fields {
                            let resolved_field_type = self.resolve_type(&field.field_type, span)?;
                            resolved_fields.push(ResolvedField {
                                name: field.name.clone(),
                                field_type: resolved_field_type,
                            });
                        }
                        ResolvedType::Struct(resolved_fields)
                    },
                    _ => { return Err(CompilerError::new(
                        span,
                        "Not implemented yet."
                    ))}
                };

                Ok(resolved_type)
            },
            UnresolvedType::Array { length, inner_type } => {
                if length.is_none() {
                    return Err(CompilerError::new(
                        span,
                        "Could not resolve type, undeclared array length."
                    ));
                }

                let resolved_inner = self.resolve_type(inner_type, span)?;

                Ok(ResolvedType::Array {
                    length: length.unwrap(),
                    inner_type: Box::new(resolved_inner)
                })
            },
            UnresolvedType::Pointer(inner_type) => {
                let resolved_inner = self.resolve_type(inner_type, span)?;
                Ok(ResolvedType::Pointer(Box::new(resolved_inner)))
            },
            UnresolvedType::Function { ret_type, params } => {
                let resolved_ret_type = self.resolve_type(ret_type, span)?;
                let mut resolved_args = Vec::new();

                for param in params {
                    resolved_args.push(self.resolve_type(param, span)?);
                }

                Ok(ResolvedType::Function {
                    ret_type: Box::new(resolved_ret_type),
                    params: resolved_args,
                })
            }
        }
    }

}