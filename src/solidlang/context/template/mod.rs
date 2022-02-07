use crate::globals::Symbol;
use crate::solidlang::ast::{ASTFunctionDef, ASTStructDef};
use crate::solidlang::lowerer::SavedScopes;
use std::collections::HashMap;

#[derive(Clone)]
pub enum TemplatedItemKind {
    Struct(ASTStructDef),
    Function(ASTFunctionDef),
}

#[derive(Clone)]
pub struct TemplatedItem {
    pub kind: TemplatedItemKind,
    pub saved_scopes: SavedScopes,
    pub params: Vec<Symbol>,
}

#[derive(Clone)]
pub struct TemplateScope {
    path_to_item: HashMap<Vec<Symbol>, TemplatedItem>,
}

pub struct TemplateContext {
    scopes: Vec<TemplateScope>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    pub fn resolve_item(&self, path: &[Symbol]) -> Option<TemplatedItem> {
        let len = self.scopes.len();
        for i in (0..len).rev() {
            if let Some(item) = self.scopes[i].path_to_item.get(path) {
                return Some(item.clone());
            }
        }

        None
    }

    pub fn register_struct_item(
        &mut self,
        item: ASTStructDef,
        params: Vec<Symbol>,
        saved_scopes: SavedScopes,
    ) {
        self.scopes.last_mut().unwrap().path_to_item.insert(
            vec![item.name],
            TemplatedItem {
                kind: TemplatedItemKind::Struct(item),
                params,
                saved_scopes,
            },
        );
    }

    pub fn register_function_item(
        &mut self,
        item: ASTFunctionDef,
        params: Vec<Symbol>,
        saved_scopes: SavedScopes,
    ) {
        self.scopes.last_mut().unwrap().path_to_item.insert(
            vec![item.name],
            TemplatedItem {
                kind: TemplatedItemKind::Function(item),
                params,
                saved_scopes,
            },
        );
    }

    pub fn start_scope(&mut self) {
        self.scopes.push(TemplateScope {
            path_to_item: HashMap::new(),
        });
    }

    pub fn close_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_current_scopes(&self) -> Vec<TemplateScope> {
        self.scopes.clone()
    }

    pub fn swap_scopes(&mut self, with: Vec<TemplateScope>) -> Vec<TemplateScope> {
        let current_scopes = self.get_current_scopes();
        self.scopes = with;
        current_scopes
    }
}
