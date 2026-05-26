# Razen Compiler (`razenc`) — Development Plan

> **Last Updated:** 2026-05-26
> **Target:** Self-hosted Razen compiler written in Rust, compiling Razen source to native code via LLVM.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Priority Tiers](#2-priority-tiers)
3. [Phase 1: Lexer — Complete](#3-phase-1-lexer--complete)
4. [Phase 2: Parser — Priority 1](#4-phase-2-parser--priority-1)
5. [Phase 3: Semantic Analysis — Priority 2](#5-phase-3-semantic-analysis--priority-2)
6. [Phase 4: CLI / Driver — Priority 3](#6-phase-4-cli--driver--priority-3)
7. [Phase 5: LLVM Codegen — Priority 4](#7-phase-5-llvm-codegen--priority-4)
8. [Phase 6: Standard Library — Priority 5](#8-phase-6-standard-library--priority-5)
9. [Phase 7: Tooling & Infrastructure — Priority 6](#9-phase-7-tooling--infrastructure--priority-6)
10. [Risk Register](#10-risk-register)
11. [Milestone Roadmap](#11-milestone-roadmap)

---

## 1. Project Overview

**razenc** is a compiler for the Razen programming language. It is written in Rust and compiles Razen source code (`*.rzn`) through four phases:

| Phase | Module | Input | Output | Priority |
|---|---|---|---|---|
| 1. Lexing | `src/lexer/` | Source text (`&str`) | `Vec<Token>` | ✅ Complete |
| 2. Parsing | `src/parser/` + `src/ast/` | `Vec<Token>` | `Program` (AST) | 🔴 Priority 1 |
| 3. Semantic Analysis | `src/sema/` | `Program` (AST) | Annotated `Program` | 🟠 Priority 2 |
| 4. LLVM Codegen | `src/llvm/` | Annotated `Program` | LLVM IR / Object code | 🟡 Priority 4 |

Supporting modules:
- **`src/cmd/`** — CLI driver, pipeline orchestration (Priority 3)
- **`src/bdg/`** — Debug/display utilities (Complete)
- **`src/std/`** — Standard library (Priority 5)
- **`syntax/`** — Language specification documents (Complete)

---

## 2. Priority Tiers

```
P0 ─────────────────────────────────────────────────────────► P5
   ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐
   │Parser│  │ Sema │  │ CLI  │  │LLVM  │  │ Std  │  │Tools │
    │ 100% │  │ 90%  │  │ 80%  │  │  0%  │  │  0%  │  │ 10%  │
   └──────┘  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘
   P1         P2        P3         P4         P5         P6
```

**Rationale:**
- **Parser (P1):** The parser is the gatekeeper — everything downstream depends on correct AST output. Reaching 100% here unblocks all later phases.
- **Semantic Analysis (P2):** Type checking and symbol resolution catch the majority of user errors. A robust sema is required before codegen.
- **CLI (P3):** The developer UX — build commands, error reporting, file watching. Can be improved incrementally.
- **LLVM Codegen (P4):** The largest single piece of work. Only makes sense once the frontend is stable.
- **Standard Library (P5):** Allocators, collections, I/O. Heavily dependent on codegen working.
- **Tooling (P6):** LSP, formatter, package manager. Nice-to-haves.

---

## 3. Phase 1: Lexer — Complete

**Status: ✅ 100% — No remaining work.**

### 3.1 Token Definitions (`src/lexer/token.rs`)
- **Lines:** 283
- **Status:** Complete. All token kinds defined including keywords (20+), type keywords (25+), operators (30+), sigils (12), grouping, literals, comments. All `Display` impls present.
- **Verification:** 75 passing tests.

### 3.2 Lexer Implementation (`src/lexer/lexer.rs`)
- **Lines:** 592
- **Status:** Complete. Whitespace, line/block comments (nested), strings with full escape sequences (`\n`, `\t`, `\r`, `\\`, `\0`, `\"`, `\'`, `\xHH`, `\u{XXXX}`), char literals, numbers (dec/hex/bin/oct with `_` separators), floats (with exponents), all sigils/operators, keyword recognition, error reporting for invalid characters, unclosed strings/comments, invalid escapes.
- **Verification:** Span tracking tested with ASCII and UTF-8 input.

### 3.3 Test Coverage (`src/lexer/tests.rs`)
- **Lines:** 946
- **Tests:** 75 test cases
- **Coverage:** Every token kind, error recovery, edge cases (empty source, unicode, mixed tokens, line/col tracking).

---

## 4. Phase 2: Parser — Priority 1

**Status: 🟢 100% — All Priority 1 items complete. No remaining gaps.**

### 4.1 AST Definitions (`src/ast/mod.rs`) — 100%
- **Lines:** 240
- **Status:** Complete. All 12 `Decl` variants, 16 `Expr` variants, 11 `Stmt` variants, 9 `Type` variants, operator enums, supporting structs.

### 4.2 Core Parser (`src/parser/mod.rs`) — 1520 lines

#### 4.2.1 Declaration Parsing — ✅ Complete
| Feature | Status | Tests |
|---|---|---|
| `Use` path parsing | ✅ | `test_use_decl` |
| `Mod` block/inline | ✅ | — |
| `Fn` with params, generics, `pub`, `ext` | ✅ | `test_fn_decl_*` (7 tests) |
| `Struct` with fields, methods, generics, `~>` behave | ✅ | `test_struct_decl_*` (4 tests) |
| `Union` with variants | ✅ | `test_union_decl` |
| `Enum` with variants, associated values, methods | ✅ | `test_enum_decl`, `test_enum_with_values` |
| `Error` declaration | ✅ | `test_error_decl` |
| `Behave` trait declaration with methods | ✅ | `test_behave_decl` |
| `Type` alias | ✅ | `test_type_alias` |
| `Test` declaration | ✅ | `test_test_decl` |

#### 4.2.2 Variable/Constant Declarations — ✅ Complete
| Syntax Form | Status | Tests |
|---|---|---|
| `name := expr` (inferred runtime) | ✅ | `test_var_decl_inferred` |
| `mut name := expr` (mutable) | ✅ | `test_var_decl_mutable` |
| `name : type = expr` (explicit type) | ✅ | `test_var_decl_explicit_type` |
| `name : type := expr` | ✅ | `test_var_decl_colon_eq` |
| `NAME :: expr` (comptime) | ✅ | `test_const_decl_inferred` |
| `NAME : type : expr` (explicit comptime) | ✅ | `test_const_decl_explicit_type` |
| `pub` modifier | ✅ | `test_var_decl_pub`, `test_pub_decl`, `test_pub_fn_decl` |

#### 4.2.3 Expression Parsing (Pratt Parser) — ✅ Complete
**Operator precedence table (lowest to highest):**

| Precedence | Operators | Associativity |
|---|---|---|
| 1-2 | `=` `:=` `+=` `-=` `*=` `/=` `%=` | Right |
| 3-4 | `\|\|` | Left |
| 5-6 | `&&` | Left |
| 7-8 | `\|` (binary OR) | Left |
| 9-10 | `^` | Left |
| 11-12 | `&` (bitwise AND) | Left |
| 13-14 | `==` `!=` | Left |
| 15-16 | `<` `>` `<=` `>=` | Left |
| 17-18 | `<<` `>>` | Left |
| 19-20 | `..` `..=` | Left |
| 21-22 | `+` `-` | Left |
| 23-24 | `*` `/` `%` | Left |

| Feature | Status | Tests |
|---|---|---|
| Binary arithmetic | ✅ | `test_binary_ops`, `test_complex_arithmetic` |
| Comparison operators | ✅ | `test_comparison_ops` |
| Logical operators | ✅ | `test_logical_ops` |
| Bitwise operators (OR, AND, XOR, shift) | ✅ | `test_bitwise_*` (4 tests) |
| Range operators (`..`, `..=`) | ✅ | `test_range_ops`, `test_dotdot_equals_range` |
| Assignment operators | ✅ | `test_assignment_ops` |
| Compound assignment | ✅ | `test_assignment_ops` |
| Unary: `-`, `!`, `~` | ✅ | `test_unary_ops` |
| Unary: `&`, `&mut` (reference) | ✅ | `test_unary_ops` |
| Unary: `*` (dereference) | ✅ | `test_unary_ops` |
| Unary: `?` (optional wrap) | ✅ | `test_optional_chaining` |
| Parenthesized expressions | ✅ | `test_paren_expr` |

#### 4.2.4 Postfix Expression Parsing — ✅ Complete
| Feature | Status | Tests |
|---|---|---|
| Function calls `f(args...)` | ✅ | `test_call_expr` |
| Field access `.name` | ✅ | `test_field_access` |
| Index access `[expr]` | ✅ | `test_index_access` |
| Slice expressions `[start..end]`, `[start..=end]` | ✅ | `test_slice_expr` |
| Explicit dereference `.*` | ✅ | `test_deref` |
| `@method` syntax | ✅ | `test_at_method` |
| `catch` postfix | ✅ | `test_try_catch_capture` |

#### 4.2.5 Control Flow — ✅ Complete
| Feature | Status | Tests |
|---|---|---|
| If-else with `\|x\|` capture | ✅ | `test_if_expr`, `test_if_else_if`, `test_if_capture`, `test_nested_if_else`, `test_if_expr_else_if_chain_deep` |
| Match with patterns and captures | ✅ | `test_match_expr_simple`, `test_match_capture`, `test_match_arm_capture` |
| Loop (infinite, conditional, range, multi-range) | ✅ | `test_loop_*` (5 tests) |
| `ret` with/without value | ✅ | `test_ret_expr`, `test_ret_void` |
| `stop` / `next` | ✅ | — |
| `defer` | ✅ | `test_defer` |
| `try` / `catch` | ✅ | `test_try_catch` |

#### 4.2.6 Type Parsing — ✅ Complete
| Syntax | Status | Tests |
|---|---|---|
| Primitive types (`i32`, `f64`, `bool`, etc.) | ✅ | — |
| Named types (`MyType`) | ✅ | — |
| Reference types (`&T`, `&mut T`) | ✅ | `test_ref_type` |
| Pointer types (`*T`) | ✅ | — |
| Optional types (`?T`) | ✅ | `test_optional_type` |
| Error unions (`E!T`, `!T`) | ✅ | `test_error_union_type` |
| Array types (`[T]`, `[T; N]`) | ✅ | — |
| Function types (`fn(T) -> U`) | ✅ | `test_fn_type` |
| Builtin collection types (`@vec[T]`, `@map{K,V}`, `@set{T}`) | ✅ | `test_vec_type`, `test_map_type`, `test_set_type` |
| `@Self` in behave context | ✅ | — |

#### 4.2.7 Miscellaneous — ✅ Complete
| Feature | Status | Tests |
|---|---|---|
| Anonymous function expressions | ✅ | `test_anon_fn_expr`, `test_anon_fn_no_return` |
| Struct initialization | ✅ | `test_struct_init` |
| Pipe capture parse detection (vs binary OR) | ✅ | `test_pipe_as_binary_not_capture`, `test_bitwise_or_without_capture` |
| Generic functions/structs with `<T, U>` | ✅ | `test_fn_decl_generic`, `test_generic_fn_multi_params`, `test_generic_struct` |
| Comment skipping | ✅ | `test_comment_only`, `test_mixed_comments_and_code` |
| `ext` (external function) | ✅ | `test_ext_fn_decl` |
| Nested blocks | ✅ | `test_nested_blocks` |
| Multi-declaration files | ✅ | `test_multi_decl` |

### 4.3 Parser Test Suite (`src/parser/tests.rs`)
- **Lines:** 1302
- **Tests:** 52 test cases
- **Coverage:** Every declaration form, expression type, control flow construct, error case.

### 4.4 Remaining Parser Work — 🔴 Priority 1 Items

#### P-PARSER-01: Statement-level error recovery
- **Description:** The parser currently recovers from errors only at the declaration level (skip to next decl). Add recovery within statements (e.g., skip to next `;` or `}` after a bad expression).
- **Impact:** Prevents "cascading errors" where one bad token poisons the rest of the file.
- **Estimate:** 2-3 days
- **Files affected:** `src/parser/mod.rs`
- **Dependencies:** None
- **Status:** ✅ Complete — `parse_block_contents` wraps `parse_stmt` in try/error with `recover_stmt()` to skip to next boundary on failure. Tests: `test_error_recovery_in_block`, `test_error_recovery_multiple_errors`.

#### P-PARSER-02: Improve pipe-capture vs binary-OR disambiguation
- **Description:** The current heuristic (`is_capture_start`) looks ahead to check if `|...|` is followed by `{` or `=>`. This can fail with nested expressions like `x | (y \| z)` or captures with complex patterns.
- **Impact:** Occasional parse failures on edge cases.
- **Estimate:** 1-2 days
- **Files affected:** `src/parser/mod.rs:685-733`
- **Dependencies:** None
- **Status:** ✅ Complete — `is_capture_start` now checks for `&`, `mut`, identifiers, commas between pipes, and requires `{`, `=>`, `,`, `|`, or `}` after closing pipe. Tests: `test_pipe_with_nested_parens`, `test_pipe_or_with_block`, `test_pipe_as_binary_not_capture`, `test_bitwise_or_without_capture`.

#### P-PARSER-03: Better error messages with span information
- **Description:** Error messages currently include line numbers but not column ranges or source snippets. Add span-based diagnostics with `--> filename:line:col` format.
- **Impact:** Developer experience — hard to locate errors in large files.
- **Estimate:** 2-3 days
- **Files affected:** `src/parser/mod.rs`, `src/bdg/mod.rs`
- **Dependencies:** None
- **Status:** ✅ Complete — `ParseError` struct with `message`, `line`, `col`. Parser stores source text, `bdg::print_parse_error` renders `--> file:line:col` with source line and blue caret indicator. All internal error methods (`expect`, `expect_ident`, etc.) produce structured errors.

#### P-PARSER-04: Attribute/annotation parsing
- **Description:** The syntax spec mentions `@` as builtin prefix and structural attributes. Parse annotation syntax like `@inline`, `@export("name")` on declarations.
- **Impact:** Future-proofing for optimization hints and metadata.
- **Estimate:** 2-3 days
- **Files affected:** `src/ast/mod.rs`, `src/parser/mod.rs`
- **Dependencies:** None
- **Status:** ✅ Complete — `Annotation` struct in AST, `parse_attrs()` in parser handles `@name` and `@name(args)` before declarations. Tests: `test_annotation_on_fn`, `test_annotation_with_args`, `test_multiple_annotations`, `test_annotation_on_struct`, `test_annotation_on_var`, `test_annotation_on_const`.

#### P-PARSER-05: Allow trailing commas consistently
- **Description:** Some parsers accept trailing commas in struct/init/enum fields, others do not. Make this consistent everywhere (recommend: allow always).
- **Impact:** User convenience — avoids friction when reordering fields.
- **Estimate:** 1 day
- **Files affected:** `src/parser/mod.rs`
- **Dependencies:** None
- **Status:** ✅ Complete — trailing commas consistently accepted via `consume_if(TokenKind::Comma)` after field/param lists. Tests: `test_trailing_comma_in_fn_params`, `test_trailing_comma_in_struct`, `test_trailing_comma_in_enum`, `test_trailing_comma_in_enum_values`, `test_trailing_comma_in_struct_init`, `test_trailing_comma_in_call`, `test_trailing_comma_in_generics`.

---

## 5. Phase 3: Semantic Analysis — Priority 2

**Status: 🟢 ~90% — All 20 SEMA errors implemented. Control flow, inference, struct validation, generics checking, test suite complete.**

### 5.1 Completed Work

#### 5.1.1 Type System (`src/sema/types.rs`) — 344 lines ✅
- TypeInfo enum with all semantic types
- Display formatting for all types
- `is_numeric()`, `is_integer()`, `is_float()`, `is_bool()`, `is_noret()`, `is_void()`, `is_reference()`, `is_pointer()`, `is_optional()`, `is_error_union()`, `is_struct()`, `is_enum()`, `is_union()`, `is_function()`, `is_string()`
- `is_assignable_to()` with structural comparison for compound types
- `resolve_builtin()` for all primitive type names
- `resolve_ast_type()` — converts AST types to semantic TypeInfo
- `literal_to_type()` — maps literal tokens to base types

#### 5.1.2 Symbol Table (`src/sema/scope.rs`) — 189 lines ✅
- `SymbolTable` with scope chain (push/pop/lookup)
- `insert()` with duplicate detection
- `insert_overwrite()` for type refinement
- `lookup_in_current()` for shadowing checks
- `lookup_type()` for convenient type queries
- Full Symbol enum: Variable, Parameter, Function, StructType, EnumType, UnionType, ErrorSet, Behave, TypeAlias, BuiltinFn
- `get_type()` for extracting TypeInfo from any symbol
- `is_mutable()`, `is_const()`, `is_function()` helper methods

#### 5.1.3 Error Checking Implemented (20/20) ✅

| Code | Name | Check |
|---|---|---|
| SEMA-0001 | TypeMismatch | Assignment, argument, return type compatibility |
| SEMA-0002 | UndefinedSymbol | Identifier lookup failure |
| SEMA-0003 | DuplicateDeclaration | Re-declaration in same scope |
| SEMA-0004 | MutationOfImmutable | Writing to non-`mut` variable |
| SEMA-0005 | ReassignmentOfConstant | Writing to `::` constant |
| SEMA-0006 | InvalidReference | `&mut` on immutable value |
| SEMA-0007 | NonBoolCondition | Non-bool in `if`/`loop` condition |
| SEMA-0008 | LoopControlFlowError | `stop`/`next` outside loop |
| SEMA-0009 | ReturnTypeMismatch | Return expression ≠ declared return type |
| SEMA-0010 | AllocatorRequired | Heap collection without allocator |
| SEMA-0011 | UnimplementedBehave | Struct missing behaved methods |
| SEMA-0012 | IncompatibleBinaryOp | Wrong types for `+`, `-`, `==`, etc.; unhandled error union |
| SEMA-0013 | IncompatibleUnaryOp | Wrong types for `-`, `!`, `~` |
| SEMA-0014 | NonCallable | Calling non-function value; unreachable code after ret |
| SEMA-0015 | FieldAccessError | Missing/private field access; missing struct fields |
| SEMA-0016 | MatchNotExhaustive | Missing `_` wildcard in match |
| SEMA-0017 | InvalidBuiltinCall | Wrong arg count/type for `@` builtins |
| SEMA-0018 | DereferenceError | `.*` on non-pointer type |
| SEMA-0019 | CaptureMismatch | `\|x\|` on non-optional type |
| SEMA-0020 | GenericParamMismatch | ✅ Generic function called without inferrable type params |

#### 5.1.4 Built-in Validation
All 38 builtins from the spec are validated for argument count:
- Reflection: `@TypeOf`, `@SizeOf`, `@AlignOf`, `@TypeName`, `@EnumCount`, `@Fields`
- Casting: `@as`, `@bitCast`, `@ptrCast`, `@intToPtr`, `@ptrToInt`
- Memory: `@memcpy`, `@memset`, `@memmove`, `@pageAlloc`, `@pageFree`, `@comptimeDefaultAllocator`
- Comptime: `@comptime`, `@compileLog`, `@compileError`, `@embedFile`
- Control: `@panic`, `@breakpoint`, `@trap`, `@sysCall`
- Materialization: `@str.from_raw`, `@vec`, `@map`, `@set`
- Math: `@addWithOverflow`, `@subWithOverflow`, `@mulWithOverflow`
- Bitwise: `@ctz`, `@clz`, `@popCount`, `@bswap`
- Concurrency: `@atomicLoad`, `@atomicStore`, `@cmpxchg`

#### 5.1.5 Control Flow Analysis ✅ (S-SEMA-02)
- `reached_end` tracking in TypeChecker: after `ret`, `stop`, or noret expression (`@panic`, `@trap`, `@breakpoint`, `@compileError`), marks path as terminated
- Unreachable statement detection: subsequent statements after termination emit SEMA-0014
- If-else branch analysis: if both branches exit (ret/stop/noret), the if-else propagates exit status; if without else never propagates (else path falls through)
- Loop body analysis: loops reset exit state (loop may not execute)
- Match arm analysis: all arms must exit (including wildcard) for match to propagate exit
- Non-void function verification: emits SEMA-0009 if function has non-void return type but `reached_end` is false after body

#### 5.1.6 Function Return Type Inference ✅ (S-SEMA-03)
- When `current_return_type` is `None` (no declared return type), the first `ret expr` infers the return type
- Subsequent `ret` expressions are checked for compatibility with the inferred type
- Emits SEMA-0009 on inference inconsistency

#### 5.1.7 Variable Type Inference ✅ (S-SEMA-04)
- Variable declarations with inferred type (`name := expr`) resolve the type from the expression's return type
- Explicit types are checked against inferred types for compatibility
- Overwrites `Void` placeholder with inferred concrete type

#### 5.1.8 Struct Literal Validation ✅ (S-SEMA-11)
- Field existence checked (existing behavior)
- **New:** Field value type matched against declared field type — emits SEMA-0001 on mismatch
- Required fields checked — missing fields emit SEMA-0015

#### 5.1.9 Error Union Propagation ✅ (S-SEMA-12)
- When an expression statement discards an `ErrorUnion` result, emits SEMA-0012 warning
- Catch expressions properly unwrap error unions

### 5.2 Remaining Semantic Analysis Work — 🟠 Priority 2 Items

#### S-SEMA-01: SEMA-0020 — Generic parameter mismatch
- **Description:** When calling a generic function/type, verify that the provided type arguments match the expected count and constraints.
- **Files affected:** `src/sema/checker.rs:697-717`
- **Dependencies:** Parser generics support (already ✅)
- **Status:** ✅ Complete — checks that generic functions with zero regular params can't have generics inferred; emits SEMA-0020

#### S-SEMA-02: Control flow analysis for return/stop/next
- **Description:** Verify that all paths through a function return a value (non-`noret`). Detect unreachable code after `ret`, `stop`, `@panic`. Warn on unused expressions.
- **Files affected:** `src/sema/checker.rs`
- **Dependencies:** Existing sema infrastructure
- **Status:** ✅ Complete — `reached_end` tracking, unreachable code detection, if-else/match branch exit propagation

#### S-SEMA-03: Type inference for function return types
- **Description:** When a function has no declared return type (`fn() { ret 42 }`), infer it from the return expressions.
- **Files affected:** `src/sema/mod.rs`, `src/sema/checker.rs`
- **Dependencies:** S-SEMA-02
- **Status:** ✅ Complete — infers from first `ret expr`, checks consistency on subsequent rets

#### S-SEMA-04: Type inference for variable declarations
- **Description:** When `x := some_fnexpr()`, infer the type of `x` from the function's return type instead of defaulting to `void` then overwriting.
- **Files affected:** `src/sema/checker.rs:70-89`
- **Dependencies:** None
- **Status:** ✅ Complete — resolves type from expression, overwrites Void placeholder

#### S-SEMA-05: Compile-time evaluation (comptime)
- **Description:** Implement actual evaluation of `::` constants at compile time.
- **Estimate:** 2-3 weeks
- **Files affected:** New file `src/sema/comptime.rs`, `src/sema/mod.rs`
- **Dependencies:** S-SEMA-03, S-SEMA-04
- **Status:** ❌ Not started

#### S-SEMA-06: Module resolution and cross-file analysis
- **Description:** Resolve `use std.mem.allocator` paths to actual files/modules.
- **Estimate:** 1-2 weeks
- **Files affected:** `src/sema/mod.rs`, `src/cmd/mod.rs`
- **Dependencies:** CLI multi-file support (C-CLI-02)
- **Status:** ❌ Not started

#### S-SEMA-07: Import resolution for `use` statements
- **Description:** When a `use path.to.symbol` is encountered, resolve it to the actual declaration.
- **Estimate:** 1 week
- **Files affected:** `src/sema/mod.rs`, `src/sema/scope.rs`
- **Dependencies:** S-SEMA-06
- **Status:** ❌ Not started

#### S-SEMA-08: Method resolution and dispatch
- **Description:** When `obj.method()` is called, resolve the method from the type's method table.
- **Estimate:** 5-7 days
- **Files affected:** `src/sema/checker.rs`
- **Dependencies:** S-SEMA-01
- **Status:** ❌ Not started

#### S-SEMA-09: Reference lifetime and borrow checking
- **Description:** Verify that `&T` / `&mut T` references do not outlive their source.
- **Estimate:** 2-3 weeks
- **Files affected:** New file `src/sema/borrow.rs`, `src/sema/checker.rs`
- **Dependencies:** S-SEMA-03
- **Status:** ❌ Not started

#### S-SEMA-10: Generic type instantiation / monomorphization
- **Description:** When `Vec<i32>` or `identity<f64>` is used, verify generic constraints.
- **Estimate:** 2-3 weeks
- **Files affected:** `src/sema/types.rs`, `src/sema/checker.rs`
- **Dependencies:** S-SEMA-01
- **Status:** ❌ Not started

#### S-SEMA-11: Struct literal validation
- **Description:** Verify struct initialization provides all required fields with matching types.
- **Files affected:** `src/sema/checker.rs:1112-1160`
- **Dependencies:** None
- **Status:** ✅ Complete — field existence, missing fields, and field type matching validated

#### S-SEMA-12: Error union propagation analysis
- **Description:** Verify that `!T` return types are properly handled.
- **Files affected:** `src/sema/checker.rs`
- **Dependencies:** S-SEMA-02
- **Status:** ✅ Complete — discarded error union results flagged with SEMA-0012

#### S-SEMA-13: Semantic test suite
- **Description:** Build a comprehensive test suite for semantic analysis.
- **Files affected:** New file `src/sema/tests.rs`
- **Dependencies:** All sema features above
- **Status:** ✅ Complete — 26 tests covering all 20 SEMA checks plus inference and structural validation

---

## 6. Phase 4: CLI / Driver — Priority 3

**Status: 🟡 ~80% — Functional but incomplete.**

### 6.1 Completed Work ✅
- File reading with error handling
- Lexer → Parser → Sema pipeline orchestration
- Verbose output mode (`-v`) with token dump and AST tree
- Error display for lexer, parser, and sema errors
- Phase status reporting

### 6.2 Remaining CLI Work — 🟡 Priority 3 Items

#### C-CLI-01: Compiler output and build artifacts
- **Description:** Generate output files (object files, executables). Currently the compiler never writes anything to disk.
- **Estimate:** Depends on LLVM backend — 1-2 days for wiring if LLVM is ready
- **Files affected:** `src/cmd/mod.rs`
- **Dependencies:** LLVM Codegen (Phase 5)
- **Status:** ❌ Not started

#### C-CLI-02: Multi-file compilation
- **Description:** Process multiple input files together, resolving symbols and types across file boundaries. Build a module dependency graph.
- **Estimate:** 1 week
- **Files affected:** `src/cmd/mod.rs`, `src/sema/mod.rs`
- **Dependencies:** S-SEMA-06
- **Status:** ❌ Not started

#### C-CLI-03: `razenc test` subcommand
- **Description:** The syntax spec describes `razenc test file.rzn` to run `test {}` blocks. Implement test discovery, execution, and reporting.
- **Estimate:** 1 week
- **Files affected:** `src/cmd/mod.rs`, `src/sema/mod.rs`
- **Dependencies:** LLVM Codegen or a simple AST interpreter
- **Status:** ❌ Not started

#### C-CLI-04: `razenc build` / `razenc run` subcommands
- **Description:** Build a single executable from source; run it immediately. Standard compiler UX.
- **Estimate:** 2-3 days (after codegen exists)
- **Files affected:** `src/cmd/mod.rs`
- **Dependencies:** C-CLI-01
- **Status:** ❌ Not started

#### C-CLI-05: Source location in diagnostics
- **Description:** Show `--> filename.rzn:10:5` style error locations with source line snippets. Currently only line numbers are printed.
- **Estimate:** 2-3 days
- **Files affected:** `src/cmd/mod.rs`, `src/bdg/mod.rs`
- **Dependencies:** None
- **Status:** ❌ Not started

#### C-CLI-06: Color configuration and `--no-color` flag
- **Description:** Allow disabling ANSI color output for non-terminal environments (CI, logs, pipe).
- **Estimate:** 1 day
- **Files affected:** `src/bdg/mod.rs`, `src/cmd/mod.rs`
- **Dependencies:** None
- **Status:** ❌ Not started

#### C-CLI-07: Warning infrastructure
- **Description:** Add compiler warning levels (`--warn-all`, `--warn-as-errors`). The casing convention warnings (snake_case, PascalCase) are specified in `syntax.md` but never emitted.
- **Estimate:** 3-4 days
- **Files affected:** `src/cmd/mod.rs`, `src/sema/mod.rs`
- **Dependencies:** None
- **Status:** ❌ Not started

#### C-CLI-08: File watching (`--watch` flag)
- **Description:** Watch source files for changes and recompile automatically. Developer productivity feature.
- **Estimate:** 2-3 days
- **Files affected:** `src/cmd/mod.rs`
- **Dependencies:** C-CLI-01
- **Status:** ❌ Not started

#### C-CLI-09: `razenc init` project scaffolding
- **Description:** Generate a basic Razen project structure (`src/main.rzn`, `razenc.json`).
- **Estimate:** 1-2 days
- **Files affected:** `src/cmd/mod.rs`
- **Dependencies:** None
- **Status:** ❌ Not started

---

## 7. Phase 5: LLVM Codegen — Priority 4

**Status: 🔴 ~0% — Nothing implemented.**

This is the largest single body of work in the compiler. The LLVM codegen takes the semantically-annotated AST and produces LLVM IR, which is then compiled to native code.

### 7.1 LLVM Codegen Architecture

```
Annotated AST (Program)
       │
       ▼
┌──────────────────┐
│  LLVM Codegen     │
│  ┌──────────────┐ │
│  │ Type Lowering │ │  AST Type → LLVM Type
│  └──────────────┘ │
│  ┌──────────────┐ │
│  │ Expr Codegen  │ │  Expr → LLVM Value
│  └──────────────┘ │
│  ┌──────────────┐ │
│  │ Stmt Codegen  │ │  Stmt → LLVM BasicBlock
│  └──────────────┘ │
│  ┌──────────────┐ │
│  │ Decl Codegen  │ │  Decl → LLVM Function/Global
│  └──────────────┘ │
└──────────────────┘
       │
       ▼
   LLVM IR (via `inkwell` crate)
       │
       ▼
   Object Code / Executable
```

### 7.2 Planned Items (in order)

#### L-LLVM-01: LLVM dependency setup (`inkwell`)
- **Description:** Add `inkwell` (safe Rust bindings for LLVM) to `Cargo.toml`. Set up the LLVM context, module, builder, and execution engine.
- **Estimate:** 2-3 days
- **Files affected:** `Cargo.toml`, `src/llvm/mod.rs`
- **Status:** ❌ Not started

#### L-LLVM-02: Type lowering
- **Description:** Convert Razen TypeInfo to LLVM types:
  - `i1`-`i128`, `u1`-`u128` → LLVM integer types
  - `f32`, `f64` → LLVM float types
  - `bool` → `i1`
  - `&T`, `*T` → LLVM pointer types
  - `?T` (optional) → struct `{ T, bool }`
  - `E!T` (error union) → struct `{ T, ErrorCode }`
  - `[T; N]` → LLVM array type
  - `struct { .. }` → LLVM struct type
  - `fn(...) -> ...` → LLVM function type
  - `@vec[T]`, `@str` → pointer to heap-allocated struct
- **Estimate:** 1 week
- **Files affected:** `src/llvm/mod.rs` (or new `src/llvm/types.rs`)
- **Dependencies:** L-LLVM-01
- **Status:** ❌ Not started

#### L-LLVM-03: Literal codegen
- **Description:** Emit LLVM IR for literal values:
  - Integer literals → `LLVMConstInt`
  - Float literals → `LLVMConstReal`
  - String literals → global constant `[u8; N]` with null terminator
  - Bool literals → `LLVMConstInt` of `i1`
  - Char literals → `LLVMConstInt` of `i32`
  - Nil → null pointer or zero struct
- **Estimate:** 2-3 days
- **Files affected:** `src/llvm/mod.rs` (or new `src/llvm/expr.rs`)
- **Dependencies:** L-LLVM-02
- **Status:** ❌ Not started

#### L-LLVM-04: Expression codegen
- **Description:** Emit LLVM IR for all expression types:
  - `Ident` → load from alloca or global
  - `Binary` → arithmetic, comparison, logical, bitwise, range
  - `Unary` → neg, not, bitnot, ref (addr), deref (load), optional wrap
  - `Call` → `LLVMBuildCall` with argument marshalling
  - `Field` → `LLVMBuildStructGEP` + load
  - `Index` → `LLVMBuildGEP` + load
  - `Slice` → struct `{ ptr, len }`
  - `StructInit` → `LLVMBuildStructGEP` + store each field
  - `Deref (.*)` → `LLVMBuildLoad`
  - `Block` → sequential IR with scoped allocas
  - `Paren` → transparent
  - `Catch` → if-else with error union unwrap
  - `Ret` → `LLVMBuildRet`
  - `Fn` (anonymous) → internal function + closure
  - `AtMethod` → builtin function lowering
- **Estimate:** 3-4 weeks
- **Files affected:** `src/llvm/expr.rs`
- **Dependencies:** L-LLVM-03
- **Status:** ❌ Not started

#### L-LLVM-05: Statement codegen
- **Description:** Emit LLVM IR for all statement types:
  - `Var` → alloca + store (or optimized SSA)
  - `Assign` → store to alloca/gep
  - `If` → conditional branch with `then`/`else` blocks
  - `Match` → switch instruction with case dispatch
  - `Loop` → `br` to header block with condition check
  - `Stop` → `br` to loop exit block
  - `Next` → `br` to loop header block
  - `Defer` → push cleanup onto scope stack; emit at block exit
  - `TryCatch` → if-else with error union unwrap
  - `Block` → scoped alloca region
  - `Expr` → expression + discard
- **Estimate:** 3-4 weeks
- **Files affected:** `src/llvm/stmt.rs`
- **Dependencies:** L-LLVM-04
- **Status:** ❌ Not started

#### L-LLVM-06: Declaration codegen
- **Description:** Emit LLVM IR for all declaration types:
  - `Fn` → LLVM function definition with params and body
  - `Struct` → LLVM struct type (only generated when instantiated)
  - `Union` → LLVM struct with tag + largest variant
  - `Enum` → LLVM integer type (tag) + associated data struct
  - `Error_` → LLVM integer enum for error codes
  - `Var` / `Const` → global variable or constant
  - `TypeAlias` → type aliasing (IR-level no-op)
  - `Test` → test harness function
  - `Use` / `Mod` → module-level scope organization
- **Estimate:** 2-3 weeks
- **Files affected:** `src/llvm/decl.rs`
- **Dependencies:** L-LLVM-05
- **Status:** ❌ Not started

#### L-LLVM-07: Builtin function lowering
- **Description:** Implement each `@builtin` as LLVM intrinsic calls or inline IR:
  - `@memcpy` → `llvm.memcpy` intrinsic
  - `@memset` → `llvm.memset` intrinsic
  - `@memmove` → `llvm.memmove` intrinsic
  - `@pageAlloc` / `@pageFree` → `mmap`/`munmap` syscall
  - `@panic` → `abort()` or `printf` + `abort()`
  - `@ctz` / `@clz` / `@popCount` / `@bswap` → LLVM intrinsics
  - `@addWithOverflow` etc. → `llvm.sadd.with.overflow` etc.
  - `@atomicLoad` etc. → LLVM atomic instructions
  - `@sysCall` → inline assembly `syscall` instruction
  - Reflection builtins (`@SizeOf`, etc.) → comptime evaluation
- **Estimate:** 2-3 weeks
- **Files affected:** `src/llvm/builtins.rs`
- **Dependencies:** L-LLVM-04
- **Status:** ❌ Not started

#### L-LLVM-08: Memory management and allocator integration
- **Description:** Generate code for heap allocation:
  - `@Allocator` interface as vtable struct
  - Concrete allocator implementations via LLVM globals
  - `alloc()`, `realloc()`, `free()` calls
  - `@vec`, `@str`, `@map`, `@set` heap operations
  - Stack-to-heap escape analysis for dynamic lifetimes
- **Estimate:** 3-4 weeks
- **Files affected:** `src/llvm/memory.rs`, `src/llvm/builtins.rs`
- **Dependencies:** L-LLVM-07
- **Status:** ❌ Not started

#### L-LLVM-09: Comptime evaluation in codegen
- **Description:** The codegen must handle `::` constants by evaluating them at compile time. This may require:
  - An interpreter for comptime-known expressions
  - Folding constant expressions to LLVM constants
  - Emitting `@comptime` blocks that run during codegen
- **Estimate:** 2-3 weeks
- **Files affected:** `src/llvm/comptime.rs`
- **Dependencies:** L-LLVM-06, S-SEMA-05
- **Status:** ❌ Not started

#### L-LLVM-10: Optimization passes
- **Description:** Run LLVM optimization passes:
  - `-O0`: No optimization (debug)
  - `-O1`: Basic optimizations
  - `-O2`: Standard optimizations (default)
  - `-O3`: Aggressive optimizations
  - `-Os`: Optimize for size
  - `-Oz`: Aggressively optimize for size
- **Estimate:** 3-5 days
- **Files affected:** `src/llvm/mod.rs`
- **Dependencies:** L-LLVM-09
- **Status:** ❌ Not started

#### L-LLVM-11: Debug information (DWARF)
- **Description:** Emit DWARF debug metadata so that `gdb`/`lldb` can debug Razen programs:
  - Source locations on each instruction
  - Variable names and types
  - Function boundaries
  - Line tables
- **Estimate:** 2-3 weeks
- **Files affected:** `src/llvm/debug.rs`
- **Dependencies:** L-LLVM-06
- **Status:** ❌ Not started

#### L-LLVM-12: Object file emission
- **Description:** Write the compiled LLVM module to an object file (`.o`), then link it using the system linker (`cc`/`ld`) to produce an executable.
- **Estimate:** 1 week
- **Files affected:** `src/llvm/mod.rs`, `src/cmd/mod.rs`
- **Dependencies:** L-LLVM-10
- **Status:** ❌ Not started

#### L-LLVM-13: LLVM codegen test suite
- **Description:** Create integration tests that compile Razen programs and verify they produce correct output (exit code, stdout).
- **Estimate:** 2-3 weeks (ongoing)
- **Files affected:** `tests/` directory, `src/llvm/tests.rs`
- **Dependencies:** L-LLVM-12
- **Status:** ❌ Not started

---

## 8. Phase 6: Standard Library — Priority 5

**Status: 🔴 ~0% — Nothing implemented.**

### 8.1 Standard Library Design

The standard library (`std/`) is written in Razen itself and ships with the compiler. It provides:

| Module | Contents | Priority |
|---|---|---|
| `std.mem` | Allocators, memory operations | P0 |
| `std.testing` | `assert`, test utilities | P0 |
| `std.os` | File I/O, processes | P1 |
| `std.collections` | Vec, Map, Set implementations | P1 |
| `std.str` | String manipulation | P1 |
| `std.math` | Math constants, operations | P2 |
| `std.time` | Time, sleep, timers | P2 |
| `std.net` | Networking sockets | P3 |
| `std.json` | JSON parsing/serialization | P3 |
| `std.fmt` | Formatting, printing | P1 |

### 8.2 Planned Items

#### STD-01: `std.mem.PageAllocator`
- **Description:** Wraps `@pageAlloc`/`@pageFree` for direct OS page allocation. Implements `@Allocator` interface.
- **Estimate:** 3-5 days
- **Files:** `std/mem/page_allocator.rzn`
- **Status:** ❌ Not started

#### STD-02: `std.mem.ArenaAllocator`
- **Description:** Bump-allocator backed by pages. Bulk-deallocate entire arena via `deinit()`. Ideal for compilers and parsers.
- **Estimate:** 3-5 days
- **Files:** `std/mem/arena_allocator.rzn`
- **Status:** ❌ Not started

#### STD-03: `std.mem.GeneralPurposeAllocator`
- **Description:** Production allocator with leak detection, fragmentation handling, and thread safety.
- **Estimate:** 1-2 weeks
- **Files:** `std/mem/gpa.rzn`
- **Status:** ❌ Not started

#### STD-04: `std.mem.FixedBufferAllocator`
- **Description:** Stack-allocated `[u8; N]` turned into an allocator. Zero OS calls.
- **Estimate:** 2-3 days
- **Files:** `std/mem/fixed_buffer_allocator.rzn`
- **Status:** ❌ Not started

#### STD-05: `std.testing` module
- **Description:** `assert(actual, expected)`, `expect(condition)`, test runner infrastructure integrated with `test {}` blocks.
- **Estimate:** 1 week
- **Files:** `std/testing.rzn`
- **Status:** ❌ Not started

#### STD-06: `std.collections.Vec` (dynamic array)
- **Description:** Generic `Vec(T)` implementation with `append`, `pop`, `insert`, `remove`, `len`, `reserve`.
- **Estimate:** 1 week
- **Files:** `std/collections/vec.rzn`
- **Status:** ❌ Not started

#### STD-07: `std.str` string utilities
- **Description:** `len`, `substring`, `find`, `replace`, `split`, `trim`, `to_upper`, `to_lower`, `contains`.
- **Estimate:** 1 week
- **Files:** `std/str.rzn`
- **Status:** ❌ Not started

#### STD-08: `std.os` file I/O
- **Description:** `open`, `read`, `write`, `close`, `seek`, `stat` wrapping `@sysCall`.
- **Estimate:** 2 weeks
- **Files:** `std/os.rzn`
- **Status:** ❌ Not started

---

## 9. Phase 7: Tooling & Infrastructure — Priority 6

**Status: ⚪ ~10% — Early stage.**

### 9.1 Planned Items

#### T-TOOL-01: Razen Language Server (razenc-ls)
- **Description:** LSP-compatible language server providing:
  - Go-to-definition
  - Hover type information
  - Autocomplete
  - Diagnostic reporting (live errors)
  - Document symbols
- **Estimate:** 4-6 weeks
- **Dependencies:** Complete sema, module resolution
- **Status:** ❌ Not started

#### T-TOOL-02: Code formatter (razenc-fmt)
- **Description:** Opinionated code formatter following Razen style conventions from `syntax.md`.
- **Estimate:** 2-3 weeks
- **Dependencies:** Parser (needs lossless syntax tree or token-aware formatting)
- **Status:** ❌ Not started

#### T-TOOL-03: Package manager (razenc-pkg)
- **Description:** Simple package manager:
  - `razenc pkg add <url>` — add dependency
  - `razenc pkg build` — build all dependencies
  - `razenc.json` manifest format
- **Estimate:** 3-4 weeks
- **Dependencies:** Module resolution, build system
- **Status:** ❌ Not started

#### T-TOOL-04: CI/CD integration
- **Description:** GitHub Actions for:
  - Running `cargo test` on every PR
  - Building release binaries for Linux/macOS/Windows
  - Benchmarking compilation speed
  - Fuzz testing the parser
- **Estimate:** 1 week
- **Dependencies:** None
- **Status:** ⚠️ Partial — basic Rust CI exists via Cargo

#### T-TOOL-05: Fuzz testing
- **Description:** Use `cargo-fuzz` or `afl` to fuzz the lexer and parser with random byte sequences. Catch panics and infinite loops.
- **Estimate:** 1 week setup, ongoing
- **Dependencies:** None
- **Status:** ❌ Not started

#### T-TOOL-06: Benchmark suite
- **Description:** Track compilation speed and memory usage over time. Compare against previous versions.
- **Estimate:** 2-3 days
- **Dependencies:** C-CLI-01
- **Status:** ❌ Not started

---

## 10. Risk Register

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | LLVM API breaking changes | Medium | High | Pin LLVM version; use `inkwell` which abstracts minor changes |
| 2 | Self-hosting complexity | High | High | Target self-hosting as a stretch goal; keep Rust as bootstrap compiler |
| 3 | Razen language design instability | Medium | Medium | Freeze syntax spec (already done); only add features through RFC process |
| 4 | Parser edge cases in real-world code | Medium | Low | Extensive fuzz testing; collect issue reports from early users |
| 5 | Codegen performance issues | Low | Medium | Leverage LLVM optimization passes; measure and profile early |
| 6 | Module system design incomplete | Medium | High | Prototype module resolution early; keep it simple initially |

---

## 11. Milestone Roadmap

### Milestone 1: "Solid Foundation" (Current — Q2 2026)
- ~~Lexer complete~~ ✅
- ~~Parser complete (100%)~~ ✅
- ~~Basic semantic analysis (70%)~~ ✅
- ~~All priority-1 parser gaps closed~~ ✅
- **Deliverable:** CLI that lexes, parses, and type-checks any valid `.rzn` file

### Milestone 2: "Type Safe" (Q3 2026)
- Semantic analysis reaches 100%
- All 20 SEMA errors implemented
- Module resolution works
- Generic type checking complete
- **Deliverable:** Compiler catches all semantic errors with precise diagnostics

### Milestone 3: "Hello, World!" (Q4 2026)
- LLVM codegen produces valid IR
- Basic function calls, arithmetic, and I/O work
- `razenc build` produces an executable
- **Deliverable:** `razenc build hello.rzn` produces a working binary

### Milestone 4: "Self Hosting Prep" (Q1 2027)
- Comptime evaluation works
- Standard library (mem allocators, basic collections)
- `razenc test` works
- Compiler can compile non-trivial Razen programs
- **Deliverable:** Compiler can compile the standard library

### Milestone 5: "Self Hosting" (Q2-Q3 2027)
- `razenc` compiles its own source code
- Full standard library
- Package manager
- Testing framework
- **Deliverable:** Bootstrap complete — Razen running on Razen

### Milestone 6: "Production Ready" (Q4 2027+)
- Language server (LSP)
- Code formatter
- Debug info (DWARF)
- Performance optimization
- Cross-compilation
- **Deliverable:** Production-ready compiler with tooling ecosystem

---

## Appendix A: Code Quality Standards

- **Testing:** All new code must have tests. Parser/Sema changes need unit tests. Codegen changes need integration tests.
- **Documentation:** Public API functions need doc comments. Non-trivial algorithms need inline comments.
- **Error Messages:** Every error message must include: error code, human-readable message, source location (file, line, column).
- **Performance:** No O(n²) algorithms in hot paths. Profile before optimizing.
- **Style:** Follow Rust standard formatting (`cargo fmt`). No warnings (`cargo clippy`).

## Appendix B: File Dependency Graph

```
syntax/*.md (specification documents)
       │
       ▼
src/lexer/token.rs (token definitions)
       │
       ▼
src/lexer/lexer.rs (tokenizer)
       │
       ▼
src/ast/mod.rs (AST node definitions)
       │
       ▼
src/parser/mod.rs (token → AST)
       │
       ▼
src/sema/types.rs (semantic types)
src/sema/scope.rs (symbol table)
src/sema/checker.rs (type checking)
src/sema/mod.rs (orchestrator)
       │
       ▼
src/llvm/mod.rs (AST → LLVM IR)
       │
       ▼
src/cmd/mod.rs (CLI driver)
       │
       ▼
src/main.rs (entry point)
```

## Appendix C: Dependency Check — What Blocks What

| Work Item | Blocked By | Blocks |
|---|---|---|
| Parser error recovery | — | — |
| SEMA-02 (control flow) | — | SEMA-03, SEMA-12 |
| SEMA-05 (comptime eval) | SEMA-03, SEMA-04 | L-LLVM-09 |
| SEMA-06 (module resolution) | C-CLI-02 | SEMA-07 |
| SEMA-10 (monomorphization) | SEMA-01 | L-LLVM-06 |
| LLVM codegen | Parser ✅, Sema (partial) | CLI build, test, std lib |
| Standard library | LLVM codegen | Self-hosting |
| Self-hosting | Std lib, LLVM codegen | — |

---

*This plan is a living document. Update it as priorities shift and progress is made.*
