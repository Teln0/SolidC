use crate::globals::Symbol;
use crate::solidlang::context::pool::PoolRef;
use crate::solidlang::context::ty::Ty;

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub struct Function {
    pub path: Vec<Symbol>,
    pub params: Vec<PoolRef<Ty>>,
    pub return_type: PoolRef<Ty>,
    pub ir_name: Symbol,
}
