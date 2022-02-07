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
    /// First indexes correspond to parameters, following indexes correspond to locals
    pub index: u64,
}

pub struct IRItemFunctionDef {
    pub name: Symbol,
    pub return_type: IRType,
    pub params: Vec<IRType>,
    pub comps: Vec<IRComp>,
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
