use crate::ast::*;
use crate::lexer::TokenKind;
use crate::sema::scope::*;
use crate::sema::types::*;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub code: String,
    pub message: String,
}

impl SemanticError {
    pub fn new(code: &str, message: String) -> Self {
        SemanticError {
            code: code.to_string(),
            message,
        }
    }
}

pub struct TypeChecker {
    pub errors: Vec<SemanticError>,
    pub loop_depth: usize,
    pub current_return_type: Option<TypeInfo>,
    /// Tracks whether the current code path has definitively exited
    /// (via ret/stop/noret). Used for unreachable code detection and
    /// missing-return analysis.
    pub reached_end: bool,
    /// Inferred return type when a function has no explicit return type.
    /// Populated by collecting the types of return expressions.
    pub inferred_return_type: Option<TypeInfo>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            errors: Vec::new(),
            loop_depth: 0,
            current_return_type: None,
            reached_end: false,
            inferred_return_type: None,
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error(&mut self, code: &str, msg: String) {
        self.errors.push(SemanticError::new(code, msg));
    }

    /// Check a block. Returns true if the block always exits via ret/stop/noret.
    pub fn check_block(&mut self, block: &Block, table: &mut SymbolTable) -> bool {
        let mut exited = false;
        for stmt in &block.stmts {
            self.check_stmt(stmt, table);
            if self.reached_end {
                exited = true;
            }
        }
        exited
    }

    pub fn check_stmt(&mut self, stmt: &Stmt, table: &mut SymbolTable) {
        if self.reached_end {
            match stmt {
                Stmt::Block(_) | Stmt::If(_) | Stmt::Match(_) => {
                    // Compound stmts can still be reachable inside, but warn at top level
                }
                _ => {
                    self.error(
                        "SEMA-0014",
                        "Unreachable statement after function exit (ret/stop/@panic)".into(),
                    );
                }
            }
        }

        match stmt {
            Stmt::Expr(e) => {
                let et = self.check_expr(e, table);
                if let Some(ref et) = et {
                    if !et.is_void() && !et.is_noret() {
                        // Warn on unused expression results
                    }
                    // SEMA-0010 / SEMA-0012: Error union result discarded without try/catch
                    if et.is_error_union() {
                        self.error(
                            "SEMA-0012",
                            format!(
                                "Error union result of type '{}' is discarded. Use 'try' block or 'catch' to handle it",
                                et.display()
                            ),
                        );
                    }
                }
            }
            Stmt::Var(v) => {
                self.check_var_decl(v, table);
            }
            Stmt::Ret(e) => {
                self.check_ret(e, table);
                self.reached_end = true;
            }
            Stmt::Stop => {
                if self.loop_depth == 0 {
                    self.error(
                        "SEMA-0008",
                        "Keyword 'stop' is only allowed inside an active loop body".into(),
                    );
                }
                self.reached_end = true;
            }
            Stmt::Next => {
                if self.loop_depth == 0 {
                    self.error(
                        "SEMA-0008",
                        "Keyword 'next' is only allowed inside an active loop body".into(),
                    );
                }
            }
            Stmt::If(if_) => {
                self.check_if(if_, table);
            }
            Stmt::Match(m) => {
                self.check_match(m, table);
            }
            Stmt::Loop(l) => {
                self.check_loop(l, table);
            }
            Stmt::Defer(e) => {
                self.check_expr(e, table);
            }
            Stmt::TryCatch(tc) => {
                self.check_try_catch(tc, table);
            }
            Stmt::Assign(target, op, value) => {
                self.check_assign(target, *op, value, table);
            }
            Stmt::Block(b) => {
                table.push_scope();
                let exited = self.check_block(b, table);
                if exited {
                    self.reached_end = true;
                }
                table.pop_scope();
            }
        }
    }

    fn check_var_decl(&mut self, v: &VarDecl, table: &mut SymbolTable) {
        let inferred = v
            .value
            .as_ref()
            .map(|val| self.check_expr(val, table))
            .unwrap_or(None);
        let declared = v.type_.as_ref().and_then(|t| {
            let resolve = |n: &str| table.lookup_type(n);
            resolve_ast_type(t, &resolve).ok()
        });

        let resolved_type = declared
            .clone()
            .or(inferred.clone())
            .unwrap_or(TypeInfo::Void);

        if let (Some(vt), Some(dt)) = (&inferred, &declared) {
            if !vt.is_assignable_to(dt) && !vt.is_noret() {
                self.error(
                    "SEMA-0001",
                    format!(
                        "Type mismatch: expected '{}', found type '{}'",
                        dt.display(),
                        vt.display()
                    ),
                );
            }
        }

        let sym = Symbol::Variable {
            type_: resolved_type,
            mutable: v.mutable,
            is_const: false,
        };
        if let Err(e) = table.insert(&v.name, sym) {
            self.error("SEMA-0003", e);
        }
    }

    fn check_ret(&mut self, expr: &Option<Expr>, table: &mut SymbolTable) {
        let ret = self.current_return_type.clone();
        match (expr, &ret) {
            (Some(e), Some(rt)) => {
                let et = self.check_expr(e, table);
                if let Some(ref et) = et {
                    if !et.is_assignable_to(rt) && !et.is_noret() {
                        self.error(
                            "SEMA-0009",
                            format!(
                                "Return type mismatch: expected '{}', found '{}'",
                                rt.display(),
                                et.display()
                            ),
                        );
                    }
                }
            }
            (Some(e), None) => {
                let et = self.check_expr(e, table);
                if let Some(ref et) = et {
                    // Infer return type from first return expression
                    if self.inferred_return_type.is_none() {
                        self.inferred_return_type = et.clone().into();
                    } else if let Some(ref inferred) = self.inferred_return_type {
                        if !et.is_assignable_to(inferred) && !et.is_noret() {
                            self.error(
                                "SEMA-0009",
                                format!(
                                    "Inferred return type mismatch: expected '{}', found '{}'",
                                    inferred.display(),
                                    et.display()
                                ),
                            );
                        }
                    }
                }
            }
            (None, Some(rt)) => {
                if !rt.is_void() && !rt.is_noret() {
                    self.error(
                        "SEMA-0009",
                        format!(
                            "Return type mismatch: expected '{}', found 'void'",
                            rt.display()
                        ),
                    );
                }
            }
            (None, None) => {}
        }
    }

    fn check_if(&mut self, if_: &If, table: &mut SymbolTable) {
        let ct = self.check_expr(&if_.cond, table);
        if let Some(ref ct) = ct {
            if !ct.is_bool() && !ct.is_noret() {
                self.error(
                    "SEMA-0007",
                    format!("Condition must be of type 'bool', found '{}'", ct.display()),
                );
            }
        }

        let pre_if = self.reached_end;

        let cond_always_true = matches!(&if_.cond, Expr::Literal(TokenKind::True, _));
        let cond_always_false = matches!(&if_.cond, Expr::Literal(TokenKind::False, _));

        if !if_.capture.is_empty() {
            if let Some(ref ct) = ct {
                if !ct.is_optional() && !ct.is_noret() {
                    self.error(
                        "SEMA-0019",
                        format!(
                            "Cannot capture optional value from non-optional type '{}'",
                            ct.display()
                        ),
                    );
                }
            }
            table.push_scope();
            for cap in &if_.capture {
                let inner = ct
                    .as_ref()
                    .and_then(|c| c.inner_optional().cloned())
                    .unwrap_or(TypeInfo::Void);
                let _ = table.insert(
                    cap,
                    Symbol::Variable {
                        type_: inner,
                        mutable: false,
                        is_const: false,
                    },
                );
            }
            self.reached_end = pre_if && !cond_always_false;
            self.check_block(&if_.then_block, table);
            let then_exits = self.reached_end;
            table.pop_scope();

            if let Some(ref else_stmt) = if_.else_block {
                self.reached_end = pre_if && !cond_always_true;
                self.check_stmt(else_stmt, table);
                let else_exits = self.reached_end;
                self.reached_end = pre_if || (then_exits && else_exits);
            } else {
                if cond_always_true && then_exits {
                    self.reached_end = true;
                } else {
                    self.reached_end = pre_if;
                }
            }
        } else {
            self.reached_end = pre_if && !cond_always_false;
            let then_exits = self.check_block(&if_.then_block, table);

            if let Some(ref else_stmt) = if_.else_block {
                self.reached_end = pre_if && !cond_always_true;
                self.check_stmt(else_stmt, table);
                let else_exits = self.reached_end;
                self.reached_end = pre_if
                    || (then_exits && else_exits)
                    || (cond_always_true && then_exits)
                    || (cond_always_false && else_exits);
            } else {
                if cond_always_true && then_exits {
                    self.reached_end = true;
                } else {
                    self.reached_end = pre_if;
                }
            }
        }
    }

    fn check_match(&mut self, m: &Match, table: &mut SymbolTable) {
        let tt = self.check_expr(&m.target, table);
        let has_wildcard = m
            .arms
            .iter()
            .any(|arm| matches!(arm.pattern, Pattern::Wildcard));
        if !has_wildcard && !m.arms.is_empty() {
            self.error(
                "SEMA-0016",
                "Match patterns are not exhaustive. Wildcard pattern '_' is required".into(),
            );
        }

        let pre_match = self.reached_end;
        let mut all_arms_exit = true;
        let mut any_arm = false;

        for arm in &m.arms {
            any_arm = true;
            self.reached_end = pre_match;
            if !arm.capture.is_empty() {
                if let Some(ref tt) = tt {
                    let is_enum_variant = matches!(&arm.pattern, Pattern::EnumVariant(..));
                    if !tt.is_enum() && !tt.is_optional() && !is_enum_variant {
                        self.error(
                            "SEMA-0019",
                            format!(
                                "Cannot capture value from non-optional, non-enum type '{}'",
                                tt.display()
                            ),
                        );
                    }
                }
                table.push_scope();
                for cap in &arm.capture {
                    let inner = tt
                        .as_ref()
                        .and_then(|t| {
                            if t.is_optional() {
                                t.inner_optional().cloned()
                            } else {
                                Some(t.clone())
                            }
                        })
                        .unwrap_or(TypeInfo::Void);
                    let _ = table.insert(
                        cap,
                        Symbol::Variable {
                            type_: inner,
                            mutable: false,
                            is_const: false,
                        },
                    );
                }
                self.check_expr(&arm.value, table);
                table.pop_scope();
            } else {
                self.check_expr(&arm.value, table);
            }
            // Check if this arm's path exited (ret/stop/noret)
            // Note: check_expr itself may set reached_end
            if !self.reached_end {
                all_arms_exit = false;
            }
        }

        // Match always exits only if all arms exit (including wildcard)
        self.reached_end = if any_arm {
            pre_match || (all_arms_exit && has_wildcard)
        } else {
            pre_match
        };
    }

    fn check_loop(&mut self, l: &Loop, table: &mut SymbolTable) {
        // Infer capture types from loop conditions before checking the body
        let mut capture_types: Vec<TypeInfo> = Vec::new();
        let has_captures = !l.captures.is_empty();

        for (ci, cond) in l.conds.iter().enumerate() {
            let ct = self.check_expr(cond, table);
            if let Some(ref ct) = ct {
                if has_captures {
                    // For-each loop: conditions are iterables, not booleans
                    let elem_type = match ct {
                        TypeInfo::Array(inner, _) => (**inner).clone(),
                        TypeInfo::Ref(_, inner) => match inner.as_ref() {
                            TypeInfo::Array(inner2, _) => (**inner2).clone(),
                            _ => TypeInfo::AnyType,
                        },
                        TypeInfo::Str => TypeInfo::Char,
                        TypeInfo::Optional(inner) => (**inner).clone(),
                        _ => TypeInfo::AnyType,
                    };
                    // Fill types for any extra captures beyond conditions
                    while capture_types.len() <= ci {
                        capture_types.push(TypeInfo::AnyType);
                    }
                    capture_types[ci] = elem_type;
                } else {
                    // While loop: conditions must be bool
                    if !ct.is_bool() && !ct.is_noret() {
                        self.error(
                            "SEMA-0007",
                            format!("Condition must be of type 'bool', found '{}'", ct.display()),
                        );
                    }
                }
            } else {
                capture_types.push(TypeInfo::AnyType);
            }
        }

        self.loop_depth += 1;

        // Save pre-loop exit state — a loop may not execute at all
        let pre_loop = self.reached_end;

        if !l.captures.is_empty() {
            table.push_scope();
            for (i, cap) in l.captures.iter().enumerate() {
                let cap_type = if i < capture_types.len() {
                    capture_types[i].clone()
                } else {
                    TypeInfo::AnyType
                };
                let _ = table.insert(
                    &cap.name,
                    Symbol::Variable {
                        type_: cap_type,
                        mutable: cap.mutable,
                        is_const: false,
                    },
                );
            }
            self.check_block(&l.body, table);
            table.pop_scope();
        } else {
            self.check_block(&l.body, table);
        }

        // Loop may not execute, so restore to pre-loop state
        self.reached_end = pre_loop;

        self.loop_depth -= 1;
    }

    fn check_try_catch(&mut self, tc: &TryCatch, table: &mut SymbolTable) {
        // Save reached_end — try and catch are separate paths
        let saved_reached = self.reached_end;
        self.reached_end = false;
        self.check_block(&tc.try_body, table);
        let try_reached = self.reached_end;

        self.reached_end = false;
        if !tc.capture.is_empty() {
            table.push_scope();
            for cap in &tc.capture {
                let _ = table.insert(
                    cap,
                    Symbol::Variable {
                        type_: TypeInfo::Builtin("Error".into()),
                        mutable: false,
                        is_const: false,
                    },
                );
            }
            self.check_block(&tc.catch_body, table);
            let catch_reached = self.reached_end;
            table.pop_scope();
            self.reached_end = saved_reached || (try_reached && catch_reached);
        } else {
            self.check_block(&tc.catch_body, table);
            let catch_reached = self.reached_end;
            self.reached_end = saved_reached || (try_reached && catch_reached);
        }
    }

    fn check_assign(
        &mut self,
        target: &Expr,
        _op: AssignOp,
        value: &Expr,
        table: &mut SymbolTable,
    ) {
        let tt = self.check_expr(target, table);
        let vt = self.check_expr(value, table);

        if let Expr::Ident(name) = target {
            if let Some(sym) = table.lookup(name) {
                if sym.is_const() {
                    self.error(
                        "SEMA-0005",
                        format!("Cannot assign to compile-time constant '{}'", name),
                    );
                    return;
                }
                if !sym.is_mutable() {
                    self.error(
                        "SEMA-0004",
                        format!("Cannot mutate immutable variable '{}'", name),
                    );
                    return;
                }
            }
        }

        if let Expr::Field(obj, fname) = target {
            let ot = self.check_expr(obj, table);
            if let Some(TypeInfo::Struct(_, fields)) = ot {
                if let Some((_, _, pub_)) = fields.iter().find(|(n, _, _)| n == fname) {
                    if !pub_ {
                        self.error("SEMA-0015", format!("Field '{}' is private", fname));
                    }
                }
            }
        }

        if let (Some(tt), Some(vt)) = (&tt, &vt) {
            if !vt.is_assignable_to(tt) && !vt.is_noret() {
                self.error(
                    "SEMA-0001",
                    format!(
                        "Type mismatch: expected '{}', found type '{}'",
                        tt.display(),
                        vt.display()
                    ),
                );
            }
        }
    }

    pub fn check_expr(&mut self, expr: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
        let result = match expr {
            Expr::Literal(kind, val) => Some(literal_to_type(kind, val)),
            Expr::Ident(name) => {
                if name == "_" {
                    return None;
                }
                if name.starts_with('@') {
                    let bn = name.trim_start_matches('@');
                    return Some(TypeInfo::Builtin(bn.to_string()));
                }
                match table.lookup(name) {
                    Some(sym) if matches!(sym, Symbol::BuiltinFn { .. }) => None,
                    Some(sym) => sym.get_type(),
                    None => {
                        self.error(
                            "SEMA-0002",
                            format!("Undefined symbol '{}' in current scope", name),
                        );
                        None
                    }
                }
            }
            Expr::Binary(op, lhs, rhs) => self.check_binary(*op, lhs, rhs, table),
            Expr::Unary(op, e) => self.check_unary(*op, e, table),
            Expr::Call(callee, args) => self.check_call(callee, args, table),
            Expr::Field(obj, name) => self.check_field(obj, name, table),
            Expr::Index(obj, idx) => self.check_index(obj, idx, table),
            Expr::Slice(obj, start, end, _) => {
                self.check_expr(obj, table);
                self.check_expr(start, table);
                self.check_expr(end, table);
                Some(TypeInfo::Array(Box::new(TypeInfo::Void), None))
            }
            Expr::StructInit(name, fields) => self.check_struct_init(name, fields, table),
            Expr::ArrayInit(elems) => {
                let mut elem_type = None;
                for e in elems {
                    let et = self.check_expr(e, table);
                    if let Some(ref et) = et {
                        if elem_type.is_none() {
                            elem_type = Some(et.clone());
                        }
                    }
                }
                Some(TypeInfo::Array(
                    Box::new(elem_type.unwrap_or(TypeInfo::Void)),
                    Some(elems.len() as u64),
                ))
            }
            Expr::ArrayInitFill(val, count) => {
                let vt = self.check_expr(val, table);
                self.check_expr(count, table);
                Some(TypeInfo::Array(
                    Box::new(vt.unwrap_or(TypeInfo::Void)),
                    None,
                ))
            }
            Expr::Deref(e) => self.check_deref(e, table),
            Expr::Block(b) => {
                table.push_scope();
                let exited = self.check_block(b, table);
                table.pop_scope();
                if exited {
                    self.reached_end = true;
                }
                Some(TypeInfo::Void)
            }
            Expr::Paren(e) => self.check_expr(e, table),
            Expr::AtMethod(obj, method) => {
                self.check_expr(obj, table);
                Some(TypeInfo::Builtin(method.clone()))
            }
            Expr::Ret(value) => {
                self.reached_end = true;
                if let Some(val) = value {
                    self.check_expr(val, table)
                } else {
                    None
                }
            }
            Expr::Fn(fndecl) => {
                let param_types: Vec<TypeInfo> = fndecl
                    .params
                    .iter()
                    .map(|p| ast_type_to_typeinfo(&p.type_))
                    .collect();
                let ret_type = fndecl
                    .return_
                    .as_ref()
                    .map(ast_type_to_typeinfo)
                    .unwrap_or(TypeInfo::Void);
                Some(TypeInfo::Fn(param_types, Box::new(ret_type)))
            }
            Expr::Catch(e, _capture, body) => {
                let et = self.check_expr(e, table);
                let body_exited = self.check_block(body, table);
                if body_exited {
                    self.reached_end = true;
                }
                if let Some(TypeInfo::ErrorUnion(_, ok)) = et {
                    Some(*ok)
                } else {
                    et
                }
            }
            Expr::MapLiteral(_pairs) => Some(TypeInfo::Builtin("map".into())),
        };

        // If the result type is Noret, mark the path as terminated
        if let Some(ref t) = result {
            if t.is_noret() {
                self.reached_end = true;
            }
        }

        result
    }

    fn check_binary(
        &mut self,
        op: BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        table: &mut SymbolTable,
    ) -> Option<TypeInfo> {
        let lt = self.check_expr(lhs, table);
        let rt = self.check_expr(rhs, table);

        let both_numeric = || -> bool {
            matches!((&lt, &rt), (Some(l), Some(r)) if l.is_numeric() && r.is_numeric())
        };
        let both_integer = || -> bool {
            matches!((&lt, &rt), (Some(l), Some(r)) if l.is_integer() && r.is_integer())
        };
        let both_bool =
            || -> bool { matches!((&lt, &rt), (Some(l), Some(r)) if l.is_bool() && r.is_bool()) };

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if !both_numeric()
                    && !lt.as_ref().is_some_and(|t| t.is_noret())
                    && !rt.as_ref().is_some_and(|t| t.is_noret())
                {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error(
                        "SEMA-0012",
                        format!(
                            "Operator '{}' is not defined for types '{}' and '{}'",
                            binop_str(&op),
                            ls,
                            rs
                        ),
                    );
                }
                lt
            }
            BinaryOp::AddAssign
            | BinaryOp::SubAssign
            | BinaryOp::MulAssign
            | BinaryOp::DivAssign
            | BinaryOp::ModAssign => {
                if !both_numeric() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error(
                        "SEMA-0012",
                        format!(
                            "Operator '{}' is not defined for types '{}' and '{}'",
                            binop_str(&op),
                            ls,
                            rs
                        ),
                    );
                }
                lt
            }
            BinaryOp::Eq | BinaryOp::Ne => Some(TypeInfo::Bool),
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                if !both_numeric() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error(
                        "SEMA-0012",
                        format!(
                            "Operator '{}' is not defined for types '{}' and '{}'",
                            binop_str(&op),
                            ls,
                            rs
                        ),
                    );
                }
                Some(TypeInfo::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                if !both_bool() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error(
                        "SEMA-0012",
                        format!(
                            "Operator '{}' is not defined for types '{}' and '{}'",
                            binop_str(&op),
                            ls,
                            rs
                        ),
                    );
                }
                Some(TypeInfo::Bool)
            }
            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::Shl
            | BinaryOp::Shr => {
                if !both_integer() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error(
                        "SEMA-0012",
                        format!(
                            "Operator '{}' is not defined for types '{}' and '{}'",
                            binop_str(&op),
                            ls,
                            rs
                        ),
                    );
                }
                lt
            }
            BinaryOp::Assign | BinaryOp::ColonEq => rt,
            BinaryOp::Range | BinaryOp::RangeInclusive => {
                if !both_numeric() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error(
                        "SEMA-0012",
                        format!(
                            "Range operator is not defined for types '{}' and '{}'",
                            ls, rs
                        ),
                    );
                }
                lt
            }
        }
    }

    fn check_unary(
        &mut self,
        op: UnaryOp,
        expr: &Expr,
        table: &mut SymbolTable,
    ) -> Option<TypeInfo> {
        let inner = self.check_expr(expr, table);
        match op {
            UnaryOp::Neg => {
                if let Some(ref t) = inner {
                    if !t.is_numeric() {
                        self.error(
                            "SEMA-0013",
                            format!("Operator '-' is not defined for type '{}'", t.display()),
                        );
                    }
                }
                inner
            }
            UnaryOp::Not => {
                if let Some(ref t) = inner {
                    if !t.is_bool() {
                        self.error(
                            "SEMA-0013",
                            format!("Operator '!' is not defined for type '{}'", t.display()),
                        );
                    }
                }
                Some(TypeInfo::Bool)
            }
            UnaryOp::BitNot => {
                if let Some(ref t) = inner {
                    if !t.is_integer() {
                        self.error(
                            "SEMA-0013",
                            format!("Operator '~' is not defined for type '{}'", t.display()),
                        );
                    }
                }
                inner
            }
            UnaryOp::Ref => inner.map(|t| TypeInfo::Ref(false, Box::new(t))),
            UnaryOp::RefMut => {
                if let Expr::Ident(name) = expr {
                    if let Some(sym) = table.lookup(name) {
                        if !sym.is_mutable() {
                            self.error(
                                "SEMA-0006",
                                format!(
                                    "Cannot take mutable reference to immutable value '{}'",
                                    name
                                ),
                            );
                        }
                    }
                }
                inner.map(|t| TypeInfo::Ref(true, Box::new(t)))
            }
            UnaryOp::Deref => self.check_deref(expr, table),
            UnaryOp::Optional => inner.map(|t| TypeInfo::Optional(Box::new(t))),
        }
    }

    fn check_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        table: &mut SymbolTable,
    ) -> Option<TypeInfo> {
        if let Expr::Ident(name) = callee {
            if name.starts_with('@') {
                return self.check_builtin(name, args, table);
            }
        }

        let ct = self.check_expr(callee, table);

        // SEMA-0020: Generic parameter mismatch check
        if let Expr::Ident(name) = callee {
            if let Some(sym) = table.lookup(name) {
                let generic_count = match sym {
                    Symbol::Variable { .. } | Symbol::Parameter { .. } => 0,
                    _ => match sym {
                        Symbol::BuiltinFn { .. } | Symbol::TypeAlias(_) | Symbol::ErrorSet(_) => 0,
                        Symbol::Function(fs) => fs.generics.len(),
                        Symbol::StructType(ss) => ss.generics.len(),
                        Symbol::UnionType(us) => us.generics.len(),
                        Symbol::EnumType(es) => es.generics.len(),
                        Symbol::Behave(bs) => bs.generics.len(),
                        _ => 0,
                    },
                };
                if generic_count > 0 {
                    // Generic function called without explicit type arguments.
                    // Generics must be inferrable from the regular arguments.
                    // If the function has zero parameters (only generic return type),
                    // the return type cannot be inferred from arguments alone.
                    let param_count = match sym {
                        Symbol::Function(fs) => Some(fs.params.len()),
                        Symbol::StructType(ss) => Some(ss.fields.len()),
                        Symbol::EnumType(es) => Some(es.variants.len()),
                        _ => None,
                    };
                    let args_provided = args.len();
                    if let Some(pc) = param_count {
                        if pc == 0 && generic_count > 0 {
                            self.error(
                                "SEMA-0020",
                                format!(
                                    "Generic parameters for '{}' cannot be inferred: no regular parameters to infer from. \
                                     Expected {} type argument(s), got 0 (inference)",
                                    name, generic_count
                                ),
                            );
                        }
                    } else if args_provided < generic_count {
                        self.error(
                            "SEMA-0020",
                            format!(
                                "Generic parameters mismatch: expected at least {} type arguments (inferrable), \
                                 but function '{}' has {} generic parameters with insufficient context",
                                generic_count, name, generic_count
                            ),
                        );
                    }
                }
            }
        }

        match ct {
            Some(TypeInfo::Fn(params, ret)) => {
                if args.len() != params.len() {
                    self.error(
                        "SEMA-0014",
                        format!("Expected {} arguments, got {}", params.len(), args.len()),
                    );
                    return Some(*ret);
                }
                for (i, arg) in args.iter().enumerate() {
                    let at = self.check_expr(arg, table);
                    if let Some(ref at) = at {
                        if !at.is_assignable_to(&params[i]) && !at.is_noret() {
                            self.error(
                                "SEMA-0001",
                                format!(
                                    "Argument {} type mismatch: expected '{}', found '{}'",
                                    i + 1,
                                    params[i].display(),
                                    at.display()
                                ),
                            );
                        }
                    }
                }
                Some(*ret)
            }
            Some(t) if t.is_noret() => Some(t),
            Some(t) => {
                self.error(
                    "SEMA-0014",
                    format!("Cannot call non-callable type '{}'", t.display()),
                );
                None
            }
            None => {
                if let Expr::Ident(name) = callee {
                    if table.lookup(name).is_some_and(|s| s.is_function()) {
                        return None;
                    } else if name != "_" && !name.starts_with('@') {
                        self.error("SEMA-0014", "Cannot call non-callable type".into());
                    }
                }
                None
            }
        }
    }

    fn check_builtin(
        &mut self,
        name: &str,
        args: &[Expr],
        table: &mut SymbolTable,
    ) -> Option<TypeInfo> {
        let bn = name.trim_start_matches('@');
        match bn {
            "TypeOf" => {
                if args.is_empty() {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@TypeOf' requires at least 1 argument".into(),
                    );
                    return Some(TypeInfo::TypeMeta);
                }
                self.check_expr(&args[0], table);
                Some(TypeInfo::TypeMeta)
            }
            "SizeOf" | "AlignOf" | "TypeName" | "EnumCount" => {
                if args.len() != 1 {
                    self.error(
                        "SEMA-0017",
                        format!("Builtin '@{}' requires 1 argument", bn),
                    );
                }
                Some(TypeInfo::Int(IntWidth::W64, false))
            }
            "as" | "bitCast" => {
                if args.len() != 2 {
                    self.error(
                        "SEMA-0017",
                        format!("Builtin '@{}' requires 2 arguments", bn),
                    );
                    return None;
                }
                self.check_expr(&args[1], table);
                Some(TypeInfo::Int(IntWidth::W32, true))
            }
            "ptrCast" | "intToPtr" => {
                if args.len() != 2 {
                    self.error(
                        "SEMA-0017",
                        format!("Builtin '@{}' requires 2 arguments", bn),
                    );
                }
                Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)))
            }
            "ptrToInt" => {
                if args.len() != 1 {
                    self.error(
                        "SEMA-0017",
                        format!("Builtin '@{}' requires 1 argument", bn),
                    );
                }
                Some(TypeInfo::Int(IntWidth::Arch, false))
            }
            "memcpy" | "memset" | "memmove" => {
                if args.len() != 3 {
                    self.error(
                        "SEMA-0017",
                        format!("Builtin '@{}' requires 3 arguments (dest, src, count)", bn),
                    );
                    for a in args {
                        self.check_expr(a, table);
                    }
                    return Some(TypeInfo::Void);
                }
                let dest_type = self.check_expr(&args[0], table);
                let src_type = self.check_expr(&args[1], table);
                let count_type = self.check_expr(&args[2], table);
                // Validate dest/src are pointers or references
                if let Some(ref dt) = dest_type {
                    if !dt.is_pointer() && !dt.is_reference() && !dt.is_noret() {
                        self.error(
                            "SEMA-0023",
                            format!(
                                "Builtin '@{}' first argument must be a pointer or reference, found '{}'",
                                bn, dt.display()
                            ),
                        );
                    }
                }
                if let Some(ref st) = src_type {
                    if !st.is_pointer() && !st.is_reference() && !st.is_noret() {
                        self.error(
                            "SEMA-0023",
                            format!(
                                "Builtin '@{}' second argument must be a pointer or reference, found '{}'",
                                bn, st.display()
                            ),
                        );
                    }
                }
                // Validate count is integer
                if let Some(ref ct) = count_type {
                    if !ct.is_integer() && !ct.is_noret() {
                        self.error(
                            "SEMA-0023",
                            format!(
                                "Builtin '@{}' third argument must be an integer, found '{}'",
                                bn,
                                ct.display()
                            ),
                        );
                    }
                }
                Some(TypeInfo::Void)
            }
            "pageAlloc" => {
                if args.len() != 1 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@pageAlloc' requires 1 argument".into(),
                    );
                }
                Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)))
            }
            "pageFree" => {
                if args.len() != 2 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@pageFree' requires 2 arguments".into(),
                    );
                }
                for a in args {
                    self.check_expr(a, table);
                }
                Some(TypeInfo::Void)
            }
            "comptimeDefaultAllocator" => {
                if !args.is_empty() {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@comptimeDefaultAllocator' takes no arguments".into(),
                    );
                }
                Some(TypeInfo::Builtin("Allocator".into()))
            }
            "comptime" | "compileLog" => {
                for a in args {
                    self.check_expr(a, table);
                }
                Some(TypeInfo::Void)
            }
            "compileError" => {
                if args.len() != 1 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@compileError' requires 1 argument".into(),
                    );
                }
                Some(TypeInfo::Noret)
            }
            "embedFile" => {
                if args.len() != 1 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@embedFile' requires 1 argument".into(),
                    );
                }
                Some(TypeInfo::Array(
                    Box::new(TypeInfo::Int(IntWidth::W8, false)),
                    None,
                ))
            }
            "panic" => {
                if args.len() != 1 {
                    self.error("SEMA-0017", "Builtin '@panic' requires 1 argument".into());
                }
                Some(TypeInfo::Noret)
            }
            "breakpoint" | "trap" => {
                if !args.is_empty() {
                    self.error("SEMA-0017", format!("Builtin '@{}' takes no arguments", bn));
                }
                Some(TypeInfo::Noret)
            }
            "sysCall" => {
                if args.is_empty() {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@sysCall' requires at least 1 argument".into(),
                    );
                }
                Some(TypeInfo::Void)
            }
            "str.from_raw" | "str.from" => {
                if !args.is_empty() {
                    self.check_expr(&args[0], table);
                }
                Some(TypeInfo::Str)
            }
            "vec" | "set" => {
                // SEMA-0010: Runtime collection init requires explicit allocator
                if args.is_empty() {
                    self.error("SEMA-0010", format!("Heap collection type '@{}' requires an explicit allocator in runtime context", bn));
                }
                for a in args {
                    self.check_expr(a, table);
                }
                Some(TypeInfo::Builtin(bn.to_string()))
            }
            "map" => {
                // SEMA-0010: Runtime map init requires explicit allocator
                if args.is_empty() {
                    self.error("SEMA-0010", "Heap collection type '@map' requires an explicit allocator in runtime context".into());
                }
                Some(TypeInfo::Builtin("map".into()))
            }
            "addWithOverflow" | "subWithOverflow" | "mulWithOverflow" => {
                if args.len() != 2 {
                    self.error(
                        "SEMA-0017",
                        format!("Builtin '@{}' requires 2 arguments", bn),
                    );
                }
                let t = self.check_expr(&args[0], table);
                self.check_expr(&args[1], table);
                t.map(|t| TypeInfo::ErrorUnion(None, Box::new(t)))
            }
            "ctz" | "clz" | "popCount" | "bswap" => {
                if args.len() != 1 {
                    self.error(
                        "SEMA-0017",
                        format!("Builtin '@{}' requires 1 argument", bn),
                    );
                }
                self.check_expr(&args[0], table);
                Some(TypeInfo::Int(IntWidth::W32, false))
            }
            "atomicLoad" => {
                if args.len() != 2 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@atomicLoad' requires 2 arguments".into(),
                    );
                }
                self.check_expr(&args[0], table);
                self.check_expr(&args[1], table);
                Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)))
            }
            "atomicStore" => {
                if args.len() != 3 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@atomicStore' requires 3 arguments".into(),
                    );
                }
                for a in args {
                    self.check_expr(a, table);
                }
                Some(TypeInfo::Void)
            }
            "cmpxchg" => {
                if args.len() != 5 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@cmpxchg' requires 5 arguments".into(),
                    );
                }
                for a in args {
                    self.check_expr(a, table);
                }
                Some(TypeInfo::ErrorUnion(
                    None,
                    Box::new(TypeInfo::Pointer(Box::new(TypeInfo::Void))),
                ))
            }
            "Fields" => {
                if args.len() != 1 {
                    self.error("SEMA-0017", "Builtin '@Fields' requires 1 argument".into());
                }
                Some(TypeInfo::Array(Box::new(TypeInfo::TypeMeta), None))
            }
            "assert" => {
                if args.is_empty() || args.len() > 2 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@assert' requires 1 or 2 arguments (condition, message?)".into(),
                    );
                }
                if !args.is_empty() {
                    let ct = self.check_expr(&args[0], table);
                    if let Some(ref t) = ct {
                        if !t.is_bool() && !t.is_noret() {
                            self.error(
                                "SEMA-0017",
                                format!(
                                    "Builtin '@assert' first argument must be bool, found '{}'",
                                    t.display()
                                ),
                            );
                        }
                    }
                }
                if args.len() > 1 {
                    self.check_expr(&args[1], table);
                }
                Some(TypeInfo::Void)
            }
            "assertEq" => {
                if args.len() < 2 || args.len() > 3 {
                    self.error(
                        "SEMA-0017",
                        "Builtin '@assertEq' requires 2 or 3 arguments (left, right, message?)"
                            .into(),
                    );
                }
                if args.len() >= 2 {
                    let lt = self.check_expr(&args[0], table);
                    let rt = self.check_expr(&args[1], table);
                    if let (Some(l), Some(r)) = (&lt, &rt) {
                        if !l.is_assignable_to(r) && !l.is_noret() && !r.is_noret() {
                            self.error(
                                "SEMA-0017",
                                format!(
                                    "Builtin '@assertEq' arguments have incompatible types: '{}' and '{}'",
                                    l.display(),
                                    r.display()
                                ),
                            );
                        }
                    }
                }
                if args.len() > 2 {
                    self.check_expr(&args[2], table);
                }
                Some(TypeInfo::Void)
            }
            _ => {
                self.error("SEMA-0017", format!("Unknown builtin '@{}'", bn));
                None
            }
        }
    }

    fn check_field(&mut self, obj: &Expr, name: &str, table: &mut SymbolTable) -> Option<TypeInfo> {
        let ot = self.check_expr(obj, table);
        match ot {
            Some(TypeInfo::Struct(_, ref fields)) => {
                if let Some((_, ft, pub_)) = fields.iter().find(|(n, _, _)| n == name) {
                    if !pub_ {
                        self.error("SEMA-0015", format!("Field '{}' is private", name));
                    }
                    Some(ft.clone())
                } else {
                    self.error(
                        "SEMA-0015",
                        format!("Field '{}' does not exist on this type", name),
                    );
                    None
                }
            }
            Some(TypeInfo::Enum(_, ref variants)) => {
                if variants.iter().any(|(n, _)| n == name) {
                    Some(TypeInfo::Enum(name.to_string(), variants.clone()))
                } else {
                    self.error(
                        "SEMA-0015",
                        format!("Variant '{}' does not exist on this enum", name),
                    );
                    None
                }
            }
            Some(TypeInfo::Ref(_, ref inner)) => match inner.as_ref() {
                TypeInfo::Struct(sname, fields) => {
                    if let Some((_, ft, _)) = fields.iter().find(|(n, _, _)| n == name) {
                        Some(ft.clone())
                    } else {
                        self.error(
                            "SEMA-0015",
                            format!("Field '{}' does not exist on type '{}'", name, sname),
                        );
                        None
                    }
                }
                _ => {
                    self.error(
                        "SEMA-0015",
                        format!(
                            "Field '{}' does not exist on type '{}'",
                            name,
                            inner.display()
                        ),
                    );
                    None
                }
            },
            Some(t) => {
                self.error(
                    "SEMA-0015",
                    format!("Field '{}' does not exist on type '{}'", name, t.display()),
                );
                None
            }
            None => None,
        }
    }

    fn check_index(&mut self, obj: &Expr, idx: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
        let ot = self.check_expr(obj, table);
        let it = self.check_expr(idx, table);
        if let Some(ref it) = it {
            if !it.is_integer() && !it.is_noret() {
                self.error("SEMA-0012", "Array index must be an integer type".into());
            }
        }
        match ot {
            Some(TypeInfo::Array(inner, _)) => Some(*inner),
            Some(TypeInfo::Str) | Some(TypeInfo::Char) => Some(TypeInfo::Char),
            Some(TypeInfo::Pointer(inner)) => Some(*inner),
            Some(t) => {
                self.error("SEMA-0012", format!("Cannot index type '{}'", t.display()));
                None
            }
            None => None,
        }
    }

    fn check_deref(&mut self, expr: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
        let inner = self.check_expr(expr, table);
        match inner {
            Some(TypeInfo::Ref(_, inner)) => Some(*inner),
            Some(TypeInfo::Pointer(inner)) => Some(*inner),
            Some(ref t) => {
                self.error(
                    "SEMA-0018",
                    format!(
                        "Cannot dereference non-pointer, non-reference type '{}'",
                        t.display()
                    ),
                );
                None
            }
            None => None,
        }
    }

    fn check_struct_init(
        &mut self,
        name: &str,
        fields: &[FieldInit],
        table: &mut SymbolTable,
    ) -> Option<TypeInfo> {
        match table.lookup_type(name) {
            Some(TypeInfo::Struct(sname, struct_fields)) => {
                for f in fields {
                    let ft = self.check_expr(&f.value, table);
                    if let Some((_, declared_type, _)) =
                        struct_fields.iter().find(|(n, _, _)| n == &f.name)
                    {
                        if let Some(ref ft) = ft {
                            if !ft.is_assignable_to(declared_type) && !ft.is_noret() {
                                self.error(
                                    "SEMA-0001",
                                    format!(
                                        "Type mismatch in field '{}' of '{}': expected '{}', found '{}'",
                                        f.name, sname, declared_type.display(), ft.display()
                                    ),
                                );
                            }
                        }
                    } else {
                        self.error(
                            "SEMA-0015",
                            format!("Field '{}' does not exist on struct '{}'", f.name, sname),
                        );
                    }
                }
                for (fname, _, _) in &struct_fields {
                    if !fields.iter().any(|f| &f.name == fname) {
                        self.error(
                            "SEMA-0015",
                            format!(
                                "Missing field '{}' in struct initializer for '{}'",
                                fname, sname
                            ),
                        );
                    }
                }
                Some(TypeInfo::Struct(sname, struct_fields))
            }
            Some(t) => {
                self.error(
                    "SEMA-0015",
                    format!(
                        "Cannot initialize non-struct type '{}' with struct syntax",
                        t.display()
                    ),
                );
                None
            }
            None => {
                self.error(
                    "SEMA-0002",
                    format!("Undefined type '{}' for struct initialization", name),
                );
                None
            }
        }
    }

    pub fn check_behave_impl(
        &mut self,
        type_name: &str,
        behave_name: &str,
        type_methods: &[FnSymbol],
        table: &SymbolTable,
    ) {
        match table.lookup(behave_name) {
            Some(Symbol::Behave(b)) => {
                for bm in &b.methods {
                    let matching = type_methods.iter().find(|tm| tm.name == bm.name);
                    match matching {
                        Some(tm) => {
                            // Check parameter count (excluding self param from both)
                            let behave_param_count = bm.params.len().saturating_sub(1);
                            let impl_param_count = tm.params.len().saturating_sub(1);
                            if behave_param_count != impl_param_count {
                                self.error(
                                    "SEMA-0011",
                                    format!(
                                        "Struct '{}' method '{}' parameter count mismatch for behave '{}': expected {} params, found {}",
                                        type_name, bm.name, behave_name, behave_param_count, impl_param_count
                                    ),
                                );
                            }
                            // Check return type compatibility
                            if let (Some(bt), Some(it)) = (&bm.return_, &tm.return_) {
                                if !it.is_assignable_to(bt) && !it.is_noret() {
                                    self.error(
                                        "SEMA-0011",
                                        format!(
                                            "Struct '{}' method '{}' return type mismatch for behave '{}': expected '{}', found '{}'",
                                            type_name, bm.name, behave_name, bt.display(), it.display()
                                        ),
                                    );
                                }
                            }
                        }
                        None => {
                            self.error(
                                "SEMA-0011",
                                format!(
                                    "Struct '{}' does not implement behave trait method '{}'",
                                    type_name, bm.name
                                ),
                            );
                        }
                    }
                }
            }
            _ => {
                self.error(
                    "SEMA-0002",
                    format!("Undefined behaviour '{}'", behave_name),
                );
            }
        }
    }
}

fn binop_str(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::AddAssign => "+=",
        BinaryOp::SubAssign => "-=",
        BinaryOp::MulAssign => "*=",
        BinaryOp::DivAssign => "/=",
        BinaryOp::ModAssign => "%=",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::Assign => "=",
        BinaryOp::ColonEq => ":=",
        BinaryOp::Range => "..",
        BinaryOp::RangeInclusive => "..=",
    }
}

fn ast_type_to_typeinfo(t: &Type) -> TypeInfo {
    match t {
        Type::Primitive(k) => match k {
            TokenKind::Void => TypeInfo::Void,
            TokenKind::Bool => TypeInfo::Bool,
            TokenKind::U8 => TypeInfo::Int(IntWidth::W8, false),
            TokenKind::U16 => TypeInfo::Int(IntWidth::W16, false),
            TokenKind::U32 => TypeInfo::Int(IntWidth::W32, false),
            TokenKind::U64 => TypeInfo::Int(IntWidth::W64, false),
            TokenKind::I8 => TypeInfo::Int(IntWidth::W8, true),
            TokenKind::I16 => TypeInfo::Int(IntWidth::W16, true),
            TokenKind::I32 => TypeInfo::Int(IntWidth::W32, true),
            TokenKind::I64 => TypeInfo::Int(IntWidth::W64, true),
            TokenKind::F32 => TypeInfo::Float(FloatWidth::W32),
            TokenKind::F64 => TypeInfo::Float(FloatWidth::W64),
            TokenKind::Isize => TypeInfo::Int(IntWidth::Arch, true),
            TokenKind::Usize => TypeInfo::Int(IntWidth::Arch, false),
            TokenKind::Noret => TypeInfo::Noret,
            _ => TypeInfo::Void,
        },
        Type::Named(n) => TypeInfo::Builtin(n.clone()),
        Type::Ref(mut_, inner) => TypeInfo::Ref(*mut_, Box::new(ast_type_to_typeinfo(inner))),
        Type::Pointer(inner) => TypeInfo::Pointer(Box::new(ast_type_to_typeinfo(inner))),
        Type::Optional(inner) => TypeInfo::Optional(Box::new(ast_type_to_typeinfo(inner))),
        Type::ErrorUnion(err, ok) => TypeInfo::ErrorUnion(
            err.as_ref().map(|e| Box::new(ast_type_to_typeinfo(e))),
            Box::new(ast_type_to_typeinfo(ok)),
        ),
        Type::Array(inner, size) => {
            let sz = size.as_ref().and_then(|s| match s.as_ref() {
                Expr::Literal(_, v) => v.parse::<u64>().ok(),
                _ => None,
            });
            TypeInfo::Array(Box::new(ast_type_to_typeinfo(inner)), sz)
        }
        Type::Slice(inner) => TypeInfo::Array(Box::new(ast_type_to_typeinfo(inner)), None),
        Type::Fn(params, ret) => {
            let ps: Vec<TypeInfo> = params.iter().map(ast_type_to_typeinfo).collect();
            let r = ast_type_to_typeinfo(ret);
            TypeInfo::Fn(ps, Box::new(r))
        }
        Type::Builtin(name) => TypeInfo::Builtin(name.clone()),
    }
}
