use crate::lexer::Lexer;
use crate::types::TokenKind;
use tower_lsp_server::lsp_types::Position;

pub fn position_to_offset(text: &str, pos: Position) -> usize {
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

pub fn is_inside_comment(text: &str, offset: usize) -> bool {
    let line_start = text[..offset].rfind('\n').map_or(0, |p| p + 1);
    text[line_start..].trim_start().starts_with("//")
}

pub fn collect_declared_vars(source: &str) -> Vec<String> {
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
            TokenKind::Ident(name) if in_var
                && i + 1 < tokens.len() && tokens[i + 1].kind == TokenKind::Colon => {
                    vars.push(name.clone());
            }
            _ => {}
        }
        i += 1;
    }
    vars
}
