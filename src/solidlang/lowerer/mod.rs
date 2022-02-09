use crate::globals::{SessionGlobals, Symbol};
use crate::solidlang::ast::{ASTItem, ASTItemKind, ASTModule, ASTType, ASTTypeKind};
use crate::solidlang::item::{ItemContext};
use crate::solidlang::pool::PoolRef;
use crate::solidlang::defs::{FunctionDef, StructDef, StructDefField};
use crate::solidlang::lowerer::codegen::Codegen;
use crate::solidlang::ty::{Ty, TyKind, TyPrimitive};

pub mod codegen;

pub struct Lowerer {
    context: ItemContext,
    codegen: Codegen
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            context: ItemContext::new(),
            codegen: Codegen::new()
        }
    }

    fn resolve_ast_type(&self, ast_type: &ASTType) -> Ty {
        match &ast_type.kind {
            ASTTypeKind::Path { symbols, generic_args } => {
                if let Some(resolved) = self.context.resolve_ty(symbols) {
                    let expected_args = match resolved.kind {
                        TyKind::Struct(struct_def) => SessionGlobals::with_struct_def_pool(|pool| pool.get(struct_def).generic_params),
                        _ => 0
                    };

                    if expected_args != generic_args.len() {
                        panic!("ERROR Generic args / params len mismatch")
                    }

                    if expected_args > 0 {
                        let struct_def = match resolved.kind {
                            TyKind::Struct(struct_def) => struct_def,
                            _ => unreachable!()
                        };

                        Ty {
                            kind: TyKind::StructWithArgs(struct_def, generic_args.iter().map(|t| self.resolve_ast_type(t)).collect())
                        }
                    }
                    else {
                        resolved.clone()
                    }
                }
                else {
                    panic!("ERROR Could not resolve {:?}", symbols);
                }
            }
            ASTTypeKind::PointerTo(ast_type) => {
                Ty {
                    kind: TyKind::PointerTo(Box::new(self.resolve_ast_type(ast_type)))
                }
            }
        }
    }

    fn register_type_items(&mut self, items: &[&ASTItem], generic_params_height: usize) {
        for item in items {
            match &item.kind {
                ASTItemKind::Template(ast_template) => {
                    let items: Box<[_]> = ast_template.items.iter().collect();
                    self.register_type_items(&items, generic_params_height + ast_template.params.len());
                }
                ASTItemKind::StructDef(ast_struct_def) => {
                    let struct_def = StructDef { fields: vec![], generic_params: generic_params_height };
                    let struct_def = SessionGlobals::with_struct_def_pool_mut(|pool| pool.add(struct_def));
                    self.context.register_ty(&[ast_struct_def.name], Ty::from_struct_def(struct_def));
                }
                _ => {}
            }
        }
    }

    fn process_type_items(&mut self, items: &[&ASTItem], mut generic_params_height: usize) {
        for item in items {
            match &item.kind {
                ASTItemKind::Template(ast_template) => {
                    let items: Box<[_]> = ast_template.items.iter().collect();
                    self.context.start_scope();

                    for param in &ast_template.params {
                        self.context.register_ty(&[*param], Ty { kind: TyKind::Param(generic_params_height) });
                        generic_params_height += 1;
                    }

                    self.process_type_items(&items, generic_params_height);

                    self.context.close_scope();
                }
                ASTItemKind::StructDef(ast_struct_def) => {
                    let fields: Vec<_> = ast_struct_def.fields.iter().map(|field| {
                        StructDefField {
                            name: field.name,
                            ty: self.resolve_ast_type(&field.ast_type)
                        }
                    }).collect();
                    let ty = self.context.resolve_ty(&[ast_struct_def.name]).unwrap();
                    match ty.kind {
                        TyKind::Struct(struct_def) => {
                            SessionGlobals::with_struct_def_pool_mut(|pool| {
                                let struct_def = pool.get_mut(struct_def);
                                struct_def.fields = fields;
                            });
                        }
                        _ => unreachable!()
                    }
                }
                _ => {}
            }
        }
    }

    fn register_function_items(&mut self, items: &[&ASTItem], generic_params: Vec<Symbol>, functions_with_no_generics: &mut Vec<(Vec<Symbol>, PoolRef<FunctionDef>)>) {
        for item in items {
            match &item.kind {
                ASTItemKind::Template(ast_template) => {
                    let items: Box<[_]> = ast_template.items.iter().collect();
                    let mut generic_params = generic_params.clone();
                    generic_params.extend(ast_template.params.iter());
                    self.register_function_items(&items, generic_params, functions_with_no_generics);
                }
                ASTItemKind::FunctionDef(ast_function_def) => {
                    let function_def = FunctionDef {
                        params: ast_function_def.params.iter().map(|param| {
                            (param.name, self.resolve_ast_type(&param.ast_type))
                        }).collect(),
                        generic_params: generic_params.clone(),
                        return_type: if let Some(return_type) = &ast_function_def.return_type {
                            self.resolve_ast_type(return_type)
                        }
                        else {
                            Ty::from_primitive(TyPrimitive::Void)
                        },
                        code: ast_function_def.statement_block.clone()
                    };
                    let function_def = SessionGlobals::with_function_def_pool_mut(|pool| pool.add(function_def));
                    if generic_params.is_empty() {
                        functions_with_no_generics.push((vec![ast_function_def.name], function_def));
                    }
                    self.context.register_function(&[ast_function_def.name], function_def);
                }
                _ => {}
            }
        }
    }

    pub fn process_function_items(&mut self, functions_with_no_generics: Vec<(Vec<Symbol>, PoolRef<FunctionDef>)>) {
        for function in &functions_with_no_generics {
            todo!()
        }
    }

    pub fn process_module(mut self, module: ASTModule) {
        self.context.start_scope();

        self.context.register_default_tys();

        let items: Box<[_]> = module.items.iter().collect();
        self.register_type_items(&items, 0);
        self.process_type_items(&items, 0);
        let mut functions_with_no_generics = vec![];
        self.register_function_items(&items, vec![], &mut functions_with_no_generics);
        self.process_function_items(functions_with_no_generics);

        self.context.close_scope();
    }
}
