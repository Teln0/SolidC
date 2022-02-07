use crate::globals::Symbol;
use crate::solidlang::context::ty::Ty;

#[derive(Clone)]
pub struct Function {
    pub path: Vec<Symbol>,
    pub params: Vec<Ty>,
    pub return_type: Ty,
    pub ir_name: Symbol,
}

#[derive(Clone)]
pub struct FunctionScope {
    functions: Vec<Function>,
}

pub struct FunctionContext {
    scopes: Vec<FunctionScope>,
}

impl FunctionContext {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    pub fn register_function(&mut self, function: Function) {
        self.scopes.last_mut().unwrap().functions.push(function);
    }

    pub fn start_scope(&mut self) {
        self.scopes.push(FunctionScope { functions: vec![] });
    }

    pub fn close_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_current_scopes(&self) -> Vec<FunctionScope> {
        self.scopes.clone()
    }

    pub fn swap_scopes(&mut self, with: Vec<FunctionScope>) -> Vec<FunctionScope> {
        let current_scopes = self.get_current_scopes();
        self.scopes = with;
        current_scopes
    }
}
