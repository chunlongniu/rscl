use crate::types::*;

use super::Parser;

impl Parser {
    pub(super) fn parse_expression(&mut self) -> Expression {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Expression {
        let mut left = self.parse_xor_expr();
        while matches!(self.peek_kind(), TokenKind::Or) {
            let start = left.span().start;
            self.advance();
            let right = self.parse_xor_expr();
            let end = right.span().end;
            left = Expression::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span: Span { start, end },
            };
        }
        left
    }

    fn parse_xor_expr(&mut self) -> Expression {
        let mut left = self.parse_and_expr();
        while matches!(self.peek_kind(), TokenKind::Xor) {
            let start = left.span().start;
            self.advance();
            let right = self.parse_and_expr();
            let end = right.span().end;
            left = Expression::Binary {
                op: BinOp::Xor,
                left: Box::new(left),
                right: Box::new(right),
                span: Span { start, end },
            };
        }
        left
    }

    fn parse_and_expr(&mut self) -> Expression {
        let mut left = self.parse_comparison();
        while matches!(self.peek_kind(), TokenKind::And) {
            let start = left.span().start;
            self.advance();
            let right = self.parse_comparison();
            let end = right.span().end;
            left = Expression::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span: Span { start, end },
            };
        }
        left
    }

    fn parse_comparison(&mut self) -> Expression {
        let mut left = self.parse_additive();
        loop {
            let op = match self.peek_kind() {
                TokenKind::Eq => BinOp::Eq,
                TokenKind::Neq => BinOp::Neq,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Ge => BinOp::Ge,
                _ => break,
            };
            let start = left.span().start;
            self.advance();
            let right = self.parse_additive();
            let end = right.span().end;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span { start, end },
            };
        }
        left
    }

    fn parse_additive(&mut self) -> Expression {
        let mut left = self.parse_multiplicative();
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            let start = left.span().start;
            self.advance();
            let right = self.parse_multiplicative();
            let end = right.span().end;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span { start, end },
            };
        }
        left
    }

    fn parse_multiplicative(&mut self) -> Expression {
        let mut left = self.parse_power();
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Mod => BinOp::Mod,
                TokenKind::Div => BinOp::IntDiv,
                _ => break,
            };
            let start = left.span().start;
            self.advance();
            let right = self.parse_power();
            let end = right.span().end;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span { start, end },
            };
        }
        left
    }

    fn parse_power(&mut self) -> Expression {
        let left = self.parse_unary();
        if matches!(self.peek_kind(), TokenKind::Power) {
            let start = left.span().start;
            self.advance();
            let right = self.parse_unary();
            let end = right.span().end;
            Expression::Binary {
                op: BinOp::Power,
                left: Box::new(left),
                right: Box::new(right),
                span: Span { start, end },
            }
        } else {
            left
        }
    }

    fn parse_unary(&mut self) -> Expression {
        match self.peek_kind().clone() {
            TokenKind::Minus => {
                let start = self.peek_span().start;
                self.advance();
                let operand = self.parse_unary();
                let end = operand.span().end;
                Expression::Unary {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: Span { start, end },
                }
            }
            TokenKind::Not => {
                let start = self.peek_span().start;
                self.advance();
                let operand = self.parse_unary();
                let end = operand.span().end;
                Expression::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span: Span { start, end },
                }
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Expression {
        let mut expr = self.parse_primary();
        loop {
            match self.peek_kind() {
                TokenKind::Dot => {
                    let start = expr.span().start;
                    self.advance();
                    let field = self.expect_ident();
                    let end = self.peek_span().start;
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field,
                        span: Span { start, end },
                    };
                }
                TokenKind::LBracket => {
                    let start = expr.span().start;
                    self.advance();
                    let mut indices = vec![self.parse_expression()];
                    while self.eat(&TokenKind::Comma) {
                        indices.push(self.parse_expression());
                    }
                    let end = self.peek_span().end;
                    self.expect(&TokenKind::RBracket);
                    expr = Expression::ArrayAccess {
                        array: Box::new(expr),
                        indices,
                        span: Span { start, end },
                    };
                }
                TokenKind::LParen => {
                    let start = expr.span().start;
                    self.advance();
                    let args = self.parse_call_args();
                    let end = self.peek_span().end;
                    self.expect(&TokenKind::RParen);
                    expr = Expression::FunctionCall {
                        name: Box::new(expr),
                        args,
                        span: Span { start, end },
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expression {
        let span = self.peek_span();
        match self.peek_kind().clone() {
            TokenKind::IntLiteral(v) => {
                self.advance();
                Expression::Literal(Literal::Int(v), span)
            }
            TokenKind::RealLiteral(v) => {
                self.advance();
                Expression::Literal(Literal::Real(v), span)
            }
            TokenKind::StringLiteral(v) => {
                self.advance();
                Expression::Literal(Literal::String(v), span)
            }
            TokenKind::True => {
                self.advance();
                Expression::Literal(Literal::Bool(true), span)
            }
            TokenKind::False => {
                self.advance();
                Expression::Literal(Literal::Bool(false), span)
            }
            TokenKind::Ident(name) => {
                self.advance();
                Expression::Ident(name, span)
            }
            TokenKind::PlcAddress(addr) => {
                self.advance();
                Expression::PlcAddress(addr, span)
            }
            TokenKind::Hash => {
                self.advance();
                let name = self.expect_ident();
                let end = self.peek_span().start;
                Expression::Ident(
                    format!("#{}", name),
                    Span {
                        start: span.start,
                        end,
                    },
                )
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(&TokenKind::RParen);
                expr
            }
            _ => {
                self.error("expected expression");
                self.advance();
                Expression::Literal(Literal::Int(0), span)
            }
        }
    }
}
