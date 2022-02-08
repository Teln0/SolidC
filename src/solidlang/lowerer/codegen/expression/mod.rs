use crate::ir::comp::{IRComp, IRCompBinaryOperation, IRCompBinaryOperationKind, IRCompConstant, IRCompFunctionCall, IRCompKind, IRCompUnaryOperation, IRCompUnaryOperationKind};
use crate::ir::IRValue;
use crate::solidlang::ast::{ASTExpression, ASTExpressionKind, ASTOperator};
use crate::solidlang::context::function::Function;
use crate::solidlang::context::ty::{Ty, TyKind, TyPrimitive};
use crate::solidlang::lowerer::codegen::{CodegenContext, CompilationResult, Value};
use crate::solidlang::lowerer::Lowerer;

impl Lowerer {
    pub(in crate::solidlang::lowerer::codegen) fn compile_expression(
        &mut self,
        expression: &ASTExpression,
        codegen_context: &mut CodegenContext,
        expected_type: Option<&Ty>,
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
                } else {
                    panic!("ERROR Could not find value for identifier");
                }
            }
            ASTExpressionKind::IntegerLiteral(integer) => {
                let i32_ty = Ty::from_primitive(TyPrimitive::I32);
                let expected_type = if let Some(expected_type) = expected_type {
                    expected_type
                } else {
                    &i32_ty
                };

                let (bytes, ty) = if let TyKind::Primitive(primitive) = &expected_type.kind {
                    match primitive {
                        TyPrimitive::U8 => (
                            u8::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::U8),
                        ),
                        TyPrimitive::I8 => (
                            i8::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::I8),
                        ),
                        TyPrimitive::U16 => (
                            u16::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::U16),
                        ),
                        TyPrimitive::I16 => (
                            i16::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::I16),
                        ),
                        TyPrimitive::U32 => (
                            u32::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::U32),
                        ),
                        TyPrimitive::I32 => (
                            i32::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::I32),
                        ),
                        TyPrimitive::U64 => (
                            u64::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::U64),
                        ),
                        TyPrimitive::I64 => (
                            i64::try_from(*integer)
                                .expect("ERROR Integer literal too big for type")
                                .to_ne_bytes()
                                .to_vec(),
                            Ty::from_primitive(TyPrimitive::I64),
                        ),
                        TyPrimitive::Bool => {
                            panic!("ERROR Cannot convert integer literal to expected type")
                        }
                        TyPrimitive::Char => {
                            panic!("ERROR Cannot convert integer literal to expected type")
                        }
                        TyPrimitive::Void => {
                            panic!("ERROR Cannot convert integer literal to expected type")
                        }
                    }
                } else {
                    panic!("ERROR Cannot convert integer literal to expected type");
                };

                let value_id = codegen_context.create_new_id_for_value();
                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::Constant(IRCompConstant { bytes: bytes }),
                    id: Some(value_id),
                });

                CompilationResult::Value(Value::new_direct(value_id, ty))
            }
            ASTExpressionKind::Boolean(bool) => {
                let bool_type = Ty::from_primitive(TyPrimitive::Bool);
                if let Some(expected_type) = expected_type {
                    if !expected_type.eq(&bool_type) {
                        panic!("ERROR Unexpected type");
                    }
                }

                let id = codegen_context.create_new_id_for_value();
                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::Constant(IRCompConstant {
                        bytes: vec![if *bool { 1 } else { 0 }]
                    }),
                    id: Some(id)
                });

                CompilationResult::Value(Value::new_direct(id, bool_type))
            }
            ASTExpressionKind::UnaryOperation(operator, operand) => {
                let compilation_result = self.compile_expression(operand, codegen_context, expected_type);
                match &compilation_result {
                    CompilationResult::Value(value) => {
                        match &value.ty.kind {
                            TyKind::Primitive(primitive) => {
                                match operator {
                                    ASTOperator::BitNot => {
                                        match primitive {
                                            TyPrimitive::U8
                                            | TyPrimitive::I8
                                            | TyPrimitive::U16
                                            | TyPrimitive::I16
                                            | TyPrimitive::U32
                                            | TyPrimitive::I32
                                            | TyPrimitive::U64
                                            | TyPrimitive::I64 => {
                                                let id = codegen_context.create_new_id_for_value();
                                                codegen_context.add_computation(IRComp {
                                                    kind: IRCompKind::UnaryOperation(IRCompUnaryOperation {
                                                        kind: IRCompUnaryOperationKind::BitNot,
                                                        operand: self.get_ir_value(value)
                                                    }),
                                                    id: Some(id)
                                                });
                                                CompilationResult::Value(Value::new_direct(id, value.ty.clone()))
                                            }
                                            _ => panic!("Cannot apply operator to this type"),
                                        }
                                    },
                                    ASTOperator::Minus => {
                                        match primitive {
                                            TyPrimitive::U8
                                            | TyPrimitive::I8
                                            | TyPrimitive::U16
                                            | TyPrimitive::I16
                                            | TyPrimitive::U32
                                            | TyPrimitive::I32
                                            | TyPrimitive::U64
                                            | TyPrimitive::I64 => {
                                                let id = codegen_context.create_new_id_for_value();
                                                codegen_context.add_computation(IRComp {
                                                    kind: IRCompKind::UnaryOperation(IRCompUnaryOperation {
                                                        kind: IRCompUnaryOperationKind::SignedNegation,
                                                        operand: self.get_ir_value(value)
                                                    }),
                                                    id: Some(id)
                                                });
                                                CompilationResult::Value(Value::new_direct(id, value.ty.clone()))
                                            }
                                            _ => panic!("Cannot apply operator to this type"),
                                        }
                                    },
                                    ASTOperator::BoolNot => {
                                        match primitive {
                                            TyPrimitive::Bool => {
                                                let id = codegen_context.create_new_id_for_value();
                                                codegen_context.add_computation(IRComp {
                                                    kind: IRCompKind::UnaryOperation(IRCompUnaryOperation {
                                                        kind: IRCompUnaryOperationKind::BoolNot,
                                                        operand: self.get_ir_value(value)
                                                    }),
                                                    id: Some(id)
                                                });
                                                CompilationResult::Value(Value::new_direct(id, value.ty.clone()))
                                            }
                                            _ => panic!("Cannot apply operator to this type"),
                                        }
                                    }
                                    _ => unreachable!()
                                }
                            }
                            _ => panic!("Cannot apply operator to this type"),
                        }
                    }
                    CompilationResult::Returning => return CompilationResult::Returning
                }
            }
            ASTExpressionKind::BinaryOperation(operator, left_operand, right_operand) => {
                match operator {
                    ASTOperator::Assign => {
                        todo!()
                    }

                    ASTOperator::Plus
                    | ASTOperator::Minus
                    | ASTOperator::Mul
                    | ASTOperator::Div
                    | ASTOperator::Mod
                    | ASTOperator::BitAnd
                    | ASTOperator::BitOr
                    | ASTOperator::BoolAnd
                    | ASTOperator::BoolOr
                    | ASTOperator::BitRShift
                    | ASTOperator::BitLShift => {
                        let left = self.compile_expression(left_operand, codegen_context, expected_type);
                        let left = match &left {
                            CompilationResult::Value(value) => value,
                            CompilationResult::Returning => return CompilationResult::Returning
                        };

                        match operator {
                            ASTOperator::Plus
                            | ASTOperator::Minus
                            | ASTOperator::Mul
                            | ASTOperator::Div
                            | ASTOperator::Mod
                            | ASTOperator::BitAnd
                            | ASTOperator::BitOr
                            | ASTOperator::BitRShift
                            | ASTOperator::BitLShift => match &left.ty.kind {
                                TyKind::Primitive(primitive) => match primitive {
                                    TyPrimitive::Bool
                                    | TyPrimitive::Char
                                    | TyPrimitive::Void => panic!("Cannot apply operator to this type"),
                                    _ => {}
                                }
                                _ => panic!("Cannot apply operator to this type")
                            }
                            ASTOperator::BoolAnd
                            | ASTOperator::BoolOr => match &left.ty.kind {
                                TyKind::Primitive(primitive) => match primitive {
                                    TyPrimitive::Bool => {}
                                    _ => panic!("Cannot apply operator to this type")
                                }
                                _ => panic!("Cannot apply operator to this type")
                            }
                            _ => unreachable!()
                        }

                        let right = self.compile_expression(right_operand, codegen_context, Some(&left.ty));
                        let right = match &right {
                            CompilationResult::Value(value) => value,
                            CompilationResult::Returning => return CompilationResult::Returning
                        };

                        let id = codegen_context.create_new_id_for_value();
                        codegen_context.add_computation(IRComp {
                            kind: IRCompKind::BinaryOperation(IRCompBinaryOperation {
                                kind: match operator {
                                    ASTOperator::Plus => IRCompBinaryOperationKind::Plus,
                                    ASTOperator::Minus => IRCompBinaryOperationKind::Minus,
                                    ASTOperator::Mul => IRCompBinaryOperationKind::Mul,
                                    ASTOperator::Div => IRCompBinaryOperationKind::Div,
                                    ASTOperator::Mod => IRCompBinaryOperationKind::Mod,
                                    ASTOperator::BitAnd => IRCompBinaryOperationKind::BitAnd,
                                    ASTOperator::BitOr => IRCompBinaryOperationKind::BitOr,
                                    ASTOperator::BoolAnd => IRCompBinaryOperationKind::BitAnd,
                                    ASTOperator::BoolOr => IRCompBinaryOperationKind::BitOr,
                                    ASTOperator::BitRShift => IRCompBinaryOperationKind::BitRShift,
                                    ASTOperator::BitLShift => IRCompBinaryOperationKind::BitLShift,
                                    _ => unreachable!()
                                },
                                left_operand: self.get_ir_value(left),
                                right_operand: self.get_ir_value(right)
                            }),
                            id: Some(id)
                        });
                        CompilationResult::Value(Value::new_direct(id, left.ty.clone()))
                    }

                    ASTOperator::Equal
                    | ASTOperator::NotEqual
                    | ASTOperator::Greater
                    | ASTOperator::Lesser
                    | ASTOperator::GreaterEqual
                    | ASTOperator::LesserEqual => {
                        if let Some(expected_type) = expected_type {
                            if !expected_type.eq(&Ty::from_primitive(TyPrimitive::Bool)) {
                                panic!("ERROR Unexpected type");
                            }
                        }

                        let left = self.compile_expression(left_operand, codegen_context, None);
                        let left = match &left {
                            CompilationResult::Value(value) => value,
                            CompilationResult::Returning => return CompilationResult::Returning
                        };

                        let right = self.compile_expression(right_operand, codegen_context, Some(&left.ty));
                        let right = match &right {
                            CompilationResult::Value(value) => value,
                            CompilationResult::Returning => return CompilationResult::Returning
                        };

                        let id = codegen_context.create_new_id_for_value();
                        codegen_context.add_computation(IRComp {
                            kind: IRCompKind::BinaryOperation(IRCompBinaryOperation {
                                kind: match operator {
                                    ASTOperator::Equal => IRCompBinaryOperationKind::Equal,
                                    ASTOperator::NotEqual => IRCompBinaryOperationKind::NotEqual,
                                    ASTOperator::Greater => IRCompBinaryOperationKind::Greater,
                                    ASTOperator::GreaterEqual => IRCompBinaryOperationKind::GreaterEqual,
                                    ASTOperator::Lesser => IRCompBinaryOperationKind::Lesser,
                                    ASTOperator::LesserEqual => IRCompBinaryOperationKind::LesserEqual,
                                    _ => unreachable!()
                                },
                                left_operand: self.get_ir_value(left),
                                right_operand: self.get_ir_value(right)
                            }),
                            id: Some(id)
                        });
                        CompilationResult::Value(Value::new_direct(id, Ty::from_primitive(TyPrimitive::Bool)))
                    }
                    _ => unreachable!()
                }
            }
            ASTExpressionKind::If(condition, if_block, else_block) => {
                let condition = self.compile_expression(condition, codegen_context, Some(&Ty::from_primitive(TyPrimitive::Bool)));
                let condition = match &condition {
                    CompilationResult::Value(value) => value,
                    CompilationResult::Returning => return CompilationResult::Returning
                };
                if let Some(expected_type) = expected_type {
                    if !expected_type.eq(&Ty::from_primitive(TyPrimitive::Void)) && else_block.is_none() {
                        panic!("ERROR If expression without an else block is always of type void");
                    }
                };

                let if_end_label = codegen_context.create_new_id_for_label();
                let else_end_label = codegen_context.create_new_id_for_label();

                let condition_negation_id = codegen_context.create_new_id_for_value();
                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::UnaryOperation(IRCompUnaryOperation {
                        kind: IRCompUnaryOperationKind::BoolNot,
                        operand: self.get_ir_value(condition)
                    }),
                    id: Some(condition_negation_id)
                });
                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::If(IRValue { id: condition_negation_id }, if_end_label),
                    id: None
                });

                let result = self.compile_statement_block(if_block, codegen_context, expected_type);
                let expect_from_else = match &result {
                    CompilationResult::Value(value) => {
                        if let Some(expected_type) = expected_type {
                            if !value.ty.eq(expected_type) {
                                panic!("ERROR Unexpected type");
                            }
                        };
                        Some(&value.ty)
                    }
                    CompilationResult::Returning => None
                };

                if else_block.is_some() {
                    codegen_context.add_computation(IRComp {
                        kind: IRCompKind::Jmp(else_end_label),
                        id: None
                    });
                }

                codegen_context.place_label(if_end_label);

                if let Some(else_block) = else_block {
                    let result = self.compile_statement_block(else_block, codegen_context, expect_from_else);
                    codegen_context.place_label(else_end_label);
                    // FIXME handle if elses that are expected to take a type other than void
                    match result {
                        CompilationResult::Value(value) => if !value.ty.eq(&Ty::from_primitive(TyPrimitive::Void)) {
                            todo!("handle if elses that are expected to take a type other than void");
                        }
                        CompilationResult::Returning => {}
                    }

                    CompilationResult::Value(Value::new_none())
                }
                else {
                    CompilationResult::Value(Value::new_none())
                }
            }
            ASTExpressionKind::While(condition, block) => {
                if let Some(expected_type) = expected_type {
                    if !expected_type.eq(&Ty::from_primitive(TyPrimitive::Void)) {
                        panic!("ERROR Type of while expression is always void");
                    }
                }

                let while_begin_label = codegen_context.create_new_id_for_label();
                let while_end_label = codegen_context.create_new_id_for_label();

                codegen_context.place_label(while_begin_label);

                let result = self.compile_expression(condition, codegen_context, Some(&Ty::from_primitive(TyPrimitive::Bool)));
                let condition = match &result {
                    CompilationResult::Value(value) => value,
                    CompilationResult::Returning => return CompilationResult::Returning,
                };

                let condition_negation_id = codegen_context.create_new_id_for_value();
                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::UnaryOperation(IRCompUnaryOperation {
                        kind: IRCompUnaryOperationKind::BoolNot,
                        operand: self.get_ir_value(condition)
                    }),
                    id: Some(condition_negation_id)
                });
                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::If(IRValue { id: condition_negation_id }, while_end_label),
                    id: None
                });

                self.compile_statement_block(block, codegen_context, Some(&Ty::from_primitive(TyPrimitive::Void)));

                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::Jmp(while_begin_label),
                    id: None
                });

                codegen_context.place_label(while_end_label);

                CompilationResult::Value(Value::new_none())
            }
            ASTExpressionKind::Loop(_) => {
                todo!()
            }
            ASTExpressionKind::For(_, _, _) => {
                todo!()
            }
            ASTExpressionKind::Block(block) => {
                self.compile_statement_block(block, codegen_context, expected_type)
            }
            ASTExpressionKind::TemplateApplication(_, _) => {
                todo!()
            }
            ASTExpressionKind::Call(expression, args) => {
                // FIXME Handle template applications
                let (path, remainder) = expression.collect_static_access_path();
                if remainder.is_some() {
                    todo!("Handle function calls that are not just a static path");
                }

                struct Candidate {
                    context: CodegenContext,
                    function: Function,
                    args: Vec<IRValue>,
                    param_returning: bool
                }

                let mut candidates = vec![];

                // NOTE cloning here won't be needed later
                let mut functions: Vec<_> = self.function_context.iter_functions().cloned().collect();
                functions.drain(..).for_each(|function| {
                    if function.path != path { return; }
                    if function.params.len() != args.len() { return; }

                    let mut context = codegen_context.clone();

                    let mut ir_args = Vec::with_capacity(args.len());
                    let mut param_returning = false;
                    for i in 0..args.len() {
                        // FIXME : When Results are used instead of panics, make it so the expected type is the one of the corresponding param
                        let result = self.compile_expression(&args[i], &mut context, None);
                        match result {
                            CompilationResult::Value(value) => {
                                if !value.ty.eq(&function.params[i]) {
                                    return;
                                }
                                ir_args.push(self.get_ir_value(&value));
                            }
                            CompilationResult::Returning => {
                                param_returning = true;
                                break;
                            }
                        }
                    }

                    candidates.push(Candidate {
                        context,
                        function,
                        args: ir_args,
                        param_returning
                    });
                });

                if candidates.is_empty() {
                    panic!("ERROR No candidates were found for function call");
                }
                if candidates.len() > 1 {
                    panic!("ERROR Function call is ambiguous")
                }

                let Candidate { function, context, args, param_returning } = candidates.pop().unwrap();

                if let Some(expected_type) = expected_type {
                    if !function.return_type.eq(expected_type) {
                        panic!("ERROR Unexpected type");
                    }
                }

                if param_returning {
                    // One of the params forced a return during it's compilation
                    return CompilationResult::Returning;
                }

                *codegen_context = context;

                let id = codegen_context.create_new_id_for_value();
                codegen_context.add_computation(IRComp {
                    kind: IRCompKind::FunctionCall(IRCompFunctionCall {
                        name: function.ir_name,
                        args
                    }),
                    id: Some(id)
                });

                CompilationResult::Value(Value::new_direct(id, function.return_type))
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
}