use std::collections::HashMap;
use crate::globals::Symbol;
use crate::ir::comp::IRComp;

pub mod assembly;
pub mod comp;
pub mod interpreter;

#[derive(Debug, Clone)]
pub struct IRType {
    pub size: u64,
    pub align: u64,
}

#[derive(Debug, Clone)]
pub struct IRValue {
    pub id: Symbol,
}

pub struct IRItemFunctionDef {
    pub name: Symbol,
    pub return_type: IRType,
    pub params: Vec<(Option<Symbol>, IRType)>,
    pub comps: Vec<IRComp>,
    pub label_defs: HashMap<Symbol, u64>
}

pub enum IRItemKind {
    FunctionDef(IRItemFunctionDef),
}

pub struct IRItem {
    pub kind: IRItemKind,
}

pub struct IRModule {
    pub items: Vec<IRItem>,
}
