use std::collections::HashMap;
use crate::globals::Symbol;
use crate::solidlang::defs::FunctionDef;
use crate::solidlang::lowerer::Lowerer;
use crate::solidlang::pool::PoolRef;
use crate::solidlang::ty::Ty;

pub struct Codegen {
    // Maps function defs and generic args to the ir name
    compiled: HashMap<(PoolRef<FunctionDef>, Vec<Ty>), Symbol>
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            compiled: HashMap::new()
        }
    }
}

impl Lowerer {
    pub(in crate::solidlang::lowerer) fn get_ir_name(&mut self, function_def: PoolRef<FunctionDef>, args: Vec<Ty>) -> Symbol {
        todo!()
    }
}