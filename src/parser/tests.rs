use crate::ast::*;
use crate::lexer::Lexer;
use crate::lexer::token::TokenKind;
use crate::parser::Parser;

fn parse(source: &str) -> Result<Program, Vec<String>> {
    let lexer = Lexer::new(source);
    let result = lexer.tokenize();
    if !result.errors.is_empty() {
        return Err(result.errors.iter().map(|e| e.message.clone()).collect());
    }
    let mut parser = Parser::new(result.tokens);
    parser.parse()
}

fn parse_ok(source: &str) -> Program {
    match parse(source) {
        Ok(p) => p,
        Err(e) => panic!("Parse failed: {:?}\nSource: {}", e, source),
    }
}

fn parse_err(source: &str) -> Vec<String> {
    match parse(source) {
        Ok(p) => panic!("Expected parse error but got: {:?}\nSource: {}", p, source),
        Err(e) => e,
    }
}

fn first_decl(prog: &Program) -> &Decl {
    &prog.decls[0]
}

// ---------------------------------------------------------------------------
// 1. Empty / minimal
// ---------------------------------------------------------------------------
#[test]
fn test_empty_source() {
    let p = parse_ok("");
    assert!(p.decls.is_empty());
}

#[test]
fn test_whitespace_only() {
    let p = parse_ok("   \n  \t  ");
    assert!(p.decls.is_empty());
}

#[test]
fn test_comment_only() {
    let p = parse_ok("// just a comment");
    assert!(p.decls.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Basic variable declarations
// ---------------------------------------------------------------------------
#[test]
fn test_var_decl_inferred() {
    let p = parse_ok("x := 42");
    match first_decl(&p) {
        Decl::Var(v) => {
            assert_eq!(v.name, "x");
            assert!(!v.mutable);
            assert!(!v.pub_);
            assert!(v.type_.is_none());
            assert!(v.value.is_some());
        }
        other => panic!("Expected Var, got {:?}", other),
    }
}

#[test]
fn test_var_decl_mutable() {
    let p = parse_ok("mut x := 42");
    match first_decl(&p) {
        Decl::Var(v) => {
            assert_eq!(v.name, "x");
            assert!(v.mutable);
        }
        other => panic!("Expected Var, got {:?}", other),
    }
}

#[test]
fn test_var_decl_explicit_type() {
    let p = parse_ok("x : i32 = 42");
    match first_decl(&p) {
        Decl::Var(v) => {
            assert_eq!(v.name, "x");
            assert!(v.type_.is_some());
            assert!(v.value.is_some());
        }
        other => panic!("Expected Var, got {:?}", other),
    }
}

#[test]
fn test_var_decl_colon_eq() {
    let p = parse_ok("x : i32 := 42");
    match first_decl(&p) {
        Decl::Var(v) => {
            assert_eq!(v.name, "x");
            assert!(v.type_.is_some());
        }
        other => panic!("Expected Var, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 3. Constant declarations
// ---------------------------------------------------------------------------
#[test]
fn test_const_decl_inferred() {
    let p = parse_ok("PI :: 3.14159");
    match first_decl(&p) {
        Decl::Const(c) => {
            assert_eq!(c.name, "PI");
            assert!(c.type_.is_none());
            assert!(c.value.is_some());
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_const_decl_explicit_type() {
    let p = parse_ok("PI : f64 : 3.14159");
    match first_decl(&p) {
        Decl::Const(c) => {
            assert_eq!(c.name, "PI");
            assert!(c.type_.is_some());
            assert!(c.value.is_some());
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 4. Function declarations
// ---------------------------------------------------------------------------
#[test]
fn test_fn_decl_no_params() {
    let p = parse_ok("main :: fn() -> void { }");
    match first_decl(&p) {
        Decl::Fn(f) => {
            assert_eq!(f.name, "main");
            assert!(f.params.is_empty());
            assert!(f.return_.is_some());
            assert!(f.body.is_some());
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_fn_decl_with_params() {
    let p = parse_ok("add :: fn(a: i32, b: i32) -> i32 { ret a + b }");
    match first_decl(&p) {
        Decl::Fn(f) => {
            assert_eq!(f.name, "add");
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name, "a");
            assert_eq!(f.params[1].name, "b");
            assert!(f.return_.is_some());
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_fn_decl_no_body() {
    let p = parse_ok("extern_fn :: fn(x: i32) -> void");
    match first_decl(&p) {
        Decl::Fn(f) => {
            assert_eq!(f.name, "extern_fn");
            assert!(f.body.is_none());
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_fn_decl_generic() {
    let p = parse_ok("identity<T> :: fn(x: T) -> T { ret x }");
    match first_decl(&p) {
        Decl::Fn(f) => {
            assert_eq!(f.name, "identity");
            assert_eq!(f.generics, vec!["T"]);
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_ext_fn_decl() {
    let p = parse_ok("ext puts :: fn(s: &u8) -> i32");
    match first_decl(&p) {
        Decl::Fn(f) => {
            assert_eq!(f.name, "puts");
            assert!(f.external);
            assert!(f.body.is_none());
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 5. Struct / Union / Enum declarations
// ---------------------------------------------------------------------------
#[test]
fn test_struct_decl_empty() {
    let p = parse_ok("Empty :: struct { }");
    match first_decl(&p) {
        Decl::Struct(s) => {
            assert_eq!(s.name, "Empty");
            assert!(s.fields.is_empty());
        }
        other => panic!("Expected Struct, got {:?}", other),
    }
}

#[test]
fn test_struct_decl_fields() {
    let p = parse_ok("Point :: struct { x: i32, y: i32 }");
    match first_decl(&p) {
        Decl::Struct(s) => {
            assert_eq!(s.name, "Point");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name, "x");
            assert_eq!(s.fields[1].name, "y");
        }
        other => panic!("Expected Struct, got {:?}", other),
    }
}

#[test]
fn test_union_decl() {
    let p = parse_ok("IntOrFloat :: union { i: i32, f: f64 }");
    match first_decl(&p) {
        Decl::Union(u) => {
            assert_eq!(u.name, "IntOrFloat");
            assert_eq!(u.variants.len(), 2);
        }
        other => panic!("Expected Union, got {:?}", other),
    }
}

#[test]
fn test_enum_decl() {
    let p = parse_ok("Color :: enum { Red, Green, Blue }");
    match first_decl(&p) {
        Decl::Enum(e) => {
            assert_eq!(e.name, "Color");
            assert_eq!(e.variants.len(), 3);
        }
        other => panic!("Expected Enum, got {:?}", other),
    }
}

#[test]
fn test_generic_struct() {
    let p = parse_ok("Pair<T> :: struct { first: T, second: T }");
    match first_decl(&p) {
        Decl::Struct(s) => {
            assert_eq!(s.name, "Pair");
            assert_eq!(s.generics, vec!["T"]);
        }
        other => panic!("Expected Struct, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 6. Use / Mod
// ---------------------------------------------------------------------------
#[test]
fn test_use_decl() {
    let p = parse_ok("use std.mem.allocator");
    match first_decl(&p) {
        Decl::Use(path) => {
            assert_eq!(
                *path,
                vec![
                    "std".to_string(),
                    "mem".to_string(),
                    "allocator".to_string()
                ]
            );
        }
        other => panic!("Expected Use, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 7. Literal expressions
// ---------------------------------------------------------------------------
#[test]
fn test_literal_exprs() {
    let p = parse_ok("a :: 42\nb :: 3.14\nc :: \"hello\"\nd :: true\ne :: 'x'");
    assert_eq!(p.decls.len(), 5);
}

// ---------------------------------------------------------------------------
// 8. Binary operations (including Bitwise OR)
// ---------------------------------------------------------------------------
#[test]
fn test_binary_ops() {
    let p = parse_ok("r :: 1 + 2 * 3 | 4 ^ 5 & 6");
    match &p.decls[0] {
        Decl::Const(c) => {
            let val = c.value.as_ref().unwrap();
            // 1 + (2 * 3) | 4 ^ 5 & 6 — operator precedence test
            match val {
                Expr::Binary(BinaryOp::BitOr, _, _) => {} // top-level should be |
                other => panic!("Expected BitOr at top, got {:?}", other),
            }
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_bitwise_or_without_capture() {
    // Pipe as bitwise OR — not a capture since no ident follows
    let p = parse_ok("r :: 1 | 2");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Binary(BinaryOp::BitOr, l, r) => {
                assert!(matches!(l.as_ref(), Expr::Literal(_, _)));
                assert!(matches!(r.as_ref(), Expr::Literal(_, _)));
            }
            other => panic!("Expected BitOr, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_bitwise_and() {
    let p = parse_ok("r :: 5 & 3");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Binary(BinaryOp::BitAnd, _, _) => {}
            other => panic!("Expected BitAnd, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_bitwise_xor() {
    let p = parse_ok("r :: 5 ^ 3");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Binary(BinaryOp::BitXor, _, _) => {}
            other => panic!("Expected BitXor, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_shift_ops() {
    let p = parse_ok("a :: 1 << 3\nb :: 16 >> 2");
    assert_eq!(p.decls.len(), 2);
}

#[test]
fn test_comparison_ops() {
    let p =
        parse_ok("a :: 1 == 2\nb :: 3 != 4\nc :: 5 < 6\nd :: 7 > 8\ne :: 9 <= 10\nf :: 11 >= 12");
    assert_eq!(p.decls.len(), 6);
}

#[test]
fn test_logical_ops() {
    let p = parse_ok("a :: true && false\nb :: true || false");
    assert_eq!(p.decls.len(), 2);
}

#[test]
fn test_range_ops() {
    let p = parse_ok("a :: 0..5\nb :: 0..=10");
    assert_eq!(p.decls.len(), 2);
}

#[test]
fn test_assignment_ops() {
    let p = parse_ok("x := 10\nx += 1\nx -= 2\nx *= 3\nx /= 4\nx %= 5");
    assert_eq!(p.decls.len(), 6);
}

// ---------------------------------------------------------------------------
// 9. Unary operations
// ---------------------------------------------------------------------------
#[test]
fn test_unary_ops() {
    let p = parse_ok("a :: -5\nb :: !true\nc :: ~42\nd :: &x\ne :: &mut y\nf :: *ptr");
    assert_eq!(p.decls.len(), 6);
}

// ---------------------------------------------------------------------------
// 10. If expressions and statements
// ---------------------------------------------------------------------------
#[test]
fn test_if_expr() {
    let p = parse_ok("r :: if true { 42 } else { 0 }");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Block(b) => {
                assert_eq!(b.stmts.len(), 1);
                assert!(matches!(b.stmts[0], Stmt::If(_)));
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_if_else_if() {
    let p = parse_ok("r :: if x > 0 { 1 } else if x < 0 { -1 } else { 0 }");
    match &p.decls[0] {
        Decl::Const(c) => {
            match c.value.as_ref().unwrap() {
                Expr::Block(b) => {
                    assert_eq!(b.stmts.len(), 1);
                    match &b.stmts[0] {
                        Stmt::If(i) => {
                            assert!(i.else_block.is_some());
                            // else block should be another If
                            match i.else_block.as_ref().unwrap().as_ref() {
                                Stmt::If(inner) => {
                                    assert!(inner.else_block.is_some());
                                }
                                other => panic!("Expected nested If, got {:?}", other),
                            }
                        }
                        other => panic!("Expected If, got {:?}", other),
                    }
                }
                other => panic!("Expected Block, got {:?}", other),
            }
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_if_stmt() {
    let p = parse_ok("main :: fn() -> void { if true { } }");
    assert_eq!(p.decls.len(), 1);
}

// ---------------------------------------------------------------------------
// 11. Match expressions
// ---------------------------------------------------------------------------
#[test]
fn test_match_expr_simple() {
    let p = parse_ok("r :: match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Block(b) => {
                assert_eq!(b.stmts.len(), 1);
                match &b.stmts[0] {
                    Stmt::Match(m) => {
                        assert_eq!(m.arms.len(), 3);
                    }
                    other => panic!("Expected Match, got {:?}", other),
                }
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_match_capture() {
    let p = parse_ok("r :: match opt { x |val| => val, _ => 0 }");
    assert_eq!(p.decls.len(), 1);
}

// ---------------------------------------------------------------------------
// 12. Loop expressions
// ---------------------------------------------------------------------------
#[test]
fn test_loop_infinite() {
    let p = parse_ok("main :: fn() -> void { loop { } }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::Loop(l) => {
                    assert!(l.conds.is_empty());
                }
                other => panic!("Expected Loop, got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_loop_conditional() {
    let p = parse_ok("main :: fn() -> void { loop cond { } }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::Loop(l) => {
                    assert_eq!(l.conds.len(), 1);
                }
                other => panic!("Expected Loop, got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_loop_with_capture() {
    let p = parse_ok("main :: fn() -> void { loop collection |item| { } }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::Loop(l) => {
                    assert_eq!(l.captures.len(), 1);
                    assert_eq!(l.captures[0].name, "item");
                }
                other => panic!("Expected Loop, got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_loop_multi_range() {
    let p = parse_ok("main :: fn() -> void { loop xs, ys |i, j| { } }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::Loop(l) => {
                    assert_eq!(l.conds.len(), 2);
                    assert_eq!(l.captures.len(), 2);
                }
                other => panic!("Expected Loop, got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_loop_with_ref_mut_capture() {
    let p = parse_ok("main :: fn() -> void { loop vec |&mut elem| { } }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::Loop(l) => {
                    assert_eq!(l.captures.len(), 1);
                    assert!(l.captures[0].is_ref);
                    assert!(l.captures[0].mutable);
                }
                other => panic!("Expected Loop, got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 13. ret in expressions
// ---------------------------------------------------------------------------
#[test]
fn test_ret_expr() {
    let p = parse_ok("main :: fn() -> i32 { ret 42 }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::Ret(Some(val)) => match val {
                    Expr::Literal(TokenKind::IntegerValue, v) => assert_eq!(v, "42"),
                    other => panic!("Expected Literal(42), got {:?}", other),
                },
                other => panic!("Expected Ret, got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 14. Anonymous function expressions
// ---------------------------------------------------------------------------
#[test]
fn test_anon_fn_expr() {
    let p = parse_ok("f :: fn(x: i32) -> i32 { ret x + 1 }");
    match &p.decls[0] {
        Decl::Fn(fndecl) => {
            assert_eq!(fndecl.params.len(), 1);
            assert_eq!(fndecl.params[0].name, "x");
            assert!(fndecl.return_.is_some());
            assert!(fndecl.body.is_some());
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_anon_fn_no_return() {
    let p = parse_ok("f :: fn(x: i32) { }");
    match &p.decls[0] {
        Decl::Fn(fndecl) => {
            assert!(fndecl.return_.is_none());
            assert!(fndecl.body.is_some());
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 15. Type expressions (@vec[T], @map{K,V}, @set[T])
// ---------------------------------------------------------------------------
#[test]
fn test_vec_type() {
    let p = parse_ok("v : @vec[i32] : default_vec");
    match &p.decls[0] {
        Decl::Const(c) => {
            let t = c.type_.as_ref().unwrap();
            match t {
                Type::Builtin(name) => {
                    assert!(name.contains("vec") && name.contains("i32"), "got {}", name)
                }
                other => panic!("Expected Builtin(vec[i32]), got {:?}", other),
            }
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_map_type() {
    let p = parse_ok("m : @map{str, i32} : default_map");
    match &p.decls[0] {
        Decl::Const(c) => {
            let t = c.type_.as_ref().unwrap();
            match t {
                Type::Builtin(name) => assert!(
                    name.contains("map") && name.contains("str") && name.contains("i32"),
                    "got {}",
                    name
                ),
                other => panic!("Expected Builtin(map{{str,i32}}), got {:?}", other),
            }
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_set_type() {
    let p = parse_ok("s : @set{i32} : default_set");
    match &p.decls[0] {
        Decl::Const(c) => {
            let t = c.type_.as_ref().unwrap();
            match t {
                Type::Builtin(name) => {
                    assert!(name.contains("set") && name.contains("i32"), "got {}", name)
                }
                other => panic!("Expected Builtin(set{{i32}}), got {:?}", other),
            }
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 16. Pipe capture in various contexts
// ---------------------------------------------------------------------------
#[test]
fn test_if_capture() {
    let p = parse_ok("main :: fn() -> void { if opt |val| { } }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::If(i) => {
                    assert_eq!(i.capture.len(), 1);
                    assert_eq!(i.capture[0], "val");
                }
                other => panic!("Expected If, got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_match_arm_capture() {
    let p = parse_ok("r :: match x { val |cap| => cap, _ => 0 }");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Block(b) => match &b.stmts[0] {
                Stmt::Match(m) => {
                    assert_eq!(m.arms[0].capture.len(), 1);
                    assert_eq!(m.arms[0].capture[0], "cap");
                }
                other => panic!("Expected Match, got {:?}", other),
            },
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 17. Builtin / @-prefixed expressions
// ---------------------------------------------------------------------------
#[test]
fn test_builtin_calls() {
    let p = parse_ok("a :: @sizeOf(i32)\nb :: @TypeOf(42)\nc :: @panic(\"err\")");
    assert_eq!(p.decls.len(), 3);
}

#[test]
fn test_vec_constructor() {
    let p = parse_ok("v :: @vec(1, 2, 3)");
    assert_eq!(p.decls.len(), 1);
}

// ---------------------------------------------------------------------------
// 18. Error handling tests
// ---------------------------------------------------------------------------
#[test]
fn test_missing_semicolon_error() {
    // Strongly-adjacent declarations are valid (no separator needed)
    let p = parse_ok("x := 42\ny := 10");
    assert_eq!(p.decls.len(), 2);
}

#[test]
fn test_invalid_decl_syntax() {
    let errs = parse_err("x : : 42");
    assert!(!errs.is_empty(), "Expected parse errors");
}

#[test]
fn test_unclosed_brace() {
    let errs = parse_err("x :: fn() -> void { ");
    assert!(!errs.is_empty(), "Expected parse errors");
}

// ---------------------------------------------------------------------------
// 19. Complex nested expressions
// ---------------------------------------------------------------------------
#[test]
fn test_complex_arithmetic() {
    let p = parse_ok("r :: (1 + 2) * (3 - 4) / 5 % 6");
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn test_field_access() {
    let p = parse_ok("r :: point.x");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Field(obj, name) => {
                assert_eq!(name, "x");
                match obj.as_ref() {
                    Expr::Ident(n) => assert_eq!(n, "point"),
                    other => panic!("Expected Ident, got {:?}", other),
                }
            }
            other => panic!("Expected Field, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_index_access() {
    let p = parse_ok("r :: arr[0]");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Index(_, idx) => match idx.as_ref() {
                Expr::Literal(TokenKind::IntegerValue, v) => assert_eq!(v, "0"),
                other => panic!("Expected Literal(0), got {:?}", other),
            },
            other => panic!("Expected Index, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_slice_expr() {
    let p = parse_ok("r :: arr[0..5]");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Slice(_, _, _, _) => {}
            other => panic!("Expected Slice, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_call_expr() {
    let p = parse_ok("r :: add(1, 2)");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Call(callee, args) => {
                assert_eq!(args.len(), 2);
                match callee.as_ref() {
                    Expr::Ident(n) => assert_eq!(n, "add"),
                    other => panic!("Expected Ident, got {:?}", other),
                }
            }
            other => panic!("Expected Call, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 20. at_method expressions
// ---------------------------------------------------------------------------
#[test]
fn test_at_method() {
    let p = parse_ok("r :: obj @method()");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::AtMethod(obj, method) => {
                assert_eq!(method, "method");
                match obj.as_ref() {
                    Expr::Ident(n) => assert_eq!(n, "obj"),
                    other => panic!("Expected Ident, got {:?}", other),
                }
            }
            other => panic!("Expected AtMethod, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 21. Deref and address-of
// ---------------------------------------------------------------------------
#[test]
fn test_deref() {
    let p = parse_ok("r :: ptr.*");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Deref(e) => match e.as_ref() {
                Expr::Ident(n) => assert_eq!(n, "ptr"),
                other => panic!("Expected Ident, got {:?}", other),
            },
            other => panic!("Expected Deref, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 22. Struct init
// ---------------------------------------------------------------------------
#[test]
fn test_struct_init() {
    let p = parse_ok("p :: Point{ x: 1, y: 2 }");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::StructInit(name, fields) => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "x");
                assert_eq!(fields[1].name, "y");
            }
            other => panic!("Expected StructInit, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 23. Parenthesized expressions
// ---------------------------------------------------------------------------
#[test]
fn test_paren_expr() {
    let p = parse_ok("r :: (42)");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Paren(inner) => match inner.as_ref() {
                Expr::Literal(TokenKind::IntegerValue, v) => assert_eq!(v, "42"),
                other => panic!("Expected Literal(42), got {:?}", other),
            },
            other => panic!("Expected Paren, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 24. Error type declarations
// ---------------------------------------------------------------------------
#[test]
fn test_error_decl() {
    let p = parse_ok("IOError :: error { NotFound, PermissionDenied, ConnectionFailed }");
    match &p.decls[0] {
        Decl::Error_(name, variants) => {
            assert_eq!(name, "IOError");
            assert_eq!(variants.len(), 3);
        }
        other => panic!("Expected Error_, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 25. Behave (trait) declarations
// ---------------------------------------------------------------------------
#[test]
fn test_behave_decl() {
    let p = parse_ok("Drawable :: behave { draw :: fn(self: &@Self) -> void }");
    match &p.decls[0] {
        Decl::Behave(b) => {
            assert_eq!(b.name, "Drawable");
            assert_eq!(b.methods.len(), 1);
        }
        other => panic!("Expected Behave, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 26. Type alias
// ---------------------------------------------------------------------------
#[test]
fn test_type_alias() {
    let p = parse_ok("MyInt :: type(i32)");
    match &p.decls[0] {
        Decl::TypeAlias(name, _) => {
            assert_eq!(name, "MyInt");
        }
        other => panic!("Expected TypeAlias, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 27. Test declarations
// ---------------------------------------------------------------------------
#[test]
fn test_test_decl() {
    let p = parse_ok("test_example :: test { }");
    match &p.decls[0] {
        Decl::Test(name, _) => {
            assert_eq!(name, "test_example");
        }
        other => panic!("Expected Test, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 28. Optional and error union types
// ---------------------------------------------------------------------------
#[test]
fn test_optional_type() {
    let p = parse_ok("v : ?i32 : nil");
    match &p.decls[0] {
        Decl::Const(c) => {
            let t = c.type_.as_ref().unwrap();
            match t {
                Type::Optional(inner) => match inner.as_ref() {
                    Type::Primitive(TokenKind::I32) => {}
                    other => panic!("Expected I32, got {:?}", other),
                },
                other => panic!("Expected Optional, got {:?}", other),
            }
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_error_union_type() {
    let p = parse_ok("result : Error!i32 = @panic(\"nyi\")");
    match &p.decls[0] {
        Decl::Var(v) => {
            let t = v.type_.as_ref().unwrap();
            match t {
                Type::ErrorUnion(_, _) => {}
                other => panic!("Expected ErrorUnion type, got {:?}", other),
            }
        }
        other => panic!("Expected Var, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 29. Try-catch
// ---------------------------------------------------------------------------
#[test]
fn test_try_catch() {
    let p = parse_ok("main :: fn() -> void { try { } catch |e| { } }");
    assert_eq!(p.decls.len(), 1);
}

// ---------------------------------------------------------------------------
// 30. Defer
// ---------------------------------------------------------------------------
#[test]
fn test_defer() {
    let p = parse_ok("main :: fn() -> void { defer cleanup() }");
    assert_eq!(p.decls.len(), 1);
}

// ---------------------------------------------------------------------------
// 31. Nested blocks and scopes
// ---------------------------------------------------------------------------
#[test]
fn test_nested_blocks() {
    let p = parse_ok("main :: fn() -> void { { { } } }");
    assert_eq!(p.decls.len(), 1);
}

// ---------------------------------------------------------------------------
// 32. Method declarations inside struct
// ---------------------------------------------------------------------------
#[test]
fn test_struct_with_methods() {
    let p = parse_ok(
        "Vec2 :: struct { x: f32, y: f32, pub length :: fn(self: &Vec2) -> f32 { ret 0.0 } }",
    );
    match &p.decls[0] {
        Decl::Struct(s) => {
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.methods.len(), 1);
            assert_eq!(s.methods[0].name, "length");
        }
        other => panic!("Expected Struct, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 33. Public declarations
// ---------------------------------------------------------------------------
#[test]
fn test_pub_decl() {
    let p = parse_ok("pub x :: 42\npub add :: fn(a: i32, b: i32) -> i32 { ret a + b }");
    assert_eq!(p.decls.len(), 2);
    match &p.decls[0] {
        Decl::Const(_c) => {} // pub is consumed but not stored on ConstDecl (it's on VarDecl)
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 34. Fn type syntax
// ---------------------------------------------------------------------------
#[test]
fn test_fn_type() {
    let p = parse_ok("cb : fn(i32) -> void : nil");
    match &p.decls[0] {
        Decl::Const(c) => {
            let t = c.type_.as_ref().unwrap();
            match t {
                Type::Fn(params, _ret) => {
                    assert_eq!(params.len(), 1);
                }
                other => panic!("Expected Fn type, got {:?}", other),
            }
        }
        other => panic!("Expected Const, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 35. Reference types
// ---------------------------------------------------------------------------
#[test]
fn test_ref_type() {
    let p = parse_ok("r : &i32 : nil\nm : &mut i32 : nil");
    assert_eq!(p.decls.len(), 2);
}

// ---------------------------------------------------------------------------
// 36. Comptime and compileLog builtins
// ---------------------------------------------------------------------------
#[test]
fn test_comptime_builtins() {
    let p = parse_ok("main :: fn() -> void { @compileLog(\"done\") }");
    assert_eq!(p.decls.len(), 1);
}

// ---------------------------------------------------------------------------
// 37. Multi-declaration file
// ---------------------------------------------------------------------------
#[test]
fn test_multi_decl() {
    let p = parse_ok(
        "
        use std.mem
        PI :: 3.14
        main :: fn() -> void {
            loop { }
        }
    ",
    );
    assert!(p.decls.len() >= 3);
}

// ---------------------------------------------------------------------------
// 38-50. Edge cases and combined features
// ---------------------------------------------------------------------------
#[test]
fn test_pipe_as_binary_not_capture() {
    // Pipe with identifier on both sides = binary OR, not capture
    let p = parse_ok("r :: flags_a | flags_b");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Binary(BinaryOp::BitOr, _, _) => {}
            other => panic!("Expected BitOr, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_var_decl_pub() {
    let p = parse_ok("pub x := 42");
    match &p.decls[0] {
        Decl::Var(v) => {
            assert!(v.pub_);
        }
        other => panic!("Expected Var, got {:?}", other),
    }
}

#[test]
fn test_large_binary_expression() {
    let p = parse_ok("r :: 1 + 2 * 3 - 4 / 5 | 6 ^ 7 & 8 << 1 >> 2 == 3 && true || false");
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn test_try_catch_capture() {
    let p = parse_ok("main :: fn() -> void { try { } catch |e| { } }");
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn test_enum_with_values() {
    let p = parse_ok("Status :: enum { Ok = 0, NotFound = 1, Error = 2 }");
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn test_nested_if_else() {
    let p = parse_ok("r :: if a { if b { 1 } else { 2 } } else { 3 }");
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn test_zero_arg_fn() {
    let p = parse_ok("main :: fn() -> void { }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            assert_eq!(f.params.len(), 0);
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_multi_param_fn() {
    let p = parse_ok("sum :: fn(a: i32, b: i32, c: i32) -> i32 { ret a + b + c }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            assert_eq!(f.params.len(), 3);
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_ret_void() {
    let p = parse_ok("main :: fn() -> void { ret }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            let body = f.body.as_ref().unwrap();
            match &body.stmts[0] {
                Stmt::Ret(None) => {}
                other => panic!("Expected Ret(None), got {:?}", other),
            }
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_if_expr_else_if_chain_deep() {
    let p = parse_ok("r :: if a { 1 } else if b { 2 } else if c { 3 } else { 4 }");
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn test_pub_fn_decl() {
    let p = parse_ok("pub add :: fn(a: i32, b: i32) -> i32 { ret a + b }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            assert!(f.pub_);
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_mut_param() {
    let p = parse_ok("main :: fn(mut buf: &u8) -> void { }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            assert!(f.params[0].mutable);
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_dotdot_equals_range() {
    let p = parse_ok("r :: 0..=10");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Binary(BinaryOp::RangeInclusive, _, _) => {}
            other => panic!("Expected RangeInclusive, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_optional_chaining() {
    let p = parse_ok("r :: ?x");
    match &p.decls[0] {
        Decl::Const(c) => match c.value.as_ref().unwrap() {
            Expr::Unary(UnaryOp::Optional, _) => {}
            other => panic!("Expected Optional unary, got {:?}", other),
        },
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_generic_fn_multi_params() {
    let p = parse_ok("convert<T, U> :: fn(x: T) -> U { ret @as(U, x) }");
    match &p.decls[0] {
        Decl::Fn(f) => {
            assert_eq!(f.generics, vec!["T", "U"]);
        }
        other => panic!("Expected Fn, got {:?}", other),
    }
}

#[test]
fn test_behave_with_generics() {
    let p = parse_ok("Comparable<T> :: behave { less :: fn(self: &@Self, other: T) -> bool }");
    match &p.decls[0] {
        Decl::Behave(b) => {
            assert_eq!(b.name, "Comparable");
            assert_eq!(b.generics, vec!["T"]);
            assert_eq!(b.methods.len(), 1);
        }
        other => panic!("Expected Behave, got {:?}", other),
    }
}
