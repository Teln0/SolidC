use crate::globals::{SessionGlobals, Symbol};
use crate::ir::comp::{IRComp, IRCompBinaryOperationKind, IRCompKind, IRCompUnaryOperationKind};
use crate::ir::{IRItem, IRItemKind, IRModule, IRType, IRValue};

pub mod assembler;

pub fn assembly_for_ir_item(ir_item: &IRItem) -> String {
    let mut result = "".to_owned();

    let dump_ir_type = |ir_type: &IRType| format!("({} {})", ir_type.size, ir_type.align);

    let dump_symbol =
        |symbol: &Symbol| SessionGlobals::with_interner(|interner| interner.get(symbol).unwrap());

    let dump_ir_value = |ir_value: &IRValue| format!("%{}", ir_value.index);

    let dump_ir_comp = |ir_comp: &IRComp, result: &mut String| match &ir_comp.kind {
        IRCompKind::FunctionCall(function_call) => {
            *result += "call ";
            *result += dump_symbol(&function_call.name);
            *result += " ";
            *result += &function_call.args.len().to_string();
            for arg in &function_call.args {
                *result += " ";
                dump_ir_value(arg);
            }
        }
        IRCompKind::BinaryOperation(operation) => {
            *result += "binop ";
            *result += match &operation.kind {
                IRCompBinaryOperationKind::Plus => "+",
                IRCompBinaryOperationKind::Minus => "-",
                IRCompBinaryOperationKind::Mul => "*",
                IRCompBinaryOperationKind::Div => "/",
                IRCompBinaryOperationKind::Mod => "mod",
                IRCompBinaryOperationKind::BitAnd => "&",
                IRCompBinaryOperationKind::BitOr => "|",
                IRCompBinaryOperationKind::BitRShift => ">>",
                IRCompBinaryOperationKind::BitLShift => "<<",
                IRCompBinaryOperationKind::Equal => "==",
                IRCompBinaryOperationKind::NotEqual => "!=",
                IRCompBinaryOperationKind::Greater => ">",
                IRCompBinaryOperationKind::Lesser => "<",
                IRCompBinaryOperationKind::GreaterEqual => ">=",
                IRCompBinaryOperationKind::LesserEqual => "<=",
            };
            *result += " ";
            *result += &dump_ir_value(&operation.left_operand);
            *result += " ";
            *result += &dump_ir_value(&operation.right_operand);
        }
        IRCompKind::UnaryOperation(operation) => {
            *result += "unop ";
            *result += match &operation.kind {
                IRCompUnaryOperationKind::BoolNot => "boolnot",
                IRCompUnaryOperationKind::SignedNegation => "signedneg",
            };
            *result += " ";
            *result += &dump_ir_value(&operation.operand);
        }
        IRCompKind::Constant(constant) => {
            *result += "const ";
            *result += &constant.bytes.len().to_string();
            for byte in &constant.bytes {
                *result += " ";
                *result += &byte.to_string();
            }
        }
        IRCompKind::Alloc(ir_type) => {
            *result += "alloc ";
            *result += &dump_ir_type(ir_type);
        }
        IRCompKind::Store(ir_type, location, value) => {
            *result += "store ";
            *result += &dump_ir_type(ir_type);
            *result += " ";
            *result += &dump_ir_value(location);
            *result += " ";
            *result += &dump_ir_value(value);
        }
        IRCompKind::Load(ir_type, location) => {
            *result += "load ";
            *result += &dump_ir_type(ir_type);
            *result += " ";
            *result += &dump_ir_value(location);
        }
        IRCompKind::OffsetStore(ir_type, location, value, offset) => {
            *result += "offsetstore ";
            *result += &dump_ir_type(ir_type);
            *result += " ";
            *result += &dump_ir_value(location);
            *result += " ";
            *result += &dump_ir_value(value);
            *result += " ";
            *result += &offset.to_string();
        }
        IRCompKind::OffsetLoad(ir_type, location, offset) => {
            *result += "offsetload ";
            *result += &dump_ir_type(ir_type);
            *result += " ";
            *result += &dump_ir_value(location);
            *result += " ";
            *result += &offset.to_string();
        }
        IRCompKind::Return(value) => {
            *result += "return ";
            *result += &dump_ir_value(value);
        }
        IRCompKind::If(cond, location) => {
            *result += "if ";
            *result += &dump_ir_value(cond);
            *result += " ";
            *result += &location.to_string();
        }
        IRCompKind::Jmp(location) => {
            *result += "jmp ";
            *result += &location.to_string();
        }
    };

    match &ir_item.kind {
        IRItemKind::FunctionDef(function_def) => {
            result += "fn ";
            result += dump_symbol(&function_def.name);
            result += ": ";
            for param in &function_def.params {
                result += &dump_ir_type(param);
                result += " ";
            }
            result += "-> ";
            result += &dump_ir_type(&function_def.return_type);

            for comp in &function_def.comps {
                result += "\n    ";
                dump_ir_comp(comp, &mut result);
            }
            result += "\nendfn";
        }
    }

    result
}

pub fn assembly_for_ir_modules(ir_module: &IRModule) -> String {
    let mut result = "".to_owned();

    for item in &ir_module.items {
        result += &assembly_for_ir_item(item);
        result += "\n\n";
    }

    result
}
