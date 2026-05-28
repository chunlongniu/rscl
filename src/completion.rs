use tower_lsp_server::lsp_types::*;

use crate::lexer::{Lexer, TokenKind};

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
        .map(|kw| CompletionItem {
            label: kw.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
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

fn is_inside_comment(text: &str, offset: usize) -> bool {
    let line_start = text[..offset].rfind('\n').map_or(0, |p| p + 1);
    text[line_start..].trim_start().starts_with("//")
}

fn collect_declared_vars(source: &str) -> Vec<String> {
    let tokens = Lexer::new(source).tokenize();
    let mut vars = Vec::new();
    let mut in_var = false;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i].kind {
            TokenKind::Var | TokenKind::VarInput | TokenKind::VarOutput
            | TokenKind::VarInOut | TokenKind::VarTemp => {
                in_var = true;
            }
            TokenKind::EndVar => {
                in_var = false;
            }
            TokenKind::Ident(name) if in_var => {
                if i + 1 < tokens.len() && tokens[i + 1].kind == TokenKind::Colon {
                    vars.push(name.clone());
                }
            }
            _ => {}
        }
        i += 1;
    }
    vars
}

fn position_to_offset(text: &str, pos: Position) -> usize {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in text.char_indices() {
        if line == pos.line && col == pos.character {
            return i;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    text.len()
}
