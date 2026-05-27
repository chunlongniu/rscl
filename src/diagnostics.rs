use tower_lsp_server::lsp_types::*;
use tower_lsp_server::Client;

use crate::lexer::Lexer;
use crate::parser::Parser;

pub async fn publish_diagnostics(client: &Client, uri: &Uri, text: &str) {
    let tokens = Lexer::new(text).tokenize();
    // Collect lexer errors
    let lex_errors: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.kind, crate::lexer::TokenKind::Error(_)))
        .map(|t| {
            let ch = if let crate::lexer::TokenKind::Error(c) = t.kind { c } else { '?' };
            crate::parser::ParseError {
                message: format!("unexpected character '{}'", ch),
                span: t.span.clone(),
            }
        })
        .collect();

    let (_, parse_errors) = Parser::new(tokens).parse();

    let all_errors = lex_errors.into_iter().chain(parse_errors.into_iter());

    let line_index = LineIndex::new(text);
    let diagnostics: Vec<Diagnostic> = all_errors
        .map(|e| {
            let start = line_index.position(e.span.start);
            let end = line_index.position(e.span.end);
            Diagnostic {
                range: Range::new(start, end),
                severity: Some(DiagnosticSeverity::ERROR),
                message: e.message,
                source: Some("scl".into()),
                ..Default::default()
            }
        })
        .collect();

    client.publish_diagnostics(uri.clone(), diagnostics, None).await;
}

struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    fn position(&self, offset: usize) -> Position {
        let line = self.line_starts.partition_point(|&s| s <= offset).saturating_sub(1);
        let col = offset - self.line_starts[line];
        Position::new(line as u32, col as u32)
    }
}
