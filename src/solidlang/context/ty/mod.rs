use crate::globals::{SessionGlobals, Symbol};
use crate::solidlang::context::function::Function;
use crate::solidlang::context::pool::{Pool, PoolRef};

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
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

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub struct TyStructField {
    pub offset: usize,
    pub name: Symbol,
    pub ty: PoolRef<Ty>,
}

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub struct TyStruct {
    pub fields: Vec<TyStructField>,
    pub align: usize,
    pub size: usize,
}

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub enum TyKind {
    Primitive(TyPrimitive),
    PointerTo(PoolRef<Ty>),
    Struct(TyStruct),
    PlaceholderUnknown,
}

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub struct Ty {
    pub kind: TyKind,
}

impl Ty {
    // TODO Optimize this whole thing
    pub fn struct_recompute_offsets_size_and_align_recursive(pool_ref: PoolRef<Ty>) {
        let to_recompute = SessionGlobals::with_ty_pool(|pool| {
            let self_ty = pool.get(pool_ref);
            match &self_ty.kind {
                TyKind::Struct(s) => Some(s.fields.clone()),
                _ => None,
            }
        });

        if let Some(to_recompute) = to_recompute {
            for to_recompute in &to_recompute {
                Self::struct_recompute_offsets_size_and_align_recursive(to_recompute.ty);
            }

            SessionGlobals::with_ty_pool_mut(|pool| {
                let mut fields = match &pool.get(pool_ref).kind {
                    TyKind::Struct(s) => s.fields.clone(),
                    _ => unreachable!(),
                };

                let mut current_offset = 0;
                let mut max_align = 1;
                for field in &mut fields {
                    let (size, align) = pool.get(field.ty).get_size_and_align();

                    if align > max_align {
                        max_align = align;
                    }

                    if current_offset % align != 0 {
                        current_offset += align - current_offset % align;
                    }

                    field.offset = current_offset;

                    current_offset += size;
                }

                if current_offset % max_align != 0 {
                    current_offset += max_align - current_offset % max_align;
                }

                match &mut pool.get_mut(pool_ref).kind {
                    TyKind::Struct(s) => {
                        s.size = current_offset;
                        s.fields = fields;
                        s.align = max_align;
                    }
                    _ => unreachable!(),
                }
            })
        }
    }

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
            TyKind::Struct(s) => (s.size, s.align),
            TyKind::PlaceholderUnknown => {
                panic!("ERROR Cannot compute size and alignment of unknown type")
            }
        }
    }

    pub fn from_primitive(primitive: TyPrimitive) -> Self {
        Self {
            kind: TyKind::Primitive(primitive),
        }
    }

    pub fn placeholder_unknown() -> Self {
        Self {
            kind: TyKind::PlaceholderUnknown,
        }
    }
}

pub struct DefaultTys {
    pub i8: PoolRef<Ty>,
    pub u8: PoolRef<Ty>,
    pub i16: PoolRef<Ty>,
    pub u16: PoolRef<Ty>,
    pub i32: PoolRef<Ty>,
    pub u32: PoolRef<Ty>,
    pub i64: PoolRef<Ty>,
    pub u64: PoolRef<Ty>,
    pub bool: PoolRef<Ty>,
    pub char: PoolRef<Ty>,
    pub void: PoolRef<Ty>,
}

impl DefaultTys {
    pub fn create() -> Self {
        SessionGlobals::with_ty_pool_mut(|pool| DefaultTys {
            i8: pool.add(Ty::from_primitive(TyPrimitive::I8)),
            u8: pool.add(Ty::from_primitive(TyPrimitive::U8)),
            i16: pool.add(Ty::from_primitive(TyPrimitive::I16)),
            u16: pool.add(Ty::from_primitive(TyPrimitive::U16)),
            i32: pool.add(Ty::from_primitive(TyPrimitive::I32)),
            u32: pool.add(Ty::from_primitive(TyPrimitive::U32)),
            i64: pool.add(Ty::from_primitive(TyPrimitive::I64)),
            u64: pool.add(Ty::from_primitive(TyPrimitive::U64)),
            bool: pool.add(Ty::from_primitive(TyPrimitive::Bool)),
            char: pool.add(Ty::from_primitive(TyPrimitive::Char)),
            void: pool.add(Ty::from_primitive(TyPrimitive::Void)),
        })
    }
}
