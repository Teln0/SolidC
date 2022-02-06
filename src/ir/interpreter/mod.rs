use std::collections::HashMap;
use std::ops::Rem;
use crate::globals::Symbol;
use crate::ir::{IRItemFunctionDef, IRItemKind, IRModule};
use crate::ir::comp::{IRCompBinaryOperationKind, IRCompKind, IRCompUnaryOperationKind};

#[derive(Debug, Clone)]
pub struct IRInterpreterValue {
    pub bytes: Vec<u8>
}

impl IRInterpreterValue {
    pub fn void() -> Self {
        Self { bytes: vec![] }
    }

    pub fn from_u8(value: u8) -> Self {
        Self {
            bytes: vec![value]
        }
    }

    pub fn from_i8(value: i8) -> Self {
        Self {
            bytes: vec![unsafe { std::mem::transmute(value) }]
        }
    }

    pub fn from_u16(value: u16) -> Self {
        let bytes: [u8; 2] = unsafe { std::mem::transmute(value) };
        Self {
            bytes: bytes.to_vec()
        }
    }

    pub fn from_i16(value: i16) -> Self {
        let bytes: [u8; 2] = unsafe { std::mem::transmute(value) };
        Self {
            bytes: bytes.to_vec()
        }
    }

    pub fn from_u32(value: u32) -> Self {
        let bytes: [u8; 4] = unsafe { std::mem::transmute(value) };
        Self {
            bytes: bytes.to_vec()
        }
    }

    pub fn from_i32(value: i32) -> Self {
        let bytes: [u8; 4] = unsafe { std::mem::transmute(value) };
        Self {
            bytes: bytes.to_vec()
        }
    }

    pub fn from_u64(value: u64) -> Self {
        let bytes: [u8; 8] = unsafe { std::mem::transmute(value) };
        Self {
            bytes: bytes.to_vec()
        }
    }

    pub fn from_i64(value: i64) -> Self {
        let bytes: [u8; 8] = unsafe { std::mem::transmute(value) };
        Self {
            bytes: bytes.to_vec()
        }
    }

    pub fn into_u8(&self) -> u8 {
        self.bytes[0]
    }

    pub fn into_i8(&self) -> i8 {
        unsafe { std::mem::transmute(self.bytes[0]) }
    }

    pub fn into_u16(&self) -> u16 {
        union U {
            val: u16,
            bytes: [u8; 2]
        }

        unsafe {
            let mut u = U { val: 0 };
            for i in 0..u.bytes.len() {
                u.bytes[i] = self.bytes[i];
            }
            u.val
        }
    }

    pub fn into_i16(&self) -> i16 {
        union U {
            val: i16,
            bytes: [u8; 2]
        }

        unsafe {
            let mut u = U { val: 0 };
            for i in 0..u.bytes.len() {
                u.bytes[i] = self.bytes[i];
            }
            u.val
        }
    }

    pub fn into_u32(&self) -> u32 {
        union U {
            val: u32,
            bytes: [u8; 4]
        }

        unsafe {
            let mut u = U { val: 0 };
            for i in 0..u.bytes.len() {
                u.bytes[i] = self.bytes[i];
            }
            u.val
        }
    }

    pub fn into_i32(&self) -> i32 {
        union U {
            val: i32,
            bytes: [u8; 4]
        }

        unsafe {
            let mut u = U { val: 0 };
            for i in 0..u.bytes.len() {
                u.bytes[i] = self.bytes[i];
            }
            u.val
        }
    }

    pub fn into_u64(&self) -> u64 {
        union U {
            val: u64,
            bytes: [u8; 8]
        }

        unsafe {
            let mut u = U { val: 0 };
            for i in 0..u.bytes.len() {
                u.bytes[i] = self.bytes[i];
            }
            u.val
        }
    }

    pub fn into_i64(&self) -> i64 {
        union U {
            val: i64,
            bytes: [u8; 8]
        }

        unsafe {
            let mut u = U { val: 0 };
            for i in 0..u.bytes.len() {
                u.bytes[i] = self.bytes[i];
            }
            u.val
        }
    }
}

struct IRInterpreterStack {
    values: Vec<IRInterpreterValue>,
    frames: Vec<usize>
}

pub struct IRInterpreter {
    functions: HashMap<Symbol, IRItemFunctionDef>,
    stack: IRInterpreterStack
}

impl IRInterpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            stack: IRInterpreterStack {
                values: vec![],
                frames: vec![]
            }
        }
    }

    pub fn load_module(&mut self, module: IRModule) {
        for item in module.items {
            match item.kind {
                IRItemKind::FunctionDef(function_def) => {
                    self.functions.insert(function_def.name, function_def);
                }
            }
        }
    }

    pub unsafe fn call_function(&mut self, function_name: Symbol, args: &[IRInterpreterValue]) -> IRInterpreterValue {
        let function_def = &self.functions[&function_name];
        let computations_offset = args.len();
        let comps_len = function_def.comps.len();
        let values_size = computations_offset + comps_len;
        let mut values = Vec::with_capacity(values_size);

        self.stack.frames.push(0);

        for arg in args {
            values.push(arg.clone());
        }
        for _ in 0..values_size {
            values.push(IRInterpreterValue::void());
        }

        // TODO : Take a reference instead of cloning
        let comps = function_def.comps.clone();
        let mut current_comp = 0;
        while current_comp < comps_len {
            let mut performed_jump = false;
            let mut target_comp = 0;
            let comp = &comps[current_comp];

            values[computations_offset + current_comp] = match &comp.kind {
                IRCompKind::FunctionCall(function_call) => {
                    let name = function_call.name;
                    let args = function_call.args
                        .iter()
                        .map(|irv| values[irv.index as usize].clone())
                        .collect::<Vec<_>>();

                    self.call_function(
                        name,
                        &args
                    )
                }
                IRCompKind::BinaryOperation(operation) => {
                    let left_operand = &values[operation.left_operand.index as usize];
                    let right_operand = &values[operation.right_operand.index as usize];

                    let size = right_operand.bytes.len();

                    if size == left_operand.bytes.len() {
                        match size {
                            1 => match operation.kind {
                                IRCompBinaryOperationKind::Plus => IRInterpreterValue::from_u8(left_operand.into_u8().wrapping_add(right_operand.into_u8())),
                                IRCompBinaryOperationKind::Minus => IRInterpreterValue::from_u8(left_operand.into_u8().wrapping_sub(right_operand.into_u8())),
                                IRCompBinaryOperationKind::Mul => IRInterpreterValue::from_u8(left_operand.into_u8().wrapping_mul(right_operand.into_u8())),
                                IRCompBinaryOperationKind::Div => IRInterpreterValue::from_u8(left_operand.into_u8().wrapping_div(right_operand.into_u8())),
                                IRCompBinaryOperationKind::Mod => IRInterpreterValue::from_u8(left_operand.into_u8().rem(right_operand.into_u8())),
                                IRCompBinaryOperationKind::BitAnd => IRInterpreterValue::from_u8(left_operand.into_u8() & right_operand.into_u8()),
                                IRCompBinaryOperationKind::BitOr => IRInterpreterValue::from_u8(left_operand.into_u8() | right_operand.into_u8()),
                                IRCompBinaryOperationKind::BitRShift => IRInterpreterValue::from_u8(left_operand.into_u8() >> right_operand.into_u8()),
                                IRCompBinaryOperationKind::BitLShift => IRInterpreterValue::from_u8(left_operand.into_u8() << right_operand.into_u8()),
                                IRCompBinaryOperationKind::Equal => IRInterpreterValue::from_u8((left_operand.into_u8() == right_operand.into_u8()) as u8),
                                IRCompBinaryOperationKind::NotEqual => IRInterpreterValue::from_u8((left_operand.into_u8() != right_operand.into_u8()) as u8),
                                IRCompBinaryOperationKind::Greater => IRInterpreterValue::from_u8((left_operand.into_u8() > right_operand.into_u8()) as u8),
                                IRCompBinaryOperationKind::Lesser => IRInterpreterValue::from_u8((left_operand.into_u8() < right_operand.into_u8()) as u8),
                                IRCompBinaryOperationKind::GreaterEqual => IRInterpreterValue::from_u8((left_operand.into_u8() >= right_operand.into_u8()) as u8),
                                IRCompBinaryOperationKind::LesserEqual => IRInterpreterValue::from_u8((left_operand.into_u8() <= right_operand.into_u8()) as u8),
                            }

                            2 => match operation.kind {
                                IRCompBinaryOperationKind::Plus => IRInterpreterValue::from_u16(left_operand.into_u16().wrapping_add(right_operand.into_u16())),
                                IRCompBinaryOperationKind::Minus => IRInterpreterValue::from_u16(left_operand.into_u16().wrapping_sub(right_operand.into_u16())),
                                IRCompBinaryOperationKind::Mul => IRInterpreterValue::from_u16(left_operand.into_u16().wrapping_mul(right_operand.into_u16())),
                                IRCompBinaryOperationKind::Div => IRInterpreterValue::from_u16(left_operand.into_u16().wrapping_div(right_operand.into_u16())),
                                IRCompBinaryOperationKind::Mod => IRInterpreterValue::from_u16(left_operand.into_u16().rem(right_operand.into_u16())),
                                IRCompBinaryOperationKind::BitAnd => IRInterpreterValue::from_u16(left_operand.into_u16() & right_operand.into_u16()),
                                IRCompBinaryOperationKind::BitOr => IRInterpreterValue::from_u16(left_operand.into_u16() | right_operand.into_u16()),
                                IRCompBinaryOperationKind::BitRShift => IRInterpreterValue::from_u16(left_operand.into_u16() >> right_operand.into_u16()),
                                IRCompBinaryOperationKind::BitLShift => IRInterpreterValue::from_u16(left_operand.into_u16() << right_operand.into_u16()),
                                IRCompBinaryOperationKind::Equal => IRInterpreterValue::from_u8((left_operand.into_u16() == right_operand.into_u16()) as u8),
                                IRCompBinaryOperationKind::NotEqual => IRInterpreterValue::from_u8((left_operand.into_u16() != right_operand.into_u16()) as u8),
                                IRCompBinaryOperationKind::Greater => IRInterpreterValue::from_u8((left_operand.into_u16() > right_operand.into_u16()) as u8),
                                IRCompBinaryOperationKind::Lesser => IRInterpreterValue::from_u8((left_operand.into_u16() < right_operand.into_u16()) as u8),
                                IRCompBinaryOperationKind::GreaterEqual => IRInterpreterValue::from_u8((left_operand.into_u16() >= right_operand.into_u16()) as u8),
                                IRCompBinaryOperationKind::LesserEqual => IRInterpreterValue::from_u8((left_operand.into_u16() <= right_operand.into_u16()) as u8),
                            }

                            4 => match operation.kind {
                                IRCompBinaryOperationKind::Plus => IRInterpreterValue::from_u32(left_operand.into_u32().wrapping_add(right_operand.into_u32())),
                                IRCompBinaryOperationKind::Minus => IRInterpreterValue::from_u32(left_operand.into_u32().wrapping_sub(right_operand.into_u32())),
                                IRCompBinaryOperationKind::Mul => IRInterpreterValue::from_u32(left_operand.into_u32().wrapping_mul(right_operand.into_u32())),
                                IRCompBinaryOperationKind::Div => IRInterpreterValue::from_u32(left_operand.into_u32().wrapping_div(right_operand.into_u32())),
                                IRCompBinaryOperationKind::Mod => IRInterpreterValue::from_u32(left_operand.into_u32().rem(right_operand.into_u32())),
                                IRCompBinaryOperationKind::BitAnd => IRInterpreterValue::from_u32(left_operand.into_u32() & right_operand.into_u32()),
                                IRCompBinaryOperationKind::BitOr => IRInterpreterValue::from_u32(left_operand.into_u32() | right_operand.into_u32()),
                                IRCompBinaryOperationKind::BitRShift => IRInterpreterValue::from_u32(left_operand.into_u32() >> right_operand.into_u32()),
                                IRCompBinaryOperationKind::BitLShift => IRInterpreterValue::from_u32(left_operand.into_u32() << right_operand.into_u32()),
                                IRCompBinaryOperationKind::Equal => IRInterpreterValue::from_u8((left_operand.into_u32() == right_operand.into_u32()) as u8),
                                IRCompBinaryOperationKind::NotEqual => IRInterpreterValue::from_u8((left_operand.into_u32() != right_operand.into_u32()) as u8),
                                IRCompBinaryOperationKind::Greater => IRInterpreterValue::from_u8((left_operand.into_u32() > right_operand.into_u32()) as u8),
                                IRCompBinaryOperationKind::Lesser => IRInterpreterValue::from_u8((left_operand.into_u32() < right_operand.into_u32()) as u8),
                                IRCompBinaryOperationKind::GreaterEqual => IRInterpreterValue::from_u8((left_operand.into_u32() >= right_operand.into_u32()) as u8),
                                IRCompBinaryOperationKind::LesserEqual => IRInterpreterValue::from_u8((left_operand.into_u32() <= right_operand.into_u32()) as u8),
                            }

                            8 => match operation.kind {
                                IRCompBinaryOperationKind::Plus => IRInterpreterValue::from_u64(left_operand.into_u64().wrapping_add(right_operand.into_u64())),
                                IRCompBinaryOperationKind::Minus => IRInterpreterValue::from_u64(left_operand.into_u64().wrapping_sub(right_operand.into_u64())),
                                IRCompBinaryOperationKind::Mul => IRInterpreterValue::from_u64(left_operand.into_u64().wrapping_mul(right_operand.into_u64())),
                                IRCompBinaryOperationKind::Div => IRInterpreterValue::from_u64(left_operand.into_u64().wrapping_div(right_operand.into_u64())),
                                IRCompBinaryOperationKind::Mod => IRInterpreterValue::from_u64(left_operand.into_u64().rem(right_operand.into_u64())),
                                IRCompBinaryOperationKind::BitAnd => IRInterpreterValue::from_u64(left_operand.into_u64() & right_operand.into_u64()),
                                IRCompBinaryOperationKind::BitOr => IRInterpreterValue::from_u64(left_operand.into_u64() | right_operand.into_u64()),
                                IRCompBinaryOperationKind::BitRShift => IRInterpreterValue::from_u64(left_operand.into_u64() >> right_operand.into_u64()),
                                IRCompBinaryOperationKind::BitLShift => IRInterpreterValue::from_u64(left_operand.into_u64() << right_operand.into_u64()),
                                IRCompBinaryOperationKind::Equal => IRInterpreterValue::from_u8((left_operand.into_u64() == right_operand.into_u64()) as u8),
                                IRCompBinaryOperationKind::NotEqual => IRInterpreterValue::from_u8((left_operand.into_u64() != right_operand.into_u64()) as u8),
                                IRCompBinaryOperationKind::Greater => IRInterpreterValue::from_u8((left_operand.into_u64() > right_operand.into_u64()) as u8),
                                IRCompBinaryOperationKind::Lesser => IRInterpreterValue::from_u8((left_operand.into_u64() < right_operand.into_u64()) as u8),
                                IRCompBinaryOperationKind::GreaterEqual => IRInterpreterValue::from_u8((left_operand.into_u64() >= right_operand.into_u64()) as u8),
                                IRCompBinaryOperationKind::LesserEqual => IRInterpreterValue::from_u8((left_operand.into_u64() <= right_operand.into_u64()) as u8),
                            }

                            _ => IRInterpreterValue::void()
                        }
                    }
                    else {
                        IRInterpreterValue::void()
                    }
                }
                IRCompKind::UnaryOperation(operation) => {
                    let operand = &values[operation.operand.index as usize];

                    let size = operand.bytes.len();

                    match size {
                        1 => match operation.kind {
                            IRCompUnaryOperationKind::BoolNot => IRInterpreterValue::from_u8(if operand.into_u8() == 0 { 1 } else { 0 }),
                            IRCompUnaryOperationKind::SignedNegation => IRInterpreterValue::from_i8(-operand.into_i8())
                        }

                        2 => match operation.kind {
                            IRCompUnaryOperationKind::BoolNot => IRInterpreterValue::void(),
                            IRCompUnaryOperationKind::SignedNegation => IRInterpreterValue::from_i16(-operand.into_i16())
                        }

                        4 => match operation.kind {
                            IRCompUnaryOperationKind::BoolNot => IRInterpreterValue::void(),
                            IRCompUnaryOperationKind::SignedNegation => IRInterpreterValue::from_i32(-operand.into_i32())
                        }

                        8 => match operation.kind {
                            IRCompUnaryOperationKind::BoolNot => IRInterpreterValue::void(),
                            IRCompUnaryOperationKind::SignedNegation => IRInterpreterValue::from_i64(-operand.into_i64())
                        }

                        _ => IRInterpreterValue::void()
                    }
                }
                IRCompKind::Constant(constant) => {
                    IRInterpreterValue {
                        bytes: constant.bytes.clone()
                    }
                }
                IRCompKind::Alloc(ir_type) => {
                    *self.stack.frames.last_mut().unwrap() += 1;

                    let bytes = vec![0; ir_type.size as usize];
                    self.stack.values.push(IRInterpreterValue { bytes });
                    let ptr = self.stack.values.last().unwrap().bytes.as_ptr() as u64;

                    IRInterpreterValue::from_u64(ptr)
                }
                IRCompKind::Store(ir_type, location, value) => {
                    let value = &values[value.index as usize];
                    let ptr = values[location.index as usize].into_u64() as *mut u8;
                    let slice = std::slice::from_raw_parts_mut(ptr, ir_type.size as usize);
                    for i in 0..slice.len() {
                        slice[i] = value.bytes[i];
                    }

                    IRInterpreterValue::void()
                }
                IRCompKind::Load(ir_type, location) => {
                    let ptr = values[location.index as usize].into_u64() as *const u8;
                    let slice = std::slice::from_raw_parts(ptr, ir_type.size as usize);

                    IRInterpreterValue {
                        bytes: slice.to_vec()
                    }
                }
                IRCompKind::OffsetStore(ir_type, location, value, offset) => {
                    let value = &values[value.index as usize];
                    let ptr = (values[location.index as usize].into_u64() + *offset) as *mut u8;
                    let slice = std::slice::from_raw_parts_mut(ptr, ir_type.size as usize);
                    for i in 0..slice.len() {
                        slice[i] = value.bytes[i];
                    }

                    IRInterpreterValue::void()
                }
                IRCompKind::OffsetLoad(ir_type, location, offset) => {
                    let ptr = (values[location.index as usize].into_u64() + *offset) as *const u8;
                    let slice = std::slice::from_raw_parts(ptr, ir_type.size as usize);

                    IRInterpreterValue {
                        bytes: slice.to_vec()
                    }
                }
                IRCompKind::Return(value) => {
                    let value = &values[value.index as usize];

                    for _ in 0..self.stack.frames.pop().unwrap() {
                        self.stack.values.pop();
                    }

                    return value.clone();
                }
                IRCompKind::If(value, location) => {
                    let value = &values[value.index as usize];
                    if value.into_u8() != 0 {
                        performed_jump = true;
                        target_comp = *location as usize;
                    }

                    IRInterpreterValue::void()
                }
                IRCompKind::Jmp(location) => {
                    performed_jump = true;
                    target_comp = *location as usize;

                    IRInterpreterValue::void()
                }
            };

            if !performed_jump {
                current_comp += 1;
            }
            else {
                current_comp = target_comp;
            }
        }

        for _ in 0..self.stack.frames.pop().unwrap() {
            self.stack.values.pop();
        }

        IRInterpreterValue::void()
    }
}