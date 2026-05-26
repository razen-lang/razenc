use crate::ast::*;
use crate::sema::types::*;
use crate::sema::scope::*;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub code: String,
    pub message: String,
}

impl SemanticError {
    pub fn new(code: &str, message: String) -> Self {
        SemanticError { code: code.to_string(), message }
    }
}

pub struct TypeChecker {
    pub errors: Vec<SemanticError>,
    pub loop_depth: usize,
    pub current_return_type: Option<TypeInfo>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker { errors: Vec::new(), loop_depth: 0, current_return_type: None }
    }

    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }

    fn error(&mut self, code: &str, msg: String) {
        self.errors.push(SemanticError::new(code, msg));
    }

    pub fn check_block(&mut self, block: &Block, table: &mut SymbolTable) {
        for stmt in &block.stmts {
            self.check_stmt(stmt, table);
        }
    }

    pub fn check_stmt(&mut self, stmt: &Stmt, table: &mut SymbolTable) {
        match stmt {
            Stmt::Expr(e) => { self.check_expr(e, table); }
            Stmt::Var(v) => { self.check_var_decl(v, table); }
            Stmt::Ret(e) => { self.check_ret(e, table); }
            Stmt::Stop => {
                if self.loop_depth == 0 {
                    self.error("SEMA-0008", "Keyword 'stop' is only allowed inside an active loop body".into());
                }
            }
            Stmt::Next => {
                if self.loop_depth == 0 {
                    self.error("SEMA-0008", "Keyword 'next' is only allowed inside an active loop body".into());
                }
            }
            Stmt::If(if_) => { self.check_if(if_, table); }
            Stmt::Match(m) => { self.check_match(m, table); }
            Stmt::Loop(l) => { self.check_loop(l, table); }
            Stmt::Defer(e) => { self.check_expr(e, table); }
            Stmt::TryCatch(tc) => { self.check_try_catch(tc, table); }
            Stmt::Assign(target, op, value) => { self.check_assign(target, *op, value, table); }
            Stmt::Block(b) => {
                table.push_scope();
                self.check_block(b, table);
                table.pop_scope();
            }
        }
    }

    fn check_var_decl(&mut self, v: &VarDecl, table: &mut SymbolTable) {
        let inferred = v.value.as_ref().map(|val| self.check_expr(val, table)).unwrap_or(None);
        let declared = v.type_.as_ref().and_then(|t| {
            let resolve = |n: &str| table.lookup_type(n);
            resolve_ast_type(t, &resolve).ok()
        });

        let resolved_type = declared.clone().or(inferred.clone()).unwrap_or(TypeInfo::Void);

        if let (Some(vt), Some(dt)) = (&inferred, &declared) {
            if !vt.is_assignable_to(dt) && !vt.is_noret() {
                self.error("SEMA-0001", format!("Type mismatch: expected '{}', found type '{}'", dt.display(), vt.display()));
            }
        }

        let sym = Symbol::Variable { type_: resolved_type, mutable: v.mutable, is_const: false };
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
                        self.error("SEMA-0009", format!("Return type mismatch: expected '{}', found '{}'", rt.display(), et.display()));
                    }
                }
            }
            (Some(e), None) => {
                let et = self.check_expr(e, table);
                if let Some(ref et) = et {
                    if !et.is_void() && !et.is_noret() {
                        self.error("SEMA-0009", format!("Return type mismatch: expected 'void', found '{}'", et.display()));
                    }
                }
            }
            (None, Some(rt)) => {
                if !rt.is_void() && !rt.is_noret() {
                    self.error("SEMA-0009", format!("Return type mismatch: expected '{}', found 'void'", rt.display()));
                }
            }
            (None, None) => {}
        }
    }

    fn check_if(&mut self, if_: &If, table: &mut SymbolTable) {
        let ct = self.check_expr(&if_.cond, table);
        if let Some(ref ct) = ct {
            if !ct.is_bool() && !ct.is_noret() {
                self.error("SEMA-0007", format!("Condition must be of type 'bool', found '{}'", ct.display()));
            }
        }

        if !if_.capture.is_empty() {
            if let Some(ref ct) = ct {
                if !ct.is_optional() && !ct.is_noret() {
                    self.error("SEMA-0019", format!("Cannot capture optional value from non-optional type '{}'", ct.display()));
                }
            }
            table.push_scope();
            for cap in &if_.capture {
                let inner = ct.as_ref().and_then(|c| c.inner_optional().cloned()).unwrap_or(TypeInfo::Void);
                let _ = table.insert(cap, Symbol::Variable { type_: inner, mutable: false, is_const: false });
            }
            self.check_block(&if_.then_block, table);
            table.pop_scope();
        } else {
            self.check_block(&if_.then_block, table);
        }

        if let Some(ref else_stmt) = if_.else_block {
            self.check_stmt(else_stmt, table);
        }
    }

    fn check_match(&mut self, m: &Match, table: &mut SymbolTable) {
        let tt = self.check_expr(&m.target, table);
        let has_wildcard = m.arms.iter().any(|arm| matches!(arm.pattern, Pattern::Wildcard));
        if !has_wildcard && !m.arms.is_empty() {
            self.error("SEMA-0016", "Match patterns are not exhaustive. Wildcard pattern '_' is required".into());
        }
        for arm in &m.arms {
            if !arm.capture.is_empty() {
                if let Some(ref tt) = tt {
                    let is_enum_variant = matches!(&arm.pattern, Pattern::EnumVariant(..));
                    if !tt.is_enum() && !tt.is_optional() && !is_enum_variant {
                        self.error("SEMA-0019", format!("Cannot capture value from non-optional, non-enum type '{}'", tt.display()));
                    }
                }
                table.push_scope();
                for cap in &arm.capture {
                    let inner = tt.as_ref().and_then(|t| {
                        if t.is_optional() { t.inner_optional().cloned() } else { Some(t.clone()) }
                    }).unwrap_or(TypeInfo::Void);
                    let _ = table.insert(cap, Symbol::Variable { type_: inner, mutable: false, is_const: false });
                }
                self.check_expr(&arm.value, table);
                table.pop_scope();
            } else {
                self.check_expr(&arm.value, table);
            }
        }
    }

    fn check_loop(&mut self, l: &Loop, table: &mut SymbolTable) {
        if let Some(ref cond) = l.cond {
            let ct = self.check_expr(cond, table);
            if let Some(ref ct) = ct {
                if !ct.is_bool() && !ct.is_noret() {
                    self.error("SEMA-0007", format!("Condition must be of type 'bool', found '{}'", ct.display()));
                }
            }
        }

        self.loop_depth += 1;

        if !l.captures.is_empty() {
            table.push_scope();
            for cap in &l.captures {
                let _ = table.insert(&cap.name, Symbol::Variable {
                    type_: TypeInfo::AnyType,
                    mutable: cap.mutable,
                    is_const: false,
                });
            }
            self.check_block(&l.body, table);
            table.pop_scope();
        } else {
            self.check_block(&l.body, table);
        }

        self.loop_depth -= 1;
    }

    fn check_try_catch(&mut self, tc: &TryCatch, table: &mut SymbolTable) {
        self.check_block(&tc.try_body, table);
        if !tc.capture.is_empty() {
            table.push_scope();
            for cap in &tc.capture {
                let _ = table.insert(cap, Symbol::Variable {
                    type_: TypeInfo::Builtin("Error".into()),
                    mutable: false,
                    is_const: false,
                });
            }
            self.check_block(&tc.catch_body, table);
            table.pop_scope();
        } else {
            self.check_block(&tc.catch_body, table);
        }
    }

    fn check_assign(&mut self, target: &Expr, _op: AssignOp, value: &Expr, table: &mut SymbolTable) {
        let tt = self.check_expr(target, table);
        let vt = self.check_expr(value, table);

        if let Expr::Ident(name) = target {
            if let Some(sym) = table.lookup(name) {
                if sym.is_const() {
                    self.error("SEMA-0005", format!("Cannot assign to compile-time constant '{}'", name));
                    return;
                }
                if !sym.is_mutable() {
                    self.error("SEMA-0004", format!("Cannot mutate immutable variable '{}'", name));
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
                self.error("SEMA-0001", format!("Type mismatch: expected '{}', found type '{}'", tt.display(), vt.display()));
            }
        }
    }

    pub fn check_expr(&mut self, expr: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
        match expr {
            Expr::Literal(kind, val) => Some(literal_to_type(kind, val)),
            Expr::Ident(name) => {
                if name == "_" || name == "ret_in_expr" { return None; }
                if name.starts_with('@') {
                    let bn = name.trim_start_matches('@');
                    return Some(TypeInfo::Builtin(bn.to_string()));
                }
                match table.lookup(name) {
                    Some(sym) if matches!(sym, Symbol::BuiltinFn { .. }) => None,
                    Some(sym) => sym.get_type(),
                    None => {
                        self.error("SEMA-0002", format!("Undefined symbol '{}' in current scope", name));
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
            Expr::Deref(e) => self.check_deref(e, table),
            Expr::Range(start, end, _) => {
                let st = self.check_expr(start, table);
                let et = self.check_expr(end, table);
                if let (Some(s), Some(e)) = (&st, &et) {
                    if !s.is_integer() || !e.is_integer() {
                        self.error("SEMA-0012", "Range bounds must be integer types".into());
                    }
                }
                st
            }
            Expr::Block(b) => {
                table.push_scope();
                for stmt in &b.stmts { self.check_stmt(stmt, table); }
                table.pop_scope();
                Some(TypeInfo::Void)
            }
            Expr::Paren(e) => self.check_expr(e, table),
            Expr::AtMethod(obj, method) => {
                self.check_expr(obj, table);
                Some(TypeInfo::Builtin(method.clone()))
            }
            Expr::Catch(e, _capture, body) => {
                let et = self.check_expr(e, table);
                for stmt in &body.stmts { self.check_stmt(stmt, table); }
                if let Some(TypeInfo::ErrorUnion(_, ok)) = et {
                    Some(*ok)
                } else { et }
            }
        }
    }

    fn check_binary(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
        let lt = self.check_expr(lhs, table);
        let rt = self.check_expr(rhs, table);

        let both_numeric = || -> bool {
            matches!((&lt, &rt), (Some(l), Some(r)) if l.is_numeric() && r.is_numeric())
        };
        let both_integer = || -> bool {
            matches!((&lt, &rt), (Some(l), Some(r)) if l.is_integer() && r.is_integer())
        };
        let both_bool = || -> bool {
            matches!((&lt, &rt), (Some(l), Some(r)) if l.is_bool() && r.is_bool())
        };

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if !both_numeric() && !lt.as_ref().map_or(false, |t| t.is_noret()) && !rt.as_ref().map_or(false, |t| t.is_noret()) {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error("SEMA-0012", format!("Operator '{}' is not defined for types '{}' and '{}'", binop_str(&op), ls, rs));
                }
                lt
            }
            BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::ModAssign => {
                if !both_numeric() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error("SEMA-0012", format!("Operator '{}' is not defined for types '{}' and '{}'", binop_str(&op), ls, rs));
                }
                lt
            }
            BinaryOp::Eq | BinaryOp::Ne => Some(TypeInfo::Bool),
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                if !both_numeric() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error("SEMA-0012", format!("Operator '{}' is not defined for types '{}' and '{}'", binop_str(&op), ls, rs));
                }
                Some(TypeInfo::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                if !both_bool() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error("SEMA-0012", format!("Operator '{}' is not defined for types '{}' and '{}'", binop_str(&op), ls, rs));
                }
                Some(TypeInfo::Bool)
            }
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
                if !both_integer() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error("SEMA-0012", format!("Operator '{}' is not defined for types '{}' and '{}'", binop_str(&op), ls, rs));
                }
                lt
            }
            BinaryOp::Assign | BinaryOp::ColonEq => rt,
            BinaryOp::Range | BinaryOp::RangeInclusive => {
                if !both_numeric() {
                    let ls = lt.as_ref().map(|t| t.display()).unwrap_or_default();
                    let rs = rt.as_ref().map(|t| t.display()).unwrap_or_default();
                    self.error("SEMA-0012", format!("Range operator is not defined for types '{}' and '{}'", ls, rs));
                }
                lt
            }
        }
    }

    fn check_unary(&mut self, op: UnaryOp, expr: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
        let inner = self.check_expr(expr, table);
        match op {
            UnaryOp::Neg => {
                if let Some(ref t) = inner { if !t.is_numeric() { self.error("SEMA-0013", format!("Operator '-' is not defined for type '{}'", t.display())); } }
                inner
            }
            UnaryOp::Not => {
                if let Some(ref t) = inner { if !t.is_bool() { self.error("SEMA-0013", format!("Operator '!' is not defined for type '{}'", t.display())); } }
                Some(TypeInfo::Bool)
            }
            UnaryOp::BitNot => {
                if let Some(ref t) = inner { if !t.is_integer() { self.error("SEMA-0013", format!("Operator '~' is not defined for type '{}'", t.display())); } }
                inner
            }
            UnaryOp::Ref => inner.map(|t| TypeInfo::Ref(false, Box::new(t))),
            UnaryOp::RefMut => {
                if let Expr::Ident(name) = expr {
                    if let Some(sym) = table.lookup(name) {
                        if !sym.is_mutable() {
                            self.error("SEMA-0006", format!("Cannot take mutable reference to immutable value '{}'", name));
                        }
                    }
                }
                inner.map(|t| TypeInfo::Ref(true, Box::new(t)))
            }
            UnaryOp::Deref => self.check_deref(expr, table),
            UnaryOp::Optional => inner.map(|t| TypeInfo::Optional(Box::new(t))),
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr], table: &mut SymbolTable) -> Option<TypeInfo> {
        if let Expr::Ident(name) = callee {
            if name.starts_with('@') {
                return self.check_builtin(name, args, table);
            }
        }

        let ct = self.check_expr(callee, table);

        match ct {
            Some(TypeInfo::Fn(params, ret)) => {
                if args.len() != params.len() {
                    self.error("SEMA-0014", format!("Expected {} arguments, got {}", params.len(), args.len()));
                    return Some(*ret);
                }
                for (i, arg) in args.iter().enumerate() {
                    let at = self.check_expr(arg, table);
                    if let Some(ref at) = at {
                        if !at.is_assignable_to(&params[i]) && !at.is_noret() {
                            self.error("SEMA-0001", format!("Argument {} type mismatch: expected '{}', found '{}'", i + 1, params[i].display(), at.display()));
                        }
                    }
                }
                Some(*ret)
            }
            Some(t) if t.is_noret() => Some(t),
            Some(t) => {
                self.error("SEMA-0014", format!("Cannot call non-callable type '{}'", t.display()));
                None
            }
            None => {
                if let Expr::Ident(name) = callee {
                    if table.lookup(name).map_or(false, |s| s.is_function()) {
                        return None;
                    } else if name != "_" && !name.starts_with('@') {
                        self.error("SEMA-0014", "Cannot call non-callable type".into());
                    }
                }
                None
            }
        }
    }

    fn check_builtin(&mut self, name: &str, args: &[Expr], table: &mut SymbolTable) -> Option<TypeInfo> {
        let bn = name.trim_start_matches('@');
        match bn {
            "TypeOf" => {
                if args.is_empty() { self.error("SEMA-0017", "Builtin '@TypeOf' requires at least 1 argument".into()); return Some(TypeInfo::TypeMeta); }
                self.check_expr(&args[0], table);
                Some(TypeInfo::TypeMeta)
            }
            "SizeOf" | "AlignOf" | "TypeName" | "EnumCount" => {
                if args.len() != 1 { self.error("SEMA-0017", format!("Builtin '@{}' requires 1 argument", bn)); }
                Some(TypeInfo::Int(IntWidth::W64, false))
            }
            "as" | "bitCast" => {
                if args.len() != 2 { self.error("SEMA-0017", format!("Builtin '@{}' requires 2 arguments", bn)); return None; }
                self.check_expr(&args[1], table);
                Some(TypeInfo::Int(IntWidth::W32, true))
            }
            "ptrCast" | "intToPtr" => {
                if args.len() != 2 { self.error("SEMA-0017", format!("Builtin '@{}' requires 2 arguments", bn)); }
                Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)))
            }
            "ptrToInt" => {
                if args.len() != 1 { self.error("SEMA-0017", format!("Builtin '@{}' requires 1 argument", bn)); }
                Some(TypeInfo::Int(IntWidth::Arch, false))
            }
            "memcpy" | "memset" | "memmove" => {
                if args.len() != 3 { self.error("SEMA-0017", format!("Builtin '@{}' requires 3 arguments", bn)); }
                for a in args { self.check_expr(a, table); }
                Some(TypeInfo::Void)
            }
            "pageAlloc" => {
                if args.len() != 1 { self.error("SEMA-0017", "Builtin '@pageAlloc' requires 1 argument".into()); }
                Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)))
            }
            "pageFree" => {
                if args.len() != 2 { self.error("SEMA-0017", "Builtin '@pageFree' requires 2 arguments".into()); }
                for a in args { self.check_expr(a, table); }
                Some(TypeInfo::Void)
            }
            "comptimeDefaultAllocator" => {
                if !args.is_empty() { self.error("SEMA-0017", "Builtin '@comptimeDefaultAllocator' takes no arguments".into()); }
                Some(TypeInfo::Builtin("Allocator".into()))
            }
            "comptime" | "compileLog" => {
                for a in args { self.check_expr(a, table); }
                Some(TypeInfo::Void)
            }
            "compileError" => {
                if args.len() != 1 { self.error("SEMA-0017", "Builtin '@compileError' requires 1 argument".into()); }
                Some(TypeInfo::Noret)
            }
            "embedFile" => {
                if args.len() != 1 { self.error("SEMA-0017", "Builtin '@embedFile' requires 1 argument".into()); }
                Some(TypeInfo::Array(Box::new(TypeInfo::Int(IntWidth::W8, false)), None))
            }
            "panic" => {
                if args.len() != 1 { self.error("SEMA-0017", "Builtin '@panic' requires 1 argument".into()); }
                Some(TypeInfo::Noret)
            }
            "breakpoint" | "trap" => {
                if !args.is_empty() { self.error("SEMA-0017", format!("Builtin '@{}' takes no arguments", bn)); }
                Some(TypeInfo::Noret)
            }
            "sysCall" => {
                if args.is_empty() { self.error("SEMA-0017", "Builtin '@sysCall' requires at least 1 argument".into()); }
                Some(TypeInfo::Void)
            }
            "str.from_raw" | "str.from" => {
                if !args.is_empty() { self.check_expr(&args[0], table); }
                Some(TypeInfo::Str)
            }
            "vec" | "set" => {
                for a in args { self.check_expr(a, table); }
                Some(TypeInfo::Builtin(bn.to_string()))
            }
            "map" => Some(TypeInfo::Builtin("map".into())),
            "addWithOverflow" | "subWithOverflow" | "mulWithOverflow" => {
                if args.len() != 2 { self.error("SEMA-0017", format!("Builtin '@{}' requires 2 arguments", bn)); }
                let t = self.check_expr(&args[0], table);
                self.check_expr(&args[1], table);
                t.map(|t| TypeInfo::ErrorUnion(None, Box::new(t)))
            }
            "ctz" | "clz" | "popCount" | "bswap" => {
                if args.len() != 1 { self.error("SEMA-0017", format!("Builtin '@{}' requires 1 argument", bn)); }
                self.check_expr(&args[0], table);
                Some(TypeInfo::Int(IntWidth::W32, false))
            }
            "atomicLoad" => {
                if args.len() != 2 { self.error("SEMA-0017", "Builtin '@atomicLoad' requires 2 arguments".into()); }
                self.check_expr(&args[0], table); self.check_expr(&args[1], table);
                Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)))
            }
            "atomicStore" => {
                if args.len() != 3 { self.error("SEMA-0017", "Builtin '@atomicStore' requires 3 arguments".into()); }
                for a in args { self.check_expr(a, table); }
                Some(TypeInfo::Void)
            }
            "cmpxchg" => {
                if args.len() != 5 { self.error("SEMA-0017", "Builtin '@cmpxchg' requires 5 arguments".into()); }
                for a in args { self.check_expr(a, table); }
                Some(TypeInfo::ErrorUnion(None, Box::new(TypeInfo::Pointer(Box::new(TypeInfo::Void)))))
            }
            "Fields" => {
                if args.len() != 1 { self.error("SEMA-0017", "Builtin '@Fields' requires 1 argument".into()); }
                Some(TypeInfo::Array(Box::new(TypeInfo::TypeMeta), None))
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
                    if !pub_ { self.error("SEMA-0015", format!("Field '{}' is private", name)); }
                    Some(ft.clone())
                } else {
                    self.error("SEMA-0015", format!("Field '{}' does not exist on this type", name));
                    None
                }
            }
            Some(TypeInfo::Enum(_, ref variants)) => {
                if variants.iter().any(|(n, _)| n == name) {
                    Some(TypeInfo::Enum(name.to_string(), variants.clone()))
                } else {
                    self.error("SEMA-0015", format!("Variant '{}' does not exist on this enum", name));
                    None
                }
            }
            Some(TypeInfo::Ref(_, ref inner)) => {
                match inner.as_ref() {
                    TypeInfo::Struct(sname, fields) => {
                        if let Some((_, ft, _)) = fields.iter().find(|(n, _, _)| n == name) {
                            Some(ft.clone())
                        } else {
                            self.error("SEMA-0015", format!("Field '{}' does not exist on type '{}'", name, sname));
                            None
                        }
                    }
                    _ => {
                        self.error("SEMA-0015", format!("Field '{}' does not exist on type '{}'", name, inner.display()));
                        None
                    }
                }
            }
            Some(t) => {
                self.error("SEMA-0015", format!("Field '{}' does not exist on type '{}'", name, t.display()));
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
            Some(t) => { self.error("SEMA-0012", format!("Cannot index type '{}'", t.display())); None }
            None => None,
        }
    }

    fn check_deref(&mut self, expr: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
        let inner = self.check_expr(expr, table);
        match inner {
            Some(TypeInfo::Ref(_, inner)) => Some(*inner),
            Some(TypeInfo::Pointer(inner)) => Some(*inner),
            Some(ref t) => { self.error("SEMA-0018", format!("Cannot dereference non-pointer, non-reference type '{}'", t.display())); None }
            None => None,
        }
    }

    fn check_struct_init(&mut self, name: &str, fields: &[FieldInit], table: &mut SymbolTable) -> Option<TypeInfo> {
        match table.lookup_type(name) {
            Some(TypeInfo::Struct(sname, struct_fields)) => {
                for f in fields {
                    self.check_expr(&f.value, table);
                    if !struct_fields.iter().any(|(n, _, _)| n == &f.name) {
                        self.error("SEMA-0015", format!("Field '{}' does not exist on struct '{}'", f.name, sname));
                    }
                }
                for (fname, _, _) in &struct_fields {
                    if !fields.iter().any(|f| &f.name == fname) {
                        self.error("SEMA-0015", format!("Missing field '{}' in struct initializer for '{}'", fname, sname));
                    }
                }
                Some(TypeInfo::Struct(sname, struct_fields))
            }
            Some(t) => { self.error("SEMA-0015", format!("Cannot initialize non-struct type '{}' with struct syntax", t.display())); None }
            None => { self.error("SEMA-0002", format!("Undefined type '{}' for struct initialization", name)); None }
        }
    }

    pub fn check_behave_impl(&mut self, type_name: &str, behave_name: &str, type_methods: &[FnSymbol], table: &SymbolTable) {
        match table.lookup(behave_name) {
            Some(Symbol::Behave(b)) => {
                for bm in &b.methods {
                    let implemented = type_methods.iter().any(|tm| tm.name == bm.name);
                    if !implemented {
                        self.error("SEMA-0011", format!("Struct '{}' does not implement behave trait method '{}'", type_name, bm.name));
                    }
                }
            }
            _ => { self.error("SEMA-0002", format!("Undefined behaviour '{}'", behave_name)); }
        }
    }
}

fn binop_str(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+", BinaryOp::Sub => "-", BinaryOp::Mul => "*", BinaryOp::Div => "/", BinaryOp::Mod => "%",
        BinaryOp::AddAssign => "+=", BinaryOp::SubAssign => "-=", BinaryOp::MulAssign => "*=",
        BinaryOp::DivAssign => "/=", BinaryOp::ModAssign => "%=",
        BinaryOp::Eq => "==", BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<", BinaryOp::Gt => ">", BinaryOp::Le => "<=", BinaryOp::Ge => ">=",
        BinaryOp::And => "&&", BinaryOp::Or => "||",
        BinaryOp::BitAnd => "&", BinaryOp::BitOr => "|", BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<", BinaryOp::Shr => ">>",
        BinaryOp::Assign => "=", BinaryOp::ColonEq => ":=",
        BinaryOp::Range => "..", BinaryOp::RangeInclusive => "..=",
    }
}
