use crate::ast::*;
use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

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

    // --- Block parsing ---

    fn parse_function_block(&mut self) -> Option<FunctionBlockDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume FUNCTION_BLOCK
        let name = self.expect_ident();
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

    fn parse_function(&mut self) -> Option<FunctionDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume FUNCTION
        let name = self.expect_ident();
        let return_type = if self.eat(&TokenKind::Colon) {
            Some(self.parse_type_spec())
        } else {
            None
        };
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

    fn parse_data_block(&mut self) -> Option<DataBlockDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume DATA_BLOCK
        let name = self.expect_ident();
        // Skip optional metadata (TITLE, VERSION, NON_RETAIN)
        loop {
            match self.peek_kind() {
                TokenKind::Title | TokenKind::Version => {
                    self.advance();
                    if *self.peek_kind() == TokenKind::Eq {
                        self.advance();
                    }
                    // skip the value
                    match self.peek_kind() {
                        TokenKind::QuotedIdent(_) | TokenKind::StringLiteral(_)
                        | TokenKind::RealLiteral(_) | TokenKind::Ident(_) => { self.advance(); }
                        _ => {}
                    }
                }
                TokenKind::NonRetain => { self.advance(); }
                _ => break,
            }
        }
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

    fn parse_org_block(&mut self) -> Option<OrgBlockDecl> {
        let start = self.peek_span().start;
        self.advance(); // consume ORGANIZATION_BLOCK
        let name = self.expect_ident();
        let var_sections = self.parse_var_sections();
        self.eat(&TokenKind::Begin);
        let body = self.parse_statement_list();
        let end = self.peek_span().end;
        // No standard END keyword for OB in TIA Portal, but handle if present
        Some(OrgBlockDecl {
            name,
            span: Span { start, end },
            var_sections,
            body,
        })
    }
}

// --- Variable section parsing ---
impl Parser {
    fn parse_var_sections(&mut self) -> Vec<VarSection> {
        let mut sections = Vec::new();
        loop {
            let kind = match self.peek_kind() {
                TokenKind::Var => VarSectionKind::Var,
                TokenKind::VarInput => VarSectionKind::VarInput,
                TokenKind::VarOutput => VarSectionKind::VarOutput,
                TokenKind::VarInOut => VarSectionKind::VarInOut,
                TokenKind::VarTemp => VarSectionKind::VarTemp,
                TokenKind::Const => VarSectionKind::Const,
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

    fn parse_var_decl(&mut self) -> Option<VarDecl> {
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

    fn parse_type_spec(&mut self) -> TypeSpec {
        match self.peek_kind().clone() {
            TokenKind::Bool => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Bool)
            }
            TokenKind::Byte => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Byte)
            }
            TokenKind::Word => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Word)
            }
            TokenKind::DWord => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::DWord)
            }
            TokenKind::Int => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Int)
            }
            TokenKind::DInt => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::DInt)
            }
            TokenKind::Real => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Real)
            }
            TokenKind::Char => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Char)
            }
            TokenKind::Time => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Time)
            }
            TokenKind::Date => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Date)
            }
            TokenKind::Tod => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::Tod)
            }
            TokenKind::DateAndTime => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::DateAndTime)
            }
            TokenKind::S5Time => {
                self.advance();
                TypeSpec::Elementary(ElementaryType::S5Time)
            }
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

// --- Statement parsing ---
impl Parser {
    fn parse_statement_list(&mut self) -> Vec<Statement> {
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
                    // Function call with named params
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

    fn parse_call_args(&mut self) -> Vec<CallArg> {
        let mut args = Vec::new();
        if matches!(self.peek_kind(), TokenKind::RParen) {
            return args;
        }
        loop {
            // Check for named arg: ident := expr
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

// --- Expression parsing (precedence climbing) ---
impl Parser {
    pub fn parse_expression(&mut self) -> Expression {
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
}
