use crate::globals::Symbol;
use crate::solidlang::ast::ASTStatementBlock;
use crate::solidlang::ty::Ty;

#[derive(Debug)]
pub struct StructDefField {
    pub name: Symbol,
    pub ty: Ty
}

#[derive(Debug)]
pub struct StructDef {
    pub fields: Vec<StructDefField>,
    pub generic_params: usize
}

pub struct FunctionDef {
    pub params: Vec<(Symbol, Ty)>,
    pub generic_params: Vec<Symbol>,
    pub return_type: Ty,

    pub code: ASTStatementBlock
}