#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords - Blocks
    Function,
    FunctionBlock,
    DataBlock,
    OrganizationBlock,
    EndFunction,
    EndFunctionBlock,
    EndDataBlock,
    // Keywords - Var sections
    Var,
    VarInput,
    VarOutput,
    VarInOut,
    VarTemp,
    Const,
    EndVar,
    EndConst,
    // Keywords - Control flow
    If,
    Then,
    Elsif,
    Else,
    EndIf,
    For,
    To,
    By,
    Do,
    EndFor,
    While,
    EndWhile,
    Repeat,
    Until,
    EndRepeat,
    Case,
    Of,
    EndCase,
    Return,
    Begin,
    // Keywords - Types
    Bool,
    Byte,
    Word,
    DWord,
    Int,
    DInt,
    Real,
    Char,
    StringKw,
    Time,
    Date,
    Tod,
    DateAndTime,
    S5Time,
    Array,
    Struct,
    EndStruct,
    EndType,
    Type,
    // Keywords - Operators
    And,
    Or,
    Xor,
    Not,
    Mod,
    Div,
    // Literals
    True,
    False,
    // Keywords - Metadata
    Title,
    Version,
    NonRetain,
    // Identifiers and literals
    Ident(String),
    QuotedIdent(String),
    IntLiteral(i64),
    RealLiteral(f64),
    StringLiteral(String),
    // PLC address (e.g. IW0, MW10, Q0.1, DB1.DBW0)
    PlcAddress(String),
    // Operators and punctuation
    Assign,    // :=
    Colon,     // :
    Semicolon, // ;
    Comma,     // ,
    Dot,       // .
    DotDot,    // ..
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Power,     // **
    Eq,        // =
    Neq,       // <>
    Lt,        // <
    Gt,        // >
    Le,        // <=
    Ge,        // >=
    Hash,      // #
    // Special
    Error(char),
    Eof,
}
