use crate::globals::Symbol;
use crate::solidlang::span::Span;

#[derive(Debug, Clone)]
pub struct ASTModule {
    pub items: Vec<ASTItem>,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ASTItemKind {
    FunctionDef(ASTFunctionDef),
    StructDef(ASTStructDef),
    Template(ASTTemplate),
}

#[derive(Debug, Clone)]
pub struct ASTItem {
    pub kind: ASTItemKind,
}

#[derive(Debug, Clone)]
pub struct ASTFunctionDef {
    pub name: Symbol,
    pub return_type: Option<ASTType>,
    pub params: Vec<ASTNameAndType>,
    pub statement_block: ASTStatementBlock,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ASTStructDef {
    pub name: Symbol,
    pub fields: Vec<ASTNameAndType>,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ASTTemplate {
    pub params: Vec<Symbol>,
    pub items: Vec<ASTItem>,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ASTTypeKind {
    Path {
        symbols: Vec<Symbol>,
        generic_args: Vec<ASTType>,
    },
    PointerTo(Box<ASTType>),
}

#[derive(Debug, Clone)]
pub struct ASTType {
    pub kind: ASTTypeKind,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ASTNameAndType {
    pub name: Symbol,
    pub ast_type: ASTType,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ASTStatementKind {
    // TODO : Matching local binding
    LocalBinding(Symbol, Option<ASTType>, Option<ASTExpression>),
    Expression(ASTExpression),
    Return(ASTExpression),
    Break,
    Continue,
    Item(ASTItem),
    Semicolon,
}

#[derive(Debug, Clone)]
pub struct ASTStatementBlock {
    pub statements: Vec<ASTStatement>,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ASTStatement {
    pub kind: ASTStatementKind,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ASTOperator {
    Assign,

    Plus,
    Minus,
    Mul,
    Div,
    Mod,

    BitAnd,
    BitOr,
    BitNot,
    BitRShift,
    BitLShift,

    BoolAnd,
    BoolOr,
    BoolNot,

    Equal,
    NotEqual,
    Greater,
    Lesser,
    GreaterEqual,
    LesserEqual,
}

#[derive(Debug, Clone)]
pub enum ASTExpressionKind {
    Ident(Symbol),
    IntegerLiteral(u64),
    Boolean(bool),

    UnaryOperation(ASTOperator, Box<ASTExpression>),
    BinaryOperation(ASTOperator, Box<ASTExpression>, Box<ASTExpression>),

    If(
        Box<ASTExpression>,
        ASTStatementBlock,
        Option<ASTStatementBlock>,
    ),
    While(Box<ASTExpression>, ASTStatementBlock),
    Loop(ASTStatementBlock),
    For(Symbol, Box<ASTExpression>, ASTStatementBlock),
    Block(ASTStatementBlock),

    TemplateApplication(Box<ASTExpression>, Vec<ASTType>),
    Call(Box<ASTExpression>, Vec<ASTExpression>),
    Index(Box<ASTExpression>, Box<ASTExpression>),

    MemberAccess(Box<ASTExpression>, Symbol),
    StaticAccess(Box<ASTExpression>, Symbol), // TODO : Match
}

#[derive(Debug, Clone)]
pub struct ASTExpression {
    pub kind: ASTExpressionKind,

    pub span: Span,
}

impl ASTExpression {
    pub fn collect_static_access_path(&self) -> (Vec<Symbol>, Option<&ASTExpression>) {
        let mut expression = self;
        let mut path = vec![];
        let remainder = loop {
            match &expression.kind {
                ASTExpressionKind::Ident(symbol) => {
                    path.push(*symbol);
                    break None;
                }
                ASTExpressionKind::StaticAccess(e, symbol) => {
                    path.push(*symbol);
                    expression = e;
                }
                _ => break Some(expression),
            }
        };
        (path, remainder)
    }
}

impl ASTStatement {
    pub fn requires_semi(&self) -> bool {
        match &self.kind {
            ASTStatementKind::Expression(e) => match e.kind {
                ASTExpressionKind::If(_, _, _) => false,
                ASTExpressionKind::While(_, _) => false,
                ASTExpressionKind::Loop(_) => false,
                ASTExpressionKind::For(_, _, _) => false,
                _ => true,
            },
            ASTStatementKind::Item(_) => false,
            ASTStatementKind::Semicolon => false,
            _ => true,
        }
    }
}
