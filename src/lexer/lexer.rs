use crate::lexer::token::{Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer { source }
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let chars: Vec<char> = self.source.chars().collect();
        let len = chars.len();
        let mut tokens = Vec::new();

        let mut pos = 0;
        let mut line = 1;
        let mut col = 1;

        while pos < len {
            let ch = chars[pos];
            let start_col = col;

            // Whitespace
            if ch == ' ' || ch == '\t' || ch == '\r' {
                pos += 1;
                col += 1;
                continue;
            }
            if ch == '\n' {
                pos += 1;
                line += 1;
                col = 1;
                continue;
            }

            // Line comment //
            if ch == '/' && pos + 1 < len && chars[pos + 1] == '/' {
                let start = pos;
                pos += 2;
                col += 2;
                while pos < len && chars[pos] != '\n' {
                    if chars[pos] == '\t' { col += 4; } else { col += 1; }
                    pos += 1;
                }
                let value: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(TokenKind::LineComment, value, line, start_col));
                continue;
            }

            // Block comment /* ... */
            if ch == '/' && pos + 1 < len && chars[pos + 1] == '*' {
                let start = pos;
                pos += 2;
                col += 2;
                let mut depth: u32 = 1;
                while pos < len && depth > 0 {
                    if chars[pos] == '\n' { line += 1; col = 1; }
                    else if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '*' {
                        depth += 1; pos += 2; col += 2; continue;
                    }
                    else if chars[pos] == '*' && pos + 1 < len && chars[pos + 1] == '/' {
                        depth -= 1; pos += 2; col += 2; continue;
                    }
                    else { if chars[pos] == '\t' { col += 4; } else { col += 1; } pos += 1; }
                }
                let value: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(TokenKind::BlockComment, value, line, start_col));
                continue;
            }

            // String literal "..."
            if ch == '"' {
                let start = pos;
                pos += 1; col += 1;
                while pos < len && chars[pos] != '"' {
                    if chars[pos] == '\n' { line += 1; col = 1; }
                    else if chars[pos] == '\\' && pos + 1 < len { pos += 1; col += 1; }
                    pos += 1; col += 1;
                }
                if pos < len { pos += 1; col += 1; }
                let value: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(TokenKind::StringValue, value, line, start_col));
                continue;
            }

            // Char literal '...'
            if ch == '\'' {
                let start = pos;
                pos += 1; col += 1;
                while pos < len && chars[pos] != '\'' {
                    if chars[pos] == '\n' { line += 1; col = 1; }
                    else if chars[pos] == '\\' && pos + 1 < len { pos += 1; col += 1; }
                    pos += 1; col += 1;
                }
                if pos < len { pos += 1; col += 1; }
                let value: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(TokenKind::CharValue, value, line, start_col));
                continue;
            }

            // Number literal
            if ch.is_ascii_digit() {
                let start = pos;

                // 0x, 0b, 0o prefixes
                if ch == '0' && pos + 1 < len {
                    let next = chars[pos + 1];
                    if next == 'x' || next == 'X' {
                        pos += 2; col += 2;
                        while pos < len && chars[pos].is_ascii_hexdigit() { pos += 1; col += 1; }
                        let value: String = chars[start..pos].iter().collect();
                        tokens.push(Token::new(TokenKind::IntegerValue, value, line, start_col));
                        continue;
                    }
                    if next == 'b' || next == 'B' {
                        pos += 2; col += 2;
                        while pos < len && (chars[pos] == '0' || chars[pos] == '1') { pos += 1; col += 1; }
                        let value: String = chars[start..pos].iter().collect();
                        tokens.push(Token::new(TokenKind::IntegerValue, value, line, start_col));
                        continue;
                    }
                    if next == 'o' || next == 'O' {
                        pos += 2; col += 2;
                        while pos < len && chars[pos] >= '0' && chars[pos] <= '7' { pos += 1; col += 1; }
                        let value: String = chars[start..pos].iter().collect();
                        tokens.push(Token::new(TokenKind::IntegerValue, value, line, start_col));
                        continue;
                    }
                }

                // Decimal digits
                while pos < len && chars[pos].is_ascii_digit() { pos += 1; col += 1; }

                // Float: check for '.' followed by digit (not '..')
                if pos + 1 < len && chars[pos] == '.' && chars[pos + 1].is_ascii_digit() {
                    pos += 1; col += 1;
                    while pos < len && chars[pos].is_ascii_digit() { pos += 1; col += 1; }
                    if pos < len && (chars[pos] == 'e' || chars[pos] == 'E') {
                        pos += 1; col += 1;
                        if pos < len && (chars[pos] == '+' || chars[pos] == '-') { pos += 1; col += 1; }
                        while pos < len && chars[pos].is_ascii_digit() { pos += 1; col += 1; }
                    }
                    let value: String = chars[start..pos].iter().collect();
                    tokens.push(Token::new(TokenKind::FloatValue, value, line, start_col));
                    continue;
                }

                let value: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(TokenKind::IntegerValue, value, line, start_col));
                continue;
            }

            // Identifier, keyword, or type keyword
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = pos;
                while pos < len && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                    pos += 1; col += 1;
                }
                let word: String = chars[start..pos].iter().collect();
                if word == "_" {
                    tokens.push(Token::new(TokenKind::Underscore, word, line, start_col));
                } else if let Some(kw) = keyword_from_str(&word) {
                    tokens.push(Token::new(kw, word, line, start_col));
                } else {
                    tokens.push(Token::new(TokenKind::Identifier, word, line, start_col));
                }
                continue;
            }

            // Multi-character operators (greedy matching)

            if ch == ':' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    tokens.push(Token::new(TokenKind::ColonEquals, ":=".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == ':' {
                    tokens.push(Token::new(TokenKind::ColonColon, "::".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                tokens.push(Token::new(TokenKind::Colon, ":".into(), line, start_col));
                pos += 1; col += 1; continue;
            }

            if ch == '~' && pos + 1 < len && chars[pos + 1] == '>' {
                tokens.push(Token::new(TokenKind::TildeArrow, "~>".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '-' {
                if pos + 1 < len && chars[pos + 1] == '>' {
                    tokens.push(Token::new(TokenKind::Arrow, "->".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '=' {
                    tokens.push(Token::new(TokenKind::MinusEquals, "-=".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                tokens.push(Token::new(TokenKind::Minus, "-".into(), line, start_col));
                pos += 1; col += 1; continue;
            }

            if ch == '.' {
                if pos + 1 < len && chars[pos + 1] == '.' {
                    if pos + 2 < len && chars[pos + 2] == '=' {
                        tokens.push(Token::new(TokenKind::DotDotEquals, "..=".into(), line, start_col));
                        pos += 3; col += 3; continue;
                    }
                    tokens.push(Token::new(TokenKind::DotDot, "..".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '*' {
                    tokens.push(Token::new(TokenKind::DotStar, ".*".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                tokens.push(Token::new(TokenKind::Dot, ".".into(), line, start_col));
                pos += 1; col += 1; continue;
            }

            if ch == '=' && pos + 1 < len && chars[pos + 1] == '=' {
                tokens.push(Token::new(TokenKind::EqualEqual, "==".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '=' && pos + 1 < len && chars[pos + 1] == '>' {
                tokens.push(Token::new(TokenKind::FatArrow, "=>".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '!' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    tokens.push(Token::new(TokenKind::NotEqual, "!=".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                tokens.push(Token::new(TokenKind::Bang, "!".into(), line, start_col));
                pos += 1; col += 1; continue;
            }

            if ch == '<' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    tokens.push(Token::new(TokenKind::LessEqual, "<=".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '<' {
                    tokens.push(Token::new(TokenKind::ShiftLeft, "<<".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                tokens.push(Token::new(TokenKind::Less, "<".into(), line, start_col));
                pos += 1; col += 1; continue;
            }

            if ch == '>' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    tokens.push(Token::new(TokenKind::GreaterEqual, ">=".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '>' {
                    tokens.push(Token::new(TokenKind::ShiftRight, ">>".into(), line, start_col));
                    pos += 2; col += 2; continue;
                }
                tokens.push(Token::new(TokenKind::Greater, ">".into(), line, start_col));
                pos += 1; col += 1; continue;
            }

            if ch == '&' && pos + 1 < len && chars[pos + 1] == '&' {
                tokens.push(Token::new(TokenKind::AndAnd, "&&".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '|' && pos + 1 < len && chars[pos + 1] == '|' {
                tokens.push(Token::new(TokenKind::OrOr, "||".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '+' && pos + 1 < len && chars[pos + 1] == '=' {
                tokens.push(Token::new(TokenKind::PlusEquals, "+=".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '*' && pos + 1 < len && chars[pos + 1] == '=' {
                tokens.push(Token::new(TokenKind::StarEquals, "*=".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '/' && pos + 1 < len && chars[pos + 1] == '=' {
                tokens.push(Token::new(TokenKind::SlashEquals, "/=".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            if ch == '%' && pos + 1 < len && chars[pos + 1] == '=' {
                tokens.push(Token::new(TokenKind::PercentEquals, "%=".into(), line, start_col));
                pos += 2; col += 2; continue;
            }

            // Single-character tokens
            let single = match ch {
                '+' => TokenKind::Plus,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '%' => TokenKind::Percent,
                '&' => TokenKind::And,
                '|' => TokenKind::Pipe,
                '^' => TokenKind::Caret,
                '~' => TokenKind::Tilde,
                '=' => TokenKind::Assign,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '@' => TokenKind::At,
                '?' => TokenKind::QuestionMark,
                '(' => TokenKind::LeftParen,
                ')' => TokenKind::RightParen,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                '[' => TokenKind::LeftBracket,
                ']' => TokenKind::RightBracket,
                _ => {
                    // Unknown character — skip
                    pos += 1; col += 1;
                    continue;
                }
            };
            let value = format!("{}", ch);
            tokens.push(Token::new(single, value, line, start_col));
            pos += 1;
            col += 1;
        }

        tokens
    }
}

fn keyword_from_str(s: &str) -> Option<TokenKind> {
    match s {
        // Control Flow
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "match" => Some(TokenKind::Match),
        "loop" => Some(TokenKind::Loop),
        "stop" => Some(TokenKind::Stop),
        "next" => Some(TokenKind::Next),
        "ret" => Some(TokenKind::Ret),

        // Memory & Error
        "mut" => Some(TokenKind::Mut),
        "try" => Some(TokenKind::Try),
        "catch" => Some(TokenKind::Catch),
        "defer" => Some(TokenKind::Defer),

        // Modules & Visibility
        "mod" => Some(TokenKind::Mod),
        "use" => Some(TokenKind::Use),
        "pub" => Some(TokenKind::Pub),
        "ext" => Some(TokenKind::Ext),

        // Definitions
        "fn" => Some(TokenKind::Fn),
        "struct" => Some(TokenKind::Struct),
        "union" => Some(TokenKind::Union),
        "enum" => Some(TokenKind::Enum),
        "error" => Some(TokenKind::ErrorKw),
        "behave" => Some(TokenKind::Behave),
        "type" => Some(TokenKind::Type),
        "test" => Some(TokenKind::Test),

        // Literals
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "nil" => Some(TokenKind::Nil),

        // Type Keywords
        "void" => Some(TokenKind::Void),
        "bool" => Some(TokenKind::Bool),
        "char" => Some(TokenKind::Char),
        "str" => Some(TokenKind::Str),
        "noret" => Some(TokenKind::Noret),
        "anytype" => Some(TokenKind::AnyType),
        "int" => Some(TokenKind::Int),
        "uint" => Some(TokenKind::Uint),
        "float" => Some(TokenKind::Float),
        "isize" => Some(TokenKind::Isize),
        "usize" => Some(TokenKind::Usize),

        "i1" => Some(TokenKind::I1),
        "i2" => Some(TokenKind::I2),
        "i4" => Some(TokenKind::I4),
        "i8" => Some(TokenKind::I8),
        "i16" => Some(TokenKind::I16),
        "i32" => Some(TokenKind::I32),
        "i64" => Some(TokenKind::I64),
        "i128" => Some(TokenKind::I128),

        "u1" => Some(TokenKind::U1),
        "u2" => Some(TokenKind::U2),
        "u4" => Some(TokenKind::U4),
        "u8" => Some(TokenKind::U8),
        "u16" => Some(TokenKind::U16),
        "u32" => Some(TokenKind::U32),
        "u64" => Some(TokenKind::U64),
        "u128" => Some(TokenKind::U128),

        "f8" => Some(TokenKind::F8),
        "f16" => Some(TokenKind::F16),
        "f32" => Some(TokenKind::F32),
        "f64" => Some(TokenKind::F64),
        "f128" => Some(TokenKind::F128),

        _ => None,
    }
}
