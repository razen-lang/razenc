# Razen Language Syntax Specification (`syntax.md`)

This document defines the complete, official syntax for the Razen programming language. Razen is designed around four core principles: **Meaningful, Accurate, Simple, and Maximum Performance**.

---

## Keywords & Symbols

### Keywords

* **Control Flow**: `if`, `else`, `match`, `loop`, `stop`, `next`, `ret`
* **Memory & Error**: `mut`, `try`, `catch`, `defer`
* **Modules & Visibility**: `mod`, `use`, `pub`, `ext`
* **Definitions**: `fn`, `struct`, `union`, `enum`, `error`, `behave`, `type`, `test`
* **Literals**: `true`, `false`, `nil`

### Core Sigils

* `:=` : Inferred variable assignment (Runtime)
* `::` : Inferred constant assignment (Comptime)
* `: type =` : Explicit type variable assignment (Runtime)
* `: type :` : Explicit type constant assignment (Comptime)
* `~>` : Implements behaviour/trait
* `->` : Function return type pointer
* `@`  : Builtin compiler macro/type prefix (see [Builtins](#builtins) for the full list)
* `?`  : Optional/Nullable type prefix (e.g., `?i32`)
* `!`  : Error union separator (e.g., `ErrorUnionName!i32`)
* `&`  : Reference sigil — `&T` immutable, `&mut T` mutable
* `.*` : Explicit postfix dereference operator (e.g., `ptr.* = value`)

### Operators

* **Math**: `+`, `-`, `*`, `/`, `%`, `+=`, `-=`, `*=`, `/=`, `%=`
* **Logic**: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`
* **Bitwise**: `&`, `|`, `^`, `~`, `<<`, `>>`
* **Range**: `..` (exclusive), `..=` (inclusive) — used in slices and loops
* **Access & Scoping**: `.`, `_`, `|`

---

## Typing Cases

Razen uses distinct casing styles to clarify code structure at a glance. These styles are recommended but not strictly enforced; violating them results in a harmless compiler warning.

* `snake_case` : Functions, variables, parameters, module names.
* `PascalCase` : Struct, Union, Error, Enum, Behave names.
* `SCREAMING_SNAKE_CASE` : Comptime constants.
* `Spine_Case` : Fields inside errors, structs, unions, or enums (Optional).

---

## Types

### Integer Types

* **Fixed Bit-Width**: `i1`, `i2`, `i4`, `i8`, `i16`, `i32`, `i64`, `i128`
* **Unsigned Fixed Bit-Width**: `u1`, `u2`, `u4`, `u8`, `u16`, `u32`, `u64`, `u128`
* **Architecture Sized**: `isize`, `usize`
* **Defaults**: `int` (alias for `i32`), `uint` (alias for `u32`)

### Float Types

* **Standard Widths**: `f8`, `f16`, `f32`, `f64`, `f128`
* **Default**: `float` (alias for `f32`)

### Pointer & Reference Types

* `&T`     : Immutable reference — safe read-only borrow
* `&mut T` : Mutable reference — safe read-write borrow
* `*T`     : Raw pointer — unsafe, direct memory / pointer arithmetic

### Special Types

* `bool`    : Boolean value (`true` or `false`)
* `char`    : Unicode character literal (code point)
* `void`    : Empty value / no return value
* `noret`   : Function never returns (diverging — e.g. infinite loops, `@panic`)
* `anytype` : Compile-time duck typing wildcard — type is inferred at comptime
* `nil`     : The absence of a value (used with `?T` optional types)
* `fn(...)` : Function signature as a first-class type
* `type(T)` : Type meta-representation / alias context

### Strings

* `str`  : Stack-allocated or static read-only string slice
* `@str` : Heap-allocated dynamic string

### Arrays & Slices

* `[T]`    : Array Slice — a runtime view into contiguous memory (non-owning)
* `[T; N]` : Fixed Array — stack-allocated array of exactly `N` elements

### Heap Collections

* `@vec[T]`    : Dynamic heap vector
* `@vec[T; N]` : Fixed-capacity heap vector
* `@map{K, V}` : Hash map — unique keys `K` to values `V`
* `@set{T}`    : Mathematical unique set
* `@set{T; N}` : Sized unique set with static capacity `N`

### Data Types

* `struct{}`  : Encapsulated custom data structure (fields private by default)
* `enum{}`    : Distinct named choices
* `union{}`   : Shared-memory tag variant container
* `error{}`   : Explicit error context definition
* `behave{}`  : Structural behavioural trait contract
* `test{}`    : Compiler-native isolated test runner block

### Error Handling & Optionals

* `?T`  : Optional wrapper — value of type `T` or `nil`
* `!T`  : Inferred implicit global error union wrapping type `T`
* `E!T` : Explicit error union `E` wrapping return type `T`

---

## Variable Declaration

### Immutable Variables

Immutable values are assigned at runtime and cannot be altered after initialization.

```rzn
// Inferred type
variable_name := value

// Explicit type
variable_name : type = value
```

### Mutable Variables

Mutable values can change their underlying data at runtime via the `mut` keyword.

```rzn
// Inferred type
mut variable_name := value

// Explicit type
mut variable_name : type = value
```

---

## Constant Declaration

Constants are entirely computed at compile time (`comptime`).

### Comptime Constants

```rzn
// Inferred type
CONSTANT_NAME :: value

// Explicit type
CONSTANT_NAME : type : value
```

### Comptime Variables (Stateful Comptime)

Stateful configurations evaluated during compilation — can change value during the compilation process itself.

```rzn
// Inferred type
mut COMPTIME_VAR :: value

// Explicit type
mut COMPTIME_VAR : type : value
```

---

## Function Declaration

### Constant Functions

All primary functions are constant and value-bound by default. They are evaluated at compile time (`comptime`). Constant functions support generics via `<>`.

```rzn
// Standard constant function
function_name :: fn(parameter: type, ...) -> type {
    ret value
}

// Generic constant function
function_name<T, ...> :: fn(parameter: T, ...) -> T {
    ret value
}

// Mutable copy parameter (modifies only the local stack copy)
function_name :: fn(mut parameter: type) -> type {
    parameter = new_value
    ret parameter
}
```

### Variable Functions

Variable functions are evaluated at runtime. They can change their assigned functional body but **do not** support generics.

```rzn
// Value parameter
function_name := fn(parameter: type) -> type {...}

// Immutable reference parameter
function_name := fn(parameter: &type) -> type {...}

// Mutable value parameter
function_name := fn(mut parameter: type) -> type {...}

// Mutable reference parameter
function_name := fn(mut parameter: &type) -> type {...}

// Dual mutability (both local variable and pointed-to data are mutable)
function_name := fn(mut parameter: &mut type) -> type {...}
```

### Function Types

Functions are first-class values. Their type is written as `fn(param_types...) -> return_type`.

```rzn
// Storing a function in a variable
operation : fn(i32, i32) -> i32 = add

// Passing a function as a parameter
apply :: fn(value: i32, transform: fn(i32) -> i32) -> i32 {
    ret transform(value)
}
```

---

## Data Structures

All fields inside custom types are **private by default** to preserve strict encapsulation. Visibility can be elevated using the `pub` keyword.

### Structs

```rzn
// Standard Struct
StructName :: struct {
    pub public_field: i32,
    private_field: str,
}

// Generic Struct
StructName<T> :: struct {
    field_name: T,
}

// Struct implementing a Behaviour
StructName :: struct ~> BehaviourName {
    field_name: i32,

    pub method_name :: fn(self: &StructName, parameter: i32) -> i32 {
        ret self.field_name + parameter
    }
}
```

### Unions

Unions hold exactly one of their declared types at any given time in shared memory space.

```rzn
UnionName :: union {
    Variant_One: i32,
    Variant_Two: f32,
}

UnionName<T> :: union {
    Variant_One: T,
    Variant_Two: str,
}
```

### Enums

Enums represent a distinct set of named choices. They can implement behaviours and methods.

```rzn
EnumName :: enum {
    State_Idle,
    State_Running,
    State_Stopped,
}

// Enum with associated values
Token :: enum {
    Identifier: str,
    Number: i64,
    Eof,
}

EnumName :: enum ~> BehaviourName {
    // If a default method from a behaviour needs to be overridden,
    // the target method must be marked 'mut' in the original behave spec.
    method_from_behave :: fn(parameter: type) -> type {
        ret value
    }
}
```

---

## Error Unions

Error declarations define explicitly isolated failure contexts.

```rzn
ErrorUnionName :: error {
    Invalid_Expression,
    Out_Of_Scope,
}
```

Functions append errors to standard return values using the `!` sigil:

```rzn
parse_data :: fn(input: i32) -> ErrorUnionName!i32 {
    if input < 0 {
        ret ErrorUnionName.Out_Of_Scope
    }
    ret input
}
```

The inferred `!T` form covers cases where the error type does not need to be named explicitly:

```rzn
quick_parse :: fn(input: str) -> !i32 {
    // compiler infers the error union from all possible error paths
}
```

---

## Trait / Behave

Behaviours define a contract of function blueprints that structs, enums, or unions must fulfill.

```rzn
BehaviourName :: behave {
    // Abstract method — must be implemented by the conforming type
    function_name :: fn(self: &@Self, parameter: type) -> type

    // Default method implementation.
    // Mark with 'mut' to allow downstream types to override it.
    mut overrideable_function :: fn(self: &@Self) -> void {
        // default logic
    }
}
```

---

## Pointers, References, & Dereferencing

Razen provides clear semantics for memory safety and raw hardware control.

* `&T` : Immutable reference — safely borrowed read-only pointer.
* `&mut T` : Mutable reference — safely borrowed write pointer.
* `*T` : Raw pointer — unchecked, for low-level pointer arithmetic.

### Dereferencing Rules

1. **Implicit Auto-Dereference**: Field and method access via `.` automatically dereferences references.
2. **Explicit Dereference**: Direct assignment or arithmetic on the pointed-to value uses postfix `ptr.*`.

```rzn
mut number := 100
ptr := &mut number
ptr.* = 200 // Explicitly updates 'number' to 200

Player :: struct { score: i32 }
mut player := Player{ score: 10 }
player_ptr := &mut player
player_ptr.score = 20 // Auto-dereferenced via dot access
```

---

## Control Flow

### If-Else

Conditionals do not require parentheses around clauses.

```rzn
if condition {
    // logic
} else if another_condition {
    // logic
} else {
    // logic
}
```

Capture unwrapped optional values or matched variants inside a condition using `|x|`:

```rzn
mut potential_value: ?i32 = 42

if potential_value |x| {
    // 'x' is a concrete i32, only exists in this block
} else {
    // value was nil
}
```

### Match (Pattern Matching)

Pattern matches must be exhaustive. The wildcard `_` is mandatory as the default fallthrough.

```rzn
match target_value {
    pattern_one => expression,
    pattern_two => {
        // block expression
    },
    _ => default_expression,
}
```

Matching on enum variants with associated values:

```rzn
match token {
    Token.Identifier |name| => process_name(name),
    Token.Number     |n|    => process_number(n),
    Token.Eof               => stop,
    _                       => @panic("unexpected token"),
}
```

### Loops

Razen unifies all iteration under the single `loop` keyword.

```rzn
// 1. Infinite loop — exit with 'stop'
loop {
    if condition { stop }
    if skip_condition { next } // skip to next iteration
}

// 2. Conditional loop (while-style)
loop condition {
    // executed while condition is true
}

// 3. Range / Collection loop with scoped capture variable
loop array_or_range |i| {
    // 'i' is the current element or index
}

// Mutate elements in place
loop vector_data |&mut element| {
    element = element * 2
}

// Multi-range looping
loop range_one, range_two |i, j| {
    // iterate both ranges in lockstep
}
```

---

## Modules and Visibility

### Declaring Modules

```rzn
mod engine

// Inline module scoping
mod network {
    // scoped components
}
```

### Importing Modules

```rzn
use std.testing.assert
use math.matrix
```

### Exporting (Public Visibility)

By default, everything in a module is private. Exposure requires the `pub` prefix.

```rzn
pub public_var := 10

pub Configuration :: struct {
    pub open_field: bool,
    hidden_field: i32,     // remains private to the module
}

pub calculate_speed :: fn() -> i32 {...}
```

---

## Error and Resource Management

### Try-Catch

Error processing isolates failures locally without hidden exception bubbles.

```rzn
// Combined catch block
try {
    risky_operation()
} catch |e| {
    match e {
        ErrorUnionName.Invalid_Expression => fix_expression(),
        _ => @panic("unhandled error"),
    }
}

// Single-line shorthand
try safe_fallback_call()
catch |err| handle_isolated_error(err)
```

### Defer

The `defer` statement schedules an expression to run at the end of the surrounding block scope. It is the primary mechanism for explicit cleanup — and it **always runs**, even if the block exits early via `ret` or an error.

```rzn
file := open_file("logs.txt")
defer file.close() // runs automatically on scope exit

process(file)
```

Multiple defers in one scope run in **reverse order** (last declared, first executed):

```rzn
defer log("step 1 cleanup") // runs second
defer log("step 2 cleanup") // runs first
```

---

## Memory Model

Razen's memory model has four explicit rules. No hidden garbage collector. No complex borrow checker. Memory is always visible, predictable, and under your control.

### Rule 1 — Stack by Default (Value Semantics)

All variables and structs live on the stack unless you explicitly heap-allocate them. Passing a value to a function gives it a local copy.

```rzn
Point :: struct { x: i32, y: i32 }

update_point :: fn(mut p: Point) -> Point {
    p.x = 100 // modifies only the local copy
    ret p
}

main :: fn() -> i32 {
    origin := Point{ x: 0, y: 0 }
    updated := update_point(origin)
    // origin.x is still 0, updated.x is 100
    ret 0
}
```

### Rule 2 — Explicit References (`&T` / `&mut T`)

Use references to avoid copying large structures. References cannot outlive the scope they were created in — they are never stored past the call stack.

```rzn
increment_score :: fn(player: &mut Player) -> void {
    player.score += 10 // auto-dereferenced via dot access
}

main :: fn() -> i32 {
    mut local_player := Player{ score: 0 }
    increment_score(&mut local_player)
    ret 0
}
```

### Rule 3 — Explicit Heap via `@Allocator`

The language **never** implicitly allocates heap memory. Any collection or struct needing heap lifetime must receive an `@Allocator` explicitly.

```rzn
init_buffer :: fn(allocator: @Allocator, size: usize) -> DynamicBuffer {
    memory := allocator.alloc(u8, size)
    ret DynamicBuffer{ ptr: memory, len: size, allocator: allocator }
}
```

### Rule 4 — `defer` for Cleanup

Pair every allocation with an immediate `defer`. It runs on block exit no matter what.

```rzn
process_tokens :: fn(allocator: @Allocator) -> void {
    mut tokens := @vec_with_allocator(allocator, Token)
    defer tokens.free()

    tokens.append(Token.Identifier)
}
```

> **Comptime memory is automatic.** Any `@comptime` block runs in an isolated compiler arena. The result is frozen into the binary; the temporary arena is wiped — you never manage it.

---

## Builtins

Builtin functions are hardcoded into the compiler and always prefixed with `@`.

**Reflection**
* `@Self`           : The implementing type within a `behave` context.
* `@TypeOf(expr)`   : Compile-time type of an expression.
* `@SizeOf(T)`      : Runtime size of a type in bytes.
* `@AlignOf(T)`     : Alignment constraint of a type.
* `@TypeName(T)`    : String representation of a type name.
* `@Fields(T)`      : Comptime array of fields inside a struct, enum, or union.
* `@EnumCount(E)`   : Total number of variants in an enum.

**Type Casting**
* `@as(T, value)`          : Safe explicit coercion to type `T`.
* `@bitCast(T, value)`     : Reinterprets raw bits of a value as type `T`.
* `@ptrCast(T, ptr)`       : Forces conversion between pointer types.
* `@intToPtr(T, address)`  : Converts a raw integer address to a typed pointer `*T`.
* `@ptrToInt(ptr)`         : Converts a pointer to a `usize` integer address.

**Memory**
* `@Allocator`                  : The universal allocator interface type.
* `@memcpy(dest, src, count)`   : High-performance block memory copy.
* `@memset(dest, value, count)` : Fills a memory block with a byte value.
* `@memmove(dest, src, count)`  : Overlapping-safe memory copy.
* `@pageAlloc(pages)`           : Allocates raw OS memory pages.
* `@pageFree(ptr, pages)`       : Returns pages to the OS kernel.
* `@comptimeDefaultAllocator()` : Returns the internal compiler arena during compilation.

**Comptime**
* `@comptime`           : Forces a block or expression to evaluate at compile time.
* `@compileLog(args...)`: Prints to terminal during the compilation phase.
* `@compileError(str)`  : Emits a compiler error and halts compilation.
* `@embedFile(path)`    : Reads a file at compile time and embeds it as a byte array.

**Control Flow**
* `@panic(str)`              : Unrecoverable crash — prints stack trace and halts.
* `@breakpoint()`            : Invokes a hardware breakpoint for attached debuggers.
* `@trap()`                  : Aborts via CPU illegal instruction signal.
* `@sysCall(num, args...)`   : Raw kernel syscall for dependency-free I/O.

**Materialization**
* `@str.from_raw(ptr, len)` : Constructs a `str` slice from a raw pointer and length.
* `@vec[elements...]`       : Instantiates a vector at stack or comptime.
* `@map{pairs...}`          : Instantiates a primitive hash map.
* `@set{elements...}`       : Instantiates a unique hash set.

**Math (Overflow-Safe)**
* `@addWithOverflow(a, b)` : Returns `(result, did_overflow)`.
* `@subWithOverflow(a, b)` : Returns `(result, did_underflow)`.
* `@mulWithOverflow(a, b)` : Returns `(result, did_overflow)`.

**Bitwise**
* `@ctz(value)`      : Count Trailing Zeros.
* `@clz(value)`      : Count Leading Zeros.
* `@popCount(value)` : Count total set bits (1s).
* `@bswap(value)`    : Reverse byte order (cross-architecture serialization).

**Concurrency**
* `@atomicLoad(ptr, order)`                          : Thread-safe atomic read.
* `@atomicStore(ptr, value, order)`                  : Thread-safe atomic write.
* `@cmpxchg(ptr, expected, new, success_ord, fail_ord)` : Atomic Compare-And-Swap.

---

## Standard Library Allocators

Concrete allocator strategies from `std.mem`. Each implements `@Allocator` and is imported via `use`.

| Allocator | Path | When to use |
| --- | --- | --- |
| `PageAllocator` | `std.mem.PageAllocator` | Base allocator. Wraps `@pageAlloc`/`@pageFree` for direct OS page requests. |
| `ArenaAllocator` | `std.mem.ArenaAllocator` | Bump-allocator. Individual frees are no-ops; destroy everything at once with `deinit()`. Ideal for parsers, compilers, and short-lived tasks. |
| `GeneralPurposeAllocator` | `std.mem.GeneralPurposeAllocator` | Production-grade. Handles fragmentation, tracks allocations, reports leaks during testing. |
| `FixedBufferAllocator` | `std.mem.FixedBufferAllocator` | Turns a stack `[u8; N]` into a local allocator. Zero OS calls — maximum performance for tightly scoped work. |
| `PoolAllocator` | `std.mem.PoolAllocator` | Same-size chunk allocator. Ideal for game entities, ECS components, or network connections. |

```rzn
use std.mem.GeneralPurposeAllocator

main :: fn() -> i32 {
    mut gpa := GeneralPurposeAllocator.init()
    defer gpa.deinit() // prints leak report if anything was not freed

    allocator := gpa.allocator()
    // pass allocator into anything that needs heap memory
    ret 0
}
```

---

## Collections

Comptime collections use the compiler arena automatically. Runtime heap collections require an explicit `@Allocator`.

### Strings

```rzn
// Stack / static — no allocator needed
name : str = "razen"

// Heap-allocated dynamic string
message : @str = @str.from("hello world")
```

### Arrays & Slices

```rzn
// Fixed stack array
buffer : [u8; 64] = [0; 64]

// Slice — a non-owning view into existing memory
view : [u8] = buffer[0..32]
```

### Vectors

```rzn
// Comptime constant vector
VEC_CONST :: @vec[1, 2, 3]         // type: @vec[i32; 3]

// Runtime vector
vector_name := @vec["a", "b", "c"]
```

### HashMaps

```rzn
map_name := @map {
    "key_one": 100,
    "key_two": 200,
}

current_val := map_name.key_one    // dot-access
```

### Sets

```rzn
set_name := @set{10, 20, 30}       // type: @set{i32}
```

---

## Type Aliasing

```rzn
IdType :: type(u64)
```

---

## Testing

Unit tests are first-class language constructs.

```rzn
use std.testing.assert

unit_test_case :: test {
    first_value  := 10
    second_value := 10
    assert(first_value, second_value)
}
```

### Running Tests

```bash
razenc test test.rzn test2.rzn
```
