use crate::ast::*;
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
        | TokenKind::Pipe
        | TokenKind::Caret
        | TokenKind::Tilde
        | TokenKind::ShiftLeft
        | TokenKind::ShiftRight
        | TokenKind::QuestionMark => YELLOW,

        // Arrows => yellow
        TokenKind::Arrow | TokenKind::FatArrow | TokenKind::TildeArrow | TokenKind::ColonColon => {
            YELLOW
        }

        // Access / separators => grey
        TokenKind::Dot
        | TokenKind::DotDot
        | TokenKind::DotDotEquals
        | TokenKind::DotStar
        | TokenKind::Colon
        | TokenKind::Comma
        | TokenKind::At
        | TokenKind::Semicolon
        | TokenKind::Underscore => GREY,

        // Grouping => grey
        TokenKind::LeftParen
        | TokenKind::RightParen
        | TokenKind::LeftBrace
        | TokenKind::RightBrace
        | TokenKind::LeftBracket
        | TokenKind::RightBracket => GREY,

        // Or is no longer emitted by lexer (Pipe is used instead)
        TokenKind::Or => YELLOW,

        // End of file
        TokenKind::Eof => MAGENTA,
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

pub fn print_phase2_header(status: &str) {
    println!(
        "{}Phase {}  {}{}\t\t\t\t{}{}{}",
        MAGENTA, "2", RST, "AST Build", GREEN, status, RST
    );
}

pub fn print_phase3_header(status: &str) {
    println!(
        "{}Phase {}  {}{}\t\t{}{}{}",
        MAGENTA, "3", RST, "Semantic Analysis", GREEN, status, RST
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
        let span = token.span;
        println!(
            "  {g}[{i:>3}]{r}  {g}Type:{r} {c}{k}{r}  \
             {g}Value:{r} {o}{v}{r}  {g}Line:{r} {l}  {g}Span:{r} {sp}:{ep}",
            g = GREY,
            r = RST,
            i = i,
            c = c,
            k = kind_str,
            o = ORNG,
            v = value_str,
            l = line,
            sp = span.0,
            ep = span.1,
        );
    }
}

pub fn print_phase4_header(status: &str) {
    println!(
        "{}Phase {}  {}{}\t\t{}{}{}",
        MAGENTA, "4", RST, "IR Generation", GREEN, status, RST
    );
}

pub fn print_ir(ir_program: &crate::ir::IrProgram) {
    println!();
    println!("{}━━━ Generated IR ━━━{}", BLUE, RST);

    for func in &ir_program.functions {
        println!();
        println!("{}{}Function:{} {}{}", BOLD, LGREEN, RST, PEACH, func.name);
        if !func.params.is_empty() {
            let params: Vec<String> = func
                .params
                .iter()
                .map(|(n, t)| format!("{}{}{}{}", PEACH, n, CYAN, crate::bdg::type_str(t)))
                .collect();
            println!("  {}params{}: {}", GREY, RST, params.join(", "));
        }
        if let Some(ref rt) = func.return_type {
            println!("  {}return{}: {}{}", GREY, RST, CYAN, crate::bdg::type_str(rt));
        }
        println!();

        for inst in &func.insts {
            match inst {
                crate::ir::IrInst::Comment(msg) => {
                    println!("  {}{}{}{}", GREY, "; ", msg, RST);
                }
                crate::ir::IrInst::Label(label) => {
                    println!("{}{}:{}", YELLOW, label, RST);
                }
                inst => {
                    println!("  {}  {}{}", GREY, RST, inst);
                }
            }
        }
    }

    if !ir_program.globals.is_empty() {
        println!();
        println!("{}{}Globals:{}", BOLD, GREEN, RST);
        for g in &ir_program.globals {
            let init_str = g.init.as_ref().map(|v| format!(" = {}", v)).unwrap_or_default();
            println!("  {}{}{}{}", CYAN, g.name, RST, init_str);
        }
    }

    println!();
}

pub fn print_footer(status: &str) {
    println!();
    println!("\t\t{}{}{}", GREEN, status, RST);
    println!();
}

pub fn print_ast(program: &Program) {
    println!();
    println!("{}━━━ AST Tree ━━━{}", BLUE, RST);
    print_decls(&program.decls, 0);
    println!();
}

fn print_decls(decls: &[Decl], indent: usize) {
    for decl in decls {
        print_decl(decl, indent);
    }
}

fn print_decl(decl: &Decl, indent: usize) {
    let i = "  ".repeat(indent);
    match decl {
        Decl::Use(path) => {
            println!("{}{}Use{} {}{}{}", i, GREEN, RST, CYAN, path.join("."), RST);
        }
        Decl::Mod(name, body) => {
            println!("{}{}Mod{} {}{}", i, GREEN, RST, PEACH, name);
            if let Some(decls) = body {
                print_decls(decls, indent + 1);
            }
            println!("{}{}{}/{}", i, GREEN, RST, "Mod");
        }
        Decl::Fn(f) => {
            println!("{}{}Fn{} {}{}", i, LGREEN, RST, PEACH, f.name);
            if !f.attrs.is_empty() {
                for a in &f.attrs {
                    print_annotation(a, indent + 1);
                }
            }
            if !f.generics.is_empty() {
                println!("{}  {}Generics{} {:?}", i, YELLOW, RST, f.generics);
            }
            if f.pub_ {
                println!("{}  {}pub{}", i, GREEN, RST);
            }
            for p in &f.params {
                println!(
                    "{}  {}Param{} {}{} {}{}",
                    i,
                    YELLOW,
                    RST,
                    PEACH,
                    p.name,
                    CYAN,
                    type_str(&p.type_)
                );
            }
            if let Some(ref rt) = f.return_ {
                println!("{}  {}Return{} {}{}", i, YELLOW, RST, CYAN, type_str(rt));
            }
            if let Some(ref b) = f.body {
                print_block(b, indent + 1);
            }
            println!("{}{}{}/{}", i, LGREEN, RST, "Fn");
        }
        Decl::Struct(s) => {
            println!("{}{}Struct{} {}{}", i, GREEN, RST, PEACH, s.name);
            if !s.attrs.is_empty() {
                for a in &s.attrs {
                    print_annotation(a, indent + 1);
                }
            }
            if !s.generics.is_empty() {
                println!("{}  {}Generics{} {:?}", i, YELLOW, RST, s.generics);
            }
            if let Some(ref ib) = s.impl_behave {
                println!("{}  {}Implements{} {}{}", i, YELLOW, RST, PEACH, ib);
            }
            for f in &s.fields {
                println!(
                    "{}  {}Field{} {}{} {}{}",
                    i,
                    YELLOW,
                    RST,
                    PEACH,
                    f.name,
                    CYAN,
                    type_str(&f.type_)
                );
            }
            for m in &s.methods {
                println!("{}  {}Method{} {}{}", i, MAGENTA, RST, PEACH, m.name);
                for p in &m.params {
                    println!(
                        "{}    {}Param{} {}{} {}{}",
                        i,
                        YELLOW,
                        RST,
                        PEACH,
                        p.name,
                        CYAN,
                        type_str(&p.type_)
                    );
                }
                if let Some(ref rt) = m.return_ {
                    println!("{}    {}Return{} {}{}", i, YELLOW, RST, CYAN, type_str(rt));
                }
                if let Some(ref b) = m.body {
                    print_block(b, indent + 2);
                }
            }
            println!("{}{}{}/{}", i, GREEN, RST, "Struct");
        }
        Decl::Union(u) => {
            println!("{}{}Union{} {}{}", i, GREEN, RST, PEACH, u.name);
            if !u.attrs.is_empty() {
                for a in &u.attrs {
                    print_annotation(a, indent + 1);
                }
            }
            for v in &u.variants {
                println!(
                    "{}  {}Variant{} {}{} {}{}",
                    i,
                    YELLOW,
                    RST,
                    PEACH,
                    v.name,
                    CYAN,
                    type_str(&v.type_)
                );
            }
        }
        Decl::Enum(e) => {
            println!("{}{}Enum{} {}{}", i, GREEN, RST, PEACH, e.name);
            if !e.attrs.is_empty() {
                for a in &e.attrs {
                    print_annotation(a, indent + 1);
                }
            }
            for v in &e.variants {
                if let Some(ref t) = v.type_ {
                    println!(
                        "{}  {}Variant{} {}{} {}{}",
                        i,
                        YELLOW,
                        RST,
                        PEACH,
                        v.name,
                        CYAN,
                        type_str(t)
                    );
                } else {
                    println!("{}  {}Variant{} {}{}", i, YELLOW, RST, PEACH, v.name);
                }
            }
        }
        Decl::Error_(name, variants) => {
            println!("{}{}Error{} {}{}", i, GREEN, RST, PEACH, name);
            for v in variants {
                println!("{}  {}Variant{} {}{}", i, YELLOW, RST, PEACH, v.name);
            }
        }
        Decl::Behave(b) => {
            println!("{}{}Behave{} {}{}", i, GREEN, RST, PEACH, b.name);
            if !b.attrs.is_empty() {
                for a in &b.attrs {
                    print_annotation(a, indent + 1);
                }
            }
            for m in &b.methods {
                println!("{}  {}Method{} {}{}", i, MAGENTA, RST, PEACH, m.name);
                for p in &m.params {
                    println!(
                        "{}    {}Param{} {}{} {}{}",
                        i,
                        YELLOW,
                        RST,
                        PEACH,
                        p.name,
                        CYAN,
                        type_str(&p.type_)
                    );
                }
                if let Some(ref rt) = m.return_ {
                    println!("{}    {}Return{} {}{}", i, YELLOW, RST, CYAN, type_str(rt));
                }
            }
        }
        Decl::Var(v) => {
            let kind = if v.mutable { "mut" } else { "let" };
            if !v.attrs.is_empty() {
                for a in &v.attrs {
                    print_annotation(a, indent);
                }
            }
            if let Some(ref t) = v.type_ {
                println!(
                    "{}{}{}{} {}{} {}{} =",
                    i,
                    YELLOW,
                    kind,
                    RST,
                    PEACH,
                    v.name,
                    CYAN,
                    type_str(t)
                );
            } else {
                println!("{}{}{}{} {}{} :=", i, YELLOW, kind, RST, PEACH, v.name);
            }
            if let Some(ref val) = v.value {
                print_expr(val, indent + 1);
            }
        }
        Decl::Const(c) => {
            if !c.attrs.is_empty() {
                for a in &c.attrs {
                    print_annotation(a, indent);
                }
            }
            if let Some(ref t) = c.type_ {
                println!(
                    "{}{}Const{} {}{} {}{} ::",
                    i,
                    GREEN,
                    RST,
                    PEACH,
                    c.name,
                    CYAN,
                    type_str(t)
                );
            } else {
                println!("{}{}Const{} {}{} ::", i, GREEN, RST, PEACH, c.name);
            }
            if let Some(ref val) = c.value {
                print_expr(val, indent + 1);
            }
        }
        Decl::TypeAlias(name, t) => {
            println!(
                "{}{}TypeAlias{} {}{} = {}{}",
                i,
                GREEN,
                RST,
                PEACH,
                name,
                CYAN,
                type_str(t)
            );
        }
        Decl::Test(name, block) => {
            println!("{}{}Test{} {}{}", i, GREEN, RST, PEACH, name);
            print_block(block, indent + 1);
        }
    }
}

fn print_block(block: &Block, indent: usize) {
    let i = "  ".repeat(indent);
    println!("{} {}Block{} {{", i, MAGENTA, RST);
    for stmt in &block.stmts {
        print_stmt(stmt, indent + 1);
    }
    println!("{} {} {} }}", i, MAGENTA, RST);
}

fn print_stmt(stmt: &Stmt, indent: usize) {
    let i = "  ".repeat(indent);
    match stmt {
        Stmt::Expr(e) => {
            print_expr(e, indent);
        }
        Stmt::Var(v) => {
            let kind = if v.mutable { "mut" } else { "let" };
            if let Some(ref t) = v.type_ {
                println!(
                    "{}{}{}{} {}{} {}{} =",
                    i,
                    YELLOW,
                    kind,
                    RST,
                    PEACH,
                    v.name,
                    CYAN,
                    type_str(t)
                );
            } else {
                println!("{}{}{}{} {}{} :=", i, YELLOW, kind, RST, PEACH, v.name);
            }
            if let Some(ref val) = v.value {
                print_expr(val, indent + 1);
            }
        }
        Stmt::Ret(e) => match e {
            Some(ex) => {
                println!("{}{}ret{}", i, MAGENTA, RST);
                print_expr(ex, indent + 1);
            }
            None => println!("{}{}ret{}", i, MAGENTA, RST),
        },
        Stmt::Stop => println!("{}{}stop{}", i, MAGENTA, RST),
        Stmt::Next => println!("{}{}next{}", i, MAGENTA, RST),
        Stmt::If(if_) => {
            println!("{}{}if{}", i, MAGENTA, RST);
            print_expr(&if_.cond, indent + 1);
            if !if_.capture.is_empty() {
                println!("{}  {}capture{} {:?}", i, YELLOW, RST, if_.capture);
            }
            print_block(&if_.then_block, indent);
            if let Some(ref else_) = if_.else_block {
                println!("{}  {}else{}", i, MAGENTA, RST);
                match else_.as_ref() {
                    Stmt::If(inner) => print_stmt(&Stmt::If(inner.clone()), indent + 1),
                    Stmt::Block(b) => print_block(b, indent),
                    _ => print_stmt(else_, indent),
                }
            }
        }
        Stmt::Match(m) => {
            println!("{}{}match{}", i, MAGENTA, RST);
            print_expr(&m.target, indent + 1);
            for arm in &m.arms {
                println!("{}  {}Arm{} =>", i, YELLOW, RST);
                print_pattern(&arm.pattern, indent + 2);
                if !arm.capture.is_empty() {
                    println!("{}  {}capture{} {:?}", i, YELLOW, RST, arm.capture);
                }
                print_expr(&arm.value, indent + 2);
            }
        }
        Stmt::Loop(l) => {
            println!("{}{}loop{}", i, MAGENTA, RST);
            for c in &l.conds {
                print_expr(c, indent + 1);
            }
            if !l.captures.is_empty() {
                println!(
                    "{}  {}captures{} {:?}",
                    i,
                    YELLOW,
                    RST,
                    l.captures.iter().map(|c| &c.name).collect::<Vec<_>>()
                );
            }
            print_block(&l.body, indent);
        }
        Stmt::Defer(e) => {
            println!("{}{}defer{}", i, MAGENTA, RST);
            print_expr(e, indent + 1);
        }
        Stmt::TryCatch(tc) => {
            println!("{}{}try{}", i, MAGENTA, RST);
            print_block(&tc.try_body, indent);
            println!("{}  {}catch{} {:?}", i, MAGENTA, RST, tc.capture);
            print_block(&tc.catch_body, indent);
        }
        Stmt::Assign(target, op, value) => {
            println!("{} {}assign{} {}", i, YELLOW, RST, assign_op_str(op));
            print_expr(target, indent + 1);
            print_expr(value, indent + 1);
        }
        Stmt::Block(b) => {
            print_block(b, indent);
        }
    }
}

fn print_pattern(pattern: &Pattern, indent: usize) {
    let i = "  ".repeat(indent);
    match pattern {
        Pattern::Wildcard => println!("{}_", i),
        Pattern::Ident(name) => println!("{}{}", i, name),
        Pattern::Literal(kind, val) => println!("{}{} '{}'", i, kind_display(kind), val),
        Pattern::EnumVariant(typ, variant, capture) => {
            print!("{}.{}", typ, variant);
            if let Some(c) = capture {
                print!(" |{}|", c);
            }
            println!();
        }
    }
}

fn print_expr(expr: &Expr, indent: usize) {
    let i = "  ".repeat(indent);
    match expr {
        Expr::Literal(kind, val) => {
            println!(
                "{}{}Literal{} {}{} '{}'",
                i,
                ORNG,
                RST,
                GREY,
                kind_display(kind),
                val
            );
        }
        Expr::Ident(name) => {
            println!("{}{}Ident{} {}", i, PEACH, RST, name);
        }
        Expr::Binary(op, l, r) => {
            println!("{}{}BinaryOp{} {}", i, YELLOW, RST, binop_str(op));
            print_expr(l, indent + 1);
            print_expr(r, indent + 1);
        }
        Expr::Unary(op, e) => {
            println!("{}{}UnaryOp{} {}", i, YELLOW, RST, unaryop_str(op));
            print_expr(e, indent + 1);
        }
        Expr::Call(callee, args) => {
            println!("{}{}Call{}", i, YELLOW, RST);
            print_expr(callee, indent + 1);
            for arg in args {
                print_expr(arg, indent + 1);
            }
        }
        Expr::Field(obj, name) => {
            println!("{}{}Field{} .{}", i, YELLOW, RST, name);
            print_expr(obj, indent + 1);
        }
        Expr::Index(obj, idx) => {
            println!("{}{}Index{}", i, YELLOW, RST);
            print_expr(obj, indent + 1);
            print_expr(idx, indent + 1);
        }
        Expr::Slice(obj, s, e, incl) => {
            let op = if *incl { "..=" } else { ".." };
            println!("{}{}Slice{}{}", i, YELLOW, RST, op);
            print_expr(obj, indent + 1);
            print_expr(s, indent + 1);
            print_expr(e, indent + 1);
        }
        Expr::StructInit(name, fields) => {
            println!("{}{}StructInit{} {}{}", i, CYAN, RST, PEACH, name);
            for f in fields {
                println!("{}  {}FieldInit{} {}{}", i, YELLOW, RST, PEACH, f.name);
                print_expr(&f.value, indent + 2);
            }
        }
        Expr::Deref(e) => {
            println!("{}{}Deref{} .*", i, YELLOW, RST);
            print_expr(e, indent + 1);
        }
        Expr::Block(b) => {
            print_block(b, indent);
        }
        Expr::Paren(e) => {
            println!("{}{}Paren{}", i, GREY, RST);
            print_expr(e, indent + 1);
        }
        Expr::AtMethod(obj, method) => {
            println!("{}{}AtMethod{} @{}", i, YELLOW, RST, method);
            print_expr(obj, indent + 1);
        }
        Expr::Catch(expr, capture, body) => {
            println!("{}{}Catch{} |{}|", i, YELLOW, RST, capture.join(", "));
            print_expr(expr, indent + 1);
            print_block(body, indent + 1);
        }
        Expr::Ret(value) => {
            println!("{}{}Ret{}", i, YELLOW, RST);
            if let Some(val) = value {
                print_expr(val, indent + 1);
            }
        }
        Expr::Fn(fndecl) => {
            println!("{}{}Fn{} {}", i, CYAN, RST, fndecl.name);
            for p in &fndecl.params {
                println!(
                    "{}  {}{}Param{} {}: {}",
                    i,
                    YELLOW,
                    if p.mutable { "mut " } else { "" },
                    RST,
                    p.name,
                    type_str(&p.type_)
                );
            }
            if let Some(ret) = &fndecl.return_ {
                println!("{}  {}Return{} {}", i, YELLOW, RST, type_str(ret));
            }
            if let Some(body) = &fndecl.body {
                print_block(body, indent + 1);
            }
        }
        Expr::MapLiteral(pairs) => {
            println!("{}{}MapLiteral{}", i, YELLOW, RST);
            for (k, v) in pairs {
                print_expr(k, indent + 1);
                print_expr(v, indent + 1);
            }
        }
    }
}

fn type_str(t: &Type) -> String {
    match t {
        Type::Primitive(k) => kind_display(k).to_string(),
        Type::Named(n) => n.clone(),
        Type::Ref(mut_, inner) => {
            if *mut_ {
                format!("&mut {}", type_str(inner))
            } else {
                format!("&{}", type_str(inner))
            }
        }
        Type::Pointer(inner) => format!("*{}", type_str(inner)),
        Type::Optional(inner) => format!("?{}", type_str(inner)),
        Type::ErrorUnion(err, ok) => {
            if let Some(e) = err {
                format!("{}!{}", type_str(e), type_str(ok))
            } else {
                format!("!{}", type_str(ok))
            }
        }
        Type::Array(inner, size) => {
            if let Some(s) = size {
                format!("[{}; {}]", type_str(inner), expr_short_str(s))
            } else {
                format!("[{}]", type_str(inner))
            }
        }
        Type::Fn(params, ret) => {
            let ps: Vec<String> = params.iter().map(type_str).collect();
            format!("fn({}) -> {}", ps.join(", "), type_str(ret))
        }
        Type::Builtin(name) => format!("@{}", name),
    }
}

fn expr_short_str(e: &Expr) -> String {
    match e {
        Expr::Literal(_, v) => v.clone(),
        Expr::Ident(name) => name.clone(),
        _ => "expr".into(),
    }
}

fn kind_display(k: &TokenKind) -> &'static str {
    match k {
        TokenKind::IntegerValue => "int",
        TokenKind::FloatValue => "float",
        TokenKind::StringValue => "string",
        TokenKind::CharValue => "char",
        TokenKind::True => "true",
        TokenKind::False => "false",
        TokenKind::Nil => "nil",
        _ => "",
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

fn unaryop_str(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
        UnaryOp::Ref => "&",
        UnaryOp::RefMut => "&mut",
        UnaryOp::Optional => "?",
        UnaryOp::Deref => ".*",
    }
}

fn assign_op_str(op: &AssignOp) -> &'static str {
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

pub fn print_error(err: &str) {
    eprintln!("{}Error:{} {}", RED, RST, err);
}

pub fn print_parse_error(source: &str, file_stem: &str, err: &crate::parser::ParseError) {
    let line = err.line;
    let col = err.col;
    eprintln!("{}Error:{} {}", RED, RST, err.message);
    if line > 0 {
        eprintln!(" {}-->{} {}:{}:{}", GREY, RST, file_stem, line, col);
        if let Some(source_line) = source.lines().nth(line - 1) {
            eprintln!(" {}|{}", GREY, RST);
            eprintln!(" {:>4}{} {}", GREY, RST, source_line);
            let mut caret = String::with_capacity(col.saturating_sub(1) + 5);
            caret.push_str(&" ".repeat(4)); // line number padding
            caret.push(' ');
            caret.push_str(&" ".repeat(col.saturating_sub(1)));
            caret.push_str(&format!("{}^{}", BLUE, RST));
            eprintln!("{}", caret);
        }
    }
}

fn print_annotation(a: &Annotation, indent: usize) {
    let i = "  ".repeat(indent);
    if a.args.is_empty() {
        println!("{}{}@{}", i, YELLOW, a.name);
    } else {
        let args_str: Vec<String> = a.args.iter().map(|e| expr_short_str(e)).collect();
        println!("{}{}@{}({})", i, YELLOW, a.name, args_str.join(", "));
    }
}
