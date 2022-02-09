use crate::globals::{SessionGlobals, Symbol};
use crate::solidlang::ast::{ASTItem, ASTItemKind, ASTModule, ASTStructDef, ASTType, ASTTypeKind};
use crate::solidlang::context::item::{ItemContext, SavedScopes};
use crate::solidlang::context::pool::PoolRef;
use crate::solidlang::context::template::{Template, TemplateKind};
use crate::solidlang::context::ty::{Ty, TyKind, TyStruct, TyStructField};

pub struct Lowerer {
    context: ItemContext,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            context: ItemContext::new(),
        }
    }

    fn process_templated_item(&mut self, mut existing_params: Vec<Symbol>, item: &ASTItem, created: &mut Vec<PoolRef<Template>>) {
        match &item.kind {
            ASTItemKind::FunctionDef(function_def) => {
                let c = SessionGlobals::with_template_pool_mut(|pool| {
                    pool.add(Template {
                        kind: TemplateKind::Function(function_def.clone()),
                        params: existing_params,
                        path: vec![function_def.name],
                        saved_scopes: SavedScopes::empty(),
                    })
                });
                created.push(c);
                self.context.register_template(c);
            }
            ASTItemKind::StructDef(struct_def) => {
                let c = SessionGlobals::with_template_pool_mut(|pool| {
                    pool.add(Template {
                        kind: TemplateKind::Struct(struct_def.clone()),
                        params: existing_params,
                        path: vec![struct_def.name],
                        saved_scopes: SavedScopes::empty(),
                    })
                });
                created.push(c);
                self.context.register_template(c);
                self.context.register_path_for_template_ty(vec![struct_def.name], c);
            }
            ASTItemKind::Template(template) => {
                existing_params.extend(template.params.iter());
                for item in &template.items {
                    self.process_templated_item(existing_params.clone(), item, created);
                }
            }
        }
    }

    fn process_template_items(&mut self, items: &[&ASTItem], created: &mut Vec<PoolRef<Template>>) {
        for item in items {
            match &item.kind {
                ASTItemKind::Template(template) => {
                    for item in &template.items {
                        self.process_templated_item(template.params.clone(), item, created);
                    }
                }
                _ => {}
            }
        }
    }

    fn struct_def_to_ty(
        &mut self,
        struct_def: &ASTStructDef,
        unresolved: &mut Vec<(Vec<Symbol>, PoolRef<Ty>)>,
        unresolved_with_generics: &mut Vec<(Vec<Symbol>, Vec<PoolRef<Ty>>, PoolRef<Ty>)>,
    ) -> PoolRef<Ty> {
        let mut ty_struct = TyStruct {
            fields: vec![],
            size: 0,
            align: 0,
        };

        for field in &struct_def.fields {
            let ty = self.ast_type_to_ty(&field.ast_type, unresolved, unresolved_with_generics);
            ty_struct.fields.push(TyStructField {
                offset: 0,
                name: field.name,
                ty,
            });
        }

         SessionGlobals::with_ty_pool_mut(|pool| {
            pool.add(Ty {
                kind: TyKind::Struct(ty_struct),
            })
        })
    }

    fn ast_type_to_ty(
        &mut self,
        ast_ty: &ASTType,
        unresolved: &mut Vec<(Vec<Symbol>, PoolRef<Ty>)>,
        unresolved_with_generics: &mut Vec<(Vec<Symbol>, Vec<PoolRef<Ty>>, PoolRef<Ty>)>
    ) -> PoolRef<Ty> {
        match &ast_ty.kind {
            ASTTypeKind::Path {
                symbols,
                generic_args,
            } => {
                if !generic_args.is_empty() {
                    let args = generic_args.iter().map(|a| self.ast_type_to_ty(a, unresolved, unresolved_with_generics)).collect();
                    let u = SessionGlobals::with_ty_pool_mut(|pool| {
                        pool.add(Ty::placeholder_unknown())
                    });
                    unresolved_with_generics.push((symbols.clone(), args, u));
                    return u;
                }

                if let Some(ty) = self.context.resolve_ty(symbols) {
                    ty
                } else {
                    let u = SessionGlobals::with_ty_pool_mut(|pool| {
                        pool.add(Ty::placeholder_unknown())
                    });
                    unresolved.push((symbols.clone(), u));
                    u
                }
            }
            ASTTypeKind::PointerTo(ty) => {
                let ty = self.ast_type_to_ty(ty, unresolved, unresolved_with_generics);
                let ty = Ty {
                    kind: TyKind::PointerTo(ty),
                };
                SessionGlobals::with_ty_pool_mut(|pool| pool.add(ty))
            }
        }
    }

    fn template_to_ty(&mut self, template: Template, args: &[PoolRef<Ty>]) -> PoolRef<Ty> {
        let saved_scopes = template.saved_scopes;
        // Swap into the scopes of the template
        let saved_scopes = self.context.swap_scopes(saved_scopes);

        if args.len() != template.params.len() {
            panic!("ERROR Template params len / args len mismatch");
        }

        self.context.start_scope();
        for i in 0..args.len() {
            self.context.register_ty(&[template.params[i]], args[i]);
        }

        let mut unresolved = vec![];
        let mut unresolved_with_generics = vec![];
        let ty = match &template.kind {
            TemplateKind::Struct(s) => {
                self.struct_def_to_ty(s, &mut unresolved, &mut unresolved_with_generics)
            }
            TemplateKind::Function(_) => panic!("Cannot create ty from function template")
        };

        if !unresolved.is_empty() {
            panic!("ERROR Could not resolve some ty while converting template to ty");
        }
        self.resolve_unresolved_generics(&unresolved_with_generics);

        self.context.close_scope();

        // Swap back to current scopes
        self.context.swap_scopes(saved_scopes);

        ty
    }

    fn resolve_unresolved_generics(&mut self, unresolved_generics: &[(Vec<Symbol>, Vec<PoolRef<Ty>>, PoolRef<Ty>)]) {
        for (path, args, resolving_for) in unresolved_generics {
            if let Some(resolved) = self.context.resolve_template_ty(path) {
                let template = SessionGlobals::with_template_pool(|pool| {
                    let resolved = pool.get(resolved);
                    resolved.clone()
                });
                let resolved = self.template_to_ty(template, args);
                SessionGlobals::with_ty_pool_mut(|pool| {
                    let resolved = pool.get(resolved).clone();
                    *pool.get_mut(*resolving_for) = resolved;
                })
            }
            else {
                panic!("ERROR Could not resolve template ty");
            }
        }
    }

    fn process_type_items(&mut self, items: &[&ASTItem], created_templates: &[PoolRef<Template>]) {
        // TODO : Get existing items (from imports and such)

        let mut resolve_later = vec![];
        let mut resolve_later_generics = vec![];
        for item in items {
            match &item.kind {
                ASTItemKind::StructDef(struct_def) => {
                    let ty = self.struct_def_to_ty(struct_def, &mut resolve_later, &mut resolve_later_generics);
                    self.context.register_ty(&[struct_def.name], ty);
                }
                _ => {}
            }
        }

        for resolve_later in &resolve_later {
            let ty = self.context.resolve_ty(&resolve_later.0);
            if let Some(ty) = ty {
                SessionGlobals::with_ty_pool_mut(|pool| {
                    let ty = pool.get(ty).clone();
                    *pool.get_mut(resolve_later.1) = ty;
                })
            } else {
                panic!("ERROR Could not resolve type {:?}", resolve_later.0);
            }
        }

        // Giving template items saved scopes ...
        for template in created_templates {
            SessionGlobals::with_template_pool_mut(|pool| {
                pool.get_mut(*template).saved_scopes = self.context.save_scopes();
            });
        }

        // ... Then resolving unresolved types with generics
        self.resolve_unresolved_generics(&resolve_later_generics);

        for item in self.context.tys_iter() {
            Ty::struct_recompute_offsets_size_and_align_recursive(item);
        }
    }

    pub fn process_items(&mut self, items: &[&ASTItem]) {
        let mut created_templates = vec![];
        self.process_template_items(items, &mut created_templates);
        self.process_type_items(items, &created_templates);
    }

    pub fn process_module(mut self, module: ASTModule) {
        self.context.start_scope();

        self.context.create_and_register_default_types();

        let items: Vec<_> = module.items.iter().collect();
        self.process_items(&items);

        self.context.close_scope();
    }
}
