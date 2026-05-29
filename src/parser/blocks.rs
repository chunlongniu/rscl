use crate::types::*;

use super::Parser;

impl Parser {
    pub(super) fn parse_function_block(&mut self) -> Option<FunctionBlockDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume FUNCTION_BLOCK
        let name = self.expect_ident();
        // Skip optional metadata (TITLE, VERSION)
        self.skip_block_metadata();
        let var_sections = self.parse_var_sections();
        self.eat(&TokenKind::Begin);
        let body = self.parse_statement_list();
        let end = self.peek_span().end;
        self.eat(&TokenKind::EndFunctionBlock);
        Some(FunctionBlockDecl {
            name,
            span: Span { start, end },
            var_sections,
            body,
        })
    }

    pub(super) fn parse_function(&mut self) -> Option<FunctionDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume FUNCTION
        let name = self.expect_ident();
        let return_type = if self.eat(&TokenKind::Colon) {
            Some(self.parse_type_spec())
        } else {
            None
        };
        // Skip optional metadata (TITLE, VERSION)
        self.skip_block_metadata();
        let var_sections = self.parse_var_sections();
        self.eat(&TokenKind::Begin);
        let body = self.parse_statement_list();
        let end = self.peek_span().end;
        self.eat(&TokenKind::EndFunction);
        Some(FunctionDecl {
            name,
            return_type,
            span: Span { start, end },
            var_sections,
            body,
        })
    }

    pub(super) fn parse_data_block(&mut self) -> Option<DataBlockDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume DATA_BLOCK
        let name = self.expect_ident();
        self.skip_block_metadata();
        let var_sections = self.parse_var_sections();
        let end = self.peek_span().end;
        self.eat(&TokenKind::Begin);
        // Skip DB body assignments for now
        self.skip_until(&[TokenKind::EndDataBlock]);
        self.eat(&TokenKind::EndDataBlock);
        Some(DataBlockDecl {
            name,
            span: Span { start, end },
            var_sections,
        })
    }

    pub(super) fn parse_org_block(&mut self) -> Option<OrgBlockDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume ORGANIZATION_BLOCK
        let name = self.expect_ident();
        self.skip_block_metadata();
        let var_sections = self.parse_var_sections();
        self.eat(&TokenKind::Begin);
        let body = self.parse_statement_list();
        let end = self.peek_span().end;
        Some(OrgBlockDecl {
            name,
            span: Span { start, end },
            var_sections,
            body,
        })
    }

    /// Skip optional block metadata: TITLE, VERSION, NON_RETAIN
    fn skip_block_metadata(&mut self) {
        loop {
            match self.peek_kind() {
                TokenKind::Title | TokenKind::Version => {
                    self.advance();
                    if *self.peek_kind() == TokenKind::Eq || *self.peek_kind() == TokenKind::Colon {
                        self.advance();
                    }
                    // skip the value
                    match self.peek_kind() {
                        TokenKind::QuotedIdent(_)
                        | TokenKind::StringLiteral(_)
                        | TokenKind::RealLiteral(_)
                        | TokenKind::IntLiteral(_)
                        | TokenKind::Ident(_) => {
                            self.advance();
                        }
                        _ => {}
                    }
                }
                TokenKind::NonRetain => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
}
