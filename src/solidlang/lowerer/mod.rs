use crate::globals::Symbol;
use crate::ir::IRModule;
use crate::solidlang::ast::{
    ASTItem, ASTItemKind, ASTModule, ASTStructDef, ASTTemplate, ASTType, ASTTypeKind,
};
use crate::solidlang::context::function::{FunctionContext, FunctionScope};
use crate::solidlang::context::template::{TemplateContext, TemplateScope, TemplatedItemKind};
use crate::solidlang::context::ty::{Ty, TyContext, TyKind, TyScope};

pub mod codegen;

#[derive(Clone)]
pub struct SavedScopes {
    pub saved_template_scopes: Vec<TemplateScope>,
    pub saved_ty_scopes: Vec<TyScope>,
    pub saved_function_scopes: Vec<FunctionScope>,
}

pub struct Lowerer {
    ty_context: TyContext,
    template_context: TemplateContext,
    function_context: FunctionContext,

    ir_module: IRModule,
    uniqueness_marker: u64,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            ty_context: TyContext::new(),
            template_context: TemplateContext::new(),
            function_context: FunctionContext::new(),
            ir_module: IRModule { items: vec![] },
            uniqueness_marker: 0,
        }
    }

    fn new_unique(&mut self) -> u64 {
        let result = self.uniqueness_marker;
        self.uniqueness_marker += 1;
        result
    }

    fn create_saved_scopes(&self) -> SavedScopes {
        SavedScopes {
            saved_ty_scopes: self.ty_context.get_current_scopes(),
            saved_template_scopes: self.template_context.get_current_scopes(),
            saved_function_scopes: self.function_context.get_current_scopes(),
        }
    }

    fn swap_saved_scopes(&mut self, with: SavedScopes) -> SavedScopes {
        SavedScopes {
            saved_ty_scopes: self.ty_context.swap_scopes(with.saved_ty_scopes),
            saved_template_scopes: self
                .template_context
                .swap_scopes(with.saved_template_scopes),
            saved_function_scopes: self
                .function_context
                .swap_scopes(with.saved_function_scopes),
        }
    }

    fn ast_struct_to_type(&mut self, ast_struct: &ASTStructDef) -> Ty {
        let mut fields = vec![];
        for field in &ast_struct.fields {
            let ty = self.resolve_type(&field.ast_type);
            fields.push((field.name, ty));
        }
        self.ty_context.create_struct_ty(&fields)
    }

    fn resolve_type(&mut self, ast_type: &ASTType) -> Ty {
        match &ast_type.kind {
            ASTTypeKind::Path {
                symbols,
                generic_args,
            } => {
                if generic_args.is_empty() {
                    if let Some(resolved) = self.ty_context.resolve_type(&symbols) {
                        resolved.clone()
                    } else {
                        panic!("ERROR Could not resolve type")
                    }
                } else {
                    let resolved_generic_args: Vec<_> = generic_args
                        .iter()
                        .map(|arg| self.resolve_type(arg))
                        .collect();
                    if let Some(resolved) = self.template_context.resolve_item(&symbols) {
                        let resolved_params = resolved.params.clone();
                        if resolved_params.len() != resolved_generic_args.len() {
                            panic!("Template params / args length mismatch");
                        }

                        let current_scopes = self.swap_saved_scopes(resolved.saved_scopes.clone());

                        self.ty_context.start_scope();

                        for i in 0..resolved_params.len() {
                            self.ty_context.register_type(
                                &[resolved_params[i]],
                                resolved_generic_args[i].clone(),
                            );
                        }

                        let ty = if let TemplatedItemKind::Struct(struct_def) = resolved.kind {
                            self.ast_struct_to_type(&struct_def)
                        } else {
                            panic!("ERROR Templated type resolved to a function instead of a type");
                        };

                        self.ty_context.close_scope();

                        self.swap_saved_scopes(current_scopes);

                        ty
                    } else {
                        panic!("ERROR Could not resolve templated type")
                    }
                }
            }
            ASTTypeKind::PointerTo(ty) => Ty {
                kind: TyKind::PointerTo(Box::new(self.resolve_type(&ty))),
            },
        }
    }

    fn preprocess_templated_item(&mut self, item: &ASTTemplate, mut present_params: Vec<Symbol>) {
        present_params.extend(item.params.iter());
        for item in &item.items {
            match &item.kind {
                ASTItemKind::FunctionDef(function_def) => {
                    self.template_context.register_function_item(
                        function_def.clone(),
                        present_params.clone(),
                        self.create_saved_scopes(),
                    );
                }
                ASTItemKind::StructDef(struct_def) => {
                    self.template_context.register_struct_item(
                        struct_def.clone(),
                        present_params.clone(),
                        self.create_saved_scopes(),
                    );
                }
                ASTItemKind::Template(template) => {
                    self.preprocess_templated_item(template, present_params.clone());
                }
            }
        }
    }

    fn preprocess_items(&mut self, items: &[&ASTItem]) {
        // FIXME : Allow the use of types defined later

        for item in items {
            match &item.kind {
                ASTItemKind::Template(template) => {
                    self.preprocess_templated_item(template, vec![]);
                }
                ASTItemKind::StructDef(struct_def) => {
                    let ty = self.ast_struct_to_type(struct_def);
                    self.ty_context.register_type(&[struct_def.name], ty);
                }
                _ => {}
            }
        }
    }

    fn process_function_items(&mut self, items: &[&ASTItem]) {
        for item in items {
            match &item.kind {
                ASTItemKind::FunctionDef(function_def) => {
                    let function = self.compile_function_item(function_def);
                    self.function_context.register_function(function);
                }
                _ => {}
            }
        }
    }

    fn start_scope(&mut self) {
        self.ty_context.start_scope();
        self.template_context.start_scope();
        self.function_context.start_scope();
    }

    fn close_scope(&mut self) {
        self.ty_context.close_scope();
        self.template_context.close_scope();
        self.function_context.close_scope();
    }

    pub fn lower(mut self, ast_module: &ASTModule) -> IRModule {
        self.start_scope();

        let items = ast_module.items.iter().collect::<Vec<&ASTItem>>();
        self.preprocess_items(&items);
        self.process_function_items(&items);

        self.close_scope();

        self.ir_module
    }
}
