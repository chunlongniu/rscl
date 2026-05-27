#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords - Blocks
    Function,
    FunctionBlock,
    DataBlock,
    OrganizationBlock,
    EndFunction,
    EndFunctionBlock,
    EndDataBlock,
    // Keywords - Var sections
    Var,
    VarInput,
    VarOutput,
    VarInOut,
    VarTemp,
    Const,
    EndVar,
    EndConst,
    // Keywords - Control flow
    If,
    Then,
    Elsif,
    Else,
    EndIf,
    For,
    To,
    By,
    Do,
    EndFor,
    While,
    EndWhile,
    Repeat,
    Until,
    EndRepeat,
    Case,
    Of,
    EndCase,
    Return,
    Begin,
    // Keywords - Types
    Bool,
    Byte,
    Word,
    DWord,
    Int,
    DInt,
    Real,
    Char,
    StringKw,
    Time,
    Date,
    Tod,
    DateAndTime,
    S5Time,
    Array,
    Struct,
    EndStruct,
    EndType,
    Type,
    // Keywords - Operators
    And,
    Or,
    Xor,
    Not,
    Mod,
    Div,
    // Literals
    True,
    False,
    // Identifiers and literals
    Ident(String),
    IntLiteral(i64),
    RealLiteral(f64),
    StringLiteral(String),
    // PLC address (e.g. IW0, MW10, Q0.1, DB1.DBW0)
    PlcAddress(String),
    // Operators and punctuation
    Assign,    // :=
    Colon,     // :
    Semicolon, // ;
    Comma,     // ,
    Dot,       // .
    DotDot,    // ..
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Power,     // **
    Eq,        // =
    Neq,       // <>
    Lt,        // <
    Gt,        // >
    Le,        // <=
    Ge,        // >=
    Hash,      // #
    // Special
    Error(char),
    Eof,
}

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        let start = self.pos;
        if self.pos >= self.input.len() {
            return Token {
                kind: TokenKind::Eof,
                span: Span { start, end: start },
            };
        }
        let ch = self.input[self.pos] as char;

        // String literal
        if ch == '\'' {
            return self.lex_string(start);
        }

        // Number
        if ch.is_ascii_digit() {
            return self.lex_number(start);
        }

        // Identifier or keyword (or PLC address)
        if ch.is_ascii_alphabetic() || ch == '_' || ch == '#' {
            return self.lex_ident_or_keyword(start);
        }

        // Operators and punctuation
        self.pos += 1;
        let kind = match ch {
            ':' => {
                if self.peek() == Some('=') {
                    self.pos += 1;
                    TokenKind::Assign
                } else {
                    TokenKind::Colon
                }
            }
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => {
                if self.peek() == Some('.') {
                    self.pos += 1;
                    TokenKind::DotDot
                } else {
                    TokenKind::Dot
                }
            }
            '(' => {
                // Check for (* comment - should have been handled but just in case
                TokenKind::LParen
            }
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => {
                if self.peek() == Some('*') {
                    self.pos += 1;
                    TokenKind::Power
                } else {
                    TokenKind::Star
                }
            }
            '/' => TokenKind::Slash,
            '=' => TokenKind::Eq,
            '<' => {
                if self.peek() == Some('>') {
                    self.pos += 1;
                    TokenKind::Neq
                } else if self.peek() == Some('=') {
                    self.pos += 1;
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.pos += 1;
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '#' => TokenKind::Hash,
            _ => TokenKind::Error(ch),
        };
        Token {
            kind,
            span: Span {
                start,
                end: self.pos,
            },
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.pos < self.input.len()
                && (self.input[self.pos] as char).is_ascii_whitespace()
            {
                self.pos += 1;
            }
            if self.pos >= self.input.len() {
                return;
            }
            // Line comment //
            if self.pos + 1 < self.input.len()
                && self.input[self.pos] == b'/'
                && self.input[self.pos + 1] == b'/'
            {
                while self.pos < self.input.len() && self.input[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            // Block comment (* ... *)
            if self.pos + 1 < self.input.len()
                && self.input[self.pos] == b'('
                && self.input[self.pos + 1] == b'*'
            {
                self.pos += 2;
                while self.pos + 1 < self.input.len() {
                    if self.input[self.pos] == b'*' && self.input[self.pos + 1] == b')' {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }
            // Block comment { ... }
            if self.input[self.pos] == b'{' {
                self.pos += 1;
                while self.pos < self.input.len() && self.input[self.pos] != b'}' {
                    self.pos += 1;
                }
                if self.pos < self.input.len() {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos] as char)
        } else {
            None
        }
    }

    fn lex_string(&mut self, start: usize) -> Token {
        self.pos += 1; // skip opening '
        let mut s = String::new();
        while self.pos < self.input.len() && self.input[self.pos] != b'\'' {
            s.push(self.input[self.pos] as char);
            self.pos += 1;
        }
        if self.pos < self.input.len() {
            self.pos += 1; // skip closing '
        }
        Token {
            kind: TokenKind::StringLiteral(s),
            span: Span {
                start,
                end: self.pos,
            },
        }
    }

    fn lex_number(&mut self, start: usize) -> Token {
        // Collect digits
        while self.pos < self.input.len() && (self.input[self.pos] as char).is_ascii_digit() {
            self.pos += 1;
        }
        // Check for real number
        if self.pos < self.input.len() && self.input[self.pos] == b'.' {
            // Peek ahead - if next is also a digit, it's a real; if it's '.', it's a range
            if self.pos + 1 < self.input.len()
                && (self.input[self.pos + 1] as char).is_ascii_digit()
            {
                self.pos += 1; // skip '.'
                while self.pos < self.input.len() && (self.input[self.pos] as char).is_ascii_digit()
                {
                    self.pos += 1;
                }
                // Exponent
                if self.pos < self.input.len()
                    && (self.input[self.pos] == b'e' || self.input[self.pos] == b'E')
                {
                    self.pos += 1;
                    if self.pos < self.input.len()
                        && (self.input[self.pos] == b'+' || self.input[self.pos] == b'-')
                    {
                        self.pos += 1;
                    }
                    while self.pos < self.input.len()
                        && (self.input[self.pos] as char).is_ascii_digit()
                    {
                        self.pos += 1;
                    }
                }
                let text = std::str::from_utf8(&self.input[start..self.pos]).unwrap_or("0");
                let val = text.parse::<f64>().unwrap_or(0.0);
                return Token {
                    kind: TokenKind::RealLiteral(val),
                    span: Span {
                        start,
                        end: self.pos,
                    },
                };
            }
        }
        // Check for hex/octal/binary prefix pattern like 16#FF
        if self.pos < self.input.len() && self.input[self.pos] == b'#' {
            self.pos += 1;
            while self.pos < self.input.len()
                && (self.input[self.pos] as char).is_ascii_alphanumeric()
                || (self.pos < self.input.len() && self.input[self.pos] == b'_')
            {
                self.pos += 1;
            }
            let text = std::str::from_utf8(&self.input[start..self.pos]).unwrap_or("0");
            let val = parse_based_integer(text);
            return Token {
                kind: TokenKind::IntLiteral(val),
                span: Span {
                    start,
                    end: self.pos,
                },
            };
        }
        let text = std::str::from_utf8(&self.input[start..self.pos]).unwrap_or("0");
        let val = text.parse::<i64>().unwrap_or(0);
        Token {
            kind: TokenKind::IntLiteral(val),
            span: Span {
                start,
                end: self.pos,
            },
        }
    }

    fn lex_ident_or_keyword(&mut self, start: usize) -> Token {
        let has_hash = self.input[self.pos] == b'#';
        if has_hash {
            self.pos += 1;
        }
        while self.pos < self.input.len() && (self.input[self.pos] as char).is_ascii_alphanumeric()
            || (self.pos < self.input.len() && self.input[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.input[start..self.pos]).unwrap_or("");
        let upper = text.trim_start_matches('#').to_ascii_uppercase();

        // Check for PLC address pattern: memory prefix + optional size + digits
        if !has_hash {
            if let Some(kind) = try_plc_address(&upper, self.input, &mut self.pos, start) {
                return Token {
                    kind,
                    span: Span {
                        start,
                        end: self.pos,
                    },
                };
            }
        }

        let kind = match upper.as_str() {
            "FUNCTION_BLOCK" => TokenKind::FunctionBlock,
            "FUNCTION" => TokenKind::Function,
            "DATA_BLOCK" => TokenKind::DataBlock,
            "ORGANIZATION_BLOCK" => TokenKind::OrganizationBlock,
            "END_FUNCTION_BLOCK" => TokenKind::EndFunctionBlock,
            "END_FUNCTION" => TokenKind::EndFunction,
            "END_DATA_BLOCK" => TokenKind::EndDataBlock,
            "VAR" => TokenKind::Var,
            "VAR_INPUT" => TokenKind::VarInput,
            "VAR_OUTPUT" => TokenKind::VarOutput,
            "VAR_IN_OUT" => TokenKind::VarInOut,
            "VAR_TEMP" => TokenKind::VarTemp,
            "CONST" => TokenKind::Const,
            "END_VAR" => TokenKind::EndVar,
            "END_CONST" => TokenKind::EndConst,
            "IF" => TokenKind::If,
            "THEN" => TokenKind::Then,
            "ELSIF" => TokenKind::Elsif,
            "ELSE" => TokenKind::Else,
            "END_IF" => TokenKind::EndIf,
            "FOR" => TokenKind::For,
            "TO" => TokenKind::To,
            "BY" => TokenKind::By,
            "DO" => TokenKind::Do,
            "END_FOR" => TokenKind::EndFor,
            "WHILE" => TokenKind::While,
            "END_WHILE" => TokenKind::EndWhile,
            "REPEAT" => TokenKind::Repeat,
            "UNTIL" => TokenKind::Until,
            "END_REPEAT" => TokenKind::EndRepeat,
            "CASE" => TokenKind::Case,
            "OF" => TokenKind::Of,
            "END_CASE" => TokenKind::EndCase,
            "RETURN" => TokenKind::Return,
            "BEGIN" => TokenKind::Begin,
            "BOOL" => TokenKind::Bool,
            "BYTE" => TokenKind::Byte,
            "WORD" => TokenKind::Word,
            "DWORD" => TokenKind::DWord,
            "INT" => TokenKind::Int,
            "DINT" => TokenKind::DInt,
            "REAL" => TokenKind::Real,
            "CHAR" => TokenKind::Char,
            "STRING" => TokenKind::StringKw,
            "TIME" => TokenKind::Time,
            "DATE" => TokenKind::Date,
            "TIME_OF_DAY" | "TOD" => TokenKind::Tod,
            "DATE_AND_TIME" | "DT" => TokenKind::DateAndTime,
            "S5TIME" => TokenKind::S5Time,
            "ARRAY" => TokenKind::Array,
            "STRUCT" => TokenKind::Struct,
            "END_STRUCT" => TokenKind::EndStruct,
            "END_TYPE" => TokenKind::EndType,
            "TYPE" => TokenKind::Type,
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            "XOR" => TokenKind::Xor,
            "NOT" => TokenKind::Not,
            "MOD" => TokenKind::Mod,
            "DIV" => TokenKind::Div,
            "TRUE" => TokenKind::True,
            "FALSE" => TokenKind::False,
            _ => TokenKind::Ident(text.to_string()),
        };
        Token {
            kind,
            span: Span {
                start,
                end: self.pos,
            },
        }
    }
}

/// Try to parse a PLC memory address like IW0, MW10, Q0.1, DB1.DBW0
fn try_plc_address(upper: &str, input: &[u8], pos: &mut usize, _start: usize) -> Option<TokenKind> {
    let prefixes = [
        "PIW", "PQW", "PID", "PQD", "IB", "IW", "ID", "QB", "QW", "QD", "MB", "MW", "MD",
    ];
    let single = ["I", "Q", "M"];

    // Find matching prefix
    let prefix_len = prefixes
        .iter()
        .find(|p| upper.starts_with(*p))
        .map(|p| p.len())
        .or_else(|| {
            single
                .iter()
                .find(|p| upper.starts_with(*p))
                .map(|p| p.len())
        });

    let prefix_len = match prefix_len {
        Some(l) => l,
        None => {
            // Check DB pattern: DB followed by digits
            if upper.starts_with("DB")
                && upper.len() > 2
                && upper[2..].chars().all(|c| c.is_ascii_digit())
            {
                let mut full = upper.to_string();
                if *pos < input.len() && input[*pos] == b'.' {
                    *pos += 1;
                    let dot_start = *pos;
                    while *pos < input.len()
                        && ((input[*pos] as char).is_ascii_alphanumeric() || input[*pos] == b'_')
                    {
                        *pos += 1;
                    }
                    let after_dot = std::str::from_utf8(&input[dot_start..*pos]).unwrap_or("");
                    full.push('.');
                    full.push_str(after_dot);
                }
                return Some(TokenKind::PlcAddress(full));
            }
            return None;
        }
    };

    // After the prefix, the rest must be digits (address)
    if upper.len() > prefix_len && upper[prefix_len..].chars().all(|c| c.is_ascii_digit()) {
        let mut addr = upper.to_string();
        // Check for .bit notation (e.g., Q0.1)
        if *pos < input.len()
            && input[*pos] == b'.'
            && *pos + 1 < input.len()
            && (input[*pos + 1] as char).is_ascii_digit()
        {
            *pos += 1;
            addr.push('.');
            while *pos < input.len() && (input[*pos] as char).is_ascii_digit() {
                addr.push(input[*pos] as char);
                *pos += 1;
            }
        }
        return Some(TokenKind::PlcAddress(addr));
    }

    None
}

fn parse_based_integer(text: &str) -> i64 {
    // Format: base#value, e.g. 16#FF, 2#1010, 8#77
    if let Some(idx) = text.find('#') {
        let base_str = &text[..idx];
        let val_str = &text[idx + 1..].replace('_', "");
        let base = base_str.parse::<u32>().unwrap_or(10);
        i64::from_str_radix(val_str, base).unwrap_or(0)
    } else {
        text.parse::<i64>().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let tokens = Lexer::new("FUNCTION_BLOCK IF THEN END_IF").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::FunctionBlock);
        assert_eq!(tokens[1].kind, TokenKind::If);
        assert_eq!(tokens[2].kind, TokenKind::Then);
        assert_eq!(tokens[3].kind, TokenKind::EndIf);
    }

    #[test]
    fn test_case_insensitive() {
        let tokens = Lexer::new("function_block If THEN end_if").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::FunctionBlock);
        assert_eq!(tokens[1].kind, TokenKind::If);
        assert_eq!(tokens[2].kind, TokenKind::Then);
        assert_eq!(tokens[3].kind, TokenKind::EndIf);
    }

    #[test]
    fn test_operators() {
        let tokens = Lexer::new(":= : ; , . .. ( ) [ ] + - * / ** = <> < > <= >=").tokenize();
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(kinds[0], &TokenKind::Assign);
        assert_eq!(kinds[1], &TokenKind::Colon);
        assert_eq!(kinds[2], &TokenKind::Semicolon);
        assert_eq!(kinds[3], &TokenKind::Comma);
        assert_eq!(kinds[4], &TokenKind::Dot);
        assert_eq!(kinds[5], &TokenKind::DotDot);
    }

    #[test]
    fn test_plc_address() {
        let tokens = Lexer::new("IW0 MW10 Q0.1").tokenize();
        assert!(matches!(&tokens[0].kind, TokenKind::PlcAddress(s) if s == "IW0"));
        assert!(matches!(&tokens[1].kind, TokenKind::PlcAddress(s) if s == "MW10"));
        assert!(matches!(&tokens[2].kind, TokenKind::PlcAddress(s) if s == "Q0.1"));
    }

    #[test]
    fn test_numbers() {
        let tokens = Lexer::new("42 3.14 16#FF").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[1].kind, TokenKind::RealLiteral(3.14));
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral(255));
    }

    #[test]
    fn test_string() {
        let tokens = Lexer::new("'hello world'").tokenize();
        assert_eq!(
            tokens[0].kind,
            TokenKind::StringLiteral("hello world".into())
        );
    }

    #[test]
    fn test_comments_skipped() {
        let tokens = Lexer::new("// comment\nIF (* block *) THEN").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::If);
        assert_eq!(tokens[1].kind, TokenKind::Then);
    }

    #[test]
    fn test_error_token() {
        let tokens = Lexer::new("@ IF").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::Error('@'));
        assert_eq!(tokens[1].kind, TokenKind::If);
    }
}
