# Parser vs Syntax Specification — Full Audit Report

## 1. MISSING FEATURES (Syntax Spec has it, Parser does NOT implement)

### 1.1 `stop` and `next` as expressions (not just statements)
**Syntax spec** (line 443-447): `stop` and `next` appear inside expressions.  
**Parser**: `stop`/`next` are only parsed as `Stmt::Stop` / `Stmt::Next` (line 1700-1706). They cannot appear in expression position (e.g., `r :: if cond { stop } else { 0 }`).  
**Fix needed**: Add `Stop` and `Next` as expression variants, or allow them in expression contexts.

### 1.2 `type` keyword for type alias syntax
**Syntax spec** (line 81, types.md line 32): `type(T)` is a valid type.  
**Parser**: `TokenKind::Type` is only handled in `parse_const_like` for `Decl::TypeAlias` (line 378-379), but `type(T)` is never handled in `parse_type()` — it can't be used inline as a type annotation.  
**Bug**: `parse_type_alias` (line 784) expects `type ( T )` with parens, which is correct for the declaration. But `type(T)` used as a type annotation elsewhere (e.g., `x : type(i32) : 5`) is not handled in `parse_type()`.

### 1.3 `@str.from_raw(ptr, len)` method call on `@str` type
**Syntax spec** (builtins.md line 39): `@str.from_raw(ptr, len)`.  
**Parser**: `@str.method(args)` in types (line 970-991) only handles `@str.method` but doesn't parse the method name followed by `(args)` — it only parses the method name, then `(`. But the implementation at line 973 does handle `@str.method(args)`. **Partially implemented** — but `@str.from` is not listed as a builtin.

### 1.4 `else` without a space after it (e.g., `else{`)
**Minor** — not really a bug, but `else {` must have whitespace. Parser handles it fine.

### 1.5 `noret` type keyword
**Syntax spec** (types.md line 28): `noret` is a valid type.  
**Parser**: `TokenKind::Noret` exists in lexer and `is_type_keyword` (line 2076). ✅ Implemented.

### 1.6 `anytype` type keyword  
**Syntax spec** (types.md line 29): `anytype` is a valid type.  
**Parser**: `TokenKind::AnyType` exists and is in `is_type_keyword` (line 2077). ✅ Implemented.

---

## 2. BUGS & INCORRECT BEHAVIOR

### 2.1 **BUG: `parse_type_alias` discards generics and pub/attrs**
`src/parser/mod.rs:784-797`:
```rust
fn parse_type_alias(
    &mut self,
    name: String,
    _generics: Vec<String>,   // DISCARDED
    _pub_: bool,               // DISCARDED
    _attrs: Vec<Annotation>,   // DISCARDED
) -> PResult<Decl> {
```
The `Decl::TypeAlias(String, Type)` AST variant (ast/mod.rs:28) only stores `name` and `type_` — it **cannot** hold `pub_`, `generics`, or `attrs`. But the syntax spec doesn't explicitly show generics on type aliases, so this may be by design. Still, `pub` support is missing from the AST.

### 2.2 **BUG: `parse_error_decl` discards generics and attrs**
`src/parser/mod.rs:723-744`:
```rust
fn parse_error_decl(
    &mut self,
    name: String,
    _generics: Vec<String>,   // DISCARDED
    _attrs: Vec<Annotation>,   // DISCARDED
) -> PResult<Decl> {
```
`Decl::Error_(String, Vec<EnumVariant>)` has no fields for generics, pub, or attrs. The spec says error types are just `error { ... }` with no generics shown, so generics may be fine. But `pub` and `attrs` are silently dropped.

### 2.3 **BUG: `ConstDecl` has no `pub_` field**
`src/ast/mod.rs:96-102`: `ConstDecl` lacks `pub_: bool`.  
The parser at line 179 consumes `pub_` for `parse_decl`, but for `Decl::Const` it's stored only in the `ConstDecl` without `pub_`. The `pub` keyword is consumed but **never stored** in the AST node. This means `pub PI :: 3.14` silently drops the `pub`.

The test at line 1066 even acknowledges this:
```rust
Decl::Const(_c) => {} // pub is consumed but not stored on ConstDecl
```

### 2.4 **BUG: `mut` comptime variable uses `::` but parser doesn't support it**
**Syntax spec** (line 164-170):
```rzn
mut COMPTIME_VAR :: value         // comptime mutable variable
mut COMPTIME_VAR : type : value   // comptime mutable variable with type
```
**Parser**: `parse_decl` (line 177) consumes `mut_` at the top level, but when `::` is encountered (line 199), it goes to `parse_const_like`. In `parse_const_like` (line 385), if no keyword follows, it creates `Decl::Const` — which **silently drops `mut_`**. The `ConstDecl` AST node has no `mutable` field.

There's no AST representation for mutable comptime variables at all.

### 2.5 **BUG: `parse_explicit_type_decl` handles `::` but creates `ConstDecl` instead of `VarDecl`**
`src/parser/mod.rs:484-493`:
```rust
} else if self.consume_if(TokenKind::Colon) {
    let value = self.parse_expr()?;
    self.expect_newline_or_semi();
    Ok(Decl::Const(ConstDecl { ... }))
```
After `name : type`, if `:` follows (constant syntax), it creates a `ConstDecl`. But `ConstDecl` has **no `pub_` field**, so any `pub` is lost. Also, `mut` comptime variables (`mut COMPTIME_VAR : type : value`) would be parsed here but the `mut` flag is dropped.

### 2.6 **BUG: `parse_if_expr` wraps in `Expr::Block` unnecessarily**
`src/parser/mod.rs:1905-1910`:
```rust
fn parse_if_expr(&mut self) -> PResult<Expr> {
    let if_ = self.parse_if_inner()?;
    Ok(Expr::Block(Block {
        stmts: vec![Stmt::If(if_)],
    }))
}
```
This wraps an `if` expression inside `Expr::Block(Stmt::If(...))`, but the syntax spec says `if` is an expression that returns a value directly. The `Expr` enum has no `If` variant — it goes through `Stmt::If` which is wrong for expressions. This causes the test at line 418-429 to pass by checking for `Expr::Block` wrapping `Stmt::If`, but it's semantically incorrect — `if` as an expression should be directly representable.

### 2.7 **BUG: `parse_match_expr` and `parse_loop_expr` have the same Block-wrapping issue**
`src/parser/mod.rs:1912-1924`: Same pattern as if — wraps in `Expr::Block` with `Stmt::Match` / `Stmt::Loop` inside. These should be first-class expression variants (`Expr::If`, `Expr::Match`, `Expr::Loop`) rather than forcing them through statement types.

### 2.8 **BUG: `expect_newline_or_semi` is a no-op for statement separation**
`src/parser/mod.rs:173-175`:
```rust
fn expect_newline_or_semi(&mut self) {
    self.consume_if(TokenKind::Semicolon);
}
```
This only optionally consumes a semicolon. It does **NOT** validate that there's a newline. In Razen, newlines are statement separators (semicolons are optional). Without checking for newlines, the parser could incorrectly treat multiple declarations on the same line as valid:
```
x := 10 y := 20   // Should be an error, but parser may accept this
```
The `at_stmt_end` function (line 2038-2043) checks for `;`, `}`, `)`, or EOF — but `parse_stmt` doesn't actually verify that the next token is on a new line after consuming the current statement. This is a structural bug.

### 2.9 **BUG: `parse_decl` fallback case creates `ConstDecl` with empty name**
`src/parser/mod.rs:210-234`: When no operator (`::`, `:=`, `:`) follows an identifier, the code falls through to:
```rust
let mut expr = Expr::Ident(name);
// ...
Ok(Decl::Const(ConstDecl {
    name: String::new(),   // EMPTY NAME
    ...
    value: Some(expr),
}))
```
This creates a constant with an **empty name**, which is clearly wrong. A standalone expression statement should be `Stmt::Expr`, not a `Decl::Const`.

### 2.10 **BUG: `ColonEquals` used in assignment operators inside `infix_bp` and `build_binary`**
`src/parser/mod.rs:2127`: `ColonEquals` has binding power `(1, 2)`, the same as regular assignment. But `:=` is the **variable declaration** operator, not an assignment operator. Having it in `infix_bp` means `a := b` in expression context becomes a binary operator, which is wrong — `:=` should only appear in declaration contexts.

Similarly, `ColonEq` appears in `build_binary` (line 2181) and `BinaryOp` (ast/mod.rs:193). This allows `:=` as a binary expression operator, which shouldn't be valid.

### 2.11 **BUG: `@vec`, `@map`, `@set` literal parsing for `@set` uses `Expr::ArrayInit`**
`src/parser/mod.rs:1469`:
```rust
return Ok(Expr::ArrayInit(elems));
```
`@set{1, 2, 3}` creates `Expr::ArrayInit`, but sets are conceptually different from arrays. There's no `Expr::SetLiteral` variant. This means a set literal and an array literal are indistinguishable in the AST.

### 2.12 **BUG: `@set[T]` type syntax parsed as `Expr::Ident` in primary expressions**
`src/parser/mod.rs:1471-1487`: After consuming `@set`, if `[` follows, it parses a **type** and returns `Expr::Ident(format!("@set[{}]", ...))`. But this is in `parse_primary_expr`, which is for **expressions**, not types. This conflates type syntax and expression syntax.

### 2.13 **BUG: `@vec`, `@map`, `@set` type syntax not handling `@vec[T; N]` correctly in all cases**
The `parse_type` function handles `@vec[T]` and `@vec[T; N]` for types (line 906-922), but the `@vec` token-specific handler in `parse_type` (line 906-922) duplicates logic that already exists in the generic `@` handler (line 856-905). This redundancy could lead to inconsistencies.

### 2.14 **BUG: `parse_struct_decl` doesn't handle `ext` keyword for external structs**
The syntax spec doesn't show `ext struct`, but `ext` is consumed at the top of `parse_decl` and passed to `parse_const_like`. In `parse_struct_decl` (line 562), the `ext` parameter is accepted but never used. External structs aren't part of the spec, so this is dead code.

### 2.15 **BUG: `parse_behave_decl` inserts synthetic `self` parameter even for non-self methods**
`src/parser/mod.rs:757-771`: Every method in a `behave` block gets a `self` parameter prepended. But the syntax spec (line 347-353) shows methods can have `self: &@Self` explicitly in the signature. If the user already writes `self: &@Self` in the method params, this code would add a **duplicate** `self` parameter.

The test at line 938-947 uses:
```
draw :: fn(self: &@Self) -> void
```
This means `self` is already in the params, and the parser would insert another `self` before it. **This is a real bug** — the `parse_behave_decl` should check if the first param is already `self` before inserting one.

### 2.16 **BUG: `else` block in `if` as statement vs expression**
In `parse_if_inner` (line 1831-1838):
```rust
let else_block = if self.consume_if(TokenKind::Else) {
    if self.consume_if(TokenKind::If) {
        let inner = self.parse_if_inner()?;
        Some(Box::new(Stmt::If(inner)))
    } else {
        self.expect(TokenKind::LeftBrace)?;
        let else_block = self.parse_block_contents()?;
        Some(Box::new(Stmt::Block(else_block)))
    }
```
The `else` branch is stored as `Stmt::Block`, but the syntax spec says `else { ... }` is a block expression. The `Stmt::Block` wrapping is fine structurally, but if the `else` is the last thing in an `if` expression, the return value semantics are unclear.

### 2.17 **BUG: `try ... catch` single-line shorthand doesn't match spec exactly**
**Syntax spec** (line 527-529):
```rzn
try safe_fallback_call()
catch |err| handle_isolated_error(err)
```
**Parser** (line 1954-1967): The single-line form parses `try_expr` then `catch` then `parse_pipe_capture()` then `parse_expr()`. This creates a `TryCatch` with the try body as a single expression block and the catch body as a single expression block. But the spec shows `catch |err| expr` — the catch block should be the **expression directly**, not wrapped in a block. The parser wraps it in a block:
```rust
let catch_body = Block {
    stmts: vec![Stmt::Expr(self.parse_expr()?)],
};
```
This is actually fine semantically but adds an unnecessary wrapping layer.

### 2.18 **BUG: `ColonEquals` in `infix_bp` causes `:=` to be parsed as binary operator**
As mentioned in 2.10, `ColonEquals` appears in `infix_bp` (line 2127). This means if you write:
```rzn
x := y := 5
```
The parser would try to parse `y := 5` as an expression (binary assignment), which is wrong. `:=` should only be valid in declaration contexts, not as a binary expression operator.

---

## 3. AST DESIGN ISSUES

### 3.1 `Expr` enum missing `If`, `Match`, `Loop` variants
`src/ast/mod.rs:144-165`: The `Expr` enum has no `If`, `Match`, or `Loop` variants. These are forced through `Expr::Block` with `Stmt::If` inside. This is incorrect — `if`, `match`, and `loop` are expressions in Razen and should be first-class expression variants.

### 3.2 `Pattern` enum is too limited
`src/ast/mod.rs:278-284`: The `Pattern` enum only has:
- `Wildcard`
- `Ident(String)`
- `Literal(TokenKind, String)`
- `EnumVariant(String, String, Option<String>)`

Missing patterns from the spec:
- **Range patterns**: `1..5` in match arms
- **Multi-pattern**: `1 | 2 | 3 =>` (alternative patterns)
- **Struct/tuple destructuring patterns** (not in current spec but likely future)

### 3.3 `Stmt::Ret` vs `Expr::Ret` duplication
`src/ast/mod.rs:162,224`: Both `Stmt::Ret(Option<Expr>)` and `Expr::Ret(Option<Box<Expr>>)` exist. This creates ambiguity — is `ret` a statement or an expression? The syntax spec doesn't show `ret` as an expression. `Expr::Ret` seems unnecessary and confusing.

### 3.4 `BinaryOp::ColonEq` shouldn't be a binary op
`src/ast/mod.rs:193`: `ColonEq` is listed as a `BinaryOp`, but `:=` is a declaration operator, not a binary expression operator. It shouldn't be in the expression AST.

### 3.5 `AssignOp::ColonEq` is suspicious
`src/ast/mod.rs:218`: `ColonEq` is an `AssignOp`, but `:=` is declaration, not assignment. Having it as an assign op is misleading.

### 3.6 No `Stmt::Break` / `Stmt::Continue` as expressions
The spec shows `stop` and `next` can appear in expressions. But `Stmt::Stop` and `Stmt::Next` are statements only.

### 3.7 `FnDecl.is_const` vs `FnDecl.is_variable_fn` is redundant
Both fields exist on `FnDecl` (ast/mod.rs:42-43). `is_const` = `!is_variable_fn` always. One field would suffice.

---

## 4. LEXER/PARSER MISMATCH

### 4.1 `DotStar` for `.*` dereference
**Syntax spec** (line 29): `.*` is explicit postfix dereference.  
**Lexer**: `TokenKind::DotStar` exists. ✅  
**Parser**: `parse_postfix` handles `DotStar` at line 1586. ✅  
No issue here.

### 4.2 `TildeArrow` for `~>` implements behaviour
**Lexer**: `TokenKind::TildeArrow` exists. ✅  
**Parser**: Handled in `parse_struct_decl` (line 570) and `parse_enum_decl` (line 669). ✅

### 4.3 `FatArrow` for `=>` in match arms
**Lexer**: `TokenKind::FatArrow` exists. ✅  
**Parser**: Handled in `parse_match_inner` (line 1859). ✅

---

## 5. SYNTAX SPEC FEATURES NOT YET IN LEXER (and thus not parseable)

### 5.1 `@Self` builtin type
**Syntax spec** (builtins.md line 12): `@Self` is a builtin.  
**Lexer/Token**: No `AtSelf` token. `@` followed by `Self` would be lexed as `At` + `Identifier("Self")`.  
**Parser**: The `@` handler in `parse_type` (line 856) would treat `@Self` as `Type::Builtin("Self")`. This works but is fragile — `Self` could clash with user identifiers.

### 5.2 `@vec_with_allocator`, `@str.from` etc. as runtime builtins
**Syntax spec** (builtins.md lines 662-664): `@vec[elements...]`, `@map{pairs...}`, `@set{elements...}` — these are materialization builtins.  
**Parser**: These are handled as expressions (line 1411-1491). But `@vec_with_allocator` (spec line 662) is not handled — it's not a recognized token or `@`-prefixed builtin.

### 5.3 Missing: `@compileError(str)` as a statement
**Syntax spec** (builtins.md line 33): `@compileError(str)` emits a compiler error.  
**Parser**: Not explicitly handled — falls through to generic `@` builtin call. This is fine at parse time but should be a special statement.

---

## 6. SUMMARY OF CRITICAL BUGS

| # | Severity | Location | Description |
|---|----------|----------|-------------|
| 1 | **HIGH** | ast/mod.rs:96-102 | `ConstDecl` has no `pub_` field — `pub` on constants is silently dropped |
| 2 | **HIGH** | parser/mod.rs:210-234 | Fallback creates `ConstDecl` with **empty name** `String::new()` |
| 3 | **HIGH** | parser/mod.rs:757-771 | `parse_behave_decl` inserts duplicate `self` param if method already has one |
| 4 | **HIGH** | parser/mod.rs:1905-1924 | `if`/`match`/`loop` expressions forced through `Expr::Block(Stmt::...)` instead of proper expression variants |
| 5 | **HIGH** | parser/mod.rs:2127,2181 | `:=` treated as binary operator in expression context via `infix_bp` |
| 6 | **MEDIUM** | ast/mod.rs:28 | `TypeAlias` AST variant lacks `pub_`, `generics`, `attrs` fields |
| 7 | **MEDIUM** | parser/mod.rs:173-175 | `expect_newline_or_semi` doesn't validate newline separation |
| 8 | **MEDIUM** | parser/mod.rs:1469 | `@set{...}` parsed as `Expr::ArrayInit` — no `SetLiteral` variant |
| 9 | **MEDIUM** | ast/mod.rs:162,224 | `Expr::Ret` and `Stmt::Ret` duplication |
| 10 | **MEDIUM** | parser/mod.rs:723-744 | `parse_error_decl` silently drops `generics` and `attrs` |
| 11 | **LOW** | ast/mod.rs:193 | `BinaryOp::ColonEq` shouldn't exist as a binary op |
| 12 | **LOW** | ast/mod.rs:278-284 | `Pattern` enum missing range patterns and multi-pattern (`\|`) |
| 13 | **LOW** | parser/mod.rs:1471-1487 | `@set[T]` type syntax parsed in expression context (`parse_primary_expr`) |

---

## 7. TEST COVERAGE GAPS

The tests cover basic happy paths but miss:
- `pub` on constants (test_acknowledges the bug at line 1066)
- Mutable comptime variables (`mut COMPTIME_VAR :: val`)
- `type(T)` used as a type annotation
- `stop`/`next` as expression values
- Behave methods with explicit `self` parameter (duplicate self bug)
- Multi-line statement separation validation
- `@compileError`, `@compileLog` as standalone statements
- Range patterns in match
- Nested generic types (`Result<T, E>!SomeError`)
- `@str.from_raw(ptr, len)` syntax
