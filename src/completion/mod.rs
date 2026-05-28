mod helpers;

use tower_lsp_server::lsp_types::*;

use crate::lexer::{Lexer, TokenKind};
use helpers::{collect_declared_vars, is_inside_comment, position_to_offset};

pub fn get_completions(text: &str, pos: Position, all_sources: &[&str]) -> Vec<CompletionItem> {
    let offset = position_to_offset(text, pos);
    if is_inside_comment(text, offset) {
        return Vec::new();
    }
    let context = detect_context(text, pos);
    let keywords = match context {
        Context::TopLevel => &[
            "FUNCTION_BLOCK",
            "FUNCTION",
            "DATA_BLOCK",
            "ORGANIZATION_BLOCK",
        ][..],
        Context::InsideBlock => &[
            "VAR",
            "VAR_INPUT",
            "VAR_OUTPUT",
            "VAR_IN_OUT",
            "VAR_TEMP",
            "CONST",
            "BEGIN",
        ][..],
        Context::VarSection => &[
            "BOOL",
            "BYTE",
            "WORD",
            "DWORD",
            "INT",
            "DINT",
            "REAL",
            "CHAR",
            "STRING",
            "TIME",
            "DATE",
            "TIME_OF_DAY",
            "DATE_AND_TIME",
            "S5TIME",
            "ARRAY",
            "STRUCT",
        ][..],
        Context::Body => &[
            "IF",
            "THEN",
            "ELSIF",
            "ELSE",
            "END_IF",
            "FOR",
            "TO",
            "BY",
            "DO",
            "END_FOR",
            "WHILE",
            "END_WHILE",
            "REPEAT",
            "UNTIL",
            "END_REPEAT",
            "CASE",
            "OF",
            "END_CASE",
            "RETURN",
            "TRUE",
            "FALSE",
            "AND",
            "OR",
            "XOR",
            "NOT",
            "MOD",
            "DIV",
        ][..],
    };

    let mut items: Vec<CompletionItem> = keywords
        .iter()
        .map(|kw| {
            if let Some(snippet) = block_snippet(kw) {
                CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    insert_text: Some(snippet.to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                }
            } else {
                CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    ..Default::default()
                }
            }
        })
        .collect();

    if matches!(context, Context::Body) {
        for src in all_sources {
            for name in collect_declared_vars(src) {
                items.push(CompletionItem {
                    label: name,
                    kind: Some(CompletionItemKind::VARIABLE),
                    ..Default::default()
                });
            }
        }
    }

    items
}

#[derive(Debug)]
enum Context {
    TopLevel,
    InsideBlock,
    VarSection,
    Body,
}

fn detect_context(text: &str, pos: Position) -> Context {
    let offset = position_to_offset(text, pos);
    let prefix = &text[..offset.min(text.len())];
    let tokens = Lexer::new(prefix).tokenize();

    let mut in_block = false;
    let mut in_var = false;
    let mut in_body = false;

    for tok in &tokens {
        match &tok.kind {
            TokenKind::FunctionBlock
            | TokenKind::Function
            | TokenKind::DataBlock
            | TokenKind::OrganizationBlock => {
                in_block = true;
                in_var = false;
                in_body = false;
            }
            TokenKind::EndFunctionBlock | TokenKind::EndFunction | TokenKind::EndDataBlock => {
                in_block = false;
                in_var = false;
                in_body = false;
            }
            TokenKind::Var
            | TokenKind::VarInput
            | TokenKind::VarOutput
            | TokenKind::VarInOut
            | TokenKind::VarTemp
            | TokenKind::Const => {
                in_var = true;
                in_body = false;
            }
            TokenKind::EndVar | TokenKind::EndConst => {
                in_var = false;
            }
            TokenKind::Begin => {
                in_var = false;
                in_body = true;
            }
            _ => {}
        }
    }

    if in_body {
        Context::Body
    } else if in_var {
        Context::VarSection
    } else if in_block {
        Context::InsideBlock
    } else {
        Context::TopLevel
    }
}

fn block_snippet(keyword: &str) -> Option<&'static str> {
    match keyword {
        "FUNCTION_BLOCK" => Some("FUNCTION_BLOCK $1\n$0\nEND_FUNCTION_BLOCK"),
        "FUNCTION" => Some("FUNCTION $1\n$0\nEND_FUNCTION"),
        "DATA_BLOCK" => Some("DATA_BLOCK $1\n$0\nEND_DATA_BLOCK"),
        "ORGANIZATION_BLOCK" => Some("ORGANIZATION_BLOCK $1\n$0\nEND_ORGANIZATION_BLOCK"),
        "VAR" => Some("VAR\n\t$0\nEND_VAR"),
        "VAR_INPUT" => Some("VAR_INPUT\n\t$0\nEND_VAR"),
        "VAR_OUTPUT" => Some("VAR_OUTPUT\n\t$0\nEND_VAR"),
        "VAR_IN_OUT" => Some("VAR_IN_OUT\n\t$0\nEND_VAR"),
        "VAR_TEMP" => Some("VAR_TEMP\n\t$0\nEND_VAR"),
        "CONST" => Some("CONST\n\t$0\nEND_CONST"),
        "IF" => Some("IF $1 THEN\n\t$0\nEND_IF"),
        "FOR" => Some("FOR $1 TO $2 DO\n\t$0\nEND_FOR"),
        "WHILE" => Some("WHILE $1 DO\n\t$0\nEND_WHILE"),
        "REPEAT" => Some("REPEAT\n\t$0\nUNTIL $1\nEND_REPEAT"),
        "STRUCT" => Some("STRUCT\n\t$0\nEND_STRUCT"),
        "CASE" => Some("CASE $1 OF\n\t$0\nEND_CASE"),
        _ => None,
    }
}
