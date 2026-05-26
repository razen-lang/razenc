use crate::lexer::token::{Token, TokenKind, LexError, TokenizationResult};

pub struct Lexer<'a> {
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer { source }
    }

    pub fn tokenize(&self) -> TokenizationResult {
        let chars: Vec<char> = self.source.chars().collect();
        let len = chars.len();
        let mut result = TokenizationResult::new();

        let mut pos = 0usize;
        let mut line = 1usize;
        let mut col = 1usize;

        while pos < len {
            let ch = chars[pos];
            let start_col = col;
            let start_byte = self.byte_offset_of(pos, &chars);

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
                let end_byte = self.byte_offset_of(pos, &chars);
                let value: String = chars[start..pos].iter().collect();
                result.push(Token::new(TokenKind::LineComment, value, line, start_col, (start_byte, end_byte)));
                continue;
            }

            // Block comment /* ... */
            if ch == '/' && pos + 1 < len && chars[pos + 1] == '*' {
                let start = pos;
                let start_byte_bc = start_byte;
                pos += 2;
                col += 2;
                let mut depth: u32 = 1;
                let mut unclosed = false;
                while pos < len && depth > 0 {
                    if chars[pos] == '\n' { line += 1; col = 1; pos += 1; continue; }
                    else if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '*' {
                        depth += 1; pos += 2; col += 2; continue;
                    }
                    else if chars[pos] == '*' && pos + 1 < len && chars[pos + 1] == '/' {
                        depth -= 1; pos += 2; col += 2; continue;
                    }
                    else { if chars[pos] == '\t' { col += 4; } else { col += 1; } pos += 1; }
                }
                if depth > 0 {
                    unclosed = true;
                }
                let end_byte = self.byte_offset_of(pos, &chars);
                if unclosed {
                    result.error(LexError::new(
                        "Unclosed block comment".into(),
                        line, start_col, (start_byte_bc, end_byte),
                    ));
                    // Still emit the partial comment token
                    let value: String = chars[start..pos].iter().collect();
                    result.push(Token::new(TokenKind::BlockComment, value, line, start_col, (start_byte_bc, end_byte)));
                    continue;
                }
                let value: String = chars[start..pos].iter().collect();
                result.push(Token::new(TokenKind::BlockComment, value, line, start_col, (start_byte_bc, end_byte)));
                continue;
            }

            // String literal "..."
            if ch == '"' {
                let start = pos;
                let start_byte_str = start_byte;
                pos += 1; col += 1;
                let mut unclosed = true;
                while pos < len {
                    if chars[pos] == '"' {
                        pos += 1; col += 1;
                        unclosed = false;
                        break;
                    }
                    if chars[pos] == '\n' { line += 1; col = 1; }
                    else if chars[pos] == '\\' && pos + 1 < len {
                        let esc_start = pos;
                        pos += 1; col += 1;
                        let esc_line = line;
                        let esc_col = col;
                        match self.validate_escape(chars[pos], &chars, pos, line, col) {
                            Ok(skip) => {
                                pos += skip;
                                col += skip;
                            }
                            Err(msg) => {
                                let esc_byte = self.byte_offset_of(esc_start, &chars);
                                result.error(LexError::new(
                                    msg, esc_line, esc_col, (esc_byte, esc_byte + 1),
                                ));
                                // Still consume the escape character to avoid infinite loop
                                pos += 1; col += 1;
                            }
                        }
                        continue;
                    }
                    pos += 1; col += 1;
                }
                let end_byte = self.byte_offset_of(pos, &chars);
                if unclosed {
                    result.error(LexError::new(
                        "Unclosed string literal".into(),
                        line, start_col, (start_byte_str, end_byte),
                    ));
                }
                let value: String = chars[start..pos].iter().collect();
                result.push(Token::new(TokenKind::StringValue, value, line, start_col, (start_byte_str, end_byte)));
                continue;
            }

            // Char literal '...'
            if ch == '\'' {
                let start = pos;
                let start_byte_ch = start_byte;
                pos += 1; col += 1;
                let mut unclosed = true;
                while pos < len {
                    if chars[pos] == '\'' {
                        pos += 1; col += 1;
                        unclosed = false;
                        break;
                    }
                    if chars[pos] == '\n' { line += 1; col = 1; }
                    else if chars[pos] == '\\' && pos + 1 < len {
                        let esc_start = pos;
                        pos += 1; col += 1;
                        let esc_line = line;
                        let esc_col = col;
                        match self.validate_escape(chars[pos], &chars, pos, line, col) {
                            Ok(skip) => {
                                pos += skip;
                                col += skip;
                            }
                            Err(msg) => {
                                let esc_byte = self.byte_offset_of(esc_start, &chars);
                                result.error(LexError::new(
                                    msg, esc_line, esc_col, (esc_byte, esc_byte + 1),
                                ));
                                pos += 1; col += 1;
                            }
                        }
                        continue;
                    }
                    if pos + 1 < len && chars[pos] == '\\' && chars[pos + 1] == '\'' {
                        // escaped single quote inside char literal: \'
                        pos += 2; col += 2;
                        continue;
                    }
                    pos += 1; col += 1;
                }
                let end_byte = self.byte_offset_of(pos, &chars);
                if unclosed {
                    result.error(LexError::new(
                        "Unclosed char literal".into(),
                        line, start_col, (start_byte_ch, end_byte),
                    ));
                }
                let value: String = chars[start..pos].iter().collect();
                result.push(Token::new(TokenKind::CharValue, value, line, start_col, (start_byte_ch, end_byte)));
                continue;
            }

            // Number literal
            if ch.is_ascii_digit() {
                let start = pos;
                let start_byte_num = start_byte;

                // 0x, 0X, 0b, 0B, 0o, 0O prefixes
                if ch == '0' && pos + 1 < len {
                    let next = chars[pos + 1];
                    if next == 'x' || next == 'X' {
                        pos += 2; col += 2;
                        while pos < len && (chars[pos].is_ascii_hexdigit() || chars[pos] == '_') { pos += 1; col += 1; }
                        let end_byte = self.byte_offset_of(pos, &chars);
                        let value: String = chars[start..pos].iter().filter(|&c| c != &'_').collect();
                        result.push(Token::new(TokenKind::IntegerValue, value, line, start_col, (start_byte_num, end_byte)));
                        continue;
                    }
                    if next == 'b' || next == 'B' {
                        pos += 2; col += 2;
                        while pos < len && (chars[pos] == '0' || chars[pos] == '1' || chars[pos] == '_') { pos += 1; col += 1; }
                        let end_byte = self.byte_offset_of(pos, &chars);
                        let value: String = chars[start..pos].iter().filter(|&c| c != &'_').collect();
                        result.push(Token::new(TokenKind::IntegerValue, value, line, start_col, (start_byte_num, end_byte)));
                        continue;
                    }
                    if next == 'o' || next == 'O' {
                        pos += 2; col += 2;
                        while pos < len && ((chars[pos] >= '0' && chars[pos] <= '7') || chars[pos] == '_') { pos += 1; col += 1; }
                        let end_byte = self.byte_offset_of(pos, &chars);
                        let value: String = chars[start..pos].iter().filter(|&c| c != &'_').collect();
                        result.push(Token::new(TokenKind::IntegerValue, value, line, start_col, (start_byte_num, end_byte)));
                        continue;
                    }
                }

                // Decimal digits (including _ separators)
                while pos < len && (chars[pos].is_ascii_digit() || chars[pos] == '_') { pos += 1; col += 1; }

                // Float: check for '.' followed by digit (not '..')
                if pos + 1 < len && chars[pos] == '.' && chars[pos + 1].is_ascii_digit() {
                    pos += 1; col += 1;
                    while pos < len && (chars[pos].is_ascii_digit() || chars[pos] == '_') { pos += 1; col += 1; }
                    if pos < len && (chars[pos] == 'e' || chars[pos] == 'E') {
                        pos += 1; col += 1;
                        if pos < len && (chars[pos] == '+' || chars[pos] == '-') { pos += 1; col += 1; }
                        while pos < len && (chars[pos].is_ascii_digit() || chars[pos] == '_') { pos += 1; col += 1; }
                    }
                    let end_byte = self.byte_offset_of(pos, &chars);
                    let value: String = chars[start..pos].iter().filter(|&c| c != &'_').collect();
                    result.push(Token::new(TokenKind::FloatValue, value, line, start_col, (start_byte_num, end_byte)));
                    continue;
                }

                let end_byte = self.byte_offset_of(pos, &chars);
                let value: String = chars[start..pos].iter().filter(|&c| c != &'_').collect();
                result.push(Token::new(TokenKind::IntegerValue, value, line, start_col, (start_byte_num, end_byte)));
                continue;
            }

            // Identifier, keyword, or type keyword
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = pos;
                let start_byte_id = start_byte;
                while pos < len && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                    pos += 1; col += 1;
                }
                let end_byte = self.byte_offset_of(pos, &chars);
                let word: String = chars[start..pos].iter().collect();
                if word == "_" {
                    result.push(Token::new(TokenKind::Underscore, word, line, start_col, (start_byte_id, end_byte)));
                } else if let Some(kw) = keyword_from_str(&word) {
                    result.push(Token::new(kw, word, line, start_col, (start_byte_id, end_byte)));
                } else {
                    result.push(Token::new(TokenKind::Identifier, word, line, start_col, (start_byte_id, end_byte)));
                }
                continue;
            }

            // Multi-character operators (greedy matching)

            if ch == ':' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::ColonEquals, ":=".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == ':' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::ColonColon, "::".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                let end_byte = self.byte_offset_of(pos + 1, &chars);
                result.push(Token::new(TokenKind::Colon, ":".into(), line, start_col, (start_byte, end_byte)));
                pos += 1; col += 1; continue;
            }

            if ch == '~' && pos + 1 < len && chars[pos + 1] == '>' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::TildeArrow, "~>".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '-' {
                if pos + 1 < len && chars[pos + 1] == '>' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::Arrow, "->".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '=' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::MinusEquals, "-=".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                let end_byte = self.byte_offset_of(pos + 1, &chars);
                result.push(Token::new(TokenKind::Minus, "-".into(), line, start_col, (start_byte, end_byte)));
                pos += 1; col += 1; continue;
            }

            if ch == '.' {
                if pos + 1 < len && chars[pos + 1] == '.' {
                    if pos + 2 < len && chars[pos + 2] == '=' {
                        let end_byte = self.byte_offset_of(pos + 3, &chars);
                        result.push(Token::new(TokenKind::DotDotEquals, "..=".into(), line, start_col, (start_byte, end_byte)));
                        pos += 3; col += 3; continue;
                    }
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::DotDot, "..".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '*' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::DotStar, ".*".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                let end_byte = self.byte_offset_of(pos + 1, &chars);
                result.push(Token::new(TokenKind::Dot, ".".into(), line, start_col, (start_byte, end_byte)));
                pos += 1; col += 1; continue;
            }

            if ch == '=' && pos + 1 < len && chars[pos + 1] == '=' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::EqualEqual, "==".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '=' && pos + 1 < len && chars[pos + 1] == '>' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::FatArrow, "=>".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '!' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::NotEqual, "!=".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                let end_byte = self.byte_offset_of(pos + 1, &chars);
                result.push(Token::new(TokenKind::Bang, "!".into(), line, start_col, (start_byte, end_byte)));
                pos += 1; col += 1; continue;
            }

            if ch == '<' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::LessEqual, "<=".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '<' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::ShiftLeft, "<<".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                let end_byte = self.byte_offset_of(pos + 1, &chars);
                result.push(Token::new(TokenKind::Less, "<".into(), line, start_col, (start_byte, end_byte)));
                pos += 1; col += 1; continue;
            }

            if ch == '>' {
                if pos + 1 < len && chars[pos + 1] == '=' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::GreaterEqual, ">=".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                if pos + 1 < len && chars[pos + 1] == '>' {
                    let end_byte = self.byte_offset_of(pos + 2, &chars);
                    result.push(Token::new(TokenKind::ShiftRight, ">>".into(), line, start_col, (start_byte, end_byte)));
                    pos += 2; col += 2; continue;
                }
                let end_byte = self.byte_offset_of(pos + 1, &chars);
                result.push(Token::new(TokenKind::Greater, ">".into(), line, start_col, (start_byte, end_byte)));
                pos += 1; col += 1; continue;
            }

            if ch == '&' && pos + 1 < len && chars[pos + 1] == '&' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::AndAnd, "&&".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '|' && pos + 1 < len && chars[pos + 1] == '|' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::OrOr, "||".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '+' && pos + 1 < len && chars[pos + 1] == '=' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::PlusEquals, "+=".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '*' && pos + 1 < len && chars[pos + 1] == '=' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::StarEquals, "*=".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '/' && pos + 1 < len && chars[pos + 1] == '=' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::SlashEquals, "/=".into(), line, start_col, (start_byte, end_byte)));
                pos += 2; col += 2; continue;
            }

            if ch == '%' && pos + 1 < len && chars[pos + 1] == '=' {
                let end_byte = self.byte_offset_of(pos + 2, &chars);
                result.push(Token::new(TokenKind::PercentEquals, "%=".into(), line, start_col, (start_byte, end_byte)));
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
                    // Unknown character — report error instead of silently skipping
                    let end_byte = start_byte + ch.len_utf8();
                    result.error(LexError::new(
                        format!("Unexpected character '{}'", ch),
                        line, start_col, (start_byte, end_byte),
                    ));
                    pos += 1;
                    col += 1;
                    continue;
                }
            };
            let end_byte = start_byte + ch.len_utf8();
            let value = format!("{}", ch);
            result.push(Token::new(single, value, line, start_col, (start_byte, end_byte)));
            pos += 1;
            col += 1;
        }

        result
    }

    fn validate_escape(&self, esc: char, chars: &[char], pos: usize, _line: usize, _col: usize) -> Result<usize, String> {
        match esc {
            'n' | 't' | 'r' | '0' | '\\' | '"' | '\'' => Ok(0),
            'x' => {
                // \xHH - exactly 2 hex digits
                if pos + 2 >= chars.len() {
                    return Err("Unexpected end of input in \\x escape sequence".into());
                }
                if !chars[pos + 1].is_ascii_hexdigit() || !chars[pos + 2].is_ascii_hexdigit() {
                    return Err("Expected 2 hex digits after \\x".into());
                }
                Ok(2)
            }
            'u' => {
                // \u{XXXX} - 1 to 6 hex digits in braces
                if pos + 1 >= chars.len() || chars[pos + 1] != '{' {
                    return Err("Expected '{' after \\u".into());
                }
                let mut i = pos + 2;
                let mut digit_count = 0usize;
                while i < chars.len() && chars[i].is_ascii_hexdigit() && digit_count < 6 {
                    i += 1;
                    digit_count += 1;
                }
                if digit_count == 0 {
                    return Err("Expected hex digits in \\u{...} escape".into());
                }
                if i >= chars.len() || chars[i] != '}' {
                    return Err("Expected '}' after \\u{hex}".into());
                }
                Ok(i - pos)
            }
            _ => Err(format!("Invalid escape sequence '\\{}'", esc)),
        }
    }

    fn byte_offset_of(&self, char_idx: usize, chars: &[char]) -> usize {
        // chars[..char_idx] gives us the characters up to this point
        // We can sum their byte lengths, or more simply use the source string
        // by finding the byte offset of the character at char_idx.
        if char_idx == 0 {
            return 0;
        }
        // chars[..char_idx].iter().map(|c| c.len_utf8()).sum()
        let mut byte_pos = 0usize;
        for i in 0..char_idx {
            byte_pos += chars[i].len_utf8();
        }
        byte_pos
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
