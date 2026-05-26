# Builtins & Allocators

These are the two categories of low-level infrastructure hardcoded into the Razen compiler (`razenc`): the 40 built-in primitives (`@`) and the standard library allocator implementations.

---

## 1. Built-in Primitives (`@`)

Builtins are hardcoded directly into `razenc`. They provide type reflection, memory control, hardware intrinsics, and optimization hooks required for self-hosting. All are prefixed with `@`.

| Built-in Signature | Category | Description |
| --- | --- | --- |
| **`@TypeOf(expr)`** | Reflection | Returns the compile-time Type of an expression. |
| **`@SizeOf(T)`** | Reflection | Returns the runtime size of type `T` in bytes. |
| **`@AlignOf(T)`** | Reflection | Returns the byte alignment constraint of type `T`. |
| **`@Fields(T)`** | Reflection | Returns a comptime array of fields inside a struct, enum, or union. |
| **`@EnumCount(E)`** | Reflection | Returns the total number of variants inside an enum. |
| **`@TypeName(T)`** | Reflection | Returns the string name of a type. |
| **`@as(T, value)`** | Type Casting | Safe, explicit type coercion/widening to type `T`. |
| **`@bitCast(T, value)`** | Type Casting | Reinterprets the raw bits of a value as type `T`. |
| **`@ptrCast(T, ptr)`** | Type Casting | Forces conversion between pointer types (e.g., `*u8` to `*i32`). |
| **`@intToPtr(T, address)`** | Type Casting | Converts a raw integer address to a typed pointer `*T`. |
| **`@ptrToInt(ptr)`** | Type Casting | Converts a pointer to a `usize` integer address. |
| **`@memcpy(dest, src, count)`** | Memory | High-performance block copy of memory. |
| **`@memset(dest, value, count)`** | Memory | Fills a memory block with a specific byte value. |
| **`@memmove(dest, src, count)`** | Memory | Overlapping-safe memory copy. |
| **`@pageAlloc(pages)`** | Memory | Allocates raw memory pages directly from the OS kernel. |
| **`@pageFree(ptr, pages)`** | Memory | Returns raw pages back to the OS kernel. |
| **`@comptimeDefaultAllocator()`** | Memory | Returns the internal compiler arena allocator during compilation. |
| **`@Allocator`** | Memory | **The universal allocator interface type** — passed explicitly to any heap-allocating construct. |
| **`@comptime`** | Comptime | Forces a block or expression to evaluate fully during compilation. Runs in an isolated compiler arena; result is frozen into the binary. |
| **`@compileLog(args...)`** | Comptime | Prints to terminal *during* the compilation phase. |
| **`@compileError(str)`** | Comptime | Emits an explicit compiler error and halts compilation. |
| **`@embedFile(path)`** | Comptime | Reads a file at compile time and embeds it as a byte array in the binary. |
| **`@panic(str)`** | Control Flow | Unrecoverable crash — prints stack trace and halts the process. |
| **`@breakpoint()`** | Control Flow | Invokes a hardware breakpoint for attached debuggers. |
| **`@trap()`** | Control Flow | Aborts via CPU illegal instruction signal. |
| **`@sysCall(num, args...)`** | OS Interface | Raw kernel syscall — dependency-free I/O without libc. |
| **`@str.from_raw(ptr, len)`** | Materialization | Constructs a `str` slice from a raw memory pointer and length. |
| **`@vec[elements...]`** | Materialization | Instantiates a vector at stack or comptime. |
| **`@map{pairs...}`** | Materialization | Instantiates a compiler-optimized primitive hash map. |
| **`@set{elements...}`** | Materialization | Instantiates a unique hash set. |
| **`@addWithOverflow(a, b)`** | Math | Returns `(result, did_overflow)`. |
| **`@subWithOverflow(a, b)`** | Math | Returns `(result, did_underflow)`. |
| **`@mulWithOverflow(a, b)`** | Math | Returns `(result, did_overflow)`. |
| **`@ctz(value)`** | Bitwise | Count Trailing Zeros — fast bitset scanning. |
| **`@clz(value)`** | Bitwise | Count Leading Zeros. |
| **`@popCount(value)`** | Bitwise | Count total set bits (1s). |
| **`@bswap(value)`** | Bitwise | Reverse byte order — cross-architecture serialization. |
| **`@atomicLoad(ptr, order)`** | Concurrency | Thread-safe atomic read. |
| **`@atomicStore(ptr, value, order)`** | Concurrency | Thread-safe atomic write. |
| **`@cmpxchg(ptr, exp, new, s, f)`** | Concurrency | Atomic Compare-And-Swap — building block for lock-free structures. |

---

## 2. Standard Library Allocators (`std.mem`)

These are concrete allocator strategies written in pure Razen user-land code. Each one exposes the `@Allocator` interface and is imported via `use std.mem.*`.

The language **never** allocates heap memory implicitly — you always choose and pass an allocator explicitly.

| Implementation | Path | When to use |
| --- | --- | --- |
| **`PageAllocator`** | `std.mem.PageAllocator` | Base allocator. Wraps `@pageAlloc` / `@pageFree` for direct OS page requests. Use as the root allocator that feeds others. |
| **`ArenaAllocator`** | `std.mem.ArenaAllocator` | Bump-allocator. Individual frees are no-ops; destroy everything at once with `deinit()`. Best for parsers, compilers (`razenc`), and short-lived batch tasks. |
| **`GeneralPurposeAllocator`** | `std.mem.GeneralPurposeAllocator` | Production-grade. Handles fragmentation, tracks every allocation, and prints memory leak reports during testing. Good default for long-running programs. |
| **`FixedBufferAllocator`** | `std.mem.FixedBufferAllocator` | Turns a stack `[u8; N]` into a local allocator. Zero OS calls — maximum performance for tightly scoped work with bounded memory. |
| **`PoolAllocator`** | `std.mem.PoolAllocator` | Same-size chunk allocator. Ideal for game entities, ECS components, or network connections where objects are created and destroyed frequently. |

### Usage Pattern

```rzn
use std.mem.ArenaAllocator

main :: fn() -> i32 {
    mut arena := ArenaAllocator.init(@pageAlloc(4))
    defer arena.deinit() // frees the entire arena at once

    allocator := arena.allocator()
    // pass allocator into collections and functions that need heap memory

    mut tokens := @vec_with_allocator(allocator, Token)
    // no need to free tokens individually — arena.deinit() handles it all
    ret 0
}
```

### Choosing an Allocator for `razenc` (Self-Hosting)

For the Razen compiler itself, the recommended strategy is:

* **`ArenaAllocator`** for per-file or per-pass allocations (AST nodes, token lists) — free the entire pass at once.
* **`GeneralPurposeAllocator`** in debug/test builds to catch leaks early.
* **`FixedBufferAllocator`** for small, bounded scratch buffers inside hot paths (e.g. identifier interning lookups).
