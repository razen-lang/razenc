use crate::ast::*;
use crate::lexer::token::TokenKind;

// ═══════════════════════════════════════════════
// IR Value — operand in IR instructions
// ═══════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IrValue {
    Temp(usize),
    Local(String),
    Param(String),
    Global(String),
    ConstInt(i64),
    ConstU64(u64),
    ConstBool(bool),
    ConstStr(String),
    Label(String),
    Void,
}

impl std::fmt::Display for IrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrValue::Temp(n) => write!(f, "%t{}", n),
            IrValue::Local(s) => write!(f, "%{}", s),
            IrValue::Param(s) => write!(f, "%{}", s),
            IrValue::Global(s) => write!(f, "@{}", s),
            IrValue::ConstInt(n) => write!(f, "{}", n),
            IrValue::ConstU64(n) => write!(f, "{}", n),
            IrValue::ConstBool(b) => write!(f, "{}", b),
            IrValue::ConstStr(s) => write!(f, "\"{}\"", s),
            IrValue::Label(s) => write!(f, "{}", s),
            IrValue::Void => write!(f, "void"),
        }
    }
}

// ═══════════════════════════════════════════════
// IR Binary Operator
// ═══════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

impl std::fmt::Display for IrOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrOp::Add => write!(f, "add"),
            IrOp::Sub => write!(f, "sub"),
            IrOp::Mul => write!(f, "mul"),
            IrOp::Div => write!(f, "div"),
            IrOp::Mod => write!(f, "mod"),
            IrOp::Eq => write!(f, "eq"),
            IrOp::Ne => write!(f, "ne"),
            IrOp::Lt => write!(f, "lt"),
            IrOp::Gt => write!(f, "gt"),
            IrOp::Le => write!(f, "le"),
            IrOp::Ge => write!(f, "ge"),
            IrOp::And => write!(f, "and"),
            IrOp::Or => write!(f, "or"),
            IrOp::BitAnd => write!(f, "bitand"),
            IrOp::BitOr => write!(f, "bitor"),
            IrOp::BitXor => write!(f, "bitxor"),
            IrOp::Shl => write!(f, "shl"),
            IrOp::Shr => write!(f, "shr"),
        }
    }
}

// ═══════════════════════════════════════════════
// IR Instruction
// ═══════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum IrInst {
    Alloca(IrValue),
    Store(IrValue, IrValue),
    Load(IrValue, IrValue),
    BinOp(IrOp, IrValue, IrValue, IrValue),
    Neg(IrValue, IrValue),
    Not(IrValue, IrValue),
    BitNot(IrValue, IrValue),
    Ref(IrValue, IrValue),
    Deref(IrValue, IrValue),
    Call(IrValue, String, Vec<IrValue>),
    CallPtr(IrValue, IrValue, Vec<IrValue>),
    RetVoid,
    Ret(IrValue),
    Branch(IrValue, String, String),
    Jump(String),
    Label(String),
    Gep(IrValue, IrValue, IrValue),
    FieldAddr(IrValue, IrValue, usize, String),
    PtrToInt(IrValue, IrValue),
    IntToPtr(IrValue, IrValue),
    BitCast(IrValue, IrValue),
    StructInit(IrValue, String, Vec<(String, IrValue)>),
    Copy(IrValue, IrValue),
    Comment(String),
    Phi(IrValue, Vec<(IrValue, String)>),
    AllocArray(IrValue, IrValue),
    SetError(IrValue, IrValue),
    Catch(IrValue, IrValue, IrValue, IrValue),
}

// ═══════════════════════════════════════════════
// IR Function & Program
// ═══════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub insts: Vec<IrInst>,
}

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: String,
    pub mutable: bool,
    pub type_: Option<Type>,
    pub init: Option<IrValue>,
}

#[derive(Debug, Clone)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
}

// ═══════════════════════════════════════════════
// Loop tracking for break/continue
// ═══════════════════════════════════════════════

#[derive(Debug, Clone)]
struct LoopInfo {
    start_label: String,
    end_label: String,
}

// ═══════════════════════════════════════════════
// IR Generator — walks the AST and emits IR
// ═══════════════════════════════════════════════

pub struct IrGenerator {
    insts: Vec<IrInst>,
    temps: usize,
    labels: usize,
    loops: Vec<LoopInfo>,
    defer_stack: Vec<Vec<Expr>>,
}

impl IrGenerator {
    pub fn new() -> Self {
        IrGenerator {
            insts: Vec::new(),
            temps: 0,
            labels: 0,
            loops: Vec::new(),
            defer_stack: Vec::new(),
        }
    }

    fn new_temp(&mut self) -> IrValue {
        let t = self.temps;
        self.temps += 1;
        IrValue::Temp(t)
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let l = self.labels;
        self.labels += 1;
        format!(".L{}_{}", prefix, l)
    }

    fn emit(&mut self, inst: IrInst) {
        self.insts.push(inst);
    }

    fn push_comment(&mut self, msg: String) {
        self.emit(IrInst::Comment(msg));
    }

    /// Generate IR for the entire program
    pub fn generate(&mut self, program: &Program) -> IrProgram {
        let mut ir_program = IrProgram {
            functions: Vec::new(),
            globals: Vec::new(),
        };

        for decl in &program.decls {
            self.gen_decl(decl, &mut ir_program);
        }

        ir_program
    }

    // ─────────────────────────────────────────────
    // Declarations
    // ─────────────────────────────────────────────

    fn gen_decl(&mut self, decl: &Decl, ir: &mut IrProgram) {
        match decl {
            Decl::Use(_) => {}
            Decl::Mod(_, body) => {
                if let Some(decls) = body {
                    for d in decls {
                        self.gen_decl(d, ir);
                    }
                }
            }
            Decl::Fn(f) => self.gen_fn(f, ir),
            Decl::Struct(_) => {}
            Decl::Union(_) => {}
            Decl::Enum(_) => {}
            Decl::Error_(_, _) => {}
            Decl::Behave(_) => {}
            Decl::Var(v) => self.gen_global_var(v, ir),
            Decl::Const(c) => self.gen_global_const(c, ir),
            Decl::TypeAlias(_, _) => { /* type alias — no runtime IR */ }
            Decl::Test(name, block) => self.gen_test(name, block, ir),
        }
    }

    fn gen_global_var(&mut self, v: &VarDecl, ir: &mut IrProgram) {
        ir.globals.push(IrGlobal {
            name: v.name.clone(),
            mutable: v.mutable,
            type_: v.type_.clone(),
            init: v.value.as_ref().map(|e| self.const_expr_to_value(e)),
        });
    }

    fn gen_global_const(&mut self, c: &ConstDecl, ir: &mut IrProgram) {
        ir.globals.push(IrGlobal {
            name: c.name.clone(),
            mutable: false,
            type_: c.type_.clone(),
            init: c.value.as_ref().map(|e| self.const_expr_to_value(e)),
        });
    }

    fn const_expr_to_value(&self, expr: &Expr) -> IrValue {
        match expr {
            Expr::Literal(TokenKind::IntegerValue, v) => {
                v.parse::<i64>().map(IrValue::ConstInt).unwrap_or(IrValue::ConstInt(0))
            }
            Expr::Literal(TokenKind::FloatValue, _) => IrValue::ConstInt(0),
            Expr::Literal(TokenKind::StringValue, v) => IrValue::ConstStr(v.clone()),
            Expr::Literal(TokenKind::True, _) => IrValue::ConstBool(true),
            Expr::Literal(TokenKind::False, _) => IrValue::ConstBool(false),
            Expr::Literal(TokenKind::Nil, _) => IrValue::ConstInt(0),
            Expr::Literal(_, v) => v.parse::<i64>().map(IrValue::ConstInt).unwrap_or(IrValue::ConstInt(0)),
            _ => IrValue::ConstInt(0),
        }
    }

    fn gen_fn(&mut self, f: &FnDecl, ir: &mut IrProgram) {
        let saved_temps = self.temps;
        let saved_labels = self.labels;
        let saved_loops = std::mem::take(&mut self.loops);
        let saved_defer = std::mem::take(&mut self.defer_stack);

        self.insts = Vec::new();
        self.temps = 0;
        self.labels = 0;

        let entry = self.new_label("entry");
        self.emit(IrInst::Label(entry));

        for p in &f.params {
            let alloca = IrValue::Local(p.name.clone());
            self.emit(IrInst::Alloca(alloca.clone()));
            let param_val = IrValue::Param(p.name.clone());
            self.emit(IrInst::Store(param_val, alloca));
        }

        if let Some(ref body) = f.body {
            self.defer_stack.push(Vec::new());
            self.gen_block(body);

            let deferred = self.defer_stack.last().cloned().unwrap_or_default();
            for expr in deferred.iter().rev() {
                self.gen_expr(expr);
            }
            self.defer_stack.pop();
        }

        let has_terminal = self.insts.last().map_or(false, |i| {
            matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_))
        });
        if !has_terminal {
            self.emit(IrInst::RetVoid);
        }

        let func_params: Vec<(String, Type)> = f.params.iter().map(|p| (p.name.clone(), p.type_.clone())).collect();

        let func = IrFunction {
            name: f.name.clone(),
            params: func_params,
            return_type: f.return_.clone(),
            insts: std::mem::take(&mut self.insts),
        };

        ir.functions.push(func);

        self.temps = saved_temps;
        self.labels = saved_labels;
        self.loops = saved_loops;
        self.defer_stack = saved_defer;
    }

    fn gen_test(&mut self, name: &str, block: &Block, ir: &mut IrProgram) {
        let _saved = std::mem::take(&mut self.insts);
        let saved_temps = self.temps;
        let saved_labels = self.labels;
        let saved_loops = std::mem::take(&mut self.loops);
        let saved_defer = std::mem::take(&mut self.defer_stack);

        self.temps = 0;
        self.labels = 0;

        let entry = self.new_label("entry");
        self.emit(IrInst::Label(entry));
        self.gen_block(block);
        self.emit(IrInst::RetVoid);

        let func = IrFunction {
            name: format!("test.{}", name),
            params: Vec::new(),
            return_type: None,
            insts: std::mem::take(&mut self.insts),
        };
        ir.functions.push(func);

        self.temps = saved_temps;
        self.labels = saved_labels;
        self.loops = saved_loops;
        self.defer_stack = saved_defer;
    }

    // ─────────────────────────────────────────────
    // Blocks
    // ─────────────────────────────────────────────

    fn gen_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.gen_stmt(stmt);
        }
    }

    // ─────────────────────────────────────────────
    // Statements
    // ─────────────────────────────────────────────

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                self.gen_expr(e);
            }
            Stmt::Var(v) => self.gen_var_decl(v),
            Stmt::Ret(e) => {
                self.run_deferred();
                match e {
                    Some(ex) => {
                        let val = self.gen_expr(ex);
                        self.emit(IrInst::Ret(val));
                    }
                    None => self.emit(IrInst::RetVoid),
                }
            }
            Stmt::Stop => {
                if let Some(loop_info) = self.loops.last() {
                    self.emit(IrInst::Jump(loop_info.end_label.clone()));
                }
            }
            Stmt::Next => {
                if let Some(loop_info) = self.loops.last() {
                    self.emit(IrInst::Jump(loop_info.start_label.clone()));
                }
            }
            Stmt::If(if_) => self.gen_if(if_),
            Stmt::Match(m) => self.gen_match(m),
            Stmt::Loop(l) => self.gen_loop(l),
            Stmt::Defer(e) => {
                if let Some(stack) = self.defer_stack.last_mut() {
                    stack.push((**e).clone());
                }
            }
            Stmt::TryCatch(tc) => self.gen_try_catch(tc),
            Stmt::Assign(target, op, value) => self.gen_assign(target, *op, value),
            Stmt::Block(b) => self.gen_block(b),
        }
    }

    fn gen_var_decl(&mut self, v: &VarDecl) {
        let local = IrValue::Local(v.name.clone());
        self.emit(IrInst::Alloca(local.clone()));
        if let Some(ref val) = v.value {
            let init = self.gen_expr(val);
            self.emit(IrInst::Store(init, local));
        }
    }

    fn gen_if(&mut self, if_: &If) {
        let cond = self.gen_expr(&if_.cond);
        let then_label = self.new_label("then");
        let else_label = self.new_label("else");
        let merge_label = self.new_label("merge");

        self.emit(IrInst::Branch(cond, then_label.clone(), else_label.clone()));

        // Then block
        self.emit(IrInst::Label(then_label));
        self.gen_block(&if_.then_block);
        let then_falls = !self.insts.last().map_or(false, |i| {
            matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_))
        });
        if then_falls {
            self.emit(IrInst::Jump(merge_label.clone()));
        }

        // Else block
        self.emit(IrInst::Label(else_label));
        if let Some(ref else_stmt) = if_.else_block {
            self.gen_stmt(else_stmt);
        }
        let else_falls = !self.insts.last().map_or(false, |i| {
            matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_))
        });
        if else_falls {
            self.emit(IrInst::Jump(merge_label.clone()));
        }

        // Merge point
        self.emit(IrInst::Label(merge_label));
    }

    fn gen_match(&mut self, m: &Match) {
        let target = self.gen_expr(&m.target);
        let arm_labels: Vec<String> = (0..m.arms.len()).map(|i| self.new_label(&format!("arm{}", i))).collect();
        let else_label = self.new_label("match_else");
        let merge_label = self.new_label("match_merge");

        for (i, arm) in m.arms.iter().enumerate() {
            let arm_cond = self.new_label(&format!("arm_cond{}", i));
            self.emit(IrInst::Label(arm_cond));

            match &arm.pattern {
                Pattern::Wildcard => {
                    self.emit(IrInst::Jump(arm_labels[i].clone()));
                }
                Pattern::Ident(name) => {
                    let local = IrValue::Local(name.clone());
                    self.emit(IrInst::Alloca(local.clone()));
                    self.emit(IrInst::Store(target.clone(), local));
                    self.emit(IrInst::Jump(arm_labels[i].clone()));
                }
                Pattern::Literal(kind, val) => {
                    let pat_val = self.literal_to_ir_value(kind, val);
                    let cmp = self.new_temp();
                    self.emit(IrInst::BinOp(IrOp::Eq, cmp.clone(), target.clone(), pat_val));
                    let arm_next = self.new_label(&format!("arm_next{}", i));
                    let arm_next_clone = arm_next.clone();
                    self.emit(IrInst::Branch(cmp, arm_labels[i].clone(), arm_next_clone));
                    self.emit(IrInst::Label(arm_next));
                }
                Pattern::EnumVariant(typ, variant, capture) => {
                    self.push_comment(format!("match enum {}.{}", typ, variant));
                    if let Some(c) = capture {
                        let local = IrValue::Local(c.clone());
                        self.emit(IrInst::Alloca(local.clone()));
                        self.emit(IrInst::Store(target.clone(), local));
                    }
                    self.emit(IrInst::Jump(arm_labels[i].clone()));
                }
            }
        }

        // Else branch (wildcard or fallback)
        self.emit(IrInst::Label(else_label));

        // Emit each arm body
        for (i, arm) in m.arms.iter().enumerate() {
            self.emit(IrInst::Label(arm_labels[i].clone()));
            self.gen_expr(&arm.value);
            let arm_falls = !self.insts.last().map_or(false, |i| {
                matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_))
            });
            if arm_falls {
                self.emit(IrInst::Jump(merge_label.clone()));
            }
        }

        self.emit(IrInst::Label(merge_label));
    }

    fn gen_loop(&mut self, l: &Loop) {
        let start_label = self.new_label("loop_start");
        let body_label = self.new_label("loop_body");
        let end_label = self.new_label("loop_end");

        // Process capture variables
        for cap in &l.captures {
            let local = IrValue::Local(cap.name.clone());
            self.emit(IrInst::Alloca(local));
        }

        self.emit(IrInst::Label(start_label.clone()));

        for cond in &l.conds {
            let cond_val = self.gen_expr(cond);
            let skip_label = self.new_label("cond_fail");
            let cont_label = self.new_label("cond_pass");
            let sl = skip_label.clone();
            let cl = cont_label.clone();
            self.emit(IrInst::Branch(cond_val.clone(), cl, sl));
            self.emit(IrInst::Label(skip_label));
            self.emit(IrInst::Jump(end_label.clone()));
            self.emit(IrInst::Label(cont_label));
        }

        self.emit(IrInst::Label(body_label.clone()));

        self.loops.push(LoopInfo {
            start_label: start_label.clone(),
            end_label: end_label.clone(),
        });

        self.defer_stack.push(Vec::new());
        self.gen_block(&l.body);
        let deferred = self.defer_stack.last().cloned().unwrap_or_default();
        for expr in deferred.iter().rev() {
            self.gen_expr(expr);
        }
        self.defer_stack.pop();

        self.loops.pop();

        // Jump back to start
        self.emit(IrInst::Jump(start_label.clone()));

        // End label
        self.emit(IrInst::Label(end_label));
    }

    fn gen_try_catch(&mut self, tc: &TryCatch) {
        self.gen_block(&tc.try_body);

        let catch_label = self.new_label("catch");
        let merge_label = self.new_label("try_merge");

        self.emit(IrInst::Jump(merge_label.clone()));

        self.emit(IrInst::Label(catch_label));
        for cap in &tc.capture {
            let local = IrValue::Local(cap.clone());
            self.emit(IrInst::Alloca(local.clone()));
            let err = IrValue::ConstInt(-1);
            self.emit(IrInst::Store(err, local));
        }
        self.gen_block(&tc.catch_body);

        self.emit(IrInst::Label(merge_label));
    }

    fn gen_assign(&mut self, target: &Expr, op: AssignOp, value: &Expr) {
        let val = self.gen_expr(value);
        match target {
            Expr::Ident(name) => {
                let local = IrValue::Local(name.clone());
                let loaded = self.new_temp();
                self.emit(IrInst::Load(loaded.clone(), local.clone()));

                let result = if op == AssignOp::Eq {
                    val
                } else {
                    let bin_op = assign_op_to_ir_op(op);
                    let tmp = self.new_temp();
                    self.emit(IrInst::BinOp(bin_op, tmp.clone(), loaded, val));
                    tmp
                };
                self.emit(IrInst::Store(result, local));
            }
            Expr::Field(obj, fname) => {
                let obj_val = self.gen_expr(obj);
                let addr = self.new_temp();
                let addr_clone = addr.clone();
                self.emit(IrInst::FieldAddr(addr_clone, obj_val, 0, fname.clone()));
                self.emit(IrInst::Store(val, addr));
            }
            Expr::Index(obj, idx) => {
                let obj_val = self.gen_expr(obj);
                let idx_val = self.gen_expr(idx);
                let addr = self.new_temp();
                let addr_clone = addr.clone();
                self.emit(IrInst::Gep(addr_clone, obj_val, idx_val));
                self.emit(IrInst::Store(val, addr));
            }
            _ => {
                self.gen_expr(target);
                self.emit(IrInst::Comment(format!("assign op {}", assign_op_display(op))));
                self.emit(IrInst::Copy(IrValue::Void, val));
            }
        }
    }

    // ─────────────────────────────────────────────
    // Expressions
    // ─────────────────────────────────────────────

    fn gen_expr(&mut self, expr: &Expr) -> IrValue {
        match expr {
            Expr::Literal(kind, val) => self.literal_to_ir_value(kind, val),

            Expr::Ident(name) => {
                if name == "_" {
                    return IrValue::Void;
                }
                if name.starts_with('@') {
                    return IrValue::Global(name.clone());
                }
                let local = IrValue::Local(name.clone());
                let temp = self.new_temp();
                self.emit(IrInst::Load(temp.clone(), local));
                temp
            }

            Expr::Binary(op, lhs, rhs) => self.gen_binary(*op, lhs, rhs),

            Expr::Unary(op, e) => self.gen_unary(*op, e),

            Expr::Call(callee, args) => self.gen_call(callee, args),

            Expr::Field(obj, name) => {
                let obj_val = self.gen_expr(obj);
                let temp = self.new_temp();
                self.emit(IrInst::FieldAddr(temp.clone(), obj_val, 0, name.clone()));
                let loaded = self.new_temp();
                self.emit(IrInst::Load(loaded.clone(), temp));
                loaded
            }

            Expr::Index(obj, idx) => {
                let obj_val = self.gen_expr(obj);
                let idx_val = self.gen_expr(idx);
                let addr = self.new_temp();
                self.emit(IrInst::Gep(addr.clone(), obj_val, idx_val));
                let loaded = self.new_temp();
                self.emit(IrInst::Load(loaded.clone(), addr));
                loaded
            }

            Expr::Slice(obj, start, end, inclusive) => {
                let obj_val = self.gen_expr(obj);
                let _start_val = self.gen_expr(start);
                let _end_val = self.gen_expr(end);
                let temp = self.new_temp();
                self.emit(IrInst::Comment(format!("slice {}{}", if *inclusive { "..=" } else { ".." }, "")));
                self.emit(IrInst::Copy(temp.clone(), obj_val));
                temp
            }

            Expr::StructInit(name, fields) => {
                let temp = self.new_temp();
                let mut ir_fields = Vec::new();
                for f in fields {
                    let fv = self.gen_expr(&f.value);
                    ir_fields.push((f.name.clone(), fv));
                }
                self.emit(IrInst::StructInit(temp.clone(), name.clone(), ir_fields));
                temp
            }

            Expr::Deref(e) => {
                let ptr = self.gen_expr(e);
                let temp = self.new_temp();
                self.emit(IrInst::Deref(temp.clone(), ptr));
                temp
            }

            Expr::Block(b) => {
                let temp = self.new_temp();
                self.gen_block(b);
                self.emit(IrInst::Copy(temp.clone(), IrValue::Void));
                temp
            }

            Expr::Paren(e) => self.gen_expr(e),

            Expr::AtMethod(obj, method) => {
                let obj_val = self.gen_expr(obj);
                let temp = self.new_temp();
                self.emit(IrInst::Call(temp, format!("@{}", method), vec![obj_val]));
                self.new_temp() // dummy return
            }

            Expr::Catch(expr, capture, body) => {
                let val = self.gen_expr(expr);
                let catch_label = self.new_label("catch_err");
                let merge_label = self.new_label("catch_merge");
                let temp = self.new_temp();

                let val_for_catch = val.clone();
                self.emit(IrInst::Catch(temp.clone(), val_for_catch, IrValue::Label(catch_label.clone()), IrValue::Label(merge_label.clone())));

                self.emit(IrInst::Label(catch_label));
                for cap in capture {
                    let local = IrValue::Local(cap.clone());
                    self.emit(IrInst::Alloca(local.clone()));
                    self.emit(IrInst::Store(val.clone(), local));
                }
                self.gen_block(body);
                self.emit(IrInst::Jump(merge_label.clone()));

                self.emit(IrInst::Label(merge_label));
                temp
            }

            Expr::Ret(value) => {
                self.run_deferred();
                match value {
                    Some(val) => {
                        let v = self.gen_expr(val);
                        self.emit(IrInst::Ret(v));
                        IrValue::Void
                    }
                    None => {
                        self.emit(IrInst::RetVoid);
                        IrValue::Void
                    }
                }
            }

            Expr::Fn(fndecl) => {
                let temp = self.new_temp();
                self.emit(IrInst::Comment(format!("fn literal: {}", fndecl.name)));
                self.emit(IrInst::Copy(temp.clone(), IrValue::ConstInt(0)));
                temp
            }

            Expr::MapLiteral(pairs) => {
                let temp = self.new_temp();
                self.emit(IrInst::Comment("map literal".to_string()));
                for (k, v) in pairs {
                    self.gen_expr(k);
                    self.gen_expr(v);
                }
                self.emit(IrInst::Copy(temp.clone(), IrValue::ConstInt(0)));
                temp
            }
        }
    }

    fn gen_binary(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr) -> IrValue {
        let l = self.gen_expr(lhs);
        let r = self.gen_expr(rhs);
        let temp = self.new_temp();

        match op {
            BinaryOp::Add => self.emit(IrInst::BinOp(IrOp::Add, temp.clone(), l, r)),
            BinaryOp::Sub => self.emit(IrInst::BinOp(IrOp::Sub, temp.clone(), l, r)),
            BinaryOp::Mul => self.emit(IrInst::BinOp(IrOp::Mul, temp.clone(), l, r)),
            BinaryOp::Div => self.emit(IrInst::BinOp(IrOp::Div, temp.clone(), l, r)),
            BinaryOp::Mod => self.emit(IrInst::BinOp(IrOp::Mod, temp.clone(), l, r)),
            BinaryOp::Eq => self.emit(IrInst::BinOp(IrOp::Eq, temp.clone(), l, r)),
            BinaryOp::Ne => self.emit(IrInst::BinOp(IrOp::Ne, temp.clone(), l, r)),
            BinaryOp::Lt => self.emit(IrInst::BinOp(IrOp::Lt, temp.clone(), l, r)),
            BinaryOp::Gt => self.emit(IrInst::BinOp(IrOp::Gt, temp.clone(), l, r)),
            BinaryOp::Le => self.emit(IrInst::BinOp(IrOp::Le, temp.clone(), l, r)),
            BinaryOp::Ge => self.emit(IrInst::BinOp(IrOp::Ge, temp.clone(), l, r)),
            BinaryOp::And => self.emit(IrInst::BinOp(IrOp::And, temp.clone(), l, r)),
            BinaryOp::Or => self.emit(IrInst::BinOp(IrOp::Or, temp.clone(), l, r)),
            BinaryOp::BitAnd => self.emit(IrInst::BinOp(IrOp::BitAnd, temp.clone(), l, r)),
            BinaryOp::BitOr => self.emit(IrInst::BinOp(IrOp::BitOr, temp.clone(), l, r)),
            BinaryOp::BitXor => self.emit(IrInst::BinOp(IrOp::BitXor, temp.clone(), l, r)),
            BinaryOp::Shl => self.emit(IrInst::BinOp(IrOp::Shl, temp.clone(), l, r)),
            BinaryOp::Shr => self.emit(IrInst::BinOp(IrOp::Shr, temp.clone(), l, r)),
            BinaryOp::Assign | BinaryOp::ColonEq => {
                self.emit(IrInst::Copy(temp.clone(), r));
            }
            BinaryOp::AddAssign => self.emit(IrInst::BinOp(IrOp::Add, temp.clone(), l, r)),
            BinaryOp::SubAssign => self.emit(IrInst::BinOp(IrOp::Sub, temp.clone(), l, r)),
            BinaryOp::MulAssign => self.emit(IrInst::BinOp(IrOp::Mul, temp.clone(), l, r)),
            BinaryOp::DivAssign => self.emit(IrInst::BinOp(IrOp::Div, temp.clone(), l, r)),
            BinaryOp::ModAssign => self.emit(IrInst::BinOp(IrOp::Mod, temp.clone(), l, r)),
            BinaryOp::Range | BinaryOp::RangeInclusive => {
                self.emit(IrInst::Comment(format!("range {}..{}", match &l { IrValue::ConstInt(n) => n.to_string(), _ => "?".to_string() }, match &r { IrValue::ConstInt(n) => n.to_string(), _ => "?".to_string() })));
                self.emit(IrInst::Copy(temp.clone(), l));
            }
        }
        temp
    }

    fn gen_unary(&mut self, op: UnaryOp, expr: &Expr) -> IrValue {
        let e = self.gen_expr(expr);
        let temp = self.new_temp();

        match op {
            UnaryOp::Neg => self.emit(IrInst::Neg(temp.clone(), e)),
            UnaryOp::Not => self.emit(IrInst::Not(temp.clone(), e)),
            UnaryOp::BitNot => self.emit(IrInst::BitNot(temp.clone(), e)),
            UnaryOp::Ref => self.emit(IrInst::Ref(temp.clone(), e)),
            UnaryOp::RefMut => self.emit(IrInst::Ref(temp.clone(), e)),
            UnaryOp::Deref => self.emit(IrInst::Deref(temp.clone(), e)),
            UnaryOp::Optional => {
                self.emit(IrInst::Comment("optional wrap".to_string()));
                self.emit(IrInst::Copy(temp.clone(), e));
            }
        }
        temp
    }

    fn gen_call(&mut self, callee: &Expr, args: &[Expr]) -> IrValue {
        let result = self.new_temp();

        match callee {
            Expr::Ident(name) => {
                if name.starts_with('@') {
                    return self.gen_builtin_call(name, args);
                }
                let mut ir_args = Vec::new();
                for arg in args {
                    ir_args.push(self.gen_expr(arg));
                }
                self.emit(IrInst::Call(result.clone(), name.clone(), ir_args));
                result
            }
            Expr::Field(obj, method_name) => {
                let obj_val = self.gen_expr(obj);
                let mut ir_args = vec![obj_val];
                for arg in args {
                    ir_args.push(self.gen_expr(arg));
                }
                self.emit(IrInst::Call(result.clone(), method_name.clone(), ir_args));
                result
            }
            _ => {
                let callee_val = self.gen_expr(callee);
                let mut ir_args = Vec::new();
                for arg in args {
                    ir_args.push(self.gen_expr(arg));
                }
                self.emit(IrInst::CallPtr(result.clone(), callee_val, ir_args));
                result
            }
        }
    }

    fn gen_builtin_call(&mut self, name: &str, args: &[Expr]) -> IrValue {
        let result = self.new_temp();
        let ir_args: Vec<IrValue> = args.iter().map(|a| self.gen_expr(a)).collect();
        self.emit(IrInst::Call(result.clone(), name.to_string(), ir_args));
        result
    }

    fn literal_to_ir_value(&self, kind: &TokenKind, val: &str) -> IrValue {
        match kind {
            TokenKind::IntegerValue => {
                if let Ok(n) = val.parse::<i64>() {
                    IrValue::ConstInt(n)
                } else if let Ok(n) = val.parse::<u64>() {
                    IrValue::ConstU64(n)
                } else {
                    IrValue::ConstInt(0)
                }
            }
            TokenKind::FloatValue => {
                // store float bits as u64 for simplicity
                if let Ok(f) = val.parse::<f64>() {
                    IrValue::ConstU64(f.to_bits())
                } else {
                    IrValue::ConstU64(0)
                }
            }
            TokenKind::StringValue => IrValue::ConstStr(val.to_string()),
            TokenKind::CharValue => {
                let chars: Vec<char> = val.chars().collect();
                if chars.len() >= 2 && chars[0] == '\'' && chars[chars.len() - 1] == '\'' {
                    let inner: String = chars[1..chars.len() - 1].iter().collect();
                    if let Some(c) = inner.chars().next() {
                        IrValue::ConstInt(c as i64)
                    } else {
                        IrValue::ConstInt(0)
                    }
                } else {
                    IrValue::ConstInt(0)
                }
            }
            TokenKind::True => IrValue::ConstBool(true),
            TokenKind::False => IrValue::ConstBool(false),
            TokenKind::Nil => IrValue::ConstInt(0),
            _ => IrValue::ConstInt(0),
        }
    }

    fn run_deferred(&mut self) {
        let deferred = self.defer_stack.last().cloned().unwrap_or_default();
        for expr in deferred.iter().rev() {
            self.gen_expr(expr);
        }
    }
}

// ═══════════════════════════════════════════════
// Display implementations for IR printing
// ═══════════════════════════════════════════════

impl std::fmt::Display for IrInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrInst::Alloca(dst) => write!(f, "{} = alloca", dst),
            IrInst::Store(src, ptr) => write!(f, "store {}, {}", src, ptr),
            IrInst::Load(dst, src) => write!(f, "{} = load {}", dst, src),
            IrInst::BinOp(op, dst, l, r) => write!(f, "{} = {} {}, {}", dst, op, l, r),
            IrInst::Neg(dst, src) => write!(f, "{} = neg {}", dst, src),
            IrInst::Not(dst, src) => write!(f, "{} = not {}", dst, src),
            IrInst::BitNot(dst, src) => write!(f, "{} = bitnot {}", dst, src),
            IrInst::Ref(dst, src) => write!(f, "{} = ref {}", dst, src),
            IrInst::Deref(dst, src) => write!(f, "{} = deref {}", dst, src),
            IrInst::Call(dst, name, args) => {
                let args_str: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
                write!(f, "{} = call @{}({})", dst, name, args_str.join(", "))
            }
            IrInst::CallPtr(dst, callee, args) => {
                let args_str: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
                write!(f, "{} = call {}({})", dst, callee, args_str.join(", "))
            }
            IrInst::RetVoid => write!(f, "ret void"),
            IrInst::Ret(val) => write!(f, "ret {}", val),
            IrInst::Branch(cond, t, f_) => write!(f, "br {}, {}:{}", cond, t, f_),
            IrInst::Jump(label) => write!(f, "br {}", label),
            IrInst::Label(label) => write!(f, "{}:", label),
            IrInst::Gep(dst, ptr, idx) => write!(f, "{} = gep {}, {}", dst, ptr, idx),
            IrInst::FieldAddr(dst, obj, _idx, name) => write!(f, "{} = fieldaddr {}, .{}", dst, obj, name),
            IrInst::PtrToInt(dst, src) => write!(f, "{} = ptrtoint {}", dst, src),
            IrInst::IntToPtr(dst, src) => write!(f, "{} = inttoptr {}", dst, src),
            IrInst::BitCast(dst, src) => write!(f, "{} = bitcast {}", dst, src),
            IrInst::StructInit(dst, name, fields) => {
                let f_str: Vec<String> = fields.iter().map(|(n, v)| format!(".{} = {}", n, v)).collect();
                write!(f, "{} = struct {} {{ {} }}", dst, name, f_str.join(", "))
            }
            IrInst::Copy(dst, src) => write!(f, "{} = copy {}", dst, src),
            IrInst::Comment(msg) => write!(f, "; {}", msg),
            IrInst::Phi(dst, incs) => {
                let inc_str: Vec<String> = incs.iter().map(|(v, l)| format!("[ {}, {} ]", v, l)).collect();
                write!(f, "{} = phi {}", dst, inc_str.join(", "))
            }
            IrInst::AllocArray(dst, size) => write!(f, "{} = allocarray {}", dst, size),
            IrInst::SetError(dst, src) => write!(f, "{} = set_error {}", dst, src),
            IrInst::Catch(dst, val, catch_l, merge_l) => write!(f, "{} = catch {}, {}, {}", dst, val, catch_l, merge_l),
        }
    }
}

// ═══════════════════════════════════════════════
// Helper functions
// ═══════════════════════════════════════════════

fn assign_op_to_ir_op(op: AssignOp) -> IrOp {
    match op {
        AssignOp::AddEq => IrOp::Add,
        AssignOp::SubEq => IrOp::Sub,
        AssignOp::MulEq => IrOp::Mul,
        AssignOp::DivEq => IrOp::Div,
        AssignOp::ModEq => IrOp::Mod,
        AssignOp::Eq | AssignOp::ColonEq => IrOp::Add, // unreachable, handled separately
    }
}

fn assign_op_display(op: AssignOp) -> &'static str {
    match op {
        AssignOp::Eq => "=",
        AssignOp::AddEq => "+=",
        AssignOp::SubEq => "-=",
        AssignOp::MulEq => "*=",
        AssignOp::DivEq => "/=",
        AssignOp::ModEq => "%=",
        AssignOp::ColonEq => ":=",
    }
}
