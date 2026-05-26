use std::fmt;

pub type Span = (usize, usize);

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Control Flow
    If, Else, Match, Loop, Stop, Next, Ret,

    // Memory & Error
    Mut, Try, Catch, Defer,

    // Modules & Visibility
    Mod, Use, Pub, Ext,

    // Definitions
    Fn, Struct, Union, Enum, ErrorKw, Behave, Type, Test,

    // Literals
    True, False, Nil,

    // Type Keywords
    Void, Bool, Char, Str, Noret, AnyType,
    Int, Uint, Float, Isize, Usize,
    I1, I2, I4, I8, I16, I32, I64, I128,
    U1, U2, U4, U8, U16, U32, U64, U128,
    F8, F16, F32, F64, F128,

    // Arithmetic
    Plus, Minus, Star, Slash, Percent,

    // Compound Assignment
    PlusEquals, MinusEquals, StarEquals, SlashEquals, PercentEquals,

    // Comparison
    EqualEqual, NotEqual, Less, Greater, LessEqual, GreaterEqual,

    // Logical
    AndAnd, OrOr, Bang,

    // Bitwise
    And, Or, Caret, Tilde, ShiftLeft, ShiftRight,

    // Sigils & Special
    ColonEquals, ColonColon, TildeArrow, Arrow, FatArrow, At,
    Dot, DotDot, DotDotEquals, DotStar,
    Colon, Comma, Semicolon, Assign, Underscore, Pipe,
    QuestionMark,

    // Grouping
    LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket,

    // Values
    Identifier, IntegerValue, FloatValue, StringValue, CharValue,

    // Comments
    LineComment, BlockComment,

    // End of file
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpannedTokenKind {
    Token(TokenKind),
    Eof,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub line: usize,
    pub col: usize,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, value: String, line: usize, col: usize, span: Span) -> Self {
        Token { kind, value, line, col, span }
    }
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub span: Span,
}

impl LexError {
    pub fn new(message: String, line: usize, col: usize, span: Span) -> Self {
        LexError { message, line, col, span }
    }
}

#[derive(Debug, Clone)]
pub struct TokenizationResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}

impl TokenizationResult {
    pub fn new() -> Self {
        TokenizationResult { tokens: Vec::new(), errors: Vec::new() }
    }

    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub fn error(&mut self, err: LexError) {
        self.errors.push(err);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Control Flow
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Stop => write!(f, "stop"),
            TokenKind::Next => write!(f, "next"),
            TokenKind::Ret => write!(f, "ret"),

            // Memory & Error
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Defer => write!(f, "defer"),

            // Modules & Visibility
            TokenKind::Mod => write!(f, "mod"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::Ext => write!(f, "ext"),

            // Definitions
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Union => write!(f, "union"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::ErrorKw => write!(f, "error"),
            TokenKind::Behave => write!(f, "behave"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Test => write!(f, "test"),

            // Literals
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Nil => write!(f, "nil"),

            // Type Keywords
            TokenKind::Void => write!(f, "void"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::Str => write!(f, "str"),
            TokenKind::Noret => write!(f, "noret"),
            TokenKind::AnyType => write!(f, "anytype"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Uint => write!(f, "uint"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Isize => write!(f, "isize"),
            TokenKind::Usize => write!(f, "usize"),
            TokenKind::I1 => write!(f, "i1"),
            TokenKind::I2 => write!(f, "i2"),
            TokenKind::I4 => write!(f, "i4"),
            TokenKind::I8 => write!(f, "i8"),
            TokenKind::I16 => write!(f, "i16"),
            TokenKind::I32 => write!(f, "i32"),
            TokenKind::I64 => write!(f, "i64"),
            TokenKind::I128 => write!(f, "i128"),
            TokenKind::U1 => write!(f, "u1"),
            TokenKind::U2 => write!(f, "u2"),
            TokenKind::U4 => write!(f, "u4"),
            TokenKind::U8 => write!(f, "u8"),
            TokenKind::U16 => write!(f, "u16"),
            TokenKind::U32 => write!(f, "u32"),
            TokenKind::U64 => write!(f, "u64"),
            TokenKind::U128 => write!(f, "u128"),
            TokenKind::F8 => write!(f, "f8"),
            TokenKind::F16 => write!(f, "f16"),
            TokenKind::F32 => write!(f, "f32"),
            TokenKind::F64 => write!(f, "f64"),
            TokenKind::F128 => write!(f, "f128"),

            // Arithmetic
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),

            // Compound Assignment
            TokenKind::PlusEquals => write!(f, "+="),
            TokenKind::MinusEquals => write!(f, "-="),
            TokenKind::StarEquals => write!(f, "*="),
            TokenKind::SlashEquals => write!(f, "/="),
            TokenKind::PercentEquals => write!(f, "%="),

            // Comparison
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::GreaterEqual => write!(f, ">="),

            // Logical
            TokenKind::AndAnd => write!(f, "&&"),
            TokenKind::OrOr => write!(f, "||"),
            TokenKind::Bang => write!(f, "!"),

            // Bitwise
            TokenKind::And => write!(f, "&"),
            TokenKind::Or => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::ShiftLeft => write!(f, "<<"),
            TokenKind::ShiftRight => write!(f, ">>"),

            // Sigils & Special
            TokenKind::ColonEquals => write!(f, ":="),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::TildeArrow => write!(f, "~>"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotEquals => write!(f, "..="),
            TokenKind::DotStar => write!(f, ".*"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::QuestionMark => write!(f, "?"),

            // Grouping
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),

            // Values
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::IntegerValue => write!(f, "integer"),
            TokenKind::FloatValue => write!(f, "float"),
            TokenKind::StringValue => write!(f, "string"),
            TokenKind::CharValue => write!(f, "char"),

            // Comments
            TokenKind::LineComment => write!(f, "line_comment"),
            TokenKind::BlockComment => write!(f, "block_comment"),

            // End of file
            TokenKind::Eof => write!(f, "eof"),
        }
    }
}

impl fmt::Display for SpannedTokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpannedTokenKind::Token(tk) => write!(f, "{}", tk),
            SpannedTokenKind::Eof => write!(f, "eof"),
            SpannedTokenKind::Invalid => write!(f, "invalid"),
        }
    }
}
