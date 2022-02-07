use crate::globals::Symbol;
use crate::ir::{IRType, IRValue};

#[derive(Debug, Clone)]
pub struct IRCompFunctionCall {
    pub name: Symbol,
    pub args: Vec<IRValue>,
}

#[derive(Debug, Clone)]
pub enum IRCompBinaryOperationKind {
    Plus,
    Minus,
    Mul,
    Div,
    Mod,

    BitAnd,
    BitOr,
    BitRShift,
    BitLShift,

    Equal,
    NotEqual,
    Greater,
    Lesser,
    GreaterEqual,
    LesserEqual,
}

#[derive(Debug, Clone)]
pub struct IRCompBinaryOperation {
    pub kind: IRCompBinaryOperationKind,
    pub left_operand: IRValue,
    pub right_operand: IRValue,
}

#[derive(Debug, Clone)]
pub enum IRCompUnaryOperationKind {
    BoolNot,
    SignedNegation,
}

#[derive(Debug, Clone)]
pub struct IRCompUnaryOperation {
    pub kind: IRCompUnaryOperationKind,
    pub operand: IRValue,
}

#[derive(Debug, Clone)]
pub struct IRCompConstant {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum IRCompKind {
    /// Yields the result of the function call
    FunctionCall(IRCompFunctionCall),
    /// Yields the result of the operation
    BinaryOperation(IRCompBinaryOperation),
    /// Yields the result of the operation
    UnaryOperation(IRCompUnaryOperation),
    /// Yields the constant
    Constant(IRCompConstant),
    /// Yields a pointer to a new location on the stack
    Alloc(IRType),
    /// Yields nothing, takes the second IRValue and writes it to the location given by the first IRValue
    Store(IRType, IRValue, IRValue),
    /// Yields the value at the location
    Load(IRType, IRValue),
    /// Yields nothing, takes the second IRValue and writes it to the location given by the first IRValue (with an offset)
    OffsetStore(IRType, IRValue, IRValue, u64),
    /// Yields the value at the location (with an offset)
    OffsetLoad(IRType, IRValue, u64),
    /// Yields nothing, returns from the function with the value
    Return(IRValue),
    /// Yields nothing, jumps to the location if the value is true
    If(IRValue, u64),
    /// Yields nothing, jumps to the location
    Jmp(u64),
}

#[derive(Debug, Clone)]
pub struct IRComp {
    pub kind: IRCompKind,
}
