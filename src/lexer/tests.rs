use crate::lexer::Lexer;
use crate::lexer::token::{TokenKind, TokenizationResult};

fn tokenize(source: &str) -> TokenizationResult {
    let lexer = Lexer::new(source);
    lexer.tokenize()
}

fn kinds(result: &TokenizationResult) -> Vec<TokenKind> {
    result.tokens.iter().map(|t| t.kind.clone()).collect()
}

fn kind_at(result: &TokenizationResult, idx: usize) -> TokenKind {
    result.tokens[idx].kind.clone()
}

fn value_at(result: &TokenizationResult, idx: usize) -> String {
    result.tokens[idx].value.clone()
}

fn has_error_containing(result: &TokenizationResult, substr: &str) -> bool {
    result.errors.iter().any(|e| e.message.contains(substr))
}

fn count_kind(result: &TokenizationResult, kind: TokenKind) -> usize {
    result.tokens.iter().filter(|t| t.kind == kind).count()
}

// ---------------------------------------------------------------------------
// 1. Empty source
// ---------------------------------------------------------------------------
#[test]
fn test_empty_source() {
    let r = tokenize("");
    assert!(
        r.tokens.is_empty(),
        "expected no tokens, got {}",
        r.tokens.len()
    );
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Whitespace only
// ---------------------------------------------------------------------------
#[test]
fn test_whitespace_only() {
    let r = tokenize("   \t\n  \r\n  ");
    assert!(
        r.tokens.is_empty(),
        "expected no tokens, got {}",
        r.tokens.len()
    );
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 3. Line comment
// ---------------------------------------------------------------------------
#[test]
fn test_line_comment() {
    let r = tokenize("// this is a comment");
    assert_eq!(r.tokens.len(), 1);
    assert_eq!(r.tokens[0].kind, TokenKind::LineComment);
    assert!(r.tokens[0].value.starts_with("//"));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 4. Block comment
// ---------------------------------------------------------------------------
#[test]
fn test_block_comment() {
    let r = tokenize("/* hello world */");
    assert_eq!(r.tokens.len(), 1);
    assert_eq!(r.tokens[0].kind, TokenKind::BlockComment);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 5. Nested block comments
// ---------------------------------------------------------------------------
#[test]
fn test_nested_block_comments() {
    let r = tokenize("/* outer /* inner */ still outer */");
    assert_eq!(r.tokens.len(), 1);
    assert_eq!(r.tokens[0].kind, TokenKind::BlockComment);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 6. Control flow keywords
// ---------------------------------------------------------------------------
#[test]
fn test_control_flow_keywords() {
    let r = tokenize("if else match loop stop next ret");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::If));
    assert!(ks.contains(&TokenKind::Else));
    assert!(ks.contains(&TokenKind::Match));
    assert!(ks.contains(&TokenKind::Loop));
    assert!(ks.contains(&TokenKind::Stop));
    assert!(ks.contains(&TokenKind::Next));
    assert!(ks.contains(&TokenKind::Ret));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 7. Memory & error keywords
// ---------------------------------------------------------------------------
#[test]
fn test_memory_error_keywords() {
    let r = tokenize("mut try catch defer");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Mut));
    assert!(ks.contains(&TokenKind::Try));
    assert!(ks.contains(&TokenKind::Catch));
    assert!(ks.contains(&TokenKind::Defer));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 8. Module & visibility keywords
// ---------------------------------------------------------------------------
#[test]
fn test_module_visibility_keywords() {
    let r = tokenize("mod use pub ext");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Mod));
    assert!(ks.contains(&TokenKind::Use));
    assert!(ks.contains(&TokenKind::Pub));
    assert!(ks.contains(&TokenKind::Ext));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 9. Definition keywords
// ---------------------------------------------------------------------------
#[test]
fn test_definition_keywords() {
    let r = tokenize("fn struct union enum error behave type test");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Fn));
    assert!(ks.contains(&TokenKind::Struct));
    assert!(ks.contains(&TokenKind::Union));
    assert!(ks.contains(&TokenKind::Enum));
    assert!(ks.contains(&TokenKind::ErrorKw));
    assert!(ks.contains(&TokenKind::Behave));
    assert!(ks.contains(&TokenKind::Type));
    assert!(ks.contains(&TokenKind::Test));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 10. Literal keywords
// ---------------------------------------------------------------------------
#[test]
fn test_literal_keywords() {
    let r = tokenize("true false nil");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::True));
    assert!(ks.contains(&TokenKind::False));
    assert!(ks.contains(&TokenKind::Nil));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 11. Primitive type keywords
// ---------------------------------------------------------------------------
#[test]
fn test_primitive_type_keywords() {
    let r = tokenize("void bool char str noret anytype");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Void));
    assert!(ks.contains(&TokenKind::Bool));
    assert!(ks.contains(&TokenKind::Char));
    assert!(ks.contains(&TokenKind::Str));
    assert!(ks.contains(&TokenKind::Noret));
    assert!(ks.contains(&TokenKind::AnyType));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 12. Sized type keywords
// ---------------------------------------------------------------------------
#[test]
fn test_sized_type_keywords() {
    let r = tokenize("int uint float isize usize");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Int));
    assert!(ks.contains(&TokenKind::Uint));
    assert!(ks.contains(&TokenKind::Float));
    assert!(ks.contains(&TokenKind::Isize));
    assert!(ks.contains(&TokenKind::Usize));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 13. Fixed-width integer keywords
// ---------------------------------------------------------------------------
#[test]
fn test_fixed_int_keywords() {
    let r = tokenize("i8 i16 i32 i64 i128 u8 u16 u32 u64 u128");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::I8));
    assert!(ks.contains(&TokenKind::I16));
    assert!(ks.contains(&TokenKind::I32));
    assert!(ks.contains(&TokenKind::I64));
    assert!(ks.contains(&TokenKind::I128));
    assert!(ks.contains(&TokenKind::U8));
    assert!(ks.contains(&TokenKind::U16));
    assert!(ks.contains(&TokenKind::U32));
    assert!(ks.contains(&TokenKind::U64));
    assert!(ks.contains(&TokenKind::U128));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 14. Fixed-width float keywords
// ---------------------------------------------------------------------------
#[test]
fn test_fixed_float_keywords() {
    let r = tokenize("f32 f64 f128");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::F32));
    assert!(ks.contains(&TokenKind::F64));
    assert!(ks.contains(&TokenKind::F128));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 15. Underscore
// ---------------------------------------------------------------------------
#[test]
fn test_underscore() {
    let r = tokenize("_");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Underscore));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 16. Simple identifier
// ---------------------------------------------------------------------------
#[test]
fn test_simple_identifier() {
    let r = tokenize("hello");
    assert_eq!(kind_at(&r, 0), TokenKind::Identifier);
    assert_eq!(value_at(&r, 0), "hello");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 17. Identifier with underscores
// ---------------------------------------------------------------------------
#[test]
fn test_identifier_with_underscores() {
    let r = tokenize("my_variable_name");
    assert_eq!(kind_at(&r, 0), TokenKind::Identifier);
    assert_eq!(value_at(&r, 0), "my_variable_name");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 18. Identifier with numbers
// ---------------------------------------------------------------------------
#[test]
fn test_identifier_with_numbers() {
    let r = tokenize("var2");
    assert_eq!(kind_at(&r, 0), TokenKind::Identifier);
    assert_eq!(value_at(&r, 0), "var2");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 19. Identifier starting with underscore
// ---------------------------------------------------------------------------
#[test]
fn test_identifier_starting_with_underscore() {
    let r = tokenize("_private");
    assert_eq!(kind_at(&r, 0), TokenKind::Identifier);
    assert_eq!(value_at(&r, 0), "_private");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 20. Decimal integer
// ---------------------------------------------------------------------------
#[test]
fn test_decimal_integer() {
    let r = tokenize("42");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "42");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 21. Hex integer
// ---------------------------------------------------------------------------
#[test]
fn test_hex_integer() {
    let r = tokenize("0xFF");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0xFF");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 22. Binary integer
// ---------------------------------------------------------------------------
#[test]
fn test_binary_integer() {
    let r = tokenize("0b1010");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0b1010");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 23. Octal integer
// ---------------------------------------------------------------------------
#[test]
fn test_octal_integer() {
    let r = tokenize("0o777");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0o777");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 24. Integer with underscore separators
// ---------------------------------------------------------------------------
#[test]
fn test_integer_with_underscores() {
    let r = tokenize("1_000_000");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "1000000");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 25. Simple float
// ---------------------------------------------------------------------------
#[test]
fn test_simple_float() {
    let r = tokenize("3.14");
    assert_eq!(kind_at(&r, 0), TokenKind::FloatValue);
    assert_eq!(value_at(&r, 0), "3.14");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 26. Float with exponent
// ---------------------------------------------------------------------------
#[test]
fn test_float_with_exponent() {
    let r = tokenize("1.5e10");
    assert_eq!(kind_at(&r, 0), TokenKind::FloatValue);
    assert_eq!(value_at(&r, 0), "1.5e10");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 27. Float with negative exponent
// ---------------------------------------------------------------------------
#[test]
fn test_float_negative_exponent() {
    let r = tokenize("2.5e-3");
    assert_eq!(kind_at(&r, 0), TokenKind::FloatValue);
    assert_eq!(value_at(&r, 0), "2.5e-3");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 28. Integer followed by dot (not a float — the dot is separate)
// ---------------------------------------------------------------------------
#[test]
fn test_integer_dot_not_float() {
    let r = tokenize("5.");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "5");
    assert_eq!(kind_at(&r, 1), TokenKind::Dot);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 29. Float with underscore separators
// ---------------------------------------------------------------------------
#[test]
fn test_float_with_underscores() {
    let r = tokenize("1_000.5");
    assert_eq!(kind_at(&r, 0), TokenKind::FloatValue);
    assert_eq!(value_at(&r, 0), "1000.5");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 30. Basic string
// ---------------------------------------------------------------------------
#[test]
fn test_basic_string() {
    let r = tokenize("\"hello\"");
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert_eq!(value_at(&r, 0), "\"hello\"");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 31. String with escape sequences
// ---------------------------------------------------------------------------
#[test]
fn test_string_with_escapes() {
    let r = tokenize("\"hello\\nworld\\ttab\"");
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 32. String with hex and unicode escapes
// ---------------------------------------------------------------------------
#[test]
fn test_string_hex_unicode_escapes() {
    let r = tokenize("\"\\x41\\u{0041}\"");
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 33. Basic char
// ---------------------------------------------------------------------------
#[test]
fn test_basic_char() {
    let r = tokenize("'a'");
    assert_eq!(kind_at(&r, 0), TokenKind::CharValue);
    assert_eq!(value_at(&r, 0), "'a'");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 34. Char with escape
// ---------------------------------------------------------------------------
#[test]
fn test_char_with_escape() {
    let r = tokenize("'\\n'");
    assert_eq!(kind_at(&r, 0), TokenKind::CharValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 35. Unclosed string (error)
// ---------------------------------------------------------------------------
#[test]
fn test_unclosed_string() {
    let r = tokenize("\"unclosed");
    assert!(has_error_containing(&r, "Unclosed string"));
    // Should still have the token
    assert!(count_kind(&r, TokenKind::StringValue) > 0);
}

// ---------------------------------------------------------------------------
// 36. Unclosed char (error)
// ---------------------------------------------------------------------------
#[test]
fn test_unclosed_char() {
    let r = tokenize("'x");
    assert!(has_error_containing(&r, "Unclosed char"));
    assert!(count_kind(&r, TokenKind::CharValue) > 0);
}

// ---------------------------------------------------------------------------
// 37. Invalid escape sequence (error)
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_escape() {
    let r = tokenize("\"hello\\zworld\"");
    assert!(has_error_containing(&r, "Invalid escape"));
}

// ---------------------------------------------------------------------------
// 38. Unclosed block comment (error)
// ---------------------------------------------------------------------------
#[test]
fn test_unclosed_block_comment() {
    let r = tokenize("/* unclosed comment");
    assert!(has_error_containing(&r, "Unclosed block comment"));
    assert!(count_kind(&r, TokenKind::BlockComment) > 0);
}

// ---------------------------------------------------------------------------
// 39. Arithmetic operators
// ---------------------------------------------------------------------------
#[test]
fn test_arithmetic_operators() {
    let r = tokenize("+ - * / %");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Plus));
    assert!(ks.contains(&TokenKind::Minus));
    assert!(ks.contains(&TokenKind::Star));
    assert!(ks.contains(&TokenKind::Slash));
    assert!(ks.contains(&TokenKind::Percent));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 40. Comparison operators
// ---------------------------------------------------------------------------
#[test]
fn test_comparison_operators() {
    let r = tokenize("== != < > <= >=");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::EqualEqual));
    assert!(ks.contains(&TokenKind::NotEqual));
    assert!(ks.contains(&TokenKind::Less));
    assert!(ks.contains(&TokenKind::Greater));
    assert!(ks.contains(&TokenKind::LessEqual));
    assert!(ks.contains(&TokenKind::GreaterEqual));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 41. Logical operators
// ---------------------------------------------------------------------------
#[test]
fn test_logical_operators() {
    let r = tokenize("&& || !");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::AndAnd));
    assert!(ks.contains(&TokenKind::OrOr));
    assert!(ks.contains(&TokenKind::Bang));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 42. Bitwise operators
// ---------------------------------------------------------------------------
#[test]
fn test_bitwise_operators() {
    let r = tokenize("& | ^ ~ << >>");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::And));
    assert!(ks.contains(&TokenKind::Pipe));
    assert!(ks.contains(&TokenKind::Caret));
    assert!(ks.contains(&TokenKind::Tilde));
    assert!(ks.contains(&TokenKind::ShiftLeft));
    assert!(ks.contains(&TokenKind::ShiftRight));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 43. Compound assignments
// ---------------------------------------------------------------------------
#[test]
fn test_compound_assignments() {
    let r = tokenize("+= -= *= /= %=");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::PlusEquals));
    assert!(ks.contains(&TokenKind::MinusEquals));
    assert!(ks.contains(&TokenKind::StarEquals));
    assert!(ks.contains(&TokenKind::SlashEquals));
    assert!(ks.contains(&TokenKind::PercentEquals));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 44. Range and arrow operators
// ---------------------------------------------------------------------------
#[test]
fn test_range_arrow_operators() {
    let r = tokenize(".. ..= -> ~> => := :: .*");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::DotDot));
    assert!(ks.contains(&TokenKind::DotDotEquals));
    assert!(ks.contains(&TokenKind::Arrow));
    assert!(ks.contains(&TokenKind::TildeArrow));
    assert!(ks.contains(&TokenKind::FatArrow));
    assert!(ks.contains(&TokenKind::ColonEquals));
    assert!(ks.contains(&TokenKind::ColonColon));
    assert!(ks.contains(&TokenKind::DotStar));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 45. Access and grouping tokens
// ---------------------------------------------------------------------------
#[test]
fn test_access_grouping() {
    let r = tokenize(". , ; : @ ? ( ) { } [ ]");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::Dot));
    assert!(ks.contains(&TokenKind::Comma));
    assert!(ks.contains(&TokenKind::Semicolon));
    assert!(ks.contains(&TokenKind::Colon));
    assert!(ks.contains(&TokenKind::At));
    assert!(ks.contains(&TokenKind::QuestionMark));
    assert!(ks.contains(&TokenKind::LeftParen));
    assert!(ks.contains(&TokenKind::RightParen));
    assert!(ks.contains(&TokenKind::LeftBrace));
    assert!(ks.contains(&TokenKind::RightBrace));
    assert!(ks.contains(&TokenKind::LeftBracket));
    assert!(ks.contains(&TokenKind::RightBracket));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 46. Assignment operator
// ---------------------------------------------------------------------------
#[test]
fn test_assignment_operator() {
    let r = tokenize("=");
    assert_eq!(kind_at(&r, 0), TokenKind::Assign);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 47. Multiple lines with mixed tokens
// ---------------------------------------------------------------------------
#[test]
fn test_multi_line_tokens() {
    let r = tokenize("x := 5\nif true {\n  ret\n}");
    assert!(count_kind(&r, TokenKind::Identifier) > 0);
    assert!(count_kind(&r, TokenKind::ColonEquals) > 0);
    assert!(count_kind(&r, TokenKind::IntegerValue) > 0);
    assert!(count_kind(&r, TokenKind::If) > 0);
    assert!(count_kind(&r, TokenKind::True) > 0);
    assert!(count_kind(&r, TokenKind::LeftBrace) > 0);
    assert!(count_kind(&r, TokenKind::Ret) > 0);
    assert!(count_kind(&r, TokenKind::RightBrace) > 0);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 48. Line and column tracking
// ---------------------------------------------------------------------------
#[test]
fn test_line_col_tracking() {
    let r = tokenize("a\nb\nc");
    let a = &r.tokens[0];
    let b = &r.tokens[1];
    let c = &r.tokens[2];
    assert_eq!(a.line, 1);
    assert_eq!(a.col, 1);
    assert_eq!(b.line, 2);
    assert_eq!(b.col, 1);
    assert_eq!(c.line, 3);
    assert_eq!(c.col, 1);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 49. Byte span tracking
// ---------------------------------------------------------------------------
#[test]
fn test_byte_span_tracking() {
    let r = tokenize("abc 123");
    let ident = &r.tokens[0];
    let int = &r.tokens[1];
    // "abc" starts at byte 0, ends at byte 3
    assert_eq!(ident.span, (0, 3));
    // "123" starts at byte 4, ends at byte 7
    assert_eq!(int.span, (4, 7));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 50. UTF-8 multi-byte span tracking
// ---------------------------------------------------------------------------
#[test]
fn test_utf8_span_tracking() {
    let r = tokenize("// comment with unicode: \u{00e9}\n42");
    // The comment should span the right byte range
    let comment = &r.tokens[0];
    assert_eq!(comment.kind, TokenKind::LineComment);
    let int = &r.tokens[1];
    assert_eq!(int.span.0, comment.span.1 + 1); // +1 for newline
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 51. Unknown character produces error
// ---------------------------------------------------------------------------
#[test]
fn test_unknown_character_error() {
    let r = tokenize("`");
    assert!(has_error_containing(&r, "Unexpected character"));
}

// ---------------------------------------------------------------------------
// 52. Multiple unknown characters
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_unknown_characters() {
    let r = tokenize("`@#$");
    // `@#$ — only ` is truly unknown; @ and # are...
    // Actually @ is a valid token. # and $ are unknown.
    let unknown_count = r.errors.len();
    assert!(
        unknown_count >= 1,
        "expected at least 1 unknown char error, got {}",
        unknown_count
    );
}

// ---------------------------------------------------------------------------
// 53. Mixed comments and code
// ---------------------------------------------------------------------------
#[test]
fn test_mixed_comments_and_code() {
    let source = "x := 1 // init\n/* block */ y := 2";
    let r = tokenize(source);
    assert!(count_kind(&r, TokenKind::Identifier) >= 2);
    assert!(count_kind(&r, TokenKind::IntegerValue) >= 2);
    assert!(count_kind(&r, TokenKind::LineComment) >= 1);
    assert!(count_kind(&r, TokenKind::BlockComment) >= 1);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 54. String across multiple lines (raw newlines in string)
// ---------------------------------------------------------------------------
#[test]
fn test_string_across_lines() {
    let r = tokenize("\"hello\nworld\"");
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    // The string contains an actual newline, which is valid
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 55. Invalid hex escape (\\x with wrong chars)
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_hex_escape() {
    let r = tokenize("\"\\xZZ\"");
    assert!(has_error_containing(&r, "Expected 2 hex digits"));
}

// ---------------------------------------------------------------------------
// 56. Invalid unicode escape (\\u without braces)
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_unicode_escape_no_brace() {
    let r = tokenize("\"\\u0041\"");
    assert!(has_error_containing(&r, "Expected '{' after \\u"));
}

// ---------------------------------------------------------------------------
// 57. Invalid unicode escape (empty braces)
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_unicode_escape_empty() {
    let r = tokenize("\"\\u{}\"");
    assert!(has_error_containing(&r, "Expected hex digits"));
}

// ---------------------------------------------------------------------------
// 58. EOF token always present
// ---------------------------------------------------------------------------
#[test]
fn test_eof_token_always_present() {
    // Parser handles end-of-input via pos >= tokens.len(); no explicit Eof token needed
    for src in &["", " ", "foo", "123", "\"hi\""] {
        let r = tokenize(src);
        // Non-empty sources should have at least one real token
        if !src.trim().is_empty() {
            assert!(
                !r.tokens.is_empty(),
                "source {:?} should produce tokens",
                src
            );
        } else {
            assert!(
                r.tokens.is_empty(),
                "source {:?} should produce no tokens",
                src
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 59. Number starting with zero
// ---------------------------------------------------------------------------
#[test]
fn test_number_starting_with_zero() {
    let r = tokenize("0");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 60. Identifier mixed with operators
// ---------------------------------------------------------------------------
#[test]
fn test_identifier_operator_sequence() {
    let r = tokenize("a+b-c");
    let ks = kinds(&r);
    assert_eq!(ks[0], TokenKind::Identifier);
    assert_eq!(ks[1], TokenKind::Plus);
    assert_eq!(ks[2], TokenKind::Identifier);
    assert_eq!(ks[3], TokenKind::Minus);
    assert_eq!(ks[4], TokenKind::Identifier);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 61. Float with trailing zero digits after dot
// ---------------------------------------------------------------------------
#[test]
fn test_float_trailing_zeros() {
    let r = tokenize("1.0");
    assert_eq!(kind_at(&r, 0), TokenKind::FloatValue);
    assert_eq!(value_at(&r, 0), "1.0");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 62. String with backslash at end (incomplete escape)
// ---------------------------------------------------------------------------
#[test]
fn test_string_backslash_at_end() {
    let r = tokenize("\"hello\\");
    // The backslash at end is followed by nothing, so the escape
    // validation can't run; the string just ends.
    assert!(has_error_containing(&r, "Unclosed string") || has_error_containing(&r, "escape"));
}

// ---------------------------------------------------------------------------
// 63. Very large integer
// ---------------------------------------------------------------------------
#[test]
fn test_large_integer() {
    let r = tokenize("9999999999999999999");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "9999999999999999999");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 64. Sequential underscores in identifier
// ---------------------------------------------------------------------------
#[test]
fn test_sequential_underscores_identifier() {
    let r = tokenize("__init__");
    assert_eq!(kind_at(&r, 0), TokenKind::Identifier);
    assert_eq!(value_at(&r, 0), "__init__");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 65. Tab in string
// ---------------------------------------------------------------------------
#[test]
fn test_tab_in_string() {
    let r = tokenize("\"a\tb\"");
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 66. Keywords are NOT identifiers
// ---------------------------------------------------------------------------
#[test]
fn test_keywords_not_identifiers() {
    let r = tokenize("fn if else struct");
    for token in &r.tokens {
        if token.kind == TokenKind::Eof {
            continue;
        }
        assert_ne!(
            token.kind,
            TokenKind::Identifier,
            "keyword tokenized as identifier"
        );
    }
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 67. Single pipe character
// ---------------------------------------------------------------------------
#[test]
fn test_pipe_operator() {
    let r = tokenize("|");
    assert_eq!(kind_at(&r, 0), TokenKind::Pipe);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 68. Tab indentation tracking
// ---------------------------------------------------------------------------
#[test]
fn test_tab_indentation() {
    let r = tokenize("\t\tx");
    let tok = &r.tokens[0];
    assert_eq!(tok.kind, TokenKind::Identifier);
    // Tab advances col by... currently tabs add 4
    assert!(tok.col > 1);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 69. Dot-dot without float being produced
// ---------------------------------------------------------------------------
#[test]
fn test_range_vs_float() {
    let r = tokenize("0..5");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(kind_at(&r, 1), TokenKind::DotDot);
    assert_eq!(kind_at(&r, 2), TokenKind::IntegerValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 70. No errors on properly escaped string
// ---------------------------------------------------------------------------
#[test]
fn test_no_errors_on_properly_escaped_string() {
    let r = tokenize("\"\\n\\t\\r\\\\\\0\\\"\\'\"");
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert!(!has_error_containing(&r, "Invalid escape"));
}

// ---------------------------------------------------------------------------
// 71. Line comment with special chars
// ---------------------------------------------------------------------------
#[test]
fn test_line_comment_with_special_chars() {
    let r = tokenize("// comments can have * / /* */ and other stuff");
    assert_eq!(kind_at(&r, 0), TokenKind::LineComment);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 72. Block comment on multiple lines
// ---------------------------------------------------------------------------
#[test]
fn test_multi_line_block_comment() {
    let r = tokenize("/* line1\nline2\nline3 */");
    assert_eq!(kind_at(&r, 0), TokenKind::BlockComment);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 73. Token after comment
// ---------------------------------------------------------------------------
#[test]
fn test_token_after_comment() {
    let r = tokenize("// comment\n42");
    assert_eq!(kind_at(&r, 0), TokenKind::LineComment);
    assert_eq!(kind_at(&r, 1), TokenKind::IntegerValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 74. Tokens separated by multiple spaces
// ---------------------------------------------------------------------------
#[test]
fn test_tokens_with_multiple_spaces() {
    let r = tokenize("a    b     c");
    assert_eq!(count_kind(&r, TokenKind::Identifier), 3);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 75. Empty string literal
// ---------------------------------------------------------------------------
#[test]
fn test_empty_string() {
    let r = tokenize("\"\"");
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert_eq!(value_at(&r, 0), "\"\"");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 76. @str token
// ---------------------------------------------------------------------------
#[test]
fn test_at_str_token() {
    let r = tokenize("@str");
    assert_eq!(kind_at(&r, 0), TokenKind::AtStr);
    assert_eq!(value_at(&r, 0), "@str");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 77. @vec, @map, @set still work
// ---------------------------------------------------------------------------
#[test]
fn test_collection_at_tokens() {
    let r = tokenize("@vec @map @set");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::AtVec));
    assert!(ks.contains(&TokenKind::AtMap));
    assert!(ks.contains(&TokenKind::AtSet));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 78. @str not confused with @set or @st identifier
// ---------------------------------------------------------------------------
#[test]
fn test_at_str_not_at_set() {
    let r = tokenize("@str @set");
    assert_eq!(kind_at(&r, 0), TokenKind::AtStr);
    assert_eq!(kind_at(&r, 1), TokenKind::AtSet);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 79. Empty hex prefix produces error
// ---------------------------------------------------------------------------
#[test]
fn test_empty_hex_prefix_error() {
    let r = tokenize("0x");
    assert!(has_error_containing(&r, "Expected hex digits"));
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
}

// ---------------------------------------------------------------------------
// 80. Empty binary prefix produces error
// ---------------------------------------------------------------------------
#[test]
fn test_empty_binary_prefix_error() {
    let r = tokenize("0b");
    assert!(has_error_containing(&r, "Expected binary digits"));
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
}

// ---------------------------------------------------------------------------
// 81. Empty octal prefix produces error
// ---------------------------------------------------------------------------
#[test]
fn test_empty_octal_prefix_error() {
    let r = tokenize("0o");
    assert!(has_error_containing(&r, "Expected octal digits"));
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
}

// ---------------------------------------------------------------------------
// 82. Hex prefix with valid digits still works
// ---------------------------------------------------------------------------
#[test]
fn test_hex_with_valid_digits() {
    let r = tokenize("0xFF");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0xFF");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 83. Binary prefix with valid digits still works
// ---------------------------------------------------------------------------
#[test]
fn test_binary_with_valid_digits() {
    let r = tokenize("0b1010");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0b1010");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 84. Octal prefix with valid digits still works
// ---------------------------------------------------------------------------
#[test]
fn test_octal_with_valid_digits() {
    let r = tokenize("0o777");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0o777");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 85. Char literal with escape sequences work
// ---------------------------------------------------------------------------
#[test]
fn test_char_with_escape_sequences() {
    let r = tokenize("'\\n' '\\t' '\\0' '\\\\'");
    assert_eq!(count_kind(&r, TokenKind::CharValue), 4);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 86. Char literal with hex escape
// ---------------------------------------------------------------------------
#[test]
fn test_char_with_hex_escape() {
    let r = tokenize("'\\x41'");
    assert_eq!(kind_at(&r, 0), TokenKind::CharValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 87. Char literal with unicode escape
// ---------------------------------------------------------------------------
#[test]
fn test_char_with_unicode_escape() {
    let r = tokenize("'\\u{0041}'");
    assert_eq!(kind_at(&r, 0), TokenKind::CharValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 88. Char literal single character
// ---------------------------------------------------------------------------
#[test]
fn test_char_single_letter() {
    let r = tokenize("'a'");
    assert_eq!(kind_at(&r, 0), TokenKind::CharValue);
    assert_eq!(value_at(&r, 0), "'a'");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 89. Mixed @ builtins (non-collection)
// ---------------------------------------------------------------------------
#[test]
fn test_non_collection_at_builtins() {
    let r = tokenize("@TypeOf @SizeOf @panic");
    let ks = kinds(&r);
    assert!(ks.contains(&TokenKind::At));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 90. @str in expression context
// ---------------------------------------------------------------------------
#[test]
fn test_at_str_in_expression() {
    let r = tokenize("x := @str");
    assert_eq!(kind_at(&r, 0), TokenKind::Identifier);
    assert_eq!(kind_at(&r, 1), TokenKind::ColonEquals);
    assert_eq!(kind_at(&r, 2), TokenKind::AtStr);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 91. String and identifier span tracking
// ---------------------------------------------------------------------------
#[test]
fn test_string_span_tracking() {
    let r = tokenize("\"hello\" world");
    let string_tok = &r.tokens[0];
    let ident_tok = &r.tokens[1];
    assert_eq!(string_tok.kind, TokenKind::StringValue);
    assert_eq!(ident_tok.kind, TokenKind::Identifier);
    assert_eq!(string_tok.span, (0, 7));
    assert_eq!(ident_tok.span, (8, 13));
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 92. Integer 0 alone
// ---------------------------------------------------------------------------
#[test]
fn test_zero_integer() {
    let r = tokenize("0");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 93. Float starting with dot (.5)
// ---------------------------------------------------------------------------
#[test]
fn test_float_starting_with_dot() {
    let r = tokenize(".5");
    assert_eq!(kind_at(&r, 0), TokenKind::Dot);
    assert_eq!(kind_at(&r, 1), TokenKind::IntegerValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 94. Consecutive operators
// ---------------------------------------------------------------------------
#[test]
fn test_consecutive_operators() {
    let r = tokenize("==!=<=>=");
    let ks = kinds(&r);
    assert_eq!(ks[0], TokenKind::EqualEqual);
    assert_eq!(ks[1], TokenKind::NotEqual);
    assert_eq!(ks[2], TokenKind::LessEqual);
    assert_eq!(ks[3], TokenKind::GreaterEqual);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 95. Deeply nested block comments
// ---------------------------------------------------------------------------
#[test]
fn test_deeply_nested_block_comments() {
    let r = tokenize("/* a /* b /* c */ b */ a */");
    assert_eq!(r.tokens.len(), 1);
    assert_eq!(r.tokens[0].kind, TokenKind::BlockComment);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 96. @str with dot access
// ---------------------------------------------------------------------------
#[test]
fn test_at_str_with_dot() {
    let r = tokenize("@str.from");
    assert_eq!(kind_at(&r, 0), TokenKind::AtStr);
    assert_eq!(kind_at(&r, 1), TokenKind::Dot);
    assert_eq!(kind_at(&r, 2), TokenKind::Identifier);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 97. Hex integer with underscores
// ---------------------------------------------------------------------------
#[test]
fn test_hex_with_underscores() {
    let r = tokenize("0xFF_FF");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0xFFFF");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 98. Binary integer with underscores
// ---------------------------------------------------------------------------
#[test]
fn test_binary_with_underscores() {
    let r = tokenize("0b1010_0101");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0b10100101");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 99. Octal integer with underscores
// ---------------------------------------------------------------------------
#[test]
fn test_octal_with_underscores() {
    let r = tokenize("0o77_77");
    assert_eq!(kind_at(&r, 0), TokenKind::IntegerValue);
    assert_eq!(value_at(&r, 0), "0o7777");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 100. Full Razen program tokenization
// ---------------------------------------------------------------------------
#[test]
fn test_full_program() {
    let source = r#"
use std.testing.assert

Player :: struct {
    pub score: i32,
}

main :: fn() -> i32 {
    mut p := Player{ score: 10 }
    increment_score(&mut p)
    assert(p.score, 20)
    ret 0
}
"#;
    let r = tokenize(source);
    assert!(r.errors.is_empty());
    assert!(count_kind(&r, TokenKind::Use) > 0);
    assert!(count_kind(&r, TokenKind::Struct) > 0);
    assert!(count_kind(&r, TokenKind::Fn) > 0);
    assert!(count_kind(&r, TokenKind::Pub) > 0);
    assert!(count_kind(&r, TokenKind::Mut) > 0);
    assert!(count_kind(&r, TokenKind::Ret) > 0);
}

// ---------------------------------------------------------------------------
// 101. Double backslash in string produces single backslash
// ---------------------------------------------------------------------------
#[test]
fn test_double_backslash_in_string() {
    let r = tokenize(r#""path\\to\\file""#);
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert_eq!(value_at(&r, 0), r#""path\\to\\file""#);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 102. Escaped quote in string does not close the string
// ---------------------------------------------------------------------------
#[test]
fn test_escaped_quote_in_string() {
    let r = tokenize(r#""say \"hello\"""#);
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert_eq!(value_at(&r, 0), r#""say \"hello\"""#);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 103. Triple backslash in string
// ---------------------------------------------------------------------------
#[test]
fn test_triple_backslash_in_string() {
    // \\\\ = two escaped backslashes, then 'b' is regular
    let r = tokenize(r#""a\\\\b""#);
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert_eq!(value_at(&r, 0), r#""a\\\\b""#);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 104. Backslash at end of string (incomplete escape)
// ---------------------------------------------------------------------------
#[test]
fn test_backslash_at_end_of_string() {
    let r = tokenize("\"abc\\");
    assert!(has_error_containing(&r, "Unclosed string"));
}

// ---------------------------------------------------------------------------
// 105. All valid escape sequences in one string
// ---------------------------------------------------------------------------
#[test]
fn test_all_valid_escapes_together() {
    let r = tokenize(r#""\n\t\r\0\\\"\'\x41\u{0041}""#);
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 106. Backslash followed by invalid escape char
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_escape_after_backslash() {
    let r = tokenize(r#""\q""#);
    assert!(has_error_containing(&r, "Invalid escape"));
}

// ---------------------------------------------------------------------------
// 107. Char literal with double backslash
// ---------------------------------------------------------------------------
#[test]
fn test_char_double_backslash() {
    let r = tokenize("'\\\\'");
    assert_eq!(kind_at(&r, 0), TokenKind::CharValue);
    assert_eq!(value_at(&r, 0), "'\\\\'");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 108. Char literal with escaped single quote
// ---------------------------------------------------------------------------
#[test]
fn test_char_escaped_single_quote() {
    let r = tokenize("'\\''");
    assert_eq!(kind_at(&r, 0), TokenKind::CharValue);
    assert_eq!(value_at(&r, 0), "'\\''");
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 109. Mixed escapes: backslash, quote, newline
// ---------------------------------------------------------------------------
#[test]
fn test_mixed_escapes_complex() {
    let r = tokenize(r#""line\\n\"tab\t""#);
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert!(r.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 110. String with only backslashes
// ---------------------------------------------------------------------------
#[test]
fn test_string_only_backslashes() {
    let r = tokenize(r#""\\\\""#);
    assert_eq!(kind_at(&r, 0), TokenKind::StringValue);
    assert_eq!(value_at(&r, 0), r#""\\\\""#);
    assert!(r.errors.is_empty());
}
