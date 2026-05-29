mod blocks;
mod expressions;
mod statements;
mod vars;

use crate::types::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: vec![],
        }
    }

    pub fn parse(mut self) -> (SourceFile, Vec<ParseError>) {
        let mut blocks = Vec::new();
        while !self.at_eof() {
            match self.peek_kind() {
                TokenKind::FunctionBlock => {
                    if let Some(b) = self.parse_function_block() {
                        blocks.push(Block::FunctionBlock(b));
                    }
                }
                TokenKind::Function => {
                    if let Some(b) = self.parse_function() {
                        blocks.push(Block::Function(b));
                    }
                }
                TokenKind::DataBlock => {
                    if let Some(b) = self.parse_data_block() {
                        blocks.push(Block::DataBlock(b));
                    }
                }
                TokenKind::OrganizationBlock => {
                    if let Some(b) = self.parse_org_block() {
                        blocks.push(Block::OrganizationBlock(b));
                    }
                }
                _ => {
                    self.error(
                        "expected FUNCTION_BLOCK, FUNCTION, DATA_BLOCK, or ORGANIZATION_BLOCK",
                    );
                    self.advance();
                }
            }
        }
        (SourceFile { blocks }, self.errors)
    }

    // --- Helpers ---

    fn peek_kind(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span.clone())
            .unwrap_or(Span { start: 0, end: 0 })
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            self.error(&format!("expected {:?}", kind));
            false
        }
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self) -> String {
        match self.peek_kind().clone() {
            TokenKind::Ident(name) | TokenKind::QuotedIdent(name) => {
                self.advance();
                name
            }
            _ => {
                self.error("expected identifier");
                String::from("<error>")
            }
        }
    }

    fn error(&mut self, msg: &str) {
        let span = self.peek_span();
        self.errors.push(ParseError {
            message: msg.to_string(),
            span,
        });
    }

    fn skip_until(&mut self, sync: &[TokenKind]) {
        while !self.at_eof() {
            if sync
                .iter()
                .any(|k| std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(k))
            {
                return;
            }
            self.advance();
        }
    }
}

impl Expression {
    pub fn span(&self) -> &Span {
        match self {
            Expression::Literal(_, s) => s,
            Expression::Ident(_, s) => s,
            Expression::PlcAddress(_, s) => s,
            Expression::Binary { span, .. } => span,
            Expression::Unary { span, .. } => span,
            Expression::ArrayAccess { span, .. } => span,
            Expression::FieldAccess { span, .. } => span,
            Expression::FunctionCall { span, .. } => span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> (SourceFile, Vec<ParseError>) {
        let tokens = Lexer::new(src).tokenize();
        Parser::new(tokens).parse()
    }

    #[test]
    fn test_parse_function_block() {
        let (sf, errors) = parse(
            "FUNCTION_BLOCK MyFB\nVAR\n  x : INT;\nEND_VAR\nBEGIN\n  x := 1;\nEND_FUNCTION_BLOCK",
        );
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(sf.blocks.len(), 1);
    }

    #[test]
    fn test_parse_if() {
        let (sf, errors) = parse("FUNCTION_BLOCK T\nBEGIN\nIF x > 0 THEN\n  y := 1;\nELSIF x < 0 THEN\n  y := -1;\nELSE\n  y := 0;\nEND_IF;\nEND_FUNCTION_BLOCK");
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(sf.blocks.len(), 1);
    }

    #[test]
    fn test_error_recovery() {
        let (sf, errors) =
            parse("FUNCTION_BLOCK T\nVAR\n  x : ;\nEND_VAR\nBEGIN\nEND_FUNCTION_BLOCK");
        assert!(!errors.is_empty());
        assert_eq!(sf.blocks.len(), 1);
    }

    #[test]
    fn test_metadata_with_semicolons() {
        let (sf, errors) = parse(
            "FUNCTION_BLOCK \"FB\"\n  TITLE = 'test';\n  VERSION : '1.0';\n  VAR_INPUT\n    x : BOOL := FALSE;\n  END_VAR\nBEGIN\n  x := TRUE;\nEND_FUNCTION_BLOCK",
        );
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(sf.blocks.len(), 1);
    }
}

