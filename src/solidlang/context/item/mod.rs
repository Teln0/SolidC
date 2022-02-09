use crate::globals::{SessionGlobals, Symbol};
use crate::solidlang::context::pool::PoolRef;
use crate::solidlang::context::template::Template;
use crate::solidlang::context::ty::{DefaultTys, Ty};
use std::collections::HashMap;

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
    tys: HashMap<Vec<Symbol>, PoolRef<Ty>>,
    templates: Vec<PoolRef<Template>>,
    functions: Vec<PoolRef<Ty>>,

    path_to_template_ty: HashMap<Vec<Symbol>, PoolRef<Template>>,
}

impl ItemScope {
    fn new() -> Self {
        Self {
            tys: HashMap::new(),
            templates: vec![],
            functions: vec![],
            path_to_template_ty: HashMap::new()
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

    pub fn create_and_register_default_types(&mut self) {
        let default_tys = DefaultTys::create();
        SessionGlobals::with_interner_mut(|i| {
            self.register_ty(&[i.intern("i8")], default_tys.i8);
            self.register_ty(&[i.intern("u8")], default_tys.u8);
            self.register_ty(&[i.intern("i16")], default_tys.i16);
            self.register_ty(&[i.intern("u16")], default_tys.u16);
            self.register_ty(&[i.intern("i32")], default_tys.i32);
            self.register_ty(&[i.intern("u32")], default_tys.u32);
            self.register_ty(&[i.intern("i64")], default_tys.i64);
            self.register_ty(&[i.intern("u64")], default_tys.u64);
            self.register_ty(&[i.intern("bool")], default_tys.bool);
            self.register_ty(&[i.intern("char")], default_tys.char);
            self.register_ty(&[i.intern("void")], default_tys.void);
        });
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

    pub fn register_ty(&mut self, path: &[Symbol], ty: PoolRef<Ty>) {
        self.scopes
            .last_mut()
            .unwrap()
            .tys
            .insert(path.to_vec(), ty);
    }

    pub fn register_template(&mut self, template: PoolRef<Template>) {
        self.scopes.last_mut().unwrap().templates.push(template);
    }

    pub fn register_path_for_template_ty(&mut self, path: Vec<Symbol>, template: PoolRef<Template>) {
        self.scopes.last_mut().unwrap().path_to_template_ty.insert(path, template);
    }

    pub fn resolve_ty(&self, path: &[Symbol]) -> Option<PoolRef<Ty>> {
        for i in (0..self.scopes.len()).rev() {
            if let Some(r) = self.scopes[i].tys.get(path) {
                return Some(*r);
            }
        }

        None
    }

    pub fn resolve_template_ty(&self, path: &[Symbol]) -> Option<PoolRef<Template>> {
        for i in (0..self.scopes.len()).rev() {
            if let Some(r) = self.scopes[i].path_to_template_ty.get(path) {
                return Some(*r);
            }
        }

        None
    }

    pub fn tys_iter(&self) -> impl Iterator<Item = PoolRef<Ty>> + '_ {
        let mut iter: Box<dyn Iterator<Item = PoolRef<Ty>>> = Box::new(std::iter::empty());

        for scope in &self.scopes {
            iter = Box::new(iter.chain(scope.tys.values().cloned()))
        }

        iter
    }
}
