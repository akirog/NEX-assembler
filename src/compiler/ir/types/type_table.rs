use std::collections::HashMap;
use crate::compiler::errors::CompilerError;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::types::{Type, TypeId, TypeKind, WrittenType, WrittenTypeKind};

pub struct TypeTable {
    types: HashMap<TypeId, TypeKind>,
    reverse_types: HashMap<TypeKind, TypeId>,
    symbol_table: HashMap<SymbolId, TypeId>,
}


impl<'a> TypeTable {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            reverse_types: HashMap::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn get_symbol_type_id(&self, id: SymbolId) -> Option<TypeId> {
        self.symbol_table.get(&id).cloned()
    }

    pub fn get_written_type_id(&mut self, ty: &WrittenType<'a>) -> Result<TypeId, CompilerError<'a>> {
        let type_id = match &ty.kind {
            WrittenTypeKind::Name(name) => {
                let type_kind = match name.as_str() {
                    "u8" => TypeKind::U8,
                    "u32" => TypeKind::U32,
                    "i8" => TypeKind::I8,
                    "i32" => TypeKind::I32,
                    "bool" => TypeKind::Bool,
                    "char" => TypeKind::Char,
                    _ => panic!("Not implemented this yet: {}", name)
                };

                self.get_type_kind_id(&type_kind)
            },
            WrittenTypeKind::Array { ty, length } => {
                let type_id = self.get_written_type_id(&*ty)?;

                if length.is_none() {
                    return Err(CompilerError::new(
                        ty.span,
                        "Array has no declared length",
                    ))
                }
                let length = length.unwrap();

                let type_kind = TypeKind::Array {
                    ty: type_id,
                    len: length
                };

                self.get_type_kind_id(&type_kind)
            },
            WrittenTypeKind::Pointer(written_type) => {
                let pointee_ty = self.get_written_type_id(&*written_type)?;

                let type_kind = TypeKind::Pointer(pointee_ty);

                self.get_type_kind_id(&type_kind)
            },
            WrittenTypeKind::FuncPtr { ret_type, params } => {
                let ret_ty_id = self.get_written_type_id(&*ret_type)?;

                let mut param_ids = Vec::with_capacity(params.len());
                for param in params {
                    param_ids.push(self.get_written_type_id(&param)?);
                };

                let type_kind = TypeKind::FuncPtr {
                    ret_ty: ret_ty_id,
                    params: param_ids,
                };

                self.get_type_kind_id(&type_kind)
            }
        };

        Ok(type_id)
    }

    /// Takes two type id's, ensures they are the same type, resolving unknowns to the other type, and returns the type both are.
    pub fn enforce_same_type(&self, ty1: &mut TypeId, ty2: &mut TypeId) -> Option<TypeId> {
        let ty = if ty1 == ty2 {
            ty1.clone()
        } else {
            match (self.get_type(ty1), self.get_type(ty2)) {
                (TypeKind::Unresolved, _) => {
                    *ty1 = ty2.clone();
                    ty1.clone()
                },
                (_, TypeKind::Unresolved) => {
                    *ty2 = ty1.clone();
                    ty1.clone()
                },
                (TypeKind::UnresolvedNum, TypeKind::U8 | TypeKind::U32 | TypeKind:: I8 | TypeKind::I32) => {

                }
            }
        };

        Some(ty)
    }

    pub fn get_type_kind_id(&mut self, ty: &TypeKind) -> TypeId {
        if self.reverse_types.contains_key(&ty) {
            self.reverse_types.get(&ty).unwrap().clone()
        } else {
            self.register_type(ty)
        }
    }

    fn register_type(&mut self, ty: &TypeKind) -> TypeId {
        let id = TypeId::new();
        self.types.insert(id.clone(), ty.clone());
        self.reverse_types.insert(ty.clone(), id.clone());
        id
    }

    pub fn can_index_type(&self, ty: &TypeId) -> bool {


        match self.get_type(ty) {
            TypeKind::Array { .. } => true,
            TypeKind::Pointer { .. } => true,
            _ => false,
        }
    }

    pub fn get_type(&self, ty: &TypeId) -> TypeKind {
        if !self.types.contains_key(ty) {
            panic!("Can't index type of non-existent type id: {:?}", ty);
        }
        self.types[ty].clone()
    }
}