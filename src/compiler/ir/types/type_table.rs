use crate::compiler::errors::CompilerError;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::types::{TypeId, TypeKind, WrittenType, WrittenTypeKind};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TypeTable {
    pub types: HashMap<TypeId, TypeKind>,
    reverse_types: HashMap<TypeKind, TypeId>,
    symbol_table: HashMap<SymbolId, TypeId>,

    struct_names: HashMap<String, TypeId>,
}


impl<'a> TypeTable {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            reverse_types: HashMap::new(),
            symbol_table: HashMap::new(),

            struct_names: HashMap::new(),
        }
    }

    pub fn get_symbol_type_id(&self, id: &SymbolId) -> Option<TypeId> {
        self.symbol_table.get(id).cloned()
    }

    pub fn define_struct(&mut self, name: String, ty: &TypeKind) {
        let id = self.get_type_kind_id(ty);
        self.struct_names.insert(name, id);
    }

    pub fn get_struct_type_id(&self, name: &str) -> Option<TypeId> {
        self.struct_names.get(name).cloned()
    }

    pub fn register_symbol_type(&mut self, id: &SymbolId, type_id: &TypeId) {
        self.symbol_table.insert(id.clone(), type_id.clone());
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
                    "void" => TypeKind::None,
                    _ => {
                        if self.struct_names.contains_key(name) {
                            self.get_type(self.struct_names.get(name).unwrap())
                        } else {
                            return Err(CompilerError::new(
                                ty.span,
                                "Undefined type."
                            ))
                        }
                    }
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
            WrittenTypeKind::Slice(written_type) => {
                let pointee_ty = self.get_written_type_id(&*written_type)?;
                let type_kind = TypeKind::Slice(pointee_ty);
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
                    *ty1 = ty2.clone();
                    ty1.clone()
                },
                (TypeKind::U8 | TypeKind::U32 | TypeKind:: I8 | TypeKind::I32, TypeKind::UnresolvedNum) => {
                    *ty2 = ty1.clone();
                    ty1.clone()
                },
                (TypeKind::Slice(slice_ty), TypeKind::Pointer(ptr_ty)) => {
                    let pointed_type = self.get_type(&ptr_ty);
                    let TypeKind::Array { ty: arr_ty, len: _ } = pointed_type else {
                        return None
                    };
                    if slice_ty != arr_ty { return None; }

                    *ty2 = ty1.clone();
                    ty1.clone()
                },
                (TypeKind::Pointer(ptr_ty), TypeKind::Slice(slice_ty)) => {
                    let pointed_type = self.get_type(&ptr_ty);
                    let TypeKind::Array { ty: arr_ty, len: _ } = pointed_type else {
                        return None
                    };
                    if slice_ty != arr_ty { return None; }

                    *ty1 = ty2.clone();
                    ty1.clone()
                },
                _ => return None
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

    pub fn register_type(&mut self, ty: &TypeKind) -> TypeId {
        let id = TypeId::new();
        self.types.insert(id.clone(), ty.clone());
        self.reverse_types.insert(ty.clone(), id.clone());
        id
    }

    pub fn can_index_type(&self, ty: &TypeId) -> bool {
        match self.get_type(ty) {
            TypeKind::Array { .. } => true,
            TypeKind::Slice(_) => true,
            _ => false,
        }
    }

    pub fn dereference_type(&self, ty: &TypeId) -> Option<TypeId> {
        match self.get_type(ty) {
            TypeKind::Pointer(ty) => Some(ty),
            TypeKind::Array { ty, .. } => Some(ty),
            TypeKind::Slice(ty) => Some(ty),
            _ => None,
        }
    }

    pub fn get_member_type(&mut self, ty: &TypeId, member: &String) -> Option<TypeId> {
        match self.get_type(ty) {
            TypeKind::Struct { fields } => {
                fields.get_member_type(member).cloned()
            },
            TypeKind::Slice(ty) => match member.as_str() {
                "len" => Some(self.get_type_kind_id(&TypeKind::U32)),
                "ptr" => Some(self.get_type_kind_id(&TypeKind::Pointer(ty))),
                _ => None
            },
            TypeKind::Array { .. } => match member.as_str() {
                "len" => Some(self.get_type_kind_id(&TypeKind::U32)),
                _ => None
            },
            _ => None
        }
    }

    pub fn get_member_offset(&self, ty: &TypeId, member: &String) -> Option<usize> {
        const SLICE_LEN_OFFSET: usize = 4;
        const SLICE_PRT_OFFSET: usize = 0;


        let fields = match self.get_type(ty) {
            TypeKind::Struct { fields } => fields,
            TypeKind::Slice(_) => return match member.as_str() {
                "len" => Some(SLICE_LEN_OFFSET),
                "ptr" => Some(SLICE_PRT_OFFSET),
                _ => None
            },
            _ => return None
        };

        let mut offset = 0;
        for (name, field_ty) in fields.names_types() {
            if name == member {
                return Some(offset);
            }
            offset += self.get_type_size(field_ty);
        }
        None
    }

    pub fn get_type(&self, ty: &TypeId) -> &TypeKind {
        if !self.types.contains_key(ty) {
            panic!("Can't get type kind of non-existent type id: {:?}", ty);
        }

        &self.types[ty]
    }

    pub fn get_type_mut(&mut self, ty: &TypeId) -> &mut TypeKind {
        if !self.types.contains_key(ty) {
            panic!("Can't get type kind of non-existent type id: {:?}", ty);
        }
        &mut self.types[ty]
    }

    pub fn get_type_size(&self, ty: &TypeId) -> usize {
        const USIZE: usize = 4;

        match self.get_type(ty) {
            TypeKind::U8 => 1,
            TypeKind::U32 => 4,
            TypeKind::I8 => 1,
            TypeKind::I32 => 4,
            TypeKind::Bool => 1,
            TypeKind::Char => 1,
            TypeKind::Array { ty, len } => { self.get_type_size(&ty) * len },
            TypeKind::Struct { fields } => {
                let mut size = 0;
                for field in fields.types {
                    size += self.get_type_size(&field)
                }
                size
            },
            TypeKind::Pointer(_) => USIZE,
            TypeKind::Slice(_) => 2*USIZE,
            TypeKind::FuncPtr {..} => USIZE,
            TypeKind::UnresolvedNum => USIZE,
            _ => {
                panic!("Attempted to get size of type \n{}", self.type_as_string(ty));
            },
        }
    }

    /// If a type is an aggregate, aka it takes up more than a register
    pub fn is_aggregate(&self, ty: &TypeId) -> bool {
        match self.get_type(ty) {
            TypeKind::Array { .. }  => true,
            TypeKind::Slice(_)      => true,
            TypeKind::Struct { .. } => true,
            _ => false
        }
    }

    pub fn type_as_string(&self, ty: &TypeId) -> String {
        match self.get_type(ty) {
            TypeKind::U8 => "u8".to_string(),
            TypeKind::U32 => "u32".to_string(),
            TypeKind::I8 => "i8".to_string(),
            TypeKind::I32 => "i32".to_string(),
            TypeKind::Bool => "bool".to_string(),
            TypeKind::Char => "char".to_string(),

            TypeKind::Pointer(pointee_type) => {
                format!("*{}", self.type_as_string(&pointee_type))
            }
            TypeKind::Slice(element_type) => {
                format!("[{}]", self.type_as_string(&element_type))
            }
            TypeKind::Array { ty: element_type, len: length } => {
                format!("[{}; {}]", self.type_as_string(&element_type), length)
            }

            TypeKind::FuncPtr { ret_ty: return_type, params: parameter_types }
            | TypeKind::Function { ret_ty: return_type, params: parameter_types } => {
                let parameter_strings: Vec<String> = parameter_types
                    .iter()
                    .map(|parameter_type| self.type_as_string(parameter_type))
                    .collect();
                format!(
                    "fn({}) -> {}",
                    parameter_strings.join(", "),
                    self.type_as_string(&return_type)
                )
            }

            TypeKind::Struct { fields } => {
                let field_strings: Vec<String> = fields.names_types()
                    .iter()
                    .map(|parameter| {
                        format!("\t{}: {}", parameter.0, self.type_as_string(&parameter.1))
                    })
                    .collect();
                format!(
                    "struct {{\n{}\n}}",

                    field_strings.join(",\n")
                )
            }

            TypeKind::Unresolved => "{unresolved}".to_string(),
            TypeKind::UnresolvedNum => "{number}".to_string(),
            TypeKind::None => "void".to_string(),
            TypeKind::Error => "{error}".to_string(),
        }
    }
}
