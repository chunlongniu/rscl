use crate::types::*;

use super::Parser;

impl Parser {
    pub(super) fn parse_var_sections(&mut self) -> Vec<VarSection> {
        let mut sections = Vec::new();
        loop {
            let kind = match self.peek_kind() {
                TokenKind::Var => VarSectionKind::Var,
                TokenKind::VarInput => VarSectionKind::VarInput,
                TokenKind::VarOutput => VarSectionKind::VarOutput,
                TokenKind::VarInOut => VarSectionKind::VarInOut,
                TokenKind::VarTemp => VarSectionKind::VarTemp,
                TokenKind::Const => VarSectionKind::Const,
                TokenKind::Struct => {
                    // DATA_BLOCK top-level STRUCT ... END_STRUCT
                    let start = self.peek_span().start;
                    self.advance();
                    let mut decls = Vec::new();
                    while !matches!(self.peek_kind(), TokenKind::EndStruct | TokenKind::Eof) {
                        if let Some(d) = self.parse_var_decl() {
                            decls.push(d);
                        } else {
                            self.skip_until(&[TokenKind::Semicolon, TokenKind::EndStruct]);
                            self.eat(&TokenKind::Semicolon);
                        }
                    }
                    let end = self.peek_span().end;
                    self.eat(&TokenKind::EndStruct);
                    self.eat(&TokenKind::Semicolon);
                    sections.push(VarSection {
                        kind: VarSectionKind::Var,
                        decls,
                        span: Span { start, end },
                    });
                    continue;
                }
                _ => break,
            };
            let start = self.peek_span().start;
            self.advance();
            let mut decls = Vec::new();
            while !matches!(
                self.peek_kind(),
                TokenKind::EndVar | TokenKind::EndConst | TokenKind::Eof
            ) {
                if let Some(d) = self.parse_var_decl() {
                    decls.push(d);
                } else {
                    self.skip_until(&[
                        TokenKind::Semicolon,
                        TokenKind::EndVar,
                        TokenKind::EndConst,
                    ]);
                    self.eat(&TokenKind::Semicolon);
                }
            }
            let end = self.peek_span().end;
            if kind == VarSectionKind::Const {
                self.eat(&TokenKind::EndConst);
            } else {
                self.eat(&TokenKind::EndVar);
            }
            sections.push(VarSection {
                kind,
                decls,
                span: Span { start, end },
            });
        }
        sections
    }

    pub(super) fn parse_var_decl(&mut self) -> Option<VarDecl> {
        let start = self.peek_span().start;
        let mut names = Vec::new();
        match self.peek_kind() {
            TokenKind::Ident(_) => {
                names.push(self.expect_ident());
            }
            _ => return None,
        }
        while self.eat(&TokenKind::Comma) {
            names.push(self.expect_ident());
        }
        if !self.expect(&TokenKind::Colon) {
            return None;
        }
        let type_spec = self.parse_type_spec();
        let init_value = if self.eat(&TokenKind::Assign) {
            Some(self.parse_expression())
        } else {
            None
        };
        let end = self.peek_span().end;
        self.expect(&TokenKind::Semicolon);
        Some(VarDecl {
            names,
            type_spec,
            init_value,
            span: Span { start, end },
        })
    }

    pub(super) fn parse_type_spec(&mut self) -> TypeSpec {
        match self.peek_kind().clone() {
            TokenKind::Bool => { self.advance(); TypeSpec::Elementary(ElementaryType::Bool) }
            TokenKind::Byte => { self.advance(); TypeSpec::Elementary(ElementaryType::Byte) }
            TokenKind::Word => { self.advance(); TypeSpec::Elementary(ElementaryType::Word) }
            TokenKind::DWord => { self.advance(); TypeSpec::Elementary(ElementaryType::DWord) }
            TokenKind::Int => { self.advance(); TypeSpec::Elementary(ElementaryType::Int) }
            TokenKind::DInt => { self.advance(); TypeSpec::Elementary(ElementaryType::DInt) }
            TokenKind::Real => { self.advance(); TypeSpec::Elementary(ElementaryType::Real) }
            TokenKind::Char => { self.advance(); TypeSpec::Elementary(ElementaryType::Char) }
            TokenKind::Time => { self.advance(); TypeSpec::Elementary(ElementaryType::Time) }
            TokenKind::Date => { self.advance(); TypeSpec::Elementary(ElementaryType::Date) }
            TokenKind::Tod => { self.advance(); TypeSpec::Elementary(ElementaryType::Tod) }
            TokenKind::DateAndTime => { self.advance(); TypeSpec::Elementary(ElementaryType::DateAndTime) }
            TokenKind::S5Time => { self.advance(); TypeSpec::Elementary(ElementaryType::S5Time) }
            TokenKind::StringKw => {
                self.advance();
                let len = if self.eat(&TokenKind::LBracket) {
                    let e = self.parse_expression();
                    self.expect(&TokenKind::RBracket);
                    Some(e)
                } else {
                    None
                };
                TypeSpec::StringType(len)
            }
            TokenKind::Array => {
                self.advance();
                self.expect(&TokenKind::LBracket);
                let mut ranges = Vec::new();
                loop {
                    let lo = self.parse_expression();
                    self.expect(&TokenKind::DotDot);
                    let hi = self.parse_expression();
                    ranges.push((lo, hi));
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket);
                self.expect(&TokenKind::Of);
                let elem = self.parse_type_spec();
                TypeSpec::Array {
                    ranges,
                    element_type: Box::new(elem),
                }
            }
            TokenKind::Struct => {
                self.advance();
                let mut fields = Vec::new();
                while !matches!(self.peek_kind(), TokenKind::EndStruct | TokenKind::Eof) {
                    if let Some(d) = self.parse_var_decl() {
                        fields.push(d);
                    } else {
                        self.skip_until(&[TokenKind::Semicolon, TokenKind::EndStruct]);
                        self.eat(&TokenKind::Semicolon);
                    }
                }
                self.eat(&TokenKind::EndStruct);
                TypeSpec::Struct(fields)
            }
            TokenKind::Ident(name) => {
                self.advance();
                TypeSpec::UserDefined(name)
            }
            _ => {
                self.error("expected type specification");
                TypeSpec::Elementary(ElementaryType::Int)
            }
        }
    }
}
