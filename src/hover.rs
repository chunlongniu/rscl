use tower_lsp_server::lsp_types::*;

use crate::completion::helpers::position_to_offset;
use crate::lexer::{Lexer, TokenKind};

pub fn get_hover(text: &str, pos: Position) -> Option<Hover> {
    let offset = position_to_offset(text, pos);
    let tokens = Lexer::new(text).tokenize();

    // Find the token at the cursor position
    let tok_idx = tokens.iter().position(|t| t.span.start <= offset && offset <= t.span.end)?;
    let token = &tokens[tok_idx];

    match &token.kind {
        TokenKind::Ident(name) => {
            // Look for this variable's declaration: `name : Type`
            let type_str = find_var_type(text, name)?;
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```scl\n{}: {}\n```", name, type_str),
                }),
                range: None,
            })
        }
        _ => None,
    }
}

fn find_var_type(source: &str, var_name: &str) -> Option<String> {
    let tokens = Lexer::new(source).tokenize();
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
            TokenKind::Ident(name) if in_var && name.eq_ignore_ascii_case(var_name)
                && i + 2 < tokens.len() && tokens[i + 1].kind == TokenKind::Colon => {
                    // Collect type tokens until semicolon
                    let start = tokens[i + 2].span.start;
                    let mut end = start;
                    let mut j = i + 2;
                    while j < tokens.len() && tokens[j].kind != TokenKind::Semicolon {
                        end = tokens[j].span.end;
                        j += 1;
                    }
                    return Some(source[start..end].trim().to_string());
            }
            _ => {}
        }
        i += 1;
    }
    None
}
