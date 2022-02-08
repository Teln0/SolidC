use crate::globals::{SessionGlobals, Symbol};
use std::collections::HashMap;

// TODO : Implement a "type interner"

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TyPrimitive {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Bool,
    Char,
    Void,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TyStructField {
    pub offset: usize,
    pub name: Symbol,
    pub ty: Ty,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TyStruct {
    pub fields: Vec<TyStructField>,
    pub align: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TyKind {
    Primitive(TyPrimitive),
    PointerTo(Box<Ty>),
    Struct(TyStruct)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ty {
    pub kind: TyKind,
}

impl Ty {
    pub fn get_size_and_align(&self) -> (usize, usize) {
        match &self.kind {
            TyKind::Primitive(primitive) => match primitive {
                TyPrimitive::U8 => (1, 1),
                TyPrimitive::I8 => (1, 1),
                TyPrimitive::U16 => (2, 2),
                TyPrimitive::I16 => (2, 2),
                TyPrimitive::U32 => (4, 4),
                TyPrimitive::I32 => (4, 4),
                TyPrimitive::U64 => (8, 8),
                TyPrimitive::I64 => (8, 8),
                TyPrimitive::Bool => (1, 1),
                TyPrimitive::Char => (4, 4),
                TyPrimitive::Void => (0, 1),
            },
            TyKind::PointerTo(_) => (8, 8),
            TyKind::Struct(s) => (s.size, s.align)
        }
    }

    pub fn from_primitive(primitive: TyPrimitive) -> Self {
        Ty {
            kind: TyKind::Primitive(primitive),
        }
    }
}

#[derive(Clone)]
pub struct TyScope {
    path_to_type: HashMap<Vec<Symbol>, Ty>,
}

pub struct TyContext {
    scopes: Vec<TyScope>,
}

impl TyContext {
    fn new_empty() -> Self {
        Self { scopes: vec![] }
    }

    pub fn new() -> Self {
        let mut result = Self::new_empty();
        result.start_scope();
        SessionGlobals::with_interner_mut(|i| {
            result.register_type(
                &[i.intern("u8")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::U8),
                },
            );
            result.register_type(
                &[i.intern("i8")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::I8),
                },
            );
            result.register_type(
                &[i.intern("u16")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::U16),
                },
            );
            result.register_type(
                &[i.intern("i16")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::I16),
                },
            );
            result.register_type(
                &[i.intern("u32")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::U32),
                },
            );
            result.register_type(
                &[i.intern("i32")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::I32),
                },
            );
            result.register_type(
                &[i.intern("u64")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::U64),
                },
            );
            result.register_type(
                &[i.intern("i64")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::I64),
                },
            );
            result.register_type(
                &[i.intern("bool")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::Bool),
                },
            );
            result.register_type(
                &[i.intern("char")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::Char),
                },
            );
            result.register_type(
                &[i.intern("void")],
                Ty {
                    kind: TyKind::Primitive(TyPrimitive::Void),
                },
            );
        });

        result
    }

    pub fn resolve_type(&self, path: &[Symbol]) -> Option<Ty> {
        let len = self.scopes.len();
        for i in (0..len).rev() {
            if let Some(ty) = self.scopes[i].path_to_type.get(path) {
                return Some(ty.clone());
            }
        }

        None
    }

    pub fn register_type(&mut self, path: &[Symbol], ty: Ty) {
        self.scopes
            .last_mut()
            .unwrap()
            .path_to_type
            .insert(path.to_vec(), ty);
    }

    pub fn create_struct_ty(&self, fields: &[(Symbol, Ty)]) -> Ty {
        // TODO : Support for non power of two alignments ? Would that even be useful ?

        let mut current_offset = 0;
        let mut max_align = 1;

        let mut struct_fields = vec![];
        for (symbol, field_ty) in fields {
            let (size, align) = field_ty.get_size_and_align();

            if align > max_align {
                max_align = align;
            }

            if current_offset % align != 0 {
                current_offset += align - current_offset % align;
            }

            struct_fields.push(TyStructField {
                offset: current_offset,
                name: *symbol,
                ty: field_ty.clone(),
            });

            current_offset += size;
        }

        // Padding at the end of the struct
        if current_offset % max_align != 0 {
            current_offset += max_align - current_offset % max_align;
        }

        Ty {
            kind: TyKind::Struct(TyStruct {
                size: current_offset,
                fields: struct_fields,
                align: max_align,
            }),
        }
    }

    pub fn start_scope(&mut self) {
        self.scopes.push(TyScope {
            path_to_type: HashMap::new(),
        });
    }

    pub fn close_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_current_scopes(&self) -> Vec<TyScope> {
        self.scopes.clone()
    }

    pub fn swap_scopes(&mut self, with: Vec<TyScope>) -> Vec<TyScope> {
        let current_scopes = self.get_current_scopes();
        self.scopes = with;
        current_scopes
    }
}
