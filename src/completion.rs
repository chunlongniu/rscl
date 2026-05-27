use tower_lsp_server::lsp_types::*;

use crate::lexer::{Lexer, TokenKind};

pub fn get_completions(text: &str, pos: Position) -> Vec<CompletionItem> {
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

    keywords
        .iter()
        .map(|kw| CompletionItem {
            label: kw.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        })
        .collect()
}

#[derive(Debug)]
enum Context {
    TopLevel,
    InsideBlock,
    VarSection,
    Body,
}

fn detect_context(text: &str, pos: Position) -> Context {
    // Get text up to cursor position
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
