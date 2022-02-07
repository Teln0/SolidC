use crate::globals::{SessionGlobals, Symbol};
use crate::solidlang::ast::{ASTExpression, ASTExpressionKind, ASTFunctionDef, ASTStatement, ASTStatementBlock, ASTStatementKind};
use crate::solidlang::context::function::Function;
use crate::solidlang::context::ty::{Ty, TyKind, TyPrimitive};
use crate::solidlang::lowerer::Lowerer;
use std::collections::HashMap;
use crate::ir::comp::{IRComp, IRCompKind};
use crate::ir::{IRItem, IRItemFunctionDef, IRItemKind, IRType, IRValue};

#[derive(Clone)]
enum ValueKind {
    Direct(u64),
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
            ty: Ty { kind: TyKind::Primitive(TyPrimitive::Void) }
        }
    }
}

struct CodegenScope {
    name_to_value_index: HashMap<Symbol, Value>,
}

struct CodegenContext {
    scopes: Vec<CodegenScope>,
    current_value: u64,
    computations: Vec<IRComp>,
    expected_return_type: Ty
}

impl CodegenContext {
    fn new(expected_return_type: Ty) -> Self {
        Self {
            scopes: vec![],
            current_value: 0,
            computations: vec![],
            expected_return_type
        }
    }

    fn get_current_computation_index(&mut self) -> u64 {
        self.computations.len() as u64 - 1
    }

    fn add_computation(&mut self, comp: IRComp) -> u64 {
        self.computations.push(comp);
        let value = self.current_value;
        self.current_value += 1;
        value
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

    fn direct_value_current(&mut self, ty: Ty) -> Value {
        let value = Value {
            kind: ValueKind::Direct(self.current_value),
            ty,
        };
        value
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
            ValueKind::Direct(index) => IRValue { index: *index },
            ValueKind::None => todo!()
        }
    }

    fn compile_expression(
        &mut self,
        expression: &ASTExpression,
        codegen_context: &mut CodegenContext,
        expected_type: Option<&Ty>
    ) -> CompilationResult {
        match &expression.kind {
            ASTExpressionKind::Ident(ident) => {
                if let Some(value) = codegen_context.resolve_name(ident) {
                    if let Some(expected_type) = expected_type {
                        if !value.ty.eq(expected_type) {
                            panic!("ERROR Unexpected type");
                        }
                    }
                    CompilationResult::Value(value)
                }
                else {
                    panic!("ERROR Could not find value for identifier");
                }
            }
            ASTExpressionKind::IntegerLiteral(_) => {
                todo!()
            }
            ASTExpressionKind::UnaryOperation(_, _) => {
                todo!()
            }
            ASTExpressionKind::BinaryOperation(_, _, _) => {
                todo!()
            }
            ASTExpressionKind::If(_, _, _) => {
                todo!()
            }
            ASTExpressionKind::While(_, _) => {
                todo!()
            }
            ASTExpressionKind::Loop(_) => {
                todo!()
            }
            ASTExpressionKind::For(_, _, _) => {
                todo!()
            }
            ASTExpressionKind::TemplateApplication(_, _) => {
                todo!()
            }
            ASTExpressionKind::Call(_, _) => {
                todo!()
            }
            ASTExpressionKind::Index(_, _) => {
                todo!()
            }
            ASTExpressionKind::MemberAccess(_, _) => {
                todo!()
            }
            ASTExpressionKind::StaticAccess(_, _) => {
                todo!()
            }
        }
    }

    fn compile_statement(
        &mut self,
        statement: &ASTStatement,
        codegen_context: &mut CodegenContext
    ) -> CompilationResult {
        match &statement.kind {
            ASTStatementKind::LocalBinding(_, _, _) => {
                todo!()
            }
            ASTStatementKind::Expression(expression) => {
                self.compile_expression(expression, codegen_context, None)
            }
            ASTStatementKind::Return(expression) => {
                let ert = codegen_context.expected_return_type.clone();
                let result = self.compile_expression(expression, codegen_context, Some(&ert));
                match result {
                    CompilationResult::Value(value) => {
                        codegen_context.add_computation(IRComp {
                            kind: IRCompKind::Return(self.get_ir_value(&value))
                        });
                        CompilationResult::Returning
                    },
                    r => r
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
            ASTStatementKind::Semicolon => {
                CompilationResult::Value(Value::new_none())
            }
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

        let mut result = CompilationResult::Value(Value::new_none());
        for statement in &statement_block.statements {
            if let ASTStatementKind::Item(_) = statement.kind { continue; }
            result = self.compile_statement(statement, codegen_context);
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
            Ty {
                kind: TyKind::Primitive(TyPrimitive::Void),
            }
        };

        let ir_name = SessionGlobals::with_interner_mut(|i| {
            let mut name = i.get(&function_def.name).unwrap().to_owned();
            name += "__";
            name += &self.new_unique().to_string();
            i.intern(&name)
        });

        let mut codegen_context = CodegenContext::new(resolved_return_type.clone());
        codegen_context.start_scope();

        for (param_sym, param_ty) in &resolved_params {
            let value = codegen_context.direct_value_current(param_ty.clone());
            codegen_context.current_value += 1;
            codegen_context.bind_name(*param_sym, value);
        }

        let result =
            self.compile_statement_block(&function_def.statement_block, &mut codegen_context, Some(&resolved_return_type));

        // No need to return anything if the return type is void
        if resolved_return_type.get_size_and_align().0 != 0 {
            match &result {
                CompilationResult::Value(value) => {
                    codegen_context.add_computation(IRComp {
                        kind: IRCompKind::Return(self.get_ir_value(value))
                    });
                }
                CompilationResult::Returning => {}
            }
        }

        codegen_context.close_scope();

        self.ir_module.items.push(IRItem {
            kind: IRItemKind::FunctionDef(IRItemFunctionDef {
                name: ir_name,
                params: resolved_params.iter().map(|(_, ty)| ty_to_ir_type(ty)).collect(),
                return_type: ty_to_ir_type(&resolved_return_type),
                comps: codegen_context.computations
            })
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
    IRType {
        size,
        align
    }
}