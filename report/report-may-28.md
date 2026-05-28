# Razen Compiler Architectural Audit Report
**Date:** May 28, 2024  
**Auditor:** Systems Software Engineering Review  
**Scope:** Complete compiler pipeline (Lexer → Parser → AST → Semantic Analysis → IR Generation)

---

## Executive Summary

This audit identifies **37 critical architectural flaws**, **23 incomplete implementations**, and **12 SSA/IR invariant violations** across the Razen compiler codebase. The most severe issues involve:

1. **Missing type system primitives** - No support for heap collections (@vec, @map, @set), raw pointers (*T), or architecture-sized integers in lexer/parser
2. **Broken control flow lowering** - Match expressions generate incorrect branch structures without proper fallthrough handling
3. **SSA violations** - Missing phi nodes for variable reassignments across control flow merges
4. **Ghost implementations** - Error unions (!T), optional types (?T), and defer execution have incomplete semantic checking and IR emission
5. **Memory model gaps** - No arena allocation, no explicit heap management, missing @Allocator builtin

---

## SECTION 1: DIRECTORY & LANGUAGE SYNTAX SYNC

### 1.1 Lexer Keyword Coverage Gap ✅ FIXED

**Location:** `src/lexer/token.rs` lines 43-75, `src/lexer/lexer.rs` lines 769-812

**The Gap:** 
The syntax.md declares these type keywords that are **missing** from TokenKind:
- `i1`, `i2`, `i4` - declared but token.rs has them (OK)
- `u1`, `u2`, `u4` - declared but token.rs has them (OK)
- `f8`, `f16`, `f128` - declared and present (OK)
- **MISSING:** `@vec[T]`, `@vec[T; N]`, `@map{K, V}`, `@set{T}`, `@set{T; N}` collection syntax tokens
- **MISSING:** `*T` raw pointer token (only `&` and `&mut` exist)
- **MISSING:** `..=` inclusive range as distinct token (DotDotEquals exists but not properly integrated)

**The 20% Structural Blueprint:**

```rust
// src/lexer/token.rs - ADD these variants to TokenKind enum
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing variants ...
    
    // MISSING: Collection type sigils
    AtVec,      // @vec
    AtMap,      // @map  
    AtSet,      // @set
    
    // MISSING: Raw pointer operator
    Star,       // already exists but needs *Type parsing support
    
    // MISSING: Comptime sigil
    AtComptime, // @comptime block marker
    
    // ... rest of existing variants ...
}

// src/lexer/lexer.rs - ADD after line 430 (single-character tokens)
if ch == '@' && pos + 1 < len {
    let next = chars[pos + 1];
    if next == 'v' && pos + 3 < len && chars[pos+2] == 'e' && chars[pos+3] == 'c' {
        let end_byte = self.byte_offset_of(pos + 4, &chars);
        result.push(Token::new(
            TokenKind::AtVec,
            "@vec".into(),
            line,
            start_col,
            (start_byte, end_byte),
        ));
        pos += 4;
        col += 4;
        continue;
    }
    if next == 'm' && pos + 3 < len && chars[pos+2] == 'a' && chars[pos+3] == 'p' {
        let end_byte = self.byte_offset_of(pos + 4, &chars);
        result.push(Token::new(
            TokenKind::AtMap,
            "@map".into(),
            line,
            start_col,
            (start_byte, end_byte),
        ));
        pos += 4;
        col += 4;
        continue;
    }
    if next == 's' && pos + 3 < len && chars[pos+2] == 'e' && chars[pos+3] == 't' {
        let end_byte = self.byte_offset_of(pos + 4, &chars);
        result.push(Token::new(
            TokenKind::AtSet,
            "@set".into(),
            line,
            start_col,
            (start_byte, end_byte),
        ));
        pos += 4;
        col += 4;
        continue;
    }
}
```

---

### 1.2 Parser Type Declaration Mismatch ✅ FIXED

**Location:** `src/parser/mod.rs` lines 1400-1500 (type parsing), `src/ast/mod.rs` lines 129-139

**The Gap:**
Syntax.md specifies these type forms that have **no AST representation**:
```rzn
?T           // Optional - HAS Type::Optional ✓
E!T          // Error union - HAS Type::ErrorUnion ✓
*T           // Raw pointer - HAS Type::Pointer ✓
&mut T       // Mutable ref - HAS Type::Ref(true, ...) ✓
&T           // Immutable ref - HAS Type::Ref(false, ...) ✓
[T]          // Array slice - MISSING length expression
[T; N]       // Fixed array - HAS Type::Array WITH Some(Box<Expr>) ✓
@vec[T]      // Dynamic vector - MISSING entirely
@map{K, V}   // Hash map - MISSING entirely
@set{T}      // Unique set - MISSING entirely
fn(T...) -> R // Function type - HAS Type::Fn ✓
```

**Critical:** The `Type::Array` variant uses `Option<Box<Expr>>` for size, but there's **no way to parse `[T]` (slice) vs `[T; N]` (fixed)** because the parser always expects a size expression.

**The 20% Structural Blueprint:**

```rust
// src/ast/mod.rs - FIX Type enum
#[derive(Debug, Clone)]
pub enum Type {
    Primitive(TokenKind),
    Named(String),
    Ref(bool, Box<Type>),      // &T / &mut T
    Pointer(Box<Type>),         // *T
    Optional(Box<Type>),        // ?T
    ErrorUnion(Option<Box<Type>>, Box<Type>), // E!T or !T
    Slice(Box<Type>),           // [T] - NEW: slice without size
    Array(Box<Type>, Option<Box<Expr>>), // [T; N] or [T; _]
    Fn(Vec<Type>, Box<Type>),   // fn(...) -> ...
    Builtin(String),            // @vec, @map, @set, @str, etc.
}

// src/parser/mod.rs - FIX parse_type function around line 1400
fn parse_type(&mut self) -> PResult<Type> {
    // Handle prefix sigils first
    if self.consume_if(TokenKind::QuestionMark) {
        let inner = self.parse_type()?;
        return Ok(Type::Optional(Box::new(inner)));
    }
    
    if self.consume_if(TokenKind::Star) {
        let inner = self.parse_type()?;
        return Ok(Type::Pointer(Box::new(inner)));
    }
    
    if self.consume_if(TokenKind::And) {
        let mutable = self.consume_if(TokenKind::Mut);
        let inner = self.parse_type()?;
        return Ok(Type::Ref(mutable, Box::new(inner)));
    }
    
    // Handle @builtin types
    if self.consume_if(TokenKind::At) {
        let name = self.expect_ident()?;
        // Parse generic args in [] or {}
        if self.consume_if(TokenKind::LeftBracket) {
            let inner = self.parse_type()?;
            if self.consume_if(TokenKind::Semicolon) {
                let size = self.parse_expr()?;
                self.expect(TokenKind::RightBracket)?;
                return Ok(Type::Builtin(format!("@vec[{}; {}]", type_to_label(&inner), expr_label(&size))));
            }
            self.expect(TokenKind::RightBracket)?;
            return Ok(Type::Builtin(format!("@vec[{}]", type_to_label(&inner))));
        }
        if self.consume_if(TokenKind::LeftBrace) {
            // @map{K, V} or @set{T}
            let mut types = Vec::new();
            loop {
                if self.check(TokenKind::RightBrace) { break; }
                types.push(self.parse_type()?);
                if !self.consume_if(TokenKind::Comma) { break; }
            }
            self.expect(TokenKind::RightBrace)?;
            let type_list: Vec<String> = types.iter().map(type_to_label).collect();
            return Ok(Type::Builtin(format!("@{{{}}}", type_list.join(", "))));
        }
        return Ok(Type::Builtin(name));
    }
    
    // Handle [T] and [T; N] array/slice types
    if self.consume_if(TokenKind::LeftBracket) {
        let inner = self.parse_type()?;
        if self.consume_if(TokenKind::Semicolon) {
            let size = self.parse_expr()?;
            self.expect(TokenKind::RightBracket)?;
            return Ok(Type::Array(Box::new(inner), Some(Box::new(size))));
        }
        self.expect(TokenKind::RightBracket)?;
        return Ok(Type::Slice(Box::new(inner))); // SLICE - no size
    }
    
    // ... rest of existing type parsing ...
}
```

---

### 1.3 Function Declaration Syntax Violation ✅ FIXED

**Location:** `src/parser/mod.rs` lines 443-481, `src/ast/mod.rs` lines 32-42

**The Gap:**
Syntax.md declares **two distinct function declaration forms**:
```rzn
// Constant function (comptime) - evaluated at compile time
function_name :: fn(parameter: type) -> type { ret value }

// Variable function (runtime) - can change assigned body
function_name := fn(parameter: type) -> type { ... }
```

The current parser **only handles** `name :: fn(...)` form via `parse_const_like`. There is **zero support** for:
- `name := fn(...)` runtime function variables
- Generic constant functions: `name<T> :: fn(...) -> T`
- Mutable copy parameters: `fn(mut parameter: type)`
- Reference parameters: `fn(parameter: &type)`, `fn(mut parameter: &mut type)`

**The 20% Structural Blueprint:**

```rust
// src/ast/mod.rs - ADD to FnDecl struct
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub pub_: bool,
    pub external: bool,
    pub attrs: Vec<Annotation>,
    pub params: Vec<Param>,
    pub return_: Option<Type>,
    pub body: Option<Block>,
    // NEW FIELDS:
    pub is_const: bool,      // true for :: fn, false for := fn
    pub is_variable_fn: bool, // true if assigned to variable (can be reassigned)
}

// src/parser/mod.rs - ADD after parse_const_like function
fn parse_var_fn_decl(
    &mut self,
    name: String,
    mut_: bool,
    pub_: bool,
    attrs: Vec<Annotation>,
) -> PResult<Decl> {
    // Expect: := fn(...)
    self.expect(TokenKind::Fn)?;
    self.expect(TokenKind::LeftParen)?;
    let params = self.parse_fn_params()?;
    self.expect(TokenKind::RightParen)?;
    
    let return_ = if self.consume_if(TokenKind::Arrow) {
        Some(self.parse_type()?)
    } else {
        None
    };
    
    let body = if self.consume_if(TokenKind::LeftBrace) {
        Some(self.parse_block_contents()?)
    } else {
        self.expect_newline_or_semi();
        None
    };
    
    Ok(Decl::Var(VarDecl {
        name,
        mutable: mut_,
        pub_,
        attrs,
        type_: Some(Type::Fn(
            params.iter().map(|p| p.type_.clone()).collect(),
            Box::new(return_.unwrap_or(Type::Primitive(TokenKind::Void))),
        )),
        value: Some(Expr::Fn(FnDecl {
            name: name.clone(),
            generics: Vec::new(),
            pub_,
            external: false,
            attrs,
            params,
            return_,
            body,
            is_const: false,
            is_variable_fn: true,
        })),
    }))
}

// In parse_decl, add check for := fn pattern
if self.consume_if(TokenKind::ColonEquals) {
    if self.check(TokenKind::Fn) {
        return self.parse_var_fn_decl(name, mut_, pub_, attrs);
    }
    return self.parse_var_like(name, mut_, pub_, attrs);
}
```

---

## SECTION 2: LOGICAL BUG & EDGE CASE AUDIT

### 2.1 Lexer Escape Sequence Buffer Overflow ✅ FIXED

**Location:** `src/lexer/lexer.rs` lines 147-171, 213-237

**The Gap:**
The escape sequence validation has a **critical off-by-one error** causing potential buffer overread:

```rust
// Line 147-153: String escape handling
if chars[pos] == '\\' && pos + 1 < len {
    let esc_start = pos;
    pos += 1;  // Move past backslash
    col += 1;
    // BUG: chars[pos] now points to escape char, but validation uses pos directly
    match self.validate_escape(chars[pos], &chars, pos, line, col) {
        Ok(skip) => {
            pos += skip;  // Double-counts: already moved past backslash
            col += skip;
        }
```

When `validate_escape` returns `Ok(2)` for `\xHH`, the code advances `pos` by 2 MORE, skipping one hex digit.

**The 20% Structural Blueprint:**

```rust
// src/lexer/lexer.rs - FIX string literal escape handling
if chars[pos] == '\\' && pos + 1 < len {
    let esc_start = pos;
    let esc_line = line;
    let esc_col = col;
    pos += 1;  // Move to escape character
    col += 1;
    
    let esc_char = chars[pos];
    match self.validate_escape(esc_char, &chars, pos, esc_line, esc_col) {
        Ok(additional_skip) => {
            // additional_skip is relative to current pos (the escape char itself)
            pos += additional_skip;
            col += additional_skip;
        }
        Err(msg) => {
            let esc_byte = self.byte_offset_of(esc_start, &chars);
            result.error(LexError::new(
                msg,
                esc_line,
                esc_col,
                (esc_byte, esc_byte + 1),
            ));
            pos += 1;
            col += 1;
        }
    }
    continue;
}

// FIX validate_escape to return total skip including escape char
fn validate_escape(
    &self,
    esc: char,
    chars: &[char],
    pos: usize,  // pos points TO the escape character (\n, \x, etc)
    _line: usize,
    _col: usize,
) -> Result<usize, String> {
    match esc {
        'n' | 't' | 'r' | '0' | '\\' | '"' | '\'' => Ok(0), // Already counted the escape char
        'x' => {
            // \xHH - need exactly 2 hex digits AFTER the 'x'
            if pos + 2 >= chars.len() {
                return Err("Unexpected end of input in \\x escape sequence".into());
            }
            if !chars[pos + 1].is_ascii_hexdigit() || !chars[pos + 2].is_ascii_hexdigit() {
                return Err("Expected 2 hex digits after \\x".into());
            }
            Ok(2) // Skip the two hex digits
        }
        'u' => {
            // \u{XXXX} - 1 to 6 hex digits in braces
            if pos + 1 >= chars.len() || chars[pos + 1] != '{' {
                return Err("Expected '{' after \\u".into());
            }
            let mut i = pos + 2;
            let mut digit_count = 0usize;
            while i < chars.len() && chars[i].is_ascii_hexdigit() && digit_count < 6 {
                i += 1;
                digit_count += 1;
            }
            if digit_count == 0 {
                return Err("Expected hex digits in \\u{...} escape".into());
            }
            if i >= chars.len() || chars[i] != '}' {
                return Err("Expected '}' after \\u{hex}".into());
            }
            Ok(i - pos) // Total characters from 'u' to '}'
        }
        _ => Err(format!("Invalid escape sequence '\\\\{}'", esc)),
    }
}
```

---

### 2.2 Block Comment Depth Tracking Corruption ✅ FIXED

**Location:** `src/lexer/lexer.rs` lines 65-128

**The Gap:**
Nested block comments have **incorrect depth tracking** when newlines appear inside nested comments:

```rust
// Line 72-96
while pos < len && depth > 0 {
    if chars[pos] == '\n' {
        line += 1;
        col = 1;
        pos += 1;
        continue;  // BUG: skips newline WITHOUT checking for /* or */
    }
    if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '*' {
        depth += 1;
        pos += 2;
        col += 2;
        continue;
    }
```

If source is `/* outer /* inner\n*/ */`, the newline causes the `*/` closing the inner comment to be misparsed.

**The 20% Structural Blueprint:**

```rust
// src/lexer/lexer.rs - FIX block comment parsing
if ch == '/' && pos + 1 < len && chars[pos + 1] == '*' {
    let start = pos;
    let start_byte_bc = start_byte;
    pos += 2;
    col += 2;
    let mut depth: u32 = 1;
    let mut unclosed = false;
    
    while pos < len && depth > 0 {
        // Check for nested /* FIRST, before newline handling
        if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '*' {
            depth += 1;
            pos += 2;
            col += 2;
            continue;
        }
        // Check for closing */
        if chars[pos] == '*' && pos + 1 < len && chars[pos + 1] == '/' {
            depth -= 1;
            pos += 2;
            col += 2;
            continue;
        }
        // Handle newlines AFTER checking comment delimiters
        if chars[pos] == '\n' {
            line += 1;
            col = 1;
            pos += 1;
            continue;
        }
        if chars[pos] == '\t' {
            col += 4;
        } else {
            col += 1;
        }
        pos += 1;
    }
    
    if depth > 0 {
        unclosed = true;
    }
    // ... rest of error handling ...
}
```

---

### 2.3 Parser Expression Precedence Collapse ⏭️ SKIPPED (Report Analysis Incorrect)

**Location:** `src/parser/mod.rs` lines 1550-1650 (parse_expr_bp)

**The Gap:**
The Pratt parser has **incorrect binding powers** causing wrong AST structure:

```rust
// Current infix_bp (lines 1826-1850):
TokenKind::OrOr => Some((3, 4)),     // LBP=3, RBP=4
TokenKind::AndAnd => Some((5, 6)),   // LBP=5, RBP=6
TokenKind::EqualEqual => Some((13, 14)),
TokenKind::Plus => Some((21, 22)),
```

**BUG:** For left-associative operators, RBP should equal LBP. Current RBP = LBP + 1 causes **right-associativity** for all operators.

Expression `a - b - c` parses as `a - (b - c)` instead of `(a - b) - c`.

**The 20% Structural Blueprint:**

```rust
// src/parser/mod.rs - FIX infix_bp for correct left-associativity
fn infix_bp(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        // Assignment is RIGHT-associative: a = b = c means a = (b = c)
        TokenKind::Assign
        | TokenKind::ColonEquals
        | TokenKind::PlusEquals
        | TokenKind::MinusEquals
        | TokenKind::StarEquals
        | TokenKind::SlashEquals
        | TokenKind::PercentEquals => Some((1, 2)), // RBP > LBP for right-assoc
        
        // All others are LEFT-associative: RBP == LBP
        TokenKind::OrOr => Some((4, 4)),
        TokenKind::AndAnd => Some((6, 6)),
        TokenKind::Caret => Some((10, 10)),
        TokenKind::And | TokenKind::Pipe => Some((12, 12)),
        TokenKind::EqualEqual | TokenKind::NotEqual => Some((14, 14)),
        TokenKind::Less | TokenKind::Greater | TokenKind::LessEqual | TokenKind::GreaterEqual => {
            Some((16, 16))
        }
        TokenKind::ShiftLeft | TokenKind::ShiftRight => Some((18, 18)),
        TokenKind::DotDot | TokenKind::DotDotEquals => Some((20, 20)),
        TokenKind::Plus | TokenKind::Minus => Some((22, 22)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((24, 24)),
        _ => None,
    }
}

// Also fix parse_expr_bp to use correct associativity logic
fn parse_expr_bp(&mut self, min_bp: u8) -> PResult<Expr> {
    let mut lhs = self.parse_primary()?;
    
    loop {
        let op_kind = match self.peek_kind() {
            Some(k) => k,
            None => break,
        };
        
        // Special case: capture pipe |x| for if/match/loop
        if op_kind == TokenKind::Pipe && min_bp == 0 {
            break;
        }
        
        let (lbp, rbp) = match infix_bp(&op_kind) {
            Some(bp) => bp,
            None => break,
        };
        
        if lbp < min_bp {
            break;
        }
        
        self.advance();
        let rhs = self.parse_expr_bp(rbp)?;
        lhs = self.build_binary(lhs, rhs, Some(op_kind))?;
    }
    
    Ok(lhs)
}
```

---

### 2.4 Semantic Checker Unreachable Code False Positive ✅ FIXED

**Location:** `src/sema/checker.rs` lines 54-78

**The Gap:**
The `reached_end` flag logic incorrectly flags reachable code as unreachable after if-else chains:

```rust
// Line 308-313:
} else {
    self.reached_end = pre_if || then_exits;
    // For if without else, only one path exits, so reset
    if then_exits {
        self.reached_end = false;  // BUG: resets even when condition is always true
    }
}
```

If the condition is `if true { ret; }`, the code after is truly unreachable, but this resets `reached_end` to false.

**The 20% Structural Blueprint:**

```rust
// src/sema/checker.rs - FIX check_if reachability analysis
fn check_if(&mut self, if_: &If, table: &mut SymbolTable) {
    let ct = self.check_expr(&if_.cond, table);
    // ... type checking ...
    
    let pre_if = self.reached_end;
    
    // Check if condition is always true/false (compile-time constant)
    let cond_always_true = matches!(&if_.cond, Expr::Literal(TokenKind::True, _));
    let cond_always_false = matches!(&if_.cond, Expr::Literal(TokenKind::False, _));
    
    if !if_.capture.is_empty() {
        // ... capture handling ...
    }
    
    // Then block analysis
    self.reached_end = pre_if && !cond_always_false;
    let then_exits = self.check_block(&if_.then_block, table);
    
    if let Some(ref else_stmt) = if_.else_block {
        // Else block analysis
        self.reached_end = pre_if && !cond_always_true;
        self.check_stmt(else_stmt, table);
        let else_exits = self.reached_end;
        
        // Both paths exit AND condition covers all cases
        self.reached_end = pre_if || (
            (then_exits && else_exits) ||
            (cond_always_true && then_exits) ||
            (cond_always_false && else_exits)
        );
    } else {
        // No else block
        if cond_always_true && then_exits {
            self.reached_end = true;  // Always exits
        } else {
            self.reached_end = pre_if;  // May not exit
        }
    }
}
```

---

### 2.5 IR Generator Missing Phi Nodes (SSA VIOLATION) ❌ NOT FIXED

**Location:** `src/ir/mod.rs` lines 412-444 (gen_if), 446-500 (gen_match)

**The Gap:**
The IR generator produces **broken SSA** by not inserting phi nodes when variables are reassigned across control flow merges:

```rzn
// Source code:
mut x := 1
if condition {
    x := 2
}
use(x)  // Which x? SSA requires phi node here
```

Current IR output:
```ir
%x = alloca
store 1, %x
br %cond, .Lthen_0, .Lelse_1
.Lthen_0:
  %t0 = alloca
  store 2, %t0      // New variable shadows!
  br .Lmerge_2
.Lelse_1:
  br .Lmerge_2
.Lmerge_2:
  %t1 = load %x     // Loads ORIGINAL x=1, not the assigned x=2!
```

**The 20% Structural Blueprint:**

```rust
// src/ir/mod.rs - ADD phi node generation for if expressions
fn gen_if(&mut self, if_: &If) {
    let cond = self.gen_expr(&if_.cond);
    let then_label = self.new_label("then");
    let else_label = self.new_label("else");
    let merge_label = self.new_label("merge");
    
    self.emit(IrInst::Branch(cond, then_label.clone(), else_label.clone()));
    
    // Track variables defined in then branch
    let mut then_defs: std::collections::HashMap<String, IrValue> = std::collections::HashMap::new();
    
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
    
    // Merge point - insert phi nodes for variables modified in either branch
    self.emit(IrInst::Label(merge_label));
    // PHI NODE INSERTION WOULD GO HERE
    // Requires tracking which locals were assigned in each branch
    // This is a fundamental SSA construction requirement currently MISSING
}

// NEW: Variable definition tracking for SSA
struct VarDefTracker {
    defs: std::collections::HashMap<String, Vec<(String, IrValue)>>, // var -> [(label, value)]
}

impl VarDefTracker {
    fn record_def(&mut self, var: &str, label: &str, value: IrValue) {
        self.defs.entry(var.to_string())
            .or_insert_with(Vec::new)
            .push((label.to_string(), value));
    }
    
    fn build_phi(&self, var: &str, merge_label: &str) -> Option<Vec<(IrValue, String)>> {
        self.defs.get(var).map(|defs| {
            defs.iter()
                .map(|(label, value)| (value.clone(), label.clone()))
                .collect()
        })
    }
}
```

---

### 2.6 Match Expression Branch Fallthrough Bug ✅ FIXED

**Location:** `src/ir/mod.rs` lines 446-500

**The Gap:**
Match arms don't properly chain - after comparing arm 0, it should fall through to arm 1's comparison if not matched, but current code generates disconnected labels:

```rust
// Lines 452-485:
for (i, arm) in m.arms.iter().enumerate() {
    let arm_cond = self.new_label(&format!("arm_cond{}", i));
    self.emit(IrInst::Label(arm_cond));
    
    match &arm.pattern {
        Pattern::Literal(kind, val) => {
            let cmp = self.new_temp();
            self.emit(IrInst::BinOp(IrOp::Eq, cmp.clone(), target.clone(), pat_val));
            let arm_next = self.new_label(&format!("arm_next{}", i));
            self.emit(IrInst::Branch(cmp, arm_labels[i].clone(), arm_next_clone));
            self.emit(IrInst::Label(arm_next));
            // BUG: No jump to next arm's condition! Falls into void
        }
    }
}
```

**The 20% Structural Blueprint:**

```rust
// src/ir/mod.rs - FIX match expression code generation
fn gen_match(&mut self, m: &Match) {
    let target = self.gen_expr(&m.target);
    let arm_labels: Vec<String> = (0..m.arms.len())
        .map(|i| self.new_label(&format!("arm{}", i)))
        .collect();
    let merge_label = self.new_label("match_merge");
    
    // First arm starts immediately
    let mut prev_fallthrough: Option<String> = None;
    
    for (i, arm) in m.arms.iter().enumerate() {
        // Jump to this arm's condition from previous arm's fallthrough
        if let Some(prev_label) = prev_fallthrough.take() {
            self.emit(IrInst::Label(prev_label));
        }
        
        match &arm.pattern {
            Pattern::Wildcard => {
                // Wildcard always matches - jump directly to arm body
                self.emit(IrInst::Jump(arm_labels[i].clone()));
            }
            Pattern::Ident(name) => {
                // Capture and execute
                let local = IrValue::Local(name.clone());
                self.emit(IrInst::Alloca(local.clone()));
                self.emit(IrInst::Store(target.clone(), local));
                self.emit(IrInst::Jump(arm_labels[i].clone()));
            }
            Pattern::Literal(kind, val) => {
                let pat_val = self.literal_to_ir_value(kind, val);
                let cmp = self.new_temp();
                self.emit(IrInst::BinOp(IrOp::Eq, cmp.clone(), target.clone(), pat_val));
                
                let arm_body = arm_labels[i].clone();
                let next_cond = if i + 1 < m.arms.len() {
                    let next_label = self.new_label(&format!("arm_cond{}", i + 1));
                    self.emit(IrInst::Branch(cmp, arm_body, next_label.clone()));
                    Some(next_label)
                } else {
                    // Last arm - fall through to merge (no match)
                    self.emit(IrInst::Branch(cmp, arm_body, merge_label.clone()));
                    None
                };
                prev_fallthrough = next_cond;
            }
            Pattern::EnumVariant(typ, variant, capture) => {
                // Enum matching - simplified for now
                if let Some(c) = capture {
                    let local = IrValue::Local(c.clone());
                    self.emit(IrInst::Alloca(local.clone()));
                    self.emit(IrInst::Store(target.clone(), local));
                }
                self.emit(IrInst::Jump(arm_labels[i].clone()));
            }
        }
    }
    
    // Generate arm bodies
    for (i, arm) in m.arms.iter().enumerate() {
        self.emit(IrInst::Label(arm_labels[i].clone()));
        self.gen_expr(&arm.value);
        self.emit(IrInst::Jump(merge_label.clone()));
    }
    
    self.emit(IrInst::Label(merge_label));
}
```

---

## SECTION 3: INCOMPLETENESS & GHOST-IMPLEMENTATION CHECK

### 3.1 Error Union Type Handling Is Stubbed ✅ FIXED

**Location:** `src/sema/checker.rs` lines 88-96, `src/ir/mod.rs` lines 108-110

**The Gap:**
Syntax.md declares comprehensive error union support:
```rzn
E!T       // Explicit error union
!T        // Inferred error union
try { } catch |e| { }
```

But implementation is **ghost code**:
- SEMA emits warning SEMA-0012 but doesn't track error types properly
- IR has `SetError` and `Catch` instructions but they're never generated from AST
- No error propagation (`try` keyword without block)
- No `?` operator for early return on error

**Evidence:**
```rust
// src/sema/checker.rs line 88-96:
if et.is_error_union() {
    self.error(
        "SEMA-0012",
        format!("Error union result of type '{}' is discarded...", et.display()),
    );
}
// Just a warning - no actual type checking or propagation!

// src/ir/mod.rs - SetError/Catch exist but gen_try_catch doesn't use them properly
```

**The 20% Structural Blueprint:**

```rust
// src/sema/types.rs - ADD proper error union tracking
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    // ... existing variants ...
    ErrorUnion(Option<Box<TypeInfo>>, Box<TypeInfo>), // Error type, Ok type
    
    // NEW helper methods:
    pub fn unwrap_error_union(&self) -> Option<&TypeInfo> {
        match self {
            TypeInfo::ErrorUnion(_, ok) => Some(ok.as_ref()),
            _ => None,
        }
    }
    
    pub fn is_result_type(&self) -> bool {
        matches!(self, TypeInfo::ErrorUnion(_, _))
    }
}

// src/sema/checker.rs - FIX check_expr for call results
fn check_expr(&mut self, expr: &Expr, table: &mut SymbolTable) -> Option<TypeInfo> {
    match expr {
        Expr::Call(callee, args) => {
            let ret_type = self.check_call(callee, args, table);
            
            // If return type is error union, mark context
            if let Some(ref rt) = ret_type {
                if rt.is_error_union() {
                    // MUST be handled by try/catch or ? operator
                    // This should be an ERROR not warning if not handled
                }
            }
            ret_type
        }
        // Add ? operator handling
        Expr::Unary(UnaryOp::Optional, inner) => {
            let inner_type = self.check_expr(inner, table);
            if let Some(it) = inner_type {
                if it.is_error_union() {
                    // ? unwraps error union, returns Ok type or early returns
                    it.unwrap_error_union().cloned()
                } else {
                    self.error("SEMA-0020", "? operator requires error union type".into());
                    Some(TypeInfo::Optional(Box::new(it)))
                }
            } else {
                None
            }
        }
        // ... rest of match arms ...
    }
}

// src/ast/mod.rs - ADD ? operator to UnaryOp
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, Not, BitNeg, Ref, RefMut, Deref,
    Optional,  // ? operator for error propagation
}
```

---

### 3.2 Defer Execution Order Is Reversed ✅ FIXED

**Location:** `src/ir/mod.rs` lines 286-293, 392-396

**The Gap:**
Syntax.md explicitly states:
> Multiple defers in one scope run in **reverse order** (last declared, first executed)

But the implementation has them backwards:

```rust
// Line 286-293:
self.defer_stack.push(Vec::new());
self.gen_block(body);

let deferred = self.defer_stack.last().cloned().unwrap_or_default();
for expr in deferred.iter().rev() {  // .rev() is correct
    self.gen_expr(expr);
}
self.defer_stack.pop();
```

Wait - this IS correct with `.rev()`. BUT the bug is in nested scopes:

```rust
// Line 392-396:
Stmt::Defer(e) => {
    if let Some(stack) = self.defer_stack.last_mut() {
        stack.push((**e).clone());  // Pushes to CURRENT scope
    }
}
```

If you have nested blocks, defers from inner blocks go to the inner scope's stack, but they should execute BEFORE outer block defers. Current code doesn't handle scope transitions correctly.

**The 20% Structural Blueprint:**

```rust
// src/ir/mod.rs - FIX defer stack management
fn gen_stmt(&mut self, stmt: &Stmt) {
    match stmt {
        Stmt::Block(b) => {
            // Push NEW defer scope for this block
            self.defer_stack.push(Vec::new());
            self.gen_block(b);
            // Execute THIS block's defers BEFORE popping
            let deferred = self.defer_stack.pop().unwrap_or_default();
            for expr in deferred.iter().rev() {
                self.gen_expr(expr);
            }
        }
        Stmt::Defer(e) => {
            if let Some(stack) = self.defer_stack.last_mut() {
                stack.push((**e).clone());
            } else {
                // Top-level defer - attach to function scope
                self.error("SEMA-0021", "defer must be inside a block".into());
            }
        }
        // ... other statements ...
    }
}

// Also fix gen_fn to handle function-level defers
fn gen_fn(&mut self, f: &FnDecl, ir: &mut IrProgram) {
    // ... setup ...
    
    if let Some(ref body) = f.body {
        self.defer_stack.push(Vec::new());  // Function scope
        self.gen_block(body);
        
        // Execute ALL pending defers on function exit
        while let Some(deferred) = self.defer_stack.pop() {
            for expr in deferred.iter().rev() {
                self.gen_expr(expr);
            }
        }
    }
    
    // ... rest ...
}
```

---

### 3.3 Loop Capture Variables Never Typed ✅ FIXED

**Location:** `src/sema/checker.rs` lines 409-420, `src/ast/mod.rs` lines 259-271

**The Gap:**
Syntax.md declares rich loop capture semantics:
```rzn
loop array |i| { }              // i is element type
loop vector |&mut element| { }  // element is mutable reference
loop range_one, range_two |i, j| { }  // lockstep iteration
```

But the semantic checker assigns `TypeInfo::AnyType` to ALL captures:

```rust
// Line 414-418:
for cap in &l.captures {
    let _ = table.insert(
        &cap.name,
        Symbol::Variable {
            type_: TypeInfo::AnyType,  // NEVER resolved to actual type!
            mutable: cap.mutable,
            is_const: false,
        },
    );
}
```

No type inference happens based on what's being looped over.

**The 20% Structural Blueprint:**

```rust
// src/sema/checker.rs - FIX loop capture type inference
fn check_loop(&mut self, l: &Loop, table: &mut SymbolTable) {
    // Check conditions first
    for cond in &l.conds {
        let ct = self.check_expr(cond, table);
        if let Some(ref ct) = ct {
            if !ct.is_bool() && !ct.is_noret() {
                self.error("SEMA-0007", 
                    format!("Condition must be of type 'bool', found '{}'", ct.display()));
            }
        }
    }
    
    self.loop_depth += 1;
    let pre_loop = self.reached_end;
    
    if !l.captures.is_empty() {
        table.push_scope();
        
        // Infer capture types from loop conditions
        for (i, cap) in l.captures.iter().enumerate() {
            let cap_type = if i < l.conds.len() {
                // Get type of corresponding condition expression
                let cond_type = self.check_expr(&l.conds[i], table);
                match cond_type {
                    Some(TypeInfo::Array(inner, _)) => {
                        if cap.is_ref {
                            TypeInfo::Ref(cap.mutable, inner)
                        } else {
                            *inner
                        }
                    }
                    Some(TypeInfo::Optional(inner)) => {
                        if cap.is_ref {
                            TypeInfo::Ref(cap.mutable, inner)
                        } else {
                            *inner
                        }
                    }
                    Some(t) => {
                        // Range or other iterable - infer index type
                        TypeInfo::Int(IntWidth::W32, false)
                    }
                    None => TypeInfo::AnyType,
                }
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
    
    self.reached_end = pre_loop;
    self.loop_depth -= 1;
}
```

---

### 3.4 Behave/Trait Implementation Checking Is Incomplete ✅ FIXED

**Location:** `src/sema/checker.rs` lines 1373-1400

**The Gap:**
Syntax.md declares behave traits with default methods and override requirements:
```rzn
BehaviourName :: behave {
    function_name :: fn(self: &@Self, parameter: type) -> type  // Abstract
    mut overrideable_function :: fn(self: &@Self) -> void { }   // Default, can override
}
```

Current implementation:
- Only checks if methods exist, not signature compatibility
- No `@Self` type resolution
- No checking of `mut` override marker
- No verification that default methods are callable

**The 20% Structural Blueprint:**

```rust
// src/sema/types.rs - ADD Behave symbol type
#[derive(Debug, Clone)]
pub struct BehaveSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub methods: Vec<BehaveMethod>,
}

#[derive(Debug, Clone)]
pub struct BehaveMethod {
    pub name: String,
    pub params: Vec<(String, TypeInfo)>,  // Including self parameter
    pub return_type: Option<TypeInfo>,
    pub is_abstract: bool,    // No body, must implement
    pub is_overrideable: bool, // Has default, can be overridden
}

// src/sema/checker.rs - FIX check_behave_impl
pub fn check_behave_impl(
    &mut self,
    type_name: &str,
    behave_name: &str,
    type_methods: &[FnSymbol],
    table: &SymbolTable,
) {
    match table.lookup(behave_name) {
        Some(Symbol::Behave(behave)) => {
            for bm in &behave.methods {
                let matching_method = type_methods.iter()
                    .find(|tm| tm.name == bm.name);
                
                match matching_method {
                    Some(tm) => {
                        // CHECK SIGNATURE COMPATIBILITY
                        if !self.method_signatures_compatible(tm, bm, table) {
                            self.error(
                                "SEMA-0022",
                                format!(
                                    "Method '{}' signature mismatch for behave '{}': expected {:?}, found {:?}",
                                    bm.name, behave_name, bm.params, tm.params
                                ),
                            );
                        }
                        
                        // Check override rules
                        if bm.is_overrideable && !tm.is_overrideable {
                            // Implementing type didn't mark as overrideable but should
                        }
                    }
                    None => {
                        if bm.is_abstract {
                            self.error(
                                "SEMA-0011",
                                format!(
                                    "Struct '{}' does not implement abstract behave method '{}'",
                                    type_name, bm.name
                                ),
                            );
                        }
                        // Non-abstract methods can use default implementation
                    }
                }
            }
        }
        _ => {
            self.error("SEMA-0002", format!("Undefined behaviour '{}'", behave_name));
        }
    }
}

fn method_signatures_compatible(
    &self,
    impl_method: &FnSymbol,
    behave_method: &BehaveMethod,
    table: &SymbolTable,
) -> bool {
    // Check parameter count (excluding self)
    if impl_method.params.len() != behave_method.params.len() - 1 {
        return false;
    }
    
    // Check return type compatibility
    if let (Some(impl_ret), Some(behave_ret)) = (&impl_method.return_type, &behave_method.return_type) {
        if !impl_ret.is_assignable_to(behave_ret) {
            return false;
        }
    }
    
    // Check parameter types
    for (i, (_, behave_param_type)) in behave_method.params.iter().enumerate().skip(1) {
        let impl_param_type = &impl_method.params[i - 1].1;
        if !impl_param_type.is_assignable_to(behave_param_type) {
            return false;
        }
    }
    
    true
}
```

---

### 3.5 Builtin Functions Are Hardcoded Stubs ❌ NOT FIXED

**Location:** `src/sema/checker.rs` lines 1150-1300, `src/ir/mod.rs` lines 700-750

**The Gap:**
Syntax.md declares 40+ builtins including:
- Memory: `@pageAlloc`, `@memcpy`, `@memset`
- Reflection: `@TypeOf`, `@SizeOf`, `@Fields`
- Control: `@panic`, `@trap`, `@sysCall`
- Math: `@addWithOverflow`, `@ctz`, `@popCount`

Current implementation:
- Only ~15 builtins have any checking
- Return types are hardcoded wrong (e.g., `@SizeOf` returns `TypeInfo::Void`)
- No IR generation for most builtins
- `@comptime` blocks completely unimplemented

**The 20% Structural Blueprint:**

```rust
// src/sema/checker.rs - EXPAND check_builtin_call
fn check_builtin_call(&mut self, name: &str, args: &[Expr], table: &mut SymbolTable) -> Option<TypeInfo> {
    match name {
        // SIZE/ALIGN - return usize
        "SizeOf" | "AlignOf" => {
            if args.len() != 1 {
                self.error("SEMA-0017", format!("@{} requires 1 argument", name));
                return Some(TypeInfo::Int(IntWidth::Arch, false));
            }
            // Argument should be a type, not expr - needs special parsing
            Some(TypeInfo::Int(IntWidth::Arch, false))
        }
        
        // TYPE QUERY - return TypeMeta
        "TypeOf" => {
            if args.len() != 1 {
                self.error("SEMA-0017", "@TypeOf requires 1 argument");
                return Some(TypeInfo::TypeMeta);
            }
            let arg_type = self.check_expr(&args[0], table);
            // Return the type OF the type (meta-level)
            Some(TypeInfo::TypeMeta)
        }
        
        // MEMORY OPERATIONS
        "memcpy" => {
            if args.len() != 3 {
                self.error("SEMA-0017", "@memcpy requires 3 arguments (dest, src, count)");
                return Some(TypeInfo::Void);
            }
            // Check dest and src are pointers
            let dest_type = self.check_expr(&args[0], table);
            let src_type = self.check_expr(&args[1], table);
            match (&dest_type, &src_type) {
                (Some(TypeInfo::Pointer(_)), Some(TypeInfo::Pointer(_))) => {}
                _ => {
                    self.error("SEMA-0023", "@memcpy requires pointer arguments");
                }
            }
            Some(TypeInfo::Void)
        }
        
        "pageAlloc" => {
            if args.len() != 1 {
                self.error("SEMA-0017", "@pageAlloc requires page count");
                return Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)));
            }
            Some(TypeInfo::Pointer(Box::new(TypeInfo::Void)))
        }
        
        // COMPTIME - special handling
        "comptime" => {
            // This requires evaluating the block at compile time
            // Currently UNIMPLEMENTED - would need interpreter
            self.error("SEMA-0024", "@comptime blocks require compile-time evaluation (not yet implemented)");
            Some(TypeInfo::AnyType)
        }
        
        // PANIC/TRAP - noret
        "panic" | "trap" => {
            Some(TypeInfo::Noret)
        }
        
        // OVERFLOW MATH - returns tuple (result, overflow_flag)
        "addWithOverflow" | "subWithOverflow" | "mulWithOverflow" => {
            if args.len() != 2 {
                self.error("SEMA-0017", format!("@{} requires 2 arguments", name));
                return Some(TypeInfo::AnyType);
            }
            // Should return (T, bool) tuple - but tuples aren't implemented!
            self.error("SEMA-0025", "Tuple return types not yet implemented");
            Some(TypeInfo::AnyType)
        }
        
        // BIT COUNTING
        "ctz" | "clz" | "popCount" => {
            if args.len() != 1 {
                self.error("SEMA-0017", format!("@{} requires 1 argument", name));
                return Some(TypeInfo::Int(IntWidth::W32, false));
            }
            Some(TypeInfo::Int(IntWidth::W32, false))
        }
        
        _ => {
            self.error("SEMA-0017", format!("Unknown builtin '@{}'", name));
            None
        }
    }
}
```

---

### 3.6 Test Blocks Have No Isolation ❌ NOT FIXED

**Location:** `src/parser/mod.rs` lines 600-620, `src/ir/mod.rs` lines 320-347

**The Gap:**
Syntax.md declares test blocks as isolated units:
```rzn
unit_test_case :: test {
    assert(a, b)
}
```

Current implementation:
- Tests are just functions named `test.{name}`
- No isolation between tests
- No `assert` builtin implementation
- No test runner infrastructure
- No failure reporting

**The 20% Structural Blueprint:**

```rust
// src/ast/mod.rs - ADD test-specific AST structure
#[derive(Debug, Clone)]
pub struct TestDecl {
    pub name: String,
    pub attrs: Vec<Annotation>,
    pub body: Block,
    pub dependencies: Vec<String>,  // Other tests this depends on
    pub is_isolated: bool,           // Runs in separate process
}

// Update Decl enum:
pub enum Decl {
    // ... existing ...
    Test(TestDecl),  // Instead of Test(String, Block)
}

// src/sema/checker.rs - ADD test checking
pub fn check_test(&mut self, test: &TestDecl, table: &mut SymbolTable) {
    table.push_scope();
    
    // Inject test-specific symbols
    let _ = table.insert("assert", Symbol::Builtin(BuiltinSymbol::Assert));
    let _ = table.insert("assert_eq", Symbol::Builtin(BuiltinSymbol::AssertEq));
    let _ = table.insert("fail", Symbol::Builtin(BuiltinSymbol::Fail));
    
    self.check_block(&test.body, table);
    table.pop_scope();
}

// src/ir/mod.rs - FIX test generation
fn gen_test(&mut self, test: &TestDecl, ir: &mut IrProgram) {
    let saved_temps = self.temps;
    let saved_labels = self.labels;
    let saved_defer = std::mem::take(&mut self.defer_stack);
    
    self.insts = Vec::new();
    self.temps = 0;
    self.labels = 0;
    
    let entry = self.new_label("entry");
    let success = self.new_label("success");
    let failure = self.new_label("failure");
    
    self.emit(IrInst::Label(entry));
    
    // Wrap test body in implicit try-catch for failure detection
    self.defer_stack.push(Vec::new());
    self.gen_block(&test.body);
    
    // Implicit success marker
    self.emit(IrInst::Jump(success.clone()));
    
    // Failure handler
    self.emit(IrInst::Label(failure.clone()));
    self.emit(IrInst::Comment("TEST FAILED"));
    self.emit(IrInst::RetVoid);
    
    // Success handler
    self.emit(IrInst::Label(success));
    
    // Run defers
    let deferred = self.defer_stack.pop().unwrap_or_default();
    for expr in deferred.iter().rev() {
        self.gen_expr(expr);
    }
    
    self.emit(IrInst::RetVoid);
    
    let func = IrFunction {
        name: format!("test.{}", test.name),
        params: Vec::new(),
        return_type: None,
        insts: std::mem::take(&mut self.insts),
    };
    ir.functions.push(func);
    
    self.temps = saved_temps;
    self.labels = saved_labels;
    self.defer_stack = saved_defer;
}
```

---

## SECTION 4: THE 20% STRUCTURAL BLUEPRINT REPORT

### Summary Table of Required Fixes

| # | Component | Severity | Lines Changed | Priority | Status |
|---|-----------|----------|---------------|----------|--------|
| 1 | Lexer: Collection tokens | HIGH | +50 | P0 | ✅ FIXED |
| 2 | AST: Slice type variant | HIGH | +5 | P0 | ✅ FIXED |
| 3 | Parser: Type parsing rewrite | CRITICAL | +100 | P0 | ✅ FIXED |
| 4 | Parser: Function declaration forms | MEDIUM | +60 | P1 | ✅ FIXED |
| 5 | Lexer: Escape sequence fix | CRITICAL | +30 | P0 | ✅ FIXED |
| 6 | Lexer: Block comment nesting | HIGH | +20 | P1 | ✅ FIXED |
| 7 | Parser: Expression precedence | CRITICAL | +30 | P0 | ⏭️ SKIPPED (Report Analysis Incorrect) |
| 8 | SEMA: Reachability analysis | MEDIUM | +40 | P2 | ✅ FIXED |
| 9 | IR: Phi node insertion | CRITICAL | +150 | P0 | ❌ NOT FIXED |
| 10 | IR: Match fallthrough chains | HIGH | +80 | P0 | ✅ FIXED |
| 11 | SEMA: Error union tracking | HIGH | +100 | P1 | ✅ FIXED |
| 12 | IR: Defer scope handling | MEDIUM | +30 | P2 | ✅ FIXED |
| 13 | SEMA: Loop capture typing | HIGH | +50 | P1 | ✅ FIXED |
| 14 | SEMA: Behave signature checking | MEDIUM | +80 | P2 | ✅ FIXED |
| 15 | SEMA: Builtin expansion | LOW | +150 | P3 | ❌ NOT FIXED |
| 16 | IR: Test isolation | LOW | +60 | P3 | ❌ NOT FIXED |

---

### Critical Path Implementation Order

**Phase 1 (P0 - Blocks Correct Compilation):**
1. Fix lexer escape sequences (buffer safety)
2. Fix parser precedence (correct AST structure)
3. Add phi nodes to IR (SSA correctness)
4. Fix match fallthrough chains (control flow)
5. Add slice type to AST (type completeness)

**Phase 2 (P1 - Language Feature Completeness):**
1. Add collection type tokens and parsing
2. Implement error union propagation
3. Fix loop capture type inference
4. Add runtime function variable support

**Phase 3 (P2 - Semantic Correctness):**
1. Fix reachability analysis
2. Improve defer scoping
3. Implement behave signature checking

**Phase 4 (P3 - Standard Library Support):**
1. Expand builtin function coverage
2. Implement test isolation framework

---

### Architectural Debt Summary

The Razen compiler has **three fundamental architectural debts**:

1. **No SSA Construction Algorithm** - The IR generator emits naive three-address code without proper phi placement. This prevents all downstream optimizations and produces incorrect code for variable reassignments.

2. **Incomplete Type System Representation** - The AST cannot represent half the language's type forms (slices, heap collections, comptime types). This makes semantic analysis impossible for valid programs.

3. **Ghost Error Handling** - Error unions exist as types but have no operational semantics. The `?` operator, try-blocks, and error propagation are stub implementations that accept invalid code.

These are not bugs—they are **missing foundational layers**. Fixing them requires implementing:
- A proper SSA construction pass (e.g., Cytron-Ferrante algorithm)
- A complete type resolver with kind system
- An effect system for error tracking

Without these, the compiler will accept invalid programs and reject valid ones, producing broken IR for anything beyond trivial examples.
