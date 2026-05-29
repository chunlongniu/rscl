#![allow(dead_code)]

use super::token::Span;

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Block {
    FunctionBlock(FunctionBlockDecl),
    Function(FunctionDecl),
    DataBlock(DataBlockDecl),
    OrganizationBlock(OrgBlockDecl),
}

#[derive(Debug, Clone)]
pub struct FunctionBlockDecl {
    pub name: String,
    pub span: Span,
    pub var_sections: Vec<VarSection>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub return_type: Option<TypeSpec>,
    pub span: Span,
    pub var_sections: Vec<VarSection>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct DataBlockDecl {
    pub name: String,
    pub span: Span,
    pub var_sections: Vec<VarSection>,
}

#[derive(Debug, Clone)]
pub struct OrgBlockDecl {
    pub name: String,
    pub span: Span,
    pub var_sections: Vec<VarSection>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct VarSection {
    pub kind: VarSectionKind,
    pub decls: Vec<VarDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarSectionKind {
    Var,
    VarInput,
    VarOutput,
    VarInOut,
    VarTemp,
    Const,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub names: Vec<String>,
    pub type_spec: TypeSpec,
    pub init_value: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeSpec {
    Elementary(ElementaryType),
    Array {
        ranges: Vec<(Expression, Expression)>,
        element_type: Box<TypeSpec>,
    },
    Struct(Vec<VarDecl>),
    UserDefined(String),
    StringType(Option<Expression>), // STRING[length]
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementaryType {
    Bool,
    Byte,
    Word,
    DWord,
    Int,
    DInt,
    Real,
    Char,
    Time,
    Date,
    Tod,
    DateAndTime,
    S5Time,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment {
        target: Expression,
        value: Expression,
        span: Span,
    },
    If(IfStatement),
    For(ForStatement),
    While(WhileStatement),
    Repeat(RepeatStatement),
    Case(CaseStatement),
    Return {
        value: Option<Expression>,
        span: Span,
    },
    Call {
        target: Expression,
        args: Vec<CallArg>,
        span: Span,
    },
    Empty,
}

#[derive(Debug, Clone)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_body: Vec<Statement>,
    pub elsif_clauses: Vec<(Expression, Vec<Statement>)>,
    pub else_body: Option<Vec<Statement>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStatement {
    pub var: String,
    pub from: Expression,
    pub to: Expression,
    pub by: Option<Expression>,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RepeatStatement {
    pub body: Vec<Statement>,
    pub condition: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseStatement {
    pub expr: Expression,
    pub branches: Vec<CaseBranch>,
    pub else_body: Option<Vec<Statement>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseBranch {
    pub values: Vec<Expression>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal, Span),
    Ident(String, Span),
    PlcAddress(String, Span),
    Binary {
        op: BinOp,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
        span: Span,
    },
    ArrayAccess {
        array: Box<Expression>,
        indices: Vec<Expression>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
        span: Span,
    },
    FunctionCall {
        name: Box<Expression>,
        args: Vec<CallArg>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Real(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    IntDiv,
    Power,
    And,
    Or,
    Xor,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}
