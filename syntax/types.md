# Types

## Integer Types

* **Fixed Bit-Width**: `i1`, `i2`, `i4`, `i8`, `i16`, `i32`, `i64`, `i128`
* **Unsigned Fixed Bit-Width**: `u1`, `u2`, `u4`, `u8`, `u16`, `u32`, `u64`, `u128`
* **Architecture Sized**: `isize`, `usize`
* **Defaults**: `int` (alias for `i32`), `uint` (alias for `u32`)

## Float Types

* **Standard Widths**: `f8`, `f16`, `f32`, `f64`, `f128`
* **Default**: `float` (alias for `f32`)

## Pointer & Reference Types

* `&T`     : Immutable reference (safe read-only borrow)
* `&mut T` : Mutable reference (safe read-write borrow)
* `*T`     : Raw pointer (unsafe, direct memory / pointer arithmetic)

> References (`&T`, `&mut T`) cannot outlive the scope they were created in and cannot be stored in struct fields that escape that scope. Raw pointers (`*T`) have no such restriction — use them only when you need direct hardware-level access.

## Special Types

* `bool`    : Boolean value (`true` or `false`)
* `char`    : Unicode character literal (code point)
* `void`    : Empty value / no return value
* `noret`   : Function never returns (diverging — e.g. `@panic`, infinite loops)
* `anytype` : Compile-time duck typing wildcard — type is inferred at comptime
* `nil`     : The absence of a value (used with `?T` optionals)
* `fn(...)` : Function signature as a first-class type
* `type(T)` : Type meta-representation / alias context

## Strings

* `str`  : Stack-allocated or static read-only string slice
* `@str` : Heap-allocated dynamic string (requires `@Allocator` internally)

## Collections

* `[T]`         : Array Slice — non-owning runtime view of contiguous memory
* `[T; N]`      : Fixed Array — stack-allocated array of exactly `N` elements
* `@vec[T]`     : Dynamic heap vector
* `@vec[T; N]`  : Fixed-capacity heap vector
* `@map{K, V}`  : Hash map — unique keys `K` mapped to values `V`
* `@set{T}`     : Mathematical unique set of values of type `T`
* `@set{T; N}`  : Sized unique set with static capacity `N`

## Data Types

* `struct{}`  : Encapsulated custom data structure (fields private by default)
* `enum{}`    : Distinct named choices list
* `union{}`   : Shared-memory tag variant container
* `error{}`   : Explicit error context definition
* `behave{}`  : Structural behavioural trait contract
* `test{}`    : Compiler-native isolated test runner block

## Error Handling & Optionals

* `?T`  : Optional type wrapper — value of type `T` or `nil`
* `!T`  : Inferred implicit global error union wrapping type `T`
* `E!T` : Explicit error union `E` wrapping return type `T`
