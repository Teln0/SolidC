use crate::globals::{SessionGlobals, Symbol};
use crate::ir::comp::{IRComp, IRCompKind};
use crate::ir::{IRItem, IRItemFunctionDef, IRItemKind, IRType, IRValue};
use crate::solidlang::ast::{
    ASTFunctionDef, ASTStatement, ASTStatementBlock,
    ASTStatementKind,
};
use crate::solidlang::context::function::Function;
use crate::solidlang::context::ty::{Ty, TyPrimitive};
use crate::solidlang::lowerer::Lowerer;
use std::collections::HashMap;

pub mod expression;

#[derive()]

#[derive(Clone)]
enum ValueKind {
    Direct(Symbol),
    None,
}

#[derive(Clone)]
struct Value {
    kind: ValueKind,
    ty: Ty,
}

impl Value {
    fn new_none() -> Self {
        Value {
            kind: ValueKind::None,
            ty: Ty::from_primitive(TyPrimitive::Void),
        }
    }

    fn new_direct(id: Symbol, ty: Ty) -> Self {
        Value {
            kind: ValueKind::Direct(id),
            ty,
        }
    }
}

#[derive(Clone)]
struct CodegenScope {
    name_to_value_index: HashMap<Symbol, Value>,
}

#[derive(Clone)]
struct CodegenContext {
    scopes: Vec<CodegenScope>,
    computations: Vec<IRComp>,
    current_value_index: usize,
    current_label_index: usize,
    expected_return_type: Ty,
    label_defs: HashMap<Symbol, u64>
}

impl CodegenContext {
    fn new(expected_return_type: Ty) -> Self {
        Self {
            scopes: vec![],
            computations: vec![],
            current_value_index: 0,
            current_label_index: 0,
            expected_return_type,
            label_defs: HashMap::new()
        }
    }

    fn create_new_id_for_value(&mut self) -> Symbol {
        let id = format!("_{}", self.current_value_index);
        let id = SessionGlobals::with_interner_mut(|i| i.intern(&id));
        self.current_value_index += 1;
        id
    }

    fn create_new_id_for_label(&mut self) -> Symbol {
        let id = format!("_label_{}", self.current_label_index);
        let id = SessionGlobals::with_interner_mut(|i| i.intern(&id));
        self.current_label_index += 1;
        id
    }

    fn place_label(&mut self, label: Symbol) {
        self.label_defs.insert(label, self.computations.len() as u64);
    }

    fn add_computation(&mut self, comp: IRComp) {
        self.computations.push(comp);
    }

    fn start_scope(&mut self) {
        self.scopes.push(CodegenScope {
            name_to_value_index: HashMap::new(),
        });
    }

    fn close_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind_name(&mut self, name: Symbol, value: Value) {
        self.scopes
            .last_mut()
            .unwrap()
            .name_to_value_index
            .insert(name, value);
    }

    fn resolve_name(&self, name: &Symbol) -> Option<Value> {
        let len = self.scopes.len();
        for i in (0..len).rev() {
            if let Some(value) = self.scopes[i].name_to_value_index.get(name) {
                return Some(value.clone());
            }
        }

        None
    }
}

enum CompilationResult {
    Value(Value),
    Returning,
}

impl CompilationResult {
    fn is_returning(&self) -> bool {
        match self {
            CompilationResult::Returning => true,
            _ => false,
        }
    }
}

impl Lowerer {
    fn get_ir_value(&mut self, value: &Value) -> IRValue {
        match &value.kind {
            ValueKind::Direct(index) => IRValue { id: *index },
            ValueKind::None => todo!(),
        }
    }

    fn compile_statement(
        &mut self,
        statement: &ASTStatement,
        codegen_context: &mut CodegenContext,
        expected_expression_type: Option<&Ty>
    ) -> CompilationResult {
        match &statement.kind {
            ASTStatementKind::LocalBinding(_, _, _) => {
                todo!()
            }
            ASTStatementKind::Expression(expression) => {
                self.compile_expression(expression, codegen_context, expected_expression_type)
            }
            ASTStatementKind::Return(expression) => {
                let ert = codegen_context.expected_return_type.clone();
                let result = self.compile_expression(expression, codegen_context, Some(&ert));
                match result {
                    CompilationResult::Value(value) => {
                        codegen_context.add_computation(IRComp {
                            kind: IRCompKind::Return(self.get_ir_value(&value)),
                            id: None,
                        });
                        CompilationResult::Returning
                    }
                    r => r,
                }
            }
            ASTStatementKind::Break => {
                todo!()
            }
            ASTStatementKind::Continue => {
                todo!()
            }
            ASTStatementKind::Item(_) => {
                unreachable!()
            }
            ASTStatementKind::Semicolon => CompilationResult::Value(Value::new_none()),
        }
    }

    fn compile_statement_block(
        &mut self,
        statement_block: &ASTStatementBlock,
        codegen_context: &mut CodegenContext,
        expected_type: Option<&Ty>,
    ) -> CompilationResult {
        self.start_scope();
        codegen_context.start_scope();

        let items: Vec<_> = statement_block
            .statements
            .iter()
            .filter_map(|statement| match &statement.kind {
                ASTStatementKind::Item(item) => Some(item),
                _ => None,
            })
            .collect();

        self.preprocess_items(&items);
        self.process_function_items(&items);

        let statements: Vec<_> = statement_block
            .statements
            .iter()
            .filter(|statement| match &statement.kind {
                ASTStatementKind::Item(_) => false,
                _ => true,
            })
            .collect();

        let mut result = CompilationResult::Value(Value::new_none());
        for i in 0..statements.len() {
            result = self.compile_statement(statements[i], codegen_context, if i == statements.len() - 1 {
                expected_type
            } else { None });
            if result.is_returning() {
                break;
            }
        }

        self.close_scope();
        codegen_context.close_scope();

        if let Some(expected_type) = expected_type {
            match &result {
                CompilationResult::Value(value) => {
                    if !value.ty.eq(expected_type) {
                        panic!("ERROR Unexpected type");
                    }
                }
                CompilationResult::Returning => {}
            }
        }

        result
    }

    pub(in crate::solidlang::lowerer) fn compile_function_item(
        &mut self,
        function_def: &ASTFunctionDef,
    ) -> Function {
        let resolved_params: Vec<_> = function_def
            .params
            .iter()
            .map(|param| (param.name, self.resolve_type(&param.ast_type)))
            .collect();

        let resolved_return_type = if let Some(return_type) = &function_def.return_type {
            self.resolve_type(return_type)
        } else {
            Ty::from_primitive(TyPrimitive::Void)
        };

        let ir_name = SessionGlobals::with_interner_mut(|i| {
            let mut name = i.get(&function_def.name).unwrap().to_owned();
            name += "__";
            name += &self.new_unique().to_string();
            i.intern(&name)
        });

        let mut codegen_context = CodegenContext::new(resolved_return_type.clone());
        codegen_context.start_scope();

        let mut param_ids = vec![];
        SessionGlobals::with_interner_mut(|i| {
            let mut current_param_id: usize = 0;
            for (param_sym, param_ty) in &resolved_params {
                let param_id = format!("_param_{}", current_param_id);
                let param_id = i.intern(&param_id);
                param_ids.push(param_id);
                let value = Value::new_direct(param_id, param_ty.clone());
                codegen_context.bind_name(*param_sym, value);
                current_param_id += 1;
            }
        });

        let result = self.compile_statement_block(
            &function_def.statement_block,
            &mut codegen_context,
            Some(&resolved_return_type),
        );

        // No need to return anything if the return type is void
        if resolved_return_type.get_size_and_align().0 != 0 {
            match &result {
                CompilationResult::Value(value) => {
                    codegen_context.add_computation(IRComp {
                        kind: IRCompKind::Return(self.get_ir_value(value)),
                        id: None,
                    });
                }
                CompilationResult::Returning => {}
            }
        }

        codegen_context.close_scope();

        self.ir_module.items.push(IRItem {
            kind: IRItemKind::FunctionDef(IRItemFunctionDef {
                name: ir_name,
                params: (0..resolved_params.len())
                    .map(|i| {
                        let param_id = Some(param_ids[i]);
                        let ir_type = ty_to_ir_type(&resolved_params[i].1);
                        (param_id, ir_type)
                    })
                    .collect(),
                return_type: ty_to_ir_type(&resolved_return_type),
                comps: codegen_context.computations,
                label_defs: codegen_context.label_defs
            }),
        });

        Function {
            path: vec![function_def.name],
            params: resolved_params.iter().map(|(_, ty)| ty.clone()).collect(),
            return_type: resolved_return_type,
            ir_name,
        }
    }
}

fn ty_to_ir_type(ty: &Ty) -> IRType {
    let (size, align) = ty.get_size_and_align();
    let size = size as u64;
    let align = align as u64;
    IRType { size, align }
}
