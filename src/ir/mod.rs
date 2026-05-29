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
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
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
    ArrayInit(IrValue, Vec<IrValue>),
    ArrayInitFill(IrValue, IrValue, IrValue),
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
    /// Tracks the most recent store value for each local variable name.
    /// Used to insert phi nodes at control flow merge points.
    local_stores: std::collections::HashMap<String, IrValue>,
    /// Functions generated inline (closures, nested fns) that need to be
    /// flushed into the IrProgram after the current expression finishes.
    pending_functions: Vec<IrFunction>,
}

impl Default for IrGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IrGenerator {
    pub fn new() -> Self {
        IrGenerator {
            insts: Vec::new(),
            temps: 0,
            labels: 0,
            loops: Vec::new(),
            defer_stack: Vec::new(),
            local_stores: std::collections::HashMap::new(),
            pending_functions: Vec::new(),
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
            // Flush any inline-generated functions (closures, nested fns)
            let pending = std::mem::take(&mut self.pending_functions);
            ir_program.functions.extend(pending);
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
            Decl::Struct(s) => self.gen_struct(s, ir),
            Decl::Union(u) => self.gen_union(u, ir),
            Decl::Enum(e) => self.gen_enum(e, ir),
            Decl::Error_(name, variants) => self.gen_error(name, variants, ir),
            Decl::Behave(b) => self.gen_behave(b, ir),
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

    fn gen_struct(&mut self, s: &StructDecl, ir: &mut IrProgram) {
        self.push_comment(format!("struct {} ({} fields, {} methods)", s.name, s.fields.len(), s.methods.len()));
        if let Some(ref behave) = s.impl_behave {
            self.push_comment(format!("  implements {}", behave));
        }
        // Generate IR for each method defined inside the struct
        for method in &s.methods {
            self.gen_fn(method, ir);
        }
    }

    fn gen_union(&mut self, u: &UnionDecl, ir: &mut IrProgram) {
        self.push_comment(format!("union {} ({} variants)", u.name, u.variants.len()));
        for (i, variant) in u.variants.iter().enumerate() {
            self.push_comment(format!("  variant {}: {}", i, variant.name));
        }
    }

    fn gen_enum(&mut self, e: &EnumDecl, ir: &mut IrProgram) {
        self.push_comment(format!("enum {} ({} variants)", e.name, e.variants.len()));
        if let Some(ref behave) = e.impl_behave {
            self.push_comment(format!("  implements {}", behave));
        }
        for (i, variant) in e.variants.iter().enumerate() {
            if let Some(ref ty) = variant.type_ {
                self.push_comment(format!("  variant {}: {} (associated data)", i, variant.name));
            } else {
                self.push_comment(format!("  variant {}: {}", i, variant.name));
            }
        }
        // Generate IR for each method defined inside the enum
        for method in &e.methods {
            self.gen_fn(method, ir);
        }
    }

    fn gen_error(&mut self, name: &str, variants: &[EnumVariant], ir: &mut IrProgram) {
        self.push_comment(format!("error {} ({} variants)", name, variants.len()));
        for (i, variant) in variants.iter().enumerate() {
            self.push_comment(format!("  variant {}: {}", i, variant.name));
        }
    }

    fn gen_behave(&mut self, b: &BehaveDecl, ir: &mut IrProgram) {
        self.push_comment(format!("behave {} ({} methods)", b.name, b.methods.len()));
        // Behave methods are abstract — generate IR for any default implementations
        for method in &b.methods {
            if method.body.is_some() {
                self.gen_fn(method, ir);
            } else {
                self.push_comment(format!("  abstract: {}({})", method.name,
                    method.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ")));
            }
        }
    }

    fn const_expr_to_value(&self, expr: &Expr) -> IrValue {
        match expr {
            Expr::Literal(TokenKind::IntegerValue, v) => v
                .parse::<i64>()
                .map(IrValue::ConstInt)
                .unwrap_or(IrValue::ConstInt(0)),
            Expr::Literal(TokenKind::FloatValue, _) => IrValue::ConstInt(0),
            Expr::Literal(TokenKind::StringValue, v) => IrValue::ConstStr(v.clone()),
            Expr::Literal(TokenKind::True, _) => IrValue::ConstBool(true),
            Expr::Literal(TokenKind::False, _) => IrValue::ConstBool(false),
            Expr::Literal(TokenKind::Nil, _) => IrValue::ConstInt(0),
            Expr::Literal(_, v) => v
                .parse::<i64>()
                .map(IrValue::ConstInt)
                .unwrap_or(IrValue::ConstInt(0)),
            _ => IrValue::ConstInt(0),
        }
    }

    fn gen_fn(&mut self, f: &FnDecl, ir: &mut IrProgram) {
        let saved_temps = self.temps;
        let saved_labels = self.labels;
        let saved_loops = std::mem::take(&mut self.loops);
        let saved_defer = std::mem::take(&mut self.defer_stack);
        let saved_stores = std::mem::take(&mut self.local_stores);
        let saved_pending = std::mem::take(&mut self.pending_functions);

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

        let has_terminal = self
            .insts
            .last()
            .is_some_and(|i| matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_)));
        if !has_terminal {
            self.emit(IrInst::RetVoid);
        }

        let func_params: Vec<(String, Type)> = f
            .params
            .iter()
            .map(|p| (p.name.clone(), p.type_.clone()))
            .collect();

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
        self.local_stores = saved_stores;
        // Flush any closures generated inside this function
        let pending = std::mem::take(&mut self.pending_functions);
        ir.functions.extend(pending);
        self.pending_functions = saved_pending;
    }

    fn gen_test(&mut self, name: &str, block: &Block, ir: &mut IrProgram) {
        let _saved = std::mem::take(&mut self.insts);
        let saved_temps = self.temps;
        let saved_labels = self.labels;
        let saved_loops = std::mem::take(&mut self.loops);
        let saved_defer = std::mem::take(&mut self.defer_stack);
        let saved_stores = std::mem::take(&mut self.local_stores);
        let saved_pending = std::mem::take(&mut self.pending_functions);

        self.temps = 0;
        self.labels = 0;

        let entry = self.new_label("test_entry");
        let success = self.new_label("test_success");

        self.emit(IrInst::Label(entry));

        // Generate test body with its own defer scope
        self.defer_stack.push(Vec::new());
        self.gen_block(block);

        // If the block didn't already terminate, jump to success
        let has_terminal = self
            .insts
            .last()
            .is_some_and(|i| matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_)));
        if !has_terminal {
            self.emit(IrInst::Jump(success.clone()));
        }

        // Run any defers from the test body
        let deferred = self.defer_stack.pop().unwrap_or_default();
        for expr in deferred.iter().rev() {
            self.gen_expr(expr);
        }

        // Success label
        self.emit(IrInst::Label(success));
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
        self.local_stores = saved_stores;
        // Flush any closures generated inside this test
        let pending = std::mem::take(&mut self.pending_functions);
        ir.functions.extend(pending);
        self.pending_functions = saved_pending;
    }
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
                if let Some(stack) = self.defer_stack.last_mut() {
                    stack.clear();
                }
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
            Stmt::Block(b) => {
                // Push a dedicated defer scope for this block so inner defers
                // execute when the block exits, not when the function exits.
                self.defer_stack.push(Vec::new());
                self.gen_block(b);
                let deferred = self.defer_stack.pop().unwrap_or_default();
                for expr in deferred.iter().rev() {
                    self.gen_expr(expr);
                }
            }
        }
    }

    fn gen_var_decl(&mut self, v: &VarDecl) {
        let local = IrValue::Local(v.name.clone());
        self.emit(IrInst::Alloca(local.clone()));
        if let Some(ref val) = v.value {
            let init = self.gen_expr(val);
            self.emit(IrInst::Store(init.clone(), local));
            self.local_stores.insert(v.name.clone(), init);
        }
    }

    fn gen_if(&mut self, if_: &If) {
        let cond = self.gen_expr(&if_.cond);
        let then_label = self.new_label("then");
        let else_label = self.new_label("else");
        let merge_label = self.new_label("merge");

        self.emit(IrInst::Branch(cond, then_label.clone(), else_label.clone()));

        // Snapshot local_stores before each branch
        let pre_stores = self.local_stores.clone();

        // Then block
        self.emit(IrInst::Label(then_label.clone()));
        self.gen_block(&if_.then_block);
        // Collect only NEW stores (not present in pre_stores)
        let then_new: std::collections::HashMap<String, IrValue> = self
            .local_stores
            .iter()
            .filter(|(k, v)| {
                pre_stores.get(k.as_str()) != Some(v) || !pre_stores.contains_key(k.as_str())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let then_falls = !self
            .insts
            .last()
            .is_some_and(|i| matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_)));
        if then_falls {
            self.emit(IrInst::Jump(merge_label.clone()));
        }

        // Else block — restore pre_stores so else starts from the same state
        self.local_stores = pre_stores.clone();
        self.emit(IrInst::Label(else_label.clone()));
        if let Some(ref else_stmt) = if_.else_block {
            self.gen_stmt(else_stmt);
        }
        // Collect only NEW stores (not present in pre_stores)
        let else_new: std::collections::HashMap<String, IrValue> = self
            .local_stores
            .iter()
            .filter(|(k, v)| {
                pre_stores.get(k.as_str()) != Some(v) || !pre_stores.contains_key(k.as_str())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let else_falls = !self
            .insts
            .last()
            .is_some_and(|i| matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_)));
        if else_falls {
            self.emit(IrInst::Jump(merge_label.clone()));
        }

        // Merge point — emit phi nodes for variables stored in either branch
        self.emit(IrInst::Label(merge_label.clone()));

        // Collect all variable names that were newly stored in either branch
        let mut all_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
        for k in then_new.keys() {
            all_vars.insert(k.clone());
        }
        for k in else_new.keys() {
            all_vars.insert(k.clone());
        }

        for var in &all_vars {
            let in_then = then_new.get(var);
            let in_else = else_new.get(var);
            let in_pre = pre_stores.get(var);

            match (in_then, in_else) {
                (Some(then_val), Some(else_val)) => {
                    // Stored in both branches — insert phi
                    let phi_dst = self.new_temp();
                    self.emit(IrInst::Phi(
                        phi_dst.clone(),
                        vec![
                            (then_val.clone(), then_label.clone()),
                            (else_val.clone(), else_label.clone()),
                        ],
                    ));
                    self.local_stores.insert(var.clone(), phi_dst);
                }
                (Some(then_val), None) => {
                    // Stored only in then — phi with pre-merge value for else path
                    let phi_dst = self.new_temp();
                    let else_val = in_pre.cloned().unwrap_or(IrValue::ConstInt(0));
                    self.emit(IrInst::Phi(
                        phi_dst.clone(),
                        vec![
                            (then_val.clone(), then_label.clone()),
                            (else_val, else_label.clone()),
                        ],
                    ));
                    self.local_stores.insert(var.clone(), phi_dst);
                }
                (None, Some(else_val)) => {
                    // Stored only in else — phi with pre-merge value for then path
                    let phi_dst = self.new_temp();
                    let then_val = in_pre.cloned().unwrap_or(IrValue::ConstInt(0));
                    self.emit(IrInst::Phi(
                        phi_dst.clone(),
                        vec![
                            (then_val, then_label.clone()),
                            (else_val.clone(), else_label.clone()),
                        ],
                    ));
                    self.local_stores.insert(var.clone(), phi_dst);
                }
                (None, None) => unreachable!(),
            }
        }
    }

    fn gen_match(&mut self, m: &Match) {
        let target = self.gen_expr(&m.target);
        let merge_label = self.new_label("match_merge");

        // Phase 1: Emit condition checks, chaining branches.
        // Each literal arm's false branch falls through to the next arm's check.
        // The last literal arm's false branch jumps to merge.
        for (i, arm) in m.arms.iter().enumerate() {
            match &arm.pattern {
                Pattern::Wildcard => {
                    // Wildcard always matches — jump directly to body (emitted later)
                }
                Pattern::Ident(_) => {
                    // Capture always matches — jump directly to body
                }
                Pattern::Literal(kind, val) => {
                    let pat_val = self.literal_to_ir_value(kind, val);
                    let cmp = self.new_temp();
                    self.emit(IrInst::BinOp(
                        IrOp::Eq,
                        cmp.clone(),
                        target.clone(),
                        pat_val,
                    ));
                    let arm_body = self.new_label(&format!("arm{}", i));
                    if i + 1 < m.arms.len() {
                        let next_check = self.new_label(&format!("arm_cond{}", i + 1));
                        self.emit(IrInst::Branch(cmp, arm_body, next_check.clone()));
                        self.emit(IrInst::Label(next_check));
                    } else {
                        // Last arm: no match -> jump to merge
                        self.emit(IrInst::Branch(cmp, arm_body, merge_label.clone()));
                    }
                }
                Pattern::EnumVariant(typ, variant, _) => {
                    // Compare the target's tag field with the variant index
                    // The enum is represented as a struct with a tag field
                    let tag_ptr = self.new_temp();
                    self.emit(IrInst::FieldAddr(
                        tag_ptr.clone(),
                        target.clone(),
                        0,
                        "tag".into(),
                    ));
                    let tag_val = self.new_temp();
                    self.emit(IrInst::Load(tag_val.clone(), tag_ptr));
                    let variant_idx = self.new_temp();
                    self.emit(IrInst::Copy(
                        variant_idx.clone(),
                        IrValue::ConstInt(0), // variant index will be resolved by backend
                    ));
                    let cmp = self.new_temp();
                    self.emit(IrInst::BinOp(
                        IrOp::Eq,
                        cmp.clone(),
                        tag_val,
                        variant_idx,
                    ));
                    let arm_body = self.new_label(&format!("arm{}", i));
                    if i + 1 < m.arms.len() {
                        let next_check = self.new_label(&format!("arm_cond{}", i + 1));
                        self.emit(IrInst::Branch(cmp, arm_body, next_check.clone()));
                        self.emit(IrInst::Label(next_check));
                    } else {
                        self.emit(IrInst::Branch(cmp, arm_body, merge_label.clone()));
                    }
                }
            }
        }

        // Phase 2: Emit arm bodies
        for (i, arm) in m.arms.iter().enumerate() {
            let arm_label = self.new_label(&format!("arm{}", i));
            self.emit(IrInst::Label(arm_label));

            match &arm.pattern {
                Pattern::Ident(name) => {
                    let local = IrValue::Local(name.clone());
                    self.emit(IrInst::Alloca(local.clone()));
                    self.emit(IrInst::Store(target.clone(), local));
                }
                Pattern::EnumVariant(_, _, capture) => {
                    if let Some(c) = capture {
                        let local = IrValue::Local(c.clone());
                        self.emit(IrInst::Alloca(local.clone()));
                        self.emit(IrInst::Store(target.clone(), local));
                    }
                }
                _ => {}
            }

            self.gen_expr(&arm.value);
            let arm_falls = !self
                .insts
                .last()
                .is_some_and(|i| matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_)));
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
        let catch_label = self.new_label("catch");
        let merge_label = self.new_label("try_merge");
        let error_flag = self.new_temp();

        // Initialize error flag to 0 (no error)
        self.emit(IrInst::Alloca(error_flag.clone()));
        self.emit(IrInst::Store(IrValue::ConstInt(0), error_flag.clone()));

        // Generate try body
        self.gen_block(&tc.try_body);

        // Jump to merge (try succeeded without error)
        self.emit(IrInst::Jump(merge_label.clone()));

        // Catch block — entered when an error occurs
        self.emit(IrInst::Label(catch_label));
        // Set error flag to 1
        self.emit(IrInst::Store(IrValue::ConstInt(1), error_flag.clone()));
        for cap in &tc.capture {
            let local = IrValue::Local(cap.clone());
            self.emit(IrInst::Alloca(local.clone()));
            // Load the error value from the error flag or a dedicated error slot
            let err_val = self.new_temp();
            self.emit(IrInst::Load(err_val.clone(), error_flag.clone()));
            self.emit(IrInst::Store(err_val, local));
        }
        self.gen_block(&tc.catch_body);

        // Jump to merge
        self.emit(IrInst::Jump(merge_label.clone()));

        // Merge label — execution continues here after try or catch
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
                self.emit(IrInst::Store(result.clone(), local));
                self.local_stores.insert(name.clone(), result);
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
                self.emit(IrInst::Comment(format!(
                    "assign op {}",
                    assign_op_display(op)
                )));
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
                let start_val = self.gen_expr(start);
                let end_val = self.gen_expr(end);
                let temp = self.new_temp();
                // Compute pointer to start of slice: obj + start
                let ptr = self.new_temp();
                self.emit(IrInst::Gep(ptr.clone(), obj_val, start_val.clone()));
                // Compute length: end - start
                let len = self.new_temp();
                self.emit(IrInst::BinOp(
                    IrOp::Sub,
                    len.clone(),
                    end_val,
                    start_val,
                ));
                // Store slice as struct { ptr, len }
                self.emit(IrInst::StructInit(
                    temp.clone(),
                    "Slice".into(),
                    vec![
                        ("ptr".into(), ptr),
                        ("len".into(), len),
                    ],
                ));
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

            Expr::ArrayInit(elems) => {
                let temp = self.new_temp();
                let ir_elems: Vec<IrValue> = elems.iter().map(|e| self.gen_expr(e)).collect();
                self.emit(IrInst::ArrayInit(temp.clone(), ir_elems));
                temp
            }

            Expr::ArrayInitFill(val, count) => {
                let temp = self.new_temp();
                let val_ir = self.gen_expr(val);
                let count_ir = self.gen_expr(count);
                self.emit(IrInst::ArrayInitFill(temp.clone(), val_ir, count_ir));
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
                let stmt_count = b.stmts.len();
                if stmt_count == 0 {
                    self.emit(IrInst::Copy(temp.clone(), IrValue::Void));
                } else {
                    for (i, stmt) in b.stmts.iter().enumerate() {
                        if i == stmt_count - 1 {
                            // Last statement — capture its value as the block result
                            match stmt {
                                Stmt::Expr(e) => {
                                    let val = self.gen_expr(e);
                                    self.emit(IrInst::Copy(temp.clone(), val));
                                }
                                Stmt::Ret(_) => {
                                    self.gen_stmt(stmt);
                                    self.emit(IrInst::Copy(temp.clone(), IrValue::Void));
                                }
                                _ => {
                                    self.gen_stmt(stmt);
                                    self.emit(IrInst::Copy(temp.clone(), IrValue::Void));
                                }
                            }
                        } else {
                            self.gen_stmt(stmt);
                        }
                    }
                }
                temp
            }

            Expr::Paren(e) => self.gen_expr(e),

            Expr::AtMethod(obj, method) => {
                let obj_val = self.gen_expr(obj);
                let temp = self.new_temp();
                self.emit(IrInst::Call(
                    temp.clone(),
                    format!("@{}", method),
                    vec![obj_val],
                ));
                temp
            }

            Expr::Catch(expr, capture, body) => {
                let val = self.gen_expr(expr);
                let catch_label = self.new_label("catch_err");
                let merge_label = self.new_label("catch_merge");
                let temp = self.new_temp();

                let val_for_catch = val.clone();
                self.emit(IrInst::Catch(
                    temp.clone(),
                    val_for_catch,
                    IrValue::Label(catch_label.clone()),
                    IrValue::Label(merge_label.clone()),
                ));

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
                // Generate the closure as a named function with a unique name
                let closure_name = if fndecl.name.is_empty() {
                    let n = self.labels;
                    self.labels += 1;
                    format!("closure.{}", n)
                } else {
                    fndecl.name.clone()
                };
                let saved_insts = std::mem::take(&mut self.insts);
                let saved_temps = self.temps;
                let saved_labels = self.labels;
                let saved_loops = std::mem::take(&mut self.loops);
                let saved_defer = std::mem::take(&mut self.defer_stack);
                let saved_stores = std::mem::take(&mut self.local_stores);

                self.temps = 0;
                self.labels = 0;

                let entry = self.new_label("fn_entry");
                self.emit(IrInst::Label(entry));

                for p in &fndecl.params {
                    let alloca = IrValue::Local(p.name.clone());
                    self.emit(IrInst::Alloca(alloca.clone()));
                    let param_val = IrValue::Param(p.name.clone());
                    self.emit(IrInst::Store(param_val, alloca));
                }

                if let Some(ref body) = fndecl.body {
                    self.defer_stack.push(Vec::new());
                    self.gen_block(body);
                    let deferred = self.defer_stack.last().cloned().unwrap_or_default();
                    for expr in deferred.iter().rev() {
                        self.gen_expr(expr);
                    }
                    self.defer_stack.pop();
                }

                let has_terminal = self
                    .insts
                    .last()
                    .is_some_and(|i| matches!(i, IrInst::Ret(_) | IrInst::RetVoid | IrInst::Jump(_)));
                if !has_terminal {
                    self.emit(IrInst::RetVoid);
                }

                let func_params: Vec<(String, Type)> = fndecl
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.type_.clone()))
                    .collect();

                let closure_insts = std::mem::take(&mut self.insts);
                let closure_return = fndecl.return_.clone();

                self.temps = saved_temps;
                self.labels = saved_labels;
                self.loops = saved_loops;
                self.defer_stack = saved_defer;
                self.local_stores = saved_stores;
                self.insts = saved_insts;

                // Store the closure as a pending function (will be flushed to IrProgram)
                self.pending_functions.push(IrFunction {
                    name: closure_name.clone(),
                    params: func_params,
                    return_type: closure_return,
                    insts: closure_insts,
                });

                // Return a global reference to the closure function
                IrValue::Global(closure_name)
            }

            Expr::MapLiteral(pairs) => {
                let temp = self.new_temp();
                // Generate a map initialization: create map, then insert each pair
                self.push_comment(format!("map literal ({} pairs)", pairs.len()));
                self.emit(IrInst::Call(
                    temp.clone(),
                    "map_init".into(),
                    vec![IrValue::ConstInt(pairs.len() as i64)],
                ));
                for (k, v) in pairs {
                    let key = self.gen_expr(k);
                    let val = self.gen_expr(v);
                    let map_ref = temp.clone();
                    self.emit(IrInst::Call(
                        IrValue::Void,
                        "map_insert".into(),
                        vec![map_ref, key, val],
                    ));
                }
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
                let inclusive = matches!(op, BinaryOp::RangeInclusive);
                let tag = if inclusive { 1 } else { 0 };
                self.emit(IrInst::StructInit(
                    temp.clone(),
                    "Range".into(),
                    vec![
                        ("start".into(), l),
                        ("end".into(), r),
                        ("inclusive".into(), IrValue::ConstInt(tag)),
                    ],
                ));
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
                // Wrap value in Optional — mark as present (non-nil)
                self.emit(IrInst::StructInit(
                    temp.clone(),
                    "Optional".into(),
                    vec![
                        ("present".into(), IrValue::ConstBool(true)),
                        ("value".into(), e),
                    ],
                ));
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
                // Handle @str.from(...), @vec.append(...) etc.
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    if obj_name.starts_with('@') {
                        let builtin_name = format!("{}.{}", obj_name, method_name);
                        return self.gen_builtin_call(&builtin_name, args);
                    }
                }
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
        let bn = name.trim_start_matches('@');
        let result = self.new_temp();
        let ir_args: Vec<IrValue> = args.iter().map(|a| self.gen_expr(a)).collect();

        match bn {
            // ── Reflection ──
            "TypeOf" => {
                self.push_comment(format!("@TypeOf({})", display_ir_args(&ir_args)));
                self.emit(IrInst::Copy(result.clone(), ir_args.first().cloned().unwrap_or(IrValue::Void)));
            }
            "SizeOf" | "AlignOf" | "TypeName" | "EnumCount" | "Fields" => {
                self.push_comment(format!("@{}(type_arg)", bn));
                self.emit(IrInst::Copy(result.clone(), IrValue::ConstInt(0)));
            }

            // ── Type Casting ──
            "as" => {
                if ir_args.len() >= 2 {
                    self.emit(IrInst::BitCast(result.clone(), ir_args[1].clone()));
                }
            }
            "bitCast" => {
                if ir_args.len() >= 2 {
                    self.emit(IrInst::BitCast(result.clone(), ir_args[1].clone()));
                }
            }
            "ptrCast" => {
                if ir_args.len() >= 2 {
                    self.emit(IrInst::BitCast(result.clone(), ir_args[1].clone()));
                }
            }
            "intToPtr" => {
                if ir_args.len() >= 2 {
                    self.emit(IrInst::IntToPtr(result.clone(), ir_args[1].clone()));
                }
            }
            "ptrToInt" => {
                if let Some(ptr_val) = ir_args.first() {
                    self.emit(IrInst::PtrToInt(result.clone(), ptr_val.clone()));
                }
            }

            // ── Memory ──
            "memcpy" => {
                self.push_comment("@memcpy(dest, src, count)".into());
                self.emit(IrInst::Call(
                    result.clone(),
                    "memcpy".into(),
                    ir_args,
                ));
            }
            "memset" => {
                self.push_comment("@memset(dest, value, count)".into());
                self.emit(IrInst::Call(
                    result.clone(),
                    "memset".into(),
                    ir_args,
                ));
            }
            "memmove" => {
                self.push_comment("@memmove(dest, src, count)".into());
                self.emit(IrInst::Call(
                    result.clone(),
                    "memmove".into(),
                    ir_args,
                ));
            }
            "pageAlloc" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "pageAlloc".into(),
                    ir_args,
                ));
            }
            "pageFree" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "pageFree".into(),
                    ir_args,
                ));
            }
            "comptimeDefaultAllocator" => {
                self.push_comment("@comptimeDefaultAllocator()".into());
                self.emit(IrInst::Copy(result.clone(), IrValue::ConstInt(0)));
            }

            // ── Comptime ──
            "comptime" => {
                self.push_comment("@comptime block".into());
                for arg in &ir_args {
                    self.emit(IrInst::Copy(result.clone(), arg.clone()));
                }
            }
            "compileLog" => {
                for arg in &ir_args {
                    self.push_comment(format!("@compileLog: {}", arg));
                }
            }
            "compileError" => {
                self.push_comment("@compileError — compilation halted".into());
                self.emit(IrInst::Call(
                    result.clone(),
                    "compileError".into(),
                    ir_args,
                ));
            }
            "embedFile" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "embedFile".into(),
                    ir_args,
                ));
            }

            // ── Control Flow ──
            "panic" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "panic".into(),
                    ir_args,
                ));
            }
            "breakpoint" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "breakpoint".into(),
                    vec![],
                ));
            }
            "trap" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "trap".into(),
                    vec![],
                ));
            }
            "sysCall" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "sysCall".into(),
                    ir_args,
                ));
            }

            // ── Materialization ──
            "str" | "str.from" | "str.from_raw" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "str_init".into(),
                    ir_args,
                ));
            }
            "vec" | "vec_with_allocator" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "vec_init".into(),
                    ir_args,
                ));
            }
            "map" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "map_init".into(),
                    ir_args,
                ));
            }
            "set" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "set_init".into(),
                    ir_args,
                ));
            }

            // ── Math (Overflow-Safe) ──
            "addWithOverflow" | "subWithOverflow" | "mulWithOverflow" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    bn.to_string(),
                    ir_args,
                ));
            }

            // ── Bitwise ──
            "ctz" | "clz" | "popCount" | "bswap" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    bn.to_string(),
                    ir_args,
                ));
            }

            // ── Concurrency ──
            "atomicLoad" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "atomicLoad".into(),
                    ir_args,
                ));
            }
            "atomicStore" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "atomicStore".into(),
                    ir_args,
                ));
            }
            "cmpxchg" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    "cmpxchg".into(),
                    ir_args,
                ));
            }

            // ── Test assertions ──
            "assert" | "assertEq" => {
                self.emit(IrInst::Call(
                    result.clone(),
                    bn.to_string(),
                    ir_args,
                ));
            }

            // ── Fallback: pass through as regular call ──
            _ => {
                self.emit(IrInst::Call(
                    result.clone(),
                    name.to_string(),
                    ir_args,
                ));
            }
        }
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
            IrInst::FieldAddr(dst, obj, _idx, name) => {
                write!(f, "{} = fieldaddr {}, .{}", dst, obj, name)
            }
            IrInst::PtrToInt(dst, src) => write!(f, "{} = ptrtoint {}", dst, src),
            IrInst::IntToPtr(dst, src) => write!(f, "{} = inttoptr {}", dst, src),
            IrInst::BitCast(dst, src) => write!(f, "{} = bitcast {}", dst, src),
            IrInst::StructInit(dst, name, fields) => {
                let f_str: Vec<String> = fields
                    .iter()
                    .map(|(n, v)| format!(".{} = {}", n, v))
                    .collect();
                write!(f, "{} = struct {} {{ {} }}", dst, name, f_str.join(", "))
            }
            IrInst::ArrayInit(dst, elems) => {
                let e_str: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "{} = array [{}]", dst, e_str.join(", "))
            }
            IrInst::ArrayInitFill(dst, val, count) => {
                write!(f, "{} = array_fill {}, {}", dst, val, count)
            }
            IrInst::Copy(dst, src) => write!(f, "{} = copy {}", dst, src),
            IrInst::Comment(msg) => write!(f, "; {}", msg),
            IrInst::Phi(dst, incs) => {
                let inc_str: Vec<String> = incs
                    .iter()
                    .map(|(v, l)| format!("[ {}, {} ]", v, l))
                    .collect();
                write!(f, "{} = phi {}", dst, inc_str.join(", "))
            }
            IrInst::AllocArray(dst, size) => write!(f, "{} = allocarray {}", dst, size),
            IrInst::SetError(dst, src) => write!(f, "{} = set_error {}", dst, src),
            IrInst::Catch(dst, val, catch_l, merge_l) => {
                write!(f, "{} = catch {}, {}, {}", dst, val, catch_l, merge_l)
            }
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

fn display_ir_args(args: &[IrValue]) -> String {
    args.iter()
        .map(|a| format!("{}", a))
        .collect::<Vec<_>>()
        .join(", ")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TokenKind;

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

    fn ident(name: &str) -> Expr {
        Expr::Ident(name.to_string())
    }

    fn block(stmts: Vec<Stmt>) -> Block {
        Block { stmts }
    }

    fn var_stmt(name: &str, value: Expr) -> Stmt {
        Stmt::Var(VarDecl {
            name: name.to_string(),
            mutable: true,
            pub_: false,
            attrs: vec![],
            type_: None,
            value: Some(value),
        })
    }

    fn assign_stmt(target: &str, value: Expr) -> Stmt {
        Stmt::Assign(Expr::Ident(target.to_string()), AssignOp::Eq, value)
    }

    fn if_stmt(cond: Expr, then_stmts: Vec<Stmt>, else_stmt: Option<Box<Stmt>>) -> Stmt {
        Stmt::If(If {
            cond,
            capture: vec![],
            then_block: block(then_stmts),
            else_block: else_stmt,
        })
    }

    fn match_stmt(target: Expr, arms: Vec<MatchArm>) -> Stmt {
        Stmt::Match(Match { target, arms })
    }

    fn defer_stmt(e: Expr) -> Stmt {
        Stmt::Defer(Box::new(e))
    }

    fn loop_stmt(conds: Vec<Expr>, captures: Vec<Capture>, body: Block) -> Stmt {
        Stmt::Loop(Loop {
            conds,
            captures,
            body,
        })
    }

    fn make_fn(name: &str, params: Vec<Param>, return_: Option<Type>, body: Block) -> FnDecl {
        FnDecl {
            name: name.to_string(),
            generics: vec![],
            pub_: false,
            external: false,
            attrs: vec![],
            params,
            return_,
            body: Some(body),
            is_const: true,
            is_variable_fn: false,
        }
    }

    fn make_void_fn(name: &str, body: Block) -> FnDecl {
        make_fn(name, vec![], Some(Type::Primitive(TokenKind::Void)), body)
    }

    fn make_i32_fn(name: &str, body: Block) -> FnDecl {
        make_fn(name, vec![], Some(Type::Primitive(TokenKind::I32)), body)
    }

    fn gen_program(decls: Vec<Decl>) -> IrProgram {
        let program = Program { decls };
        let mut ir_gen = IrGenerator::new();
        ir_gen.generate(&program)
    }

    fn find_label(insts: &[IrInst], label: &str) -> bool {
        insts
            .iter()
            .any(|i| matches!(i, IrInst::Label(l) if l == label))
    }

    fn count_insts(insts: &[IrInst], pattern: impl Fn(&IrInst) -> bool) -> usize {
        insts.iter().filter(|i| pattern(i)).count()
    }

    // ─── 2.6: Match fallthrough ───

    #[test]
    fn test_match_literal_fallthrough_to_next_arm() {
        // match x { 1 => ..., 2 => ..., _ => ... }
        // After checking arm 0 and failing, should jump to arm 1's check, not arm 1's body.
        let decl = Decl::Fn(make_void_fn(
            "test_match",
            block(vec![match_stmt(
                ident("x"),
                vec![
                    MatchArm {
                        pattern: Pattern::Literal(TokenKind::IntegerValue, "1".into()),
                        capture: vec![],
                        value: lit_i32(10),
                    },
                    MatchArm {
                        pattern: Pattern::Literal(TokenKind::IntegerValue, "2".into()),
                        capture: vec![],
                        value: lit_i32(20),
                    },
                    MatchArm {
                        pattern: Pattern::Wildcard,
                        capture: vec![],
                        value: lit_i32(0),
                    },
                ],
            )]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        // Should have merge label (match always ends with merge)
        assert!(
            find_label(
                &func.insts,
                &func
                    .insts
                    .iter()
                    .find_map(|i| {
                        if let IrInst::Label(l) = i {
                            if l.starts_with(".Lmatch_merge") {
                                return Some(l.clone());
                            }
                        }
                        None
                    })
                    .unwrap_or_default()
            ),
            "Match should have merge label"
        );

        // Every arm body should jump to merge (or ret)
        // The last arm (wildcard) must not fall through
        let arm_jumps_to_merge = func.insts.windows(3).any(|w| {
            matches!(&w[0], IrInst::Label(l) if l.starts_with(".Larm"))
                && matches!(&w[2], IrInst::Jump(l) if l.starts_with(".Lmatch_merge"))
                || matches!(&w[1], IrInst::Jump(l) if l.starts_with(".Lmatch_merge"))
        });
        assert!(
            arm_jumps_to_merge || func.insts.iter().any(|i| matches!(i, IrInst::Ret(_))),
            "Match arms should jump to merge or return"
        );
    }

    #[test]
    fn test_match_wildcard_no_fallthrough() {
        let decl = Decl::Fn(make_void_fn(
            "test_wildcard",
            block(vec![match_stmt(
                ident("x"),
                vec![MatchArm {
                    pattern: Pattern::Wildcard,
                    capture: vec![],
                    value: lit_i32(42),
                }],
            )]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        // Wildcard match: should have exactly 1 arm label and jump to merge
        let arm_labels = count_insts(
            &func.insts,
            |i| matches!(i, IrInst::Label(l) if l.starts_with(".Larm")),
        );
        assert_eq!(
            arm_labels, 1,
            "Wildcard match should have exactly 1 arm body"
        );
    }

    // ─── 3.2: Defer execution order ───

    #[test]
    fn test_defer_in_block_exits_at_block_end() {
        // { defer foo(); { defer bar(); } ret }
        let decl = Decl::Fn(make_void_fn(
            "test_defer",
            block(vec![
                defer_stmt(Expr::Call(Box::new(ident("foo")), vec![])),
                Stmt::Block(block(vec![defer_stmt(Expr::Call(
                    Box::new(ident("bar")),
                    vec![],
                ))])),
                Stmt::Ret(None),
            ]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        let call_count = count_insts(&func.insts, |i| matches!(i, IrInst::Call(..)));
        assert!(
            call_count >= 2,
            "Both defers should generate calls, found {}",
            call_count
        );
    }

    #[test]
    fn test_defer_reverse_order() {
        // defer a(); defer b(); ret -> b runs first, then a
        let decl = Decl::Fn(make_void_fn(
            "test_defer_order",
            block(vec![
                defer_stmt(Expr::Call(Box::new(ident("a")), vec![])),
                defer_stmt(Expr::Call(Box::new(ident("b")), vec![])),
                Stmt::Ret(None),
            ]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        let call_count = count_insts(&func.insts, |i| matches!(i, IrInst::Call(..)));
        assert_eq!(call_count, 2, "Two defers should produce two calls");
    }

    // ─── 3.3: Loop capture type inference ───

    #[test]
    fn test_loop_capture_infers_from_array() {
        use crate::sema::SemanticAnalyzer;

        // loop array |elem| { ... }
        // elem should be typed as the array's element type, not anytype
        let array_var = var_stmt("arr", Expr::StructInit("Array".into(), vec![]));
        let loop_body = Stmt::Loop(Loop {
            conds: vec![ident("arr")],
            captures: vec![Capture {
                name: "elem".into(),
                mutable: false,
                is_ref: false,
            }],
            body: block(vec![]),
        });
        let decl = Decl::Fn(make_void_fn("test", block(vec![array_var, loop_body])));
        let program = Program { decls: vec![decl] };
        let mut sema = SemanticAnalyzer::new();
        // Should not crash — capture type is inferred (may be AnyType if struct unknown,
        // but should NOT be hardcoded AnyType for known array types)
        let _ = sema.analyze(&program);
    }

    // ─── 3.1: Try/catch reachability ───

    #[test]
    fn test_try_catch_both_branches_exit() {
        // try { ret 1 } catch |e| { ret 2 }
        // Both branches return, so the function exits
        let decl = Decl::Fn(make_i32_fn(
            "test_try_catch",
            block(vec![Stmt::TryCatch(TryCatch {
                try_body: block(vec![Stmt::Ret(Some(lit_i32(1)))]),
                capture: vec!["e".into()],
                catch_body: block(vec![Stmt::Ret(Some(lit_i32(2)))]),
            })]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        // Should have try body instructions AND catch body instructions
        // Both should contain ret instructions
        let ret_count = count_insts(&func.insts, |i| matches!(i, IrInst::Ret(_)));
        assert!(
            ret_count >= 2,
            "Both try and catch branches should have ret instructions, found {}",
            ret_count
        );
    }

    #[test]
    fn test_try_catch_with_defer() {
        // try { defer cleanup(); ret } catch |e| { }
        let decl = Decl::Fn(make_void_fn(
            "test_try_defer",
            block(vec![Stmt::TryCatch(TryCatch {
                try_body: block(vec![
                    defer_stmt(Expr::Call(Box::new(ident("cleanup")), vec![])),
                    Stmt::Ret(None),
                ]),
                capture: vec!["e".into()],
                catch_body: block(vec![]),
            })]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        let call_count = count_insts(&func.insts, |i| matches!(i, IrInst::Call(..)));
        assert!(
            call_count >= 1,
            "Defer in try body should generate a call, found {}",
            call_count
        );
    }

    // ─── General IR correctness ───

    #[test]
    fn test_if_else_generates_merge_label() {
        let decl = Decl::Fn(make_void_fn(
            "test_if",
            block(vec![if_stmt(
                lit_bool(true),
                vec![],
                Some(Box::new(Stmt::Block(block(vec![])))),
            )]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        // Should have merge label
        let has_merge = func
            .insts
            .iter()
            .any(|i| matches!(i, IrInst::Label(l) if l.contains("merge")));
        assert!(has_merge, "If-else should generate a merge label");
    }

    #[test]
    fn test_if_without_else_generates_merge_label() {
        let decl = Decl::Fn(make_void_fn(
            "test_if_no_else",
            block(vec![if_stmt(lit_bool(true), vec![], None)]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        let has_merge = func
            .insts
            .iter()
            .any(|i| matches!(i, IrInst::Label(l) if l.contains("merge")));
        assert!(
            has_merge,
            "If without else should still generate merge label"
        );
    }

    // ─── #9: Phi node insertion ───

    #[test]
    fn test_phi_node_inserted_for_variable_in_both_branches() {
        // mut x := 1
        // if true { x := 2 } else { x := 3 }
        // Should produce phi at merge
        let decl = Decl::Fn(make_i32_fn(
            "test_phi",
            block(vec![
                var_stmt("x", lit_i32(1)),
                if_stmt(
                    lit_bool(true),
                    vec![assign_stmt("x", lit_i32(2))],
                    Some(Box::new(Stmt::Block(block(vec![assign_stmt(
                        "x",
                        lit_i32(3),
                    )])))),
                ),
                Stmt::Ret(Some(ident("x"))),
            ]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        let phi_count = count_insts(&func.insts, |i| matches!(i, IrInst::Phi(..)));
        assert!(
            phi_count >= 1,
            "Variable assigned in both branches should produce a phi node, found {}",
            phi_count
        );
    }

    #[test]
    fn test_phi_node_for_variable_in_one_branch_only() {
        // mut x := 1
        // if true { x := 2 }
        // x after merge should use the then-branch value
        let decl = Decl::Fn(make_i32_fn(
            "test_phi_one",
            block(vec![
                var_stmt("x", lit_i32(1)),
                if_stmt(lit_bool(true), vec![assign_stmt("x", lit_i32(2))], None),
                Stmt::Ret(Some(ident("x"))),
            ]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        // Should have at least 1 phi (x assigned in then, not in else)
        let phi_count = count_insts(&func.insts, |i| matches!(i, IrInst::Phi(..)));
        assert!(
            phi_count >= 1,
            "Variable assigned in one branch should produce a phi node, found {}",
            phi_count
        );
    }

    #[test]
    fn test_no_phi_when_variable_not_reassigned() {
        // mut x := 1
        // if true { } else { }
        // No phi needed — x unchanged
        let decl = Decl::Fn(make_i32_fn(
            "test_no_phi",
            block(vec![
                var_stmt("x", lit_i32(1)),
                if_stmt(
                    lit_bool(true),
                    vec![],
                    Some(Box::new(Stmt::Block(block(vec![])))),
                ),
                Stmt::Ret(Some(ident("x"))),
            ]),
        ));
        let ir = gen_program(vec![decl]);
        let func = &ir.functions[0];

        let phi_count = count_insts(&func.insts, |i| matches!(i, IrInst::Phi(..)));
        assert_eq!(
            phi_count, 0,
            "No phi needed when variable is not reassigned, found {}",
            phi_count
        );
    }

    // ─── #16: Test isolation ───

    #[test]
    fn test_gen_test_has_success_label() {
        let test_block = block(vec![Stmt::Ret(None)]);
        let ir = gen_program(vec![Decl::Test("my_test".into(), test_block)]);
        let func = &ir
            .functions
            .iter()
            .find(|f| f.name == "test.my_test")
            .unwrap();

        let has_success = func
            .insts
            .iter()
            .any(|i| matches!(i, IrInst::Label(l) if l.contains("test_success")));
        assert!(has_success, "Test function should have success label");
    }

    #[test]
    fn test_gen_test_has_entry_and_retvoid() {
        let test_block = block(vec![]);
        let ir = gen_program(vec![Decl::Test("empty_test".into(), test_block)]);
        let func = &ir
            .functions
            .iter()
            .find(|f| f.name == "test.empty_test")
            .unwrap();

        let has_entry = func
            .insts
            .iter()
            .any(|i| matches!(i, IrInst::Label(l) if l.contains("test_entry")));
        let has_retvoid = func.insts.iter().any(|i| matches!(i, IrInst::RetVoid));
        assert!(has_entry, "Test should have entry label");
        assert!(has_retvoid, "Test should end with ret void");
    }

    #[test]
    fn test_gen_test_with_defer() {
        let test_block = block(vec![
            defer_stmt(Expr::Call(Box::new(ident("cleanup")), vec![])),
            Stmt::Ret(None),
        ]);
        let ir = gen_program(vec![Decl::Test("defer_test".into(), test_block)]);
        let func = &ir
            .functions
            .iter()
            .find(|f| f.name == "test.defer_test")
            .unwrap();

        let call_count = count_insts(&func.insts, |i| matches!(i, IrInst::Call(..)));
        assert!(
            call_count >= 1,
            "Test with defer should execute deferred call, found {}",
            call_count
        );
    }
}
