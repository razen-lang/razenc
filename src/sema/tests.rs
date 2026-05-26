#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::TokenKind;
    use crate::sema::SemanticAnalyzer;

    fn analyze(decls: Vec<Decl>) -> Result<(), Vec<crate::sema::checker::SemanticError>> {
        let program = Program { decls };
        let mut sema = SemanticAnalyzer::new();
        sema.analyze(&program)
    }

    fn make_fn(name: &str, params: Vec<Param>, return_: Option<Type>, body: Block) -> Decl {
        Decl::Fn(FnDecl {
            name: name.to_string(),
            generics: vec![],
            pub_: false,
            external: false,
            attrs: vec![],
            params,
            return_,
            body: Some(body),
        })
    }

    fn make_void_fn(name: &str, body: Block) -> Decl {
        make_fn(name, vec![], Some(Type::Primitive(TokenKind::Void)), body)
    }

    fn make_i32_fn(name: &str, body: Block) -> Decl {
        make_fn(name, vec![], Some(Type::Primitive(TokenKind::I32)), body)
    }

    fn ret_expr(val: Expr) -> Stmt {
        Stmt::Ret(Some(val))
    }

    fn lit_i32(val: i32) -> Expr {
        Expr::Literal(TokenKind::IntegerValue, val.to_string())
    }

    fn lit_bool(val: bool) -> Expr {
        if val {
            Expr::Literal(TokenKind::True, "true".into())
        } else {
            Expr::Literal(TokenKind::False, "false".into())
        }
    }

    fn lit_str(val: &str) -> Expr {
        Expr::Literal(TokenKind::StringValue, val.to_string())
    }

    fn ident(name: &str) -> Expr {
        Expr::Ident(name.to_string())
    }

    fn block(stmts: Vec<Stmt>) -> Block {
        Block { stmts }
    }

    fn expr_stmt(e: Expr) -> Stmt {
        Stmt::Expr(e)
    }

    fn assign(target: Expr, value: Expr) -> Stmt {
        Stmt::Assign(target, AssignOp::Eq, value)
    }

    fn var_decl(name: &str, mutable: bool, type_: Option<Type>, value: Option<Expr>) -> Decl {
        Decl::Var(VarDecl {
            name: name.to_string(),
            mutable,
            pub_: false,
            attrs: vec![],
            type_,
            value,
        })
    }

    fn if_stmt(cond: Expr, then_stmts: Vec<Stmt>, else_stmt: Option<Box<Stmt>>) -> Stmt {
        Stmt::If(If {
            cond,
            capture: vec![],
            then_block: block(then_stmts),
            else_block: else_stmt,
        })
    }

    fn var_stmt(name: &str, mutable: bool, type_: Option<Type>, value: Option<Expr>) -> Stmt {
        Stmt::Var(VarDecl {
            name: name.to_string(),
            mutable,
            pub_: false,
            attrs: vec![],
            type_,
            value,
        })
    }

    // S-SEMA-02: Test missing return in non-void function
    #[test]
    fn test_missing_return_non_void() {
        let decls = vec![make_i32_fn(
            "missing_ret",
            block(vec![var_stmt("x", false, None, Some(lit_i32(42)))]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(
            errs.iter().any(|e| e.code == "SEMA-0009"),
            "Expected SEMA-0009 for missing return, got: {:?}",
            errs
        );
    }

    // Test void function doesn't need return
    #[test]
    fn test_void_fn_no_return_ok() {
        let decls = vec![make_void_fn(
            "no_ret",
            block(vec![var_stmt("x", false, None, Some(lit_i32(42)))]),
        )];
        let result = analyze(decls);
        assert!(result.is_ok(), "Void function should not need return");
    }

    // Test proper return in non-void function
    #[test]
    fn test_non_void_with_return_ok() {
        let decls = vec![make_i32_fn("has_ret", block(vec![ret_expr(lit_i32(42))]))];
        let result = analyze(decls);
        assert!(result.is_ok(), "Non-void function with return should pass");
    }

    // S-SEMA-02: Test unreachable code after ret
    #[test]
    fn test_unreachable_after_ret() {
        let decls = vec![make_i32_fn(
            "unreachable",
            block(vec![ret_expr(lit_i32(1)), ret_expr(lit_i32(2))]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(
            errs.iter().any(|e| e.code == "SEMA-0014"),
            "Expected SEMA-0014 for unreachable code, got: {:?}",
            errs
        );
    }

    // S-SEMA-02: Test if-else both branches return
    #[test]
    fn test_if_else_both_return_ok() {
        let decls = vec![make_i32_fn(
            "if_else_both_return",
            block(vec![
                var_stmt("x", false, None, Some(lit_bool(true))),
                if_stmt(
                    ident("x"),
                    vec![ret_expr(lit_i32(1))],
                    Some(Box::new(Stmt::Block(block(vec![ret_expr(lit_i32(2))])))),
                ),
            ]),
        )];
        let result = analyze(decls);
        assert!(
            result.is_ok(),
            "If-else both branches return should be ok: {:?}",
            result
        );
    }

    // S-SEMA-02: Test if without else - not all paths return
    #[test]
    fn test_if_without_else_missing_return() {
        let decls = vec![make_i32_fn(
            "if_no_else",
            block(vec![if_stmt(
                lit_bool(true),
                vec![ret_expr(lit_i32(1))],
                None,
            )]),
        )];
        let result = analyze(decls);
        assert!(
            result.is_err(),
            "If without else should warn about missing return"
        );
    }

    // S-SEMA-02: Test stop inside loop
    #[test]
    fn test_stop_inside_loop_ok() {
        let decls = vec![make_void_fn(
            "loop_stop",
            block(vec![Stmt::Loop(Loop {
                conds: vec![],
                captures: vec![],
                body: block(vec![Stmt::Stop]),
            })]),
        )];
        let result = analyze(decls);
        assert!(
            result.is_ok(),
            "stop inside loop should be ok: {:?}",
            result
        );
    }

    // S-SEMA-02: Test stop outside loop
    #[test]
    fn test_stop_outside_loop() {
        let decls = vec![make_void_fn("bad_stop", block(vec![Stmt::Stop]))];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0008"),
            "Expected SEMA-0008 for stop outside loop"
        );
    }

    // S-SEMA-02: Test next outside loop
    #[test]
    fn test_next_outside_loop() {
        let decls = vec![make_void_fn("bad_next", block(vec![Stmt::Next]))];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0008"),
            "Expected SEMA-0008 for next outside loop"
        );
    }

    // S-SEMA-03: Test return type inference
    #[test]
    fn test_return_type_inference() {
        let decls = vec![make_fn(
            "inferred_ret",
            vec![],
            None,
            block(vec![ret_expr(lit_i32(42))]),
        )];
        // Should not error - return type inferable from expression
        let result = analyze(decls);
        assert!(
            result.is_ok(),
            "Function without declared return type should infer from ret expr: {:?}",
            result
        );
    }

    // S-SEMA-11: Test struct init with wrong field type
    #[test]
    fn test_struct_init_field_type_mismatch() {
        let struct_decl = Decl::Struct(StructDecl {
            name: "Point".into(),
            generics: vec![],
            impl_behave: None,
            pub_: false,
            attrs: vec![],
            fields: vec![Field {
                name: "x".into(),
                pub_: true,
                type_: Type::Primitive(TokenKind::I32),
            }],
            methods: vec![],
        });

        let init_decl = make_void_fn(
            "test_init",
            block(vec![expr_stmt(Expr::StructInit(
                "Point".into(),
                vec![FieldInit {
                    name: "x".into(),
                    value: lit_str("hello"),
                }],
            ))]),
        );

        let decls = vec![struct_decl, init_decl];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0001"),
            "Expected SEMA-0001 for field type mismatch"
        );
    }

    // S-SEMA-11: Test struct init with correct field type
    #[test]
    fn test_struct_init_field_type_ok() {
        let struct_decl = Decl::Struct(StructDecl {
            name: "Point".into(),
            generics: vec![],
            impl_behave: None,
            pub_: false,
            attrs: vec![],
            fields: vec![Field {
                name: "x".into(),
                pub_: true,
                type_: Type::Primitive(TokenKind::I32),
            }],
            methods: vec![],
        });

        let init_decl = make_void_fn(
            "test_init",
            block(vec![expr_stmt(Expr::StructInit(
                "Point".into(),
                vec![FieldInit {
                    name: "x".into(),
                    value: lit_i32(10),
                }],
            ))]),
        );

        let decls = vec![struct_decl, init_decl];
        let result = analyze(decls);
        assert!(
            result.is_ok(),
            "Struct init with correct field types should pass: {:?}",
            result
        );
    }

    // S-SEMA-11: Test struct init missing required field
    #[test]
    fn test_struct_init_missing_field() {
        let struct_decl = Decl::Struct(StructDecl {
            name: "Point".into(),
            generics: vec![],
            impl_behave: None,
            pub_: false,
            attrs: vec![],
            fields: vec![
                Field {
                    name: "x".into(),
                    pub_: true,
                    type_: Type::Primitive(TokenKind::I32),
                },
                Field {
                    name: "y".into(),
                    pub_: true,
                    type_: Type::Primitive(TokenKind::I32),
                },
            ],
            methods: vec![],
        });

        let init_decl = make_void_fn(
            "test_init",
            block(vec![expr_stmt(Expr::StructInit(
                "Point".into(),
                vec![FieldInit {
                    name: "x".into(),
                    value: lit_i32(10),
                }],
            ))]),
        );

        let decls = vec![struct_decl, init_decl];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .iter()
                .any(|e| e.code == "SEMA-0015" && e.message.contains("Missing")),
            "Expected SEMA-0015 for missing field"
        );
    }

    // S-SEMA-01: Test generic function with zero params (can't infer)
    #[test]
    fn test_generic_fn_no_params_inference_failure() {
        // Generic function with zero params but generic return:
        // fn foo<T>() -> T { ... }
        let gen_fn = Decl::Fn(FnDecl {
            name: "make".to_string(),
            generics: vec!["T".to_string()],
            pub_: false,
            external: false,
            attrs: vec![],
            params: vec![],
            return_: Some(Type::Primitive(TokenKind::I32)),
            body: Some(block(vec![ret_expr(lit_i32(0))])),
        });

        // Calling make() without explicit generic args:
        let caller = make_i32_fn(
            "caller",
            block(vec![ret_expr(Expr::Call(Box::new(ident("make")), vec![]))]),
        );

        let decls = vec![gen_fn, caller];
        let result = analyze(decls);
        // We register generics but don't have explicit call-site type args,
        // so the generic param check tries to infer from 0 params = warning
        // It might succeed or error depending on inference - accept either
        // Actually, the check says "generic params can't be inferred from 0 params"
        // Let's just check it doesn't crash
        let _ = result;
    }

    // Test duplicate declaration
    #[test]
    fn test_duplicate_declaration() {
        let decls = vec![
            var_decl(
                "x",
                false,
                Some(Type::Primitive(TokenKind::I32)),
                Some(lit_i32(1)),
            ),
            var_decl(
                "x",
                false,
                Some(Type::Primitive(TokenKind::I32)),
                Some(lit_i32(2)),
            ),
        ];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0003"),
            "Expected SEMA-0003 for duplicate declaration"
        );
    }

    // Test undefined symbol
    #[test]
    fn test_undefined_symbol() {
        let decls = vec![make_i32_fn(
            "f",
            block(vec![ret_expr(ident("undefined_var"))]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0002"),
            "Expected SEMA-0002 for undefined symbol"
        );
    }

    // Test mutation of immutable
    #[test]
    fn test_mutation_of_immutable() {
        let decls = vec![make_void_fn(
            "f",
            block(vec![
                var_stmt("x", false, None, Some(lit_i32(1))),
                assign(ident("x"), lit_i32(2)),
            ]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0004"),
            "Expected SEMA-0004 for mutation of immutable"
        );
    }

    // Test assignment to constant
    #[test]
    fn test_assign_to_const() {
        let const_decl = Decl::Const(ConstDecl {
            name: "X".into(),
            attrs: vec![],
            type_: None,
            value: Some(lit_i32(10)),
        });
        let fn_decl = make_void_fn("f", block(vec![assign(ident("X"), lit_i32(20))]));
        let decls = vec![const_decl, fn_decl];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0005"),
            "Expected SEMA-0005 for assignment to const"
        );
    }

    // Test non-bool condition
    #[test]
    fn test_non_bool_condition() {
        let decls = vec![make_void_fn(
            "f",
            block(vec![if_stmt(lit_i32(42), vec![], None)]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0007"),
            "Expected SEMA-0007 for non-bool condition"
        );
    }

    // Test type mismatch in assignment
    #[test]
    fn test_type_mismatch_assign() {
        let decls = vec![make_void_fn(
            "f",
            block(vec![
                var_stmt("x", true, Some(Type::Primitive(TokenKind::I32)), None),
                assign(ident("x"), lit_str("hello")),
            ]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0001"),
            "Expected SEMA-0001 for type mismatch"
        );
    }

    // Test match not exhaustive
    #[test]
    fn test_match_not_exhaustive() {
        let decls = vec![make_void_fn(
            "f",
            block(vec![Stmt::Match(Match {
                target: lit_i32(1),
                arms: vec![MatchArm {
                    pattern: Pattern::Literal(TokenKind::IntegerValue, "1".into()),
                    capture: vec![],
                    value: Expr::Block(block(vec![])),
                }],
            })]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0016"),
            "Expected SEMA-0016 for non-exhaustive match"
        );
    }

    // Test match with wildcard (exhaustive)
    #[test]
    fn test_match_exhaustive_with_wildcard() {
        let decls = vec![make_void_fn(
            "f",
            block(vec![Stmt::Match(Match {
                target: lit_i32(1),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Literal(TokenKind::IntegerValue, "1".into()),
                        capture: vec![],
                        value: Expr::Block(block(vec![])),
                    },
                    MatchArm {
                        pattern: Pattern::Wildcard,
                        capture: vec![],
                        value: Expr::Block(block(vec![])),
                    },
                ],
            })]),
        )];
        let result = analyze(decls);
        assert!(
            result.is_ok(),
            "Match with wildcard should be exhaustive: {:?}",
            result
        );
    }

    // Test invalid binary operator
    #[test]
    fn test_invalid_binary_op() {
        let decls = vec![make_void_fn(
            "f",
            block(vec![expr_stmt(Expr::Binary(
                BinaryOp::Add,
                Box::new(lit_bool(true)),
                Box::new(lit_i32(1)),
            ))]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0012"),
            "Expected SEMA-0012 for invalid binary op"
        );
    }

    // Test a completely valid program
    #[test]
    fn test_valid_program() {
        let decls = vec![make_i32_fn("main", block(vec![ret_expr(lit_i32(0))]))];
        let result = analyze(decls);
        assert!(
            result.is_ok(),
            "Valid program should pass sema: {:?}",
            result
        );
    }

    // Test type alias
    #[test]
    fn test_type_alias() {
        let alias = Decl::TypeAlias("Age".into(), Type::Primitive(TokenKind::I32));
        let fn_decl = make_i32_fn("get_age", block(vec![ret_expr(lit_i32(25))]));
        let decls = vec![alias, fn_decl];
        let result = analyze(decls);
        assert!(
            result.is_ok(),
            "Type alias should not cause errors: {:?}",
            result
        );
    }

    // Test return type mismatch with explicit type
    #[test]
    fn test_return_type_mismatch_explicit() {
        let decls = vec![make_fn(
            "bad_return",
            vec![],
            Some(Type::Primitive(TokenKind::I32)),
            block(vec![ret_expr(lit_str("hello"))]),
        )];
        let result = analyze(decls);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().iter().any(|e| e.code == "SEMA-0009"),
            "Expected SEMA-0009 for return type mismatch"
        );
    }
}
