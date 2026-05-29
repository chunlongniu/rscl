use crate::types::*;

use super::Parser;

impl Parser {
    pub(super) fn parse_statement_list(&mut self) -> Vec<Statement> {
        let mut stmts = Vec::new();
        loop {
            match self.peek_kind() {
                TokenKind::EndFunctionBlock
                | TokenKind::EndFunction
                | TokenKind::EndIf
                | TokenKind::Elsif
                | TokenKind::Else
                | TokenKind::EndFor
                | TokenKind::EndWhile
                | TokenKind::EndRepeat
                | TokenKind::Until
                | TokenKind::EndCase
                | TokenKind::Eof => break,
                TokenKind::Semicolon => {
                    self.advance();
                    continue;
                }
                _ => {}
            }
            match self.parse_statement() {
                Some(s) => stmts.push(s),
                None => {
                    self.skip_until(&[
                        TokenKind::Semicolon,
                        TokenKind::EndIf,
                        TokenKind::EndFor,
                        TokenKind::EndWhile,
                        TokenKind::EndRepeat,
                        TokenKind::EndCase,
                        TokenKind::EndFunctionBlock,
                        TokenKind::EndFunction,
                    ]);
                    self.eat(&TokenKind::Semicolon);
                }
            }
        }
        stmts
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        let start = self.peek_span().start;
        match self.peek_kind().clone() {
            TokenKind::If => Some(self.parse_if_statement()),
            TokenKind::For => Some(self.parse_for_statement()),
            TokenKind::While => Some(self.parse_while_statement()),
            TokenKind::Repeat => Some(self.parse_repeat_statement()),
            TokenKind::Case => Some(self.parse_case_statement()),
            TokenKind::Return => {
                self.advance();
                let value = if !matches!(self.peek_kind(), TokenKind::Semicolon | TokenKind::Eof) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                let end = self.peek_span().end;
                self.eat(&TokenKind::Semicolon);
                Some(Statement::Return {
                    value,
                    span: Span { start, end },
                })
            }
            TokenKind::Ident(_) | TokenKind::PlcAddress(_) | TokenKind::Hash => {
                let expr = self.parse_expression();
                if self.eat(&TokenKind::Assign) {
                    let value = self.parse_expression();
                    let end = self.peek_span().end;
                    self.eat(&TokenKind::Semicolon);
                    Some(Statement::Assignment {
                        target: expr,
                        value,
                        span: Span { start, end },
                    })
                } else if self.eat(&TokenKind::LParen) {
                    let args = self.parse_call_args();
                    self.expect(&TokenKind::RParen);
                    let end = self.peek_span().end;
                    self.eat(&TokenKind::Semicolon);
                    Some(Statement::Call {
                        target: expr,
                        args,
                        span: Span { start, end },
                    })
                } else {
                    let end = self.peek_span().end;
                    self.eat(&TokenKind::Semicolon);
                    Some(Statement::Call {
                        target: expr,
                        args: vec![],
                        span: Span { start, end },
                    })
                }
            }
            _ => {
                self.error("expected statement");
                None
            }
        }
    }

    fn parse_if_statement(&mut self) -> Statement {
        let start = self.peek_span().start;
        self.advance(); // IF
        let condition = self.parse_expression();
        self.expect(&TokenKind::Then);
        let then_body = self.parse_statement_list();
        let mut elsif_clauses = Vec::new();
        while self.eat(&TokenKind::Elsif) {
            let cond = self.parse_expression();
            self.expect(&TokenKind::Then);
            let body = self.parse_statement_list();
            elsif_clauses.push((cond, body));
        }
        let else_body = if self.eat(&TokenKind::Else) {
            Some(self.parse_statement_list())
        } else {
            None
        };
        let end = self.peek_span().end;
        self.expect(&TokenKind::EndIf);
        self.eat(&TokenKind::Semicolon);
        Statement::If(IfStatement {
            condition,
            then_body,
            elsif_clauses,
            else_body,
            span: Span { start, end },
        })
    }

    fn parse_for_statement(&mut self) -> Statement {
        let start = self.peek_span().start;
        self.advance(); // FOR
        let var = self.expect_ident();
        self.expect(&TokenKind::Assign);
        let from = self.parse_expression();
        self.expect(&TokenKind::To);
        let to = self.parse_expression();
        let by = if self.eat(&TokenKind::By) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(&TokenKind::Do);
        let body = self.parse_statement_list();
        let end = self.peek_span().end;
        self.expect(&TokenKind::EndFor);
        self.eat(&TokenKind::Semicolon);
        Statement::For(ForStatement {
            var,
            from,
            to,
            by,
            body,
            span: Span { start, end },
        })
    }

    fn parse_while_statement(&mut self) -> Statement {
        let start = self.peek_span().start;
        self.advance(); // WHILE
        let condition = self.parse_expression();
        self.expect(&TokenKind::Do);
        let body = self.parse_statement_list();
        let end = self.peek_span().end;
        self.expect(&TokenKind::EndWhile);
        self.eat(&TokenKind::Semicolon);
        Statement::While(WhileStatement {
            condition,
            body,
            span: Span { start, end },
        })
    }

    fn parse_repeat_statement(&mut self) -> Statement {
        let start = self.peek_span().start;
        self.advance(); // REPEAT
        let body = self.parse_statement_list();
        self.expect(&TokenKind::Until);
        let condition = self.parse_expression();
        let end = self.peek_span().end;
        self.expect(&TokenKind::EndRepeat);
        self.eat(&TokenKind::Semicolon);
        Statement::Repeat(RepeatStatement {
            body,
            condition,
            span: Span { start, end },
        })
    }

    fn parse_case_statement(&mut self) -> Statement {
        let start = self.peek_span().start;
        self.advance(); // CASE
        let expr = self.parse_expression();
        self.expect(&TokenKind::Of);
        let mut branches = Vec::new();
        let mut else_body = None;
        loop {
            match self.peek_kind() {
                TokenKind::EndCase | TokenKind::Eof => break,
                TokenKind::Else => {
                    self.advance();
                    else_body = Some(self.parse_statement_list());
                }
                _ => {
                    let mut values = vec![self.parse_expression()];
                    while self.eat(&TokenKind::Comma) {
                        values.push(self.parse_expression());
                    }
                    self.expect(&TokenKind::Colon);
                    let body = self.parse_statement_list();
                    branches.push(CaseBranch { values, body });
                }
            }
        }
        let end = self.peek_span().end;
        self.expect(&TokenKind::EndCase);
        self.eat(&TokenKind::Semicolon);
        Statement::Case(CaseStatement {
            expr,
            branches,
            else_body,
            span: Span { start, end },
        })
    }

    pub(super) fn parse_call_args(&mut self) -> Vec<CallArg> {
        let mut args = Vec::new();
        if matches!(self.peek_kind(), TokenKind::RParen) {
            return args;
        }
        loop {
            let arg = if let TokenKind::Ident(name) = self.peek_kind().clone() {
                let saved = self.pos;
                self.advance();
                if self.eat(&TokenKind::Assign) {
                    let value = self.parse_expression();
                    CallArg {
                        name: Some(name),
                        value,
                    }
                } else {
                    self.pos = saved;
                    let value = self.parse_expression();
                    CallArg { name: None, value }
                }
            } else {
                let value = self.parse_expression();
                CallArg { name: None, value }
            };
            args.push(arg);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        args
    }
}
