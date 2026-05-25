use crate::lexer::token::{SpannedTokenKind, Token, TokenKind};

const RST: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

const GREY: &str = "\x1b[38;5;244m";
const ORNG: &str = "\x1b[38;5;214m";
const PEACH: &str = "\x1b[38;5;216m";
const LGREEN: &str = "\x1b[38;5;120m";

const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";

fn kind_color(kind: &TokenKind) -> &'static str {
    match kind {
        // Declaration keywords => green
        TokenKind::Type
        | TokenKind::Enum
        | TokenKind::Union
        | TokenKind::ErrorKw
        | TokenKind::Struct
        | TokenKind::Behave
        | TokenKind::Ext
        | TokenKind::Pub
        | TokenKind::Mod
        | TokenKind::Use
        | TokenKind::Mut
        | TokenKind::Test => GREEN,

        // Function keyword => light green
        TokenKind::Fn => LGREEN,

        // Control flow => magenta
        TokenKind::If
        | TokenKind::Else
        | TokenKind::Match
        | TokenKind::Loop
        | TokenKind::Ret
        | TokenKind::Stop
        | TokenKind::Next
        | TokenKind::Try
        | TokenKind::Catch
        | TokenKind::Defer => MAGENTA,

        // Type keywords => cyan
        TokenKind::Void
        | TokenKind::Bool
        | TokenKind::Char
        | TokenKind::Str
        | TokenKind::Noret
        | TokenKind::AnyType
        | TokenKind::Int
        | TokenKind::Uint
        | TokenKind::Float
        | TokenKind::Isize
        | TokenKind::Usize
        | TokenKind::I1
        | TokenKind::I2
        | TokenKind::I4
        | TokenKind::I8
        | TokenKind::I16
        | TokenKind::I32
        | TokenKind::I64
        | TokenKind::I128
        | TokenKind::U1
        | TokenKind::U2
        | TokenKind::U4
        | TokenKind::U8
        | TokenKind::U16
        | TokenKind::U32
        | TokenKind::U64
        | TokenKind::U128
        | TokenKind::F8
        | TokenKind::F16
        | TokenKind::F32
        | TokenKind::F64
        | TokenKind::F128 => CYAN,

        // Literal keywords => orange
        TokenKind::True | TokenKind::False | TokenKind::Nil => ORNG,

        // Identifiers => peach
        TokenKind::Identifier => PEACH,

        // Values => orange
        TokenKind::IntegerValue
        | TokenKind::FloatValue
        | TokenKind::StringValue
        | TokenKind::CharValue => ORNG,

        // Comments => dim/grey
        TokenKind::LineComment | TokenKind::BlockComment => GREY,

        // Assignment operators => yellow
        TokenKind::ColonEquals
        | TokenKind::Assign
        | TokenKind::PlusEquals
        | TokenKind::MinusEquals
        | TokenKind::StarEquals
        | TokenKind::SlashEquals
        | TokenKind::PercentEquals => YELLOW,

        // Arithmetic => yellow
        TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Percent => YELLOW,

        // Comparison => yellow
        TokenKind::EqualEqual
        | TokenKind::NotEqual
        | TokenKind::Less
        | TokenKind::LessEqual
        | TokenKind::Greater
        | TokenKind::GreaterEqual => YELLOW,

        // Logical / bitwise => yellow
        TokenKind::Bang
        | TokenKind::AndAnd
        | TokenKind::OrOr
        | TokenKind::And
        | TokenKind::Or
        | TokenKind::Caret
        | TokenKind::Tilde
        | TokenKind::ShiftLeft
        | TokenKind::ShiftRight
        | TokenKind::QuestionMark => YELLOW,

        // Arrows => yellow
        TokenKind::Arrow | TokenKind::FatArrow | TokenKind::TildeArrow | TokenKind::ColonColon => YELLOW,

        // Access / separators => grey
        TokenKind::Dot
        | TokenKind::DotDot
        | TokenKind::DotDotEquals
        | TokenKind::DotStar
        | TokenKind::Colon
        | TokenKind::Comma
        | TokenKind::At
        | TokenKind::Semicolon
        | TokenKind::Underscore
        | TokenKind::Pipe => GREY,

        // Grouping => grey
        TokenKind::LeftParen
        | TokenKind::RightParen
        | TokenKind::LeftBrace
        | TokenKind::RightBrace
        | TokenKind::LeftBracket
        | TokenKind::RightBracket => GREY,
    }
}

fn spanned_color(kind: &SpannedTokenKind) -> &'static str {
    match kind {
        SpannedTokenKind::Token(tk) => kind_color(tk),
        SpannedTokenKind::Eof => MAGENTA,
        SpannedTokenKind::Invalid => RED,
    }
}

pub fn print_source(source: &str, label: &str) {
    println!();
    println!("{}━━━ {} ━━━{}", BLUE, label, RST);
    println!("{}Source:{}", BOLD, RST);
    for line in source.lines() {
        println!("{}", line);
    }
    println!();
}

pub fn print_phase_header(phase: &str, status: &str) {
    println!(
        "{}Phase {}  {}{}\t\t\t{}{}{}",
        MAGENTA, "1", RST, phase, GREEN, status, RST
    );
}

pub fn print_token_count(count: usize) {
    println!();
    println!("{}Tokens{} ({})", CYAN, RST, count);
}

pub fn print_tokens(tokens: &[Token]) {
    for (i, token) in tokens.iter().enumerate() {
        let kind_str = format!("{}", token.kind);
        let value_str = if token.value.is_empty() {
            "''".to_string()
        } else {
            format!("'{}'", token.value)
        };
        let c = spanned_color(&SpannedTokenKind::Token(token.kind.clone()));

        let line = token.line;
        println!(
            "  {g}[{i:>3}]{r}  {g}Type:{r} {c}{k}{r}  \
             {g}Value:{r} {o}{v}{r}  {g}Line:{r} {l}",
            g = GREY, r = RST, i = i, c = c, k = kind_str,
            o = ORNG, v = value_str, l = line,
        );
    }
}

pub fn print_footer(status: &str) {
    println!();
    println!("\t\t{}{}{}", GREEN, status, RST);
    println!();
}

pub fn print_error(err: &str) {
    eprintln!("{}Error:{} {}", RED, RST, err);
}
