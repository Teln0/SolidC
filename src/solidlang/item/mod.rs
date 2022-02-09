use crate::globals::{SessionGlobals, Symbol};
use crate::solidlang::ty::{Ty, TyPrimitive};
use std::collections::HashMap;
use crate::solidlang::pool::PoolRef;
use crate::solidlang::defs::FunctionDef;

#[derive(Clone)]
pub struct SavedScopes {
    scopes: Vec<ItemScope>,
}

impl SavedScopes {
    pub fn empty() -> Self {
        Self { scopes: vec![] }
    }
}

#[derive(Clone)]
pub struct ItemScope {
    tys: HashMap<Vec<Symbol>, Ty>,
    functions: HashMap<Vec<Symbol>, Vec<PoolRef<FunctionDef>>>
}

impl ItemScope {
    fn new() -> Self {
        Self {
            tys: HashMap::new(),
            functions: HashMap::new()
        }
    }
}

pub struct ItemContext {
    scopes: Vec<ItemScope>,
}

impl ItemContext {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    pub fn start_scope(&mut self) {
        self.scopes.push(ItemScope::new());
    }

    pub fn close_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn save_scopes(&self) -> SavedScopes {
        SavedScopes {
            scopes: self.scopes.clone(),
        }
    }

    pub fn swap_scopes(&mut self, with: SavedScopes) -> SavedScopes {
        let mut scopes = with;
        std::mem::swap(&mut scopes.scopes, &mut self.scopes);
        scopes
    }

    pub fn register_default_tys(&mut self) {
        SessionGlobals::with_interner_mut(|i| {
            self.register_ty(&[i.intern("u8")], Ty::from_primitive(TyPrimitive::U8));
            self.register_ty(&[i.intern("i8")], Ty::from_primitive(TyPrimitive::I8));
            self.register_ty(&[i.intern("u16")], Ty::from_primitive(TyPrimitive::U16));
            self.register_ty(&[i.intern("i16")], Ty::from_primitive(TyPrimitive::I16));
            self.register_ty(&[i.intern("u32")], Ty::from_primitive(TyPrimitive::U32));
            self.register_ty(&[i.intern("i32")], Ty::from_primitive(TyPrimitive::I32));
            self.register_ty(&[i.intern("u64")], Ty::from_primitive(TyPrimitive::U64));
            self.register_ty(&[i.intern("i64")], Ty::from_primitive(TyPrimitive::I64));
            self.register_ty(&[i.intern("bool")], Ty::from_primitive(TyPrimitive::Bool));
            self.register_ty(&[i.intern("char")], Ty::from_primitive(TyPrimitive::Char));
            self.register_ty(&[i.intern("void")], Ty::from_primitive(TyPrimitive::Void));
        });
    }

    pub fn register_ty(&mut self, path: &[Symbol], ty: Ty) {
        if self.resolve_ty(path).is_some() {
            panic!("ERROR Type already defined");
        }

        self.scopes
            .last_mut()
            .unwrap()
            .tys
            .insert(path.to_vec(), ty);
    }

    pub fn resolve_ty(&self, path: &[Symbol]) -> Option<&Ty> {
        for i in (0..self.scopes.len()).rev() {
            if let Some(r) = self.scopes[i].tys.get(path) {
                return Some(r);
            }
        }

        None
    }

    pub fn register_function(&mut self, path: &[Symbol], fun: PoolRef<FunctionDef>) {

        let functions = &mut self.scopes
            .last_mut()
            .unwrap()
            .functions;
        if functions.contains_key(path) {
            functions.get_mut(path).unwrap().push(fun);
        }
        else {
            functions.insert(path.to_vec(), vec![fun]);
        }
    }

    pub fn resolve_function(&self, path: &[Symbol]) -> Vec<PoolRef<FunctionDef>> {
        let mut result = vec![];

        for i in (0..self.scopes.len()).rev() {
            if let Some(r) = self.scopes[i].functions.get(path) {
                result.extend(r.iter());
            }
        }

        result
    }
}
