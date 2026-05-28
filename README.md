# razenc

The Razen programming language compiler. Written in Rust.

Razen is a statically typed, systems-level language built around four principles: **Meaningful, Accurate, Simple, Maximum Performance**. No hidden allocations. No garbage collector. No exceptions.

## Quick Start

```bash
# build
cargo build --release

# compile a .rzn file (lex + parse + type-check + IR gen)
./target/release/razenc -f hello.rzn

# verbose mode (tokens, AST, IR dump)
./target/release/razenc -v -f hello.rzn
```

## Language at a Glance

```rzn
use std.fmt

MAX_SIZE :: 1024                    // comptime constant
Age :: type(u8)                     // type alias

Point :: struct {                   // private fields by default
    pub x: f64,
    pub y: f64,
}

distance :: fn(a: Point, b: Point) -> f64 {
    dx := b.x - a.x
    dy := b.y - a.y
    ret (dx * dx + dy * dy) @as(f64)  // sqrt omitted for brevity
}

main :: fn() -> void {
    mut p := Point{ x: 0.0, y: 0.0 }
    p.x = 5.0                       // mut required for reassignment
    fmt.println("x is {}", p.x)
}
```

### Key Syntax

| Construct | Syntax | Notes |
|---|---|---|
| Variable | `x := expr` | Inferred type, runtime |
| Mutable | `mut x := expr` | Reassignable |
| Constant | `X :: expr` | Compile-time, immutable |
| Explicit type | `x : i32 = 42` | Runtime with type annotation |
| Explicit const | `X : i32 : 42` | Comptime with type annotation |
| Function | `name :: fn(p: T) -> R { ... }` | Comptime (default) |
| Variable fn | `name := fn(p: T) -> R { ... }` | Runtime, reassignable |
| Behaviour | `T :: struct ~> Trait { ... }` | Structural trait impl |

### Types

**Integers:** `i1`..`i128`, `u1`..`u128`, `isize`, `usize`
**Floats:** `f8`, `f16`, `f32`, `f64`, `f128`
**Special:** `bool`, `char`, `void`, `noret`, `anytype`, `nil`, `str`

```rzn
&T          // immutable reference
&mut T      // mutable reference
*T          // raw pointer
?T          // optional (value or nil)
E!T         // explicit error union
!T          // inferred error union
[T]         // slice (view into memory)
[T; N]      // fixed-size array
@vec[T]     // dynamic heap vector
@map{K, V}  // hash map
@set{T}     // hash set
```

### Control Flow

```rzn
// if/else (no parens needed)
if x > 0 { ... } else if x == 0 { ... } else { ... }

// optional capture
if maybe_value |x| {
    fmt.println("got: {}", x)
}

// match (wildcard _ is mandatory)
match token {
    Token.Identifier |name| => process(name),
    Token.Eof             => stop,
    _                     => @panic("unexpected"),
}

// single keyword for all loops
loop { ... }                    // infinite
loop condition { ... }          // while
loop array |i| { ... }         // for-each
loop a, b |i, j| { ... }      // lockstep

stop    // break
next    // continue
ret     // return
```

### Error Handling

```rzn
ParseError :: error { Invalid_Input, Overflow }

parse :: fn(input: str) -> ParseError!i32 {
    if bad(input) {
        ret ParseError.Invalid_Input
    }
    ret 42
}

// try/catch
try {
    val := parse(raw)
} catch |e| {
    match e {
        ParseError.Invalid_Input => recover(),
        _                        => @panic("unhandled"),
    }
}

// defer (runs on scope exit, always)
file := open("data.txt")
defer file.close()
```

### Behaviours (Traits)

```rzn
Describable :: behave {
    describe :: fn(self: &@Self) -> void   // abstract
}

Person :: struct ~> Describable {
    name: str,
    pub describe :: fn(self: &Person) -> void {
        fmt.println("Person: {}", self.name)
    }
}
```

### Builtins

40+ compiler builtins, all prefixed with `@`:

| Category | Examples |
|---|---|
| Reflection | `@TypeOf`, `@SizeOf`, `@AlignOf`, `@Fields`, `@EnumCount` |
| Casting | `@as`, `@bitCast`, `@ptrCast`, `@intToPtr`, `@ptrToInt` |
| Memory | `@memcpy`, `@memset`, `@pageAlloc`, `@Allocator` |
| Comptime | `@comptime`, `@compileLog`, `@compileError`, `@embedFile` |
| Control | `@panic`, `@trap`, `@breakpoint`, `@sysCall` |
| Math | `@addWithOverflow`, `@subWithOverflow`, `@mulWithOverflow` |
| Bitwise | `@ctz`, `@clz`, `@popCount`, `@bswap` |
| Concurrency | `@atomicLoad`, `@atomicStore`, `@cmpxchg` |

Full list: [syntax/builtins.md](syntax/builtins.md)

## Project Structure

```
razenc/
├── src/
│   ├── lexer/          # Tokenizer
│   │   ├── token.rs    # Token definitions (TokenKind enum)
│   │   └── lexer.rs    # Lexer implementation
│   ├── ast/            # Abstract syntax tree nodes
│   │   └── mod.rs      # Decl, Expr, Stmt, Type enums
│   ├── parser/         # Token → AST (Pratt parser)
│   │   ├── mod.rs      # Parser implementation
│   │   └── tests.rs    # Parser test suite
│   ├── sema/           # Semantic analysis
│   │   ├── types.rs    # TypeInfo enum and type utilities
│   │   ├── scope.rs    # Symbol table with scope chain
│   │   ├── checker.rs  # Type checker and error reporting
│   │   ├── mod.rs      # Semantic analyzer orchestrator
│   │   └── tests.rs    # Sema test suite
│   ├── ir/             # IR generation (three-address code)
│   │   └── mod.rs      # IR generator
│   ├── bdg/            # Debug/display utilities
│   │   └── mod.rs      # Colored output, AST/IR printing
│   ├── cmd/            # CLI driver
│   │   └── mod.rs      # clap-based argument parsing, pipeline
│   ├── llvm/           # LLVM codegen (placeholder)
│   │   └── mod.rs
│   ├── std/            # Standard library (placeholder)
│   │   └── mod.rs
│   └── main.rs         # Entry point
├── syntax/             # Language specification
│   ├── syntax.md       # Full syntax reference
│   ├── types.md        # Type system spec
│   ├── keywords.md     # Keywords and operators
│   ├── builtins.md     # Builtin functions and allocators
│   └── errors.md       # Semantic error codes
├── samples/            # Example programs
│   ├── sample_01_basics.rzn
│   ├── sample_02_functions.rzn
│   ├── sample_03_control_flow.rzn
│   ├── sample_04_data_types.rzn
│   └── sample_05_builtins.rzn
├── report/             # Audit reports
└── demo.rzn            # Full feature demo
```

## Compiler Pipeline

```
source.rzn → Lexer → Tokens → Parser → AST → Sema → Annotated AST → IR → (LLVM)
```

| Phase | Module | Input | Output | Status |
|---|---|---|---|---|
| Lexing | `src/lexer/` | Source text | `Vec<Token>` | Done |
| Parsing | `src/parser/` + `src/ast/` | Tokens | `Program` AST | Done |
| Semantic Analysis | `src/sema/` | AST | Typed AST | ~90% |
| IR Generation | `src/ir/` | Typed AST | Three-address IR | Working |
| LLVM Codegen | `src/llvm/` | IR | Native code | Not started |

**217 tests passing.**

## Tests

```bash
cargo test
```

Tests cover lexer (75), parser (52), and semantic analysis (26). Each module has its own test file.

## Current Limitations

- No LLVM codegen yet — the compiler lexes, parses, type-checks, and generates IR, but does not emit native code
- Module resolution (`use std.fmt`) is not implemented — imports are parsed but not resolved
- Comptime evaluation is not implemented — `::` constants are parsed and type-checked but not evaluated
- Reference lifetime checking is not implemented
- Generic type instantiation / monomorphization is not implemented

## License

Not yet specified.
