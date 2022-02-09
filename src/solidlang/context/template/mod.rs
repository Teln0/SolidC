use crate::globals::Symbol;
use crate::solidlang::ast::{ASTFunctionDef, ASTStructDef};
use crate::solidlang::context::item::SavedScopes;
use std::collections::HashMap;

#[derive(Clone)]
pub enum TemplateKind {
    Struct(ASTStructDef),
    Function(ASTFunctionDef),
}

#[derive(Clone)]
pub struct Template {
    pub kind: TemplateKind,
    pub saved_scopes: SavedScopes,
    pub params: Vec<Symbol>,
    pub path: Vec<Symbol>,
}
