# Razen Compiler Semantic Errors (`errors.md`)

This document defines the complete list of semantic compiler errors emitted during **Phase 3 (Semantic Analysis)** of `razenc`. Each error has a unique code, clear diagnostic message, detailed explanation, and code examples.

---

## Error Codes Reference

| Error Code | Error Name | Description |
| --- | --- | --- |
| **`SEMA-0001`** | `TypeMismatch` | Assigned value or argument type does not match the expected type. |
| **`SEMA-0002`** | `UndefinedSymbol` | Use of an identifier that is not declared in the current or outer scopes. |
| **`SEMA-0003`** | `DuplicateDeclaration` | Redefining a symbol (variable, constant, function, type) in the same scope. |
| **`SEMA-0004`** | `MutationOfImmutable` | Attempting to modify or reassign a variable not declared with the `mut` keyword. |
| **`SEMA-0005`** | `ReassignmentOfConstant` | Attempting to reassign or modify a compile-time constant (`::`). |
| **`SEMA-0006`** | `InvalidReference` | Invalid reference borrowing, such as taking a mutable borrow of an immutable value. |
| **`SEMA-0007`** | `NonBoolCondition` | Control flow condition (in `if` or `loop`) is not of type `bool`. |
| **`SEMA-0008`** | `LoopControlFlowError` | Use of `stop` or `next` keywords outside of an active `loop` block. |
| **`SEMA-0009`** | `ReturnTypeMismatch` | Returned expression type does not match the declared return type of the function. |
| **`SEMA-0010`** | `AllocatorRequired` | Initializing a heap collection (`@vec`, `@map`, `@set`, `@str`) without an explicit allocator. |
| **`SEMA-0011`** | `UnimplementedBehave` | A type claiming to implement a `behave` trait fails to implement all required methods. |
| **`SEMA-0012`** | `IncompatibleBinaryOp` | Applying a binary operator to incompatible types (e.g. adding `bool` and `i32`). |
| **`SEMA-0013`** | `IncompatibleUnaryOp` | Applying a unary operator to an invalid operand type. |
| **`SEMA-0014`** | `NonCallable` | Attempting to invoke a call expression on a symbol that is not a function. |
| **`SEMA-0015`** | `FieldAccessError` | Accessing a non-existent field, or a private field from outside its defining module. |
| **`SEMA-0016`** | `MatchNotExhaustive` | Match expression is not exhaustive (missing patterns or the wildcard `_`). |
| **`SEMA-0017`** | `InvalidBuiltinCall` | Wrong number/type of arguments supplied to a compiler builtin function (`@`). |
| **`SEMA-0018`** | `DereferenceError` | Attempting to perform explicit dereference `.*` on a non-pointer/non-reference value. |
| **`SEMA-0019`** | `CaptureMismatch` | Invalid use of optional capture `|x|` or matching variants on non-matching types. |
| **`SEMA-0020`** | `GenericParamMismatch` | Incorrect type parameter count or constraints provided in generic function calls. |

---

## Detailed Specifications

### SEMA-0001: TypeMismatch
**Message**: `Type mismatch: expected type '{expected}', found type '{found}'`
* **Incorrect**:
  ```rzn
  mut x : i32 = 10
  x = "hello" // Error: expected i32, found str
  ```
* **Correct**:
  ```rzn
  mut x : i32 = 10
  x = 20
  ```

### SEMA-0002: UndefinedSymbol
**Message**: `Undefined symbol '{name}' in current scope`
* **Incorrect**:
  ```rzn
  main :: fn() -> void {
      result := value + 10 // Error: 'value' is undefined
  }
  ```
* **Correct**:
  ```rzn
  value := 5
  main :: fn() -> void {
      result := value + 10
  }
  ```

### SEMA-0003: DuplicateDeclaration
**Message**: `Duplicate declaration: symbol '{name}' is already defined in this scope`
* **Incorrect**:
  ```rzn
  x := 10
  x := 20 // Error: 'x' is already declared in this scope
  ```
* **Correct**:
  ```rzn
  mut x := 10
  x = 20
  ```

### SEMA-0004: MutationOfImmutable
**Message**: `Cannot mutate immutable variable '{name}'`
* **Incorrect**:
  ```rzn
  x := 100
  x = 200 // Error: 'x' is immutable
  ```
* **Correct**:
  ```rzn
  mut x := 100
  x = 200
  ```

### SEMA-0005: ReassignmentOfConstant
**Message**: `Cannot assign to compile-time constant '{name}'`
* **Incorrect**:
  ```rzn
  PI :: 3.14
  main :: fn() -> void {
      PI = 3.14159 // Error: Cannot assign to constant PI
  }
  ```
* **Correct**:
  ```rzn
  mut CURRENT_PI :: 3.14
  main :: fn() -> void {
      CURRENT_PI = 3.14159 // Allowed for stateful comptime variables
  }
  ```

### SEMA-0006: InvalidReference
**Message**: `Cannot take mutable reference to immutable value`
* **Incorrect**:
  ```rzn
  val := 42
  ptr := &mut val // Error: 'val' is immutable, cannot borrow mutably
  ```
* **Correct**:
  ```rzn
  mut val := 42
  ptr := &mut val
  ```

### SEMA-0007: NonBoolCondition
**Message**: `Condition must be of type 'bool', found '{type}'`
* **Incorrect**:
  ```rzn
  if 123 { // Error: expected bool, found i32
  }
  ```
* **Correct**:
  ```rzn
  if true {
  }
  ```

### SEMA-0008: LoopControlFlowError
**Message**: `Keyword '{keyword}' is only allowed inside an active loop body`
* **Incorrect**:
  ```rzn
  main :: fn() -> void {
      stop // Error: 'stop' outside of a loop
  }
  ```
* **Correct**:
  ```rzn
  loop {
      stop // Allowed
  }
  ```

### SEMA-0009: ReturnTypeMismatch
**Message**: `Return type mismatch: expected '{expected}', found '{found}'`
* **Incorrect**:
  ```rzn
  get_number :: fn() -> i32 {
      ret "not a number" // Error: expected i32, found str
  }
  ```
* **Correct**:
  ```rzn
  get_number :: fn() -> i32 {
      ret 42
  }
  ```

### SEMA-0010: AllocatorRequired
**Message**: `Heap collection type '{type}' requires an explicit allocator`
* **Incorrect**:
  ```rzn
  // Attempting to instantiate heap vector without allocator
  mut numbers := @vec[1, 2, 3]
  ```
* **Correct**:
  ```rzn
  // Comptime allocations use compiler arena allocator automatically
  NUMBERS :: @vec[1, 2, 3]
  
  // Runtime heap allocations require allocator
  main :: fn() -> void {
      allocator := @comptimeDefaultAllocator() // or page allocator
      mut tokens := @vec_with_allocator(allocator, Token)
  }
  ```

### SEMA-0011: UnimplementedBehave
**Message**: `Struct '{name}' does not implement behave trait method '{method}'`
* **Incorrect**:
  ```rzn
  Drawable :: behave {
      draw :: fn(self: &@Self) -> void
  }
  Vector :: struct ~> Drawable {
      x: f32, // Error: Vector does not implement draw method
  }
  ```
* **Correct**:
  ```rzn
  Drawable :: behave {
      draw :: fn(self: &@Self) -> void
  }
  Vector :: struct ~> Drawable {
      x: f32,
      pub draw :: fn(self: &Vector) -> void {
          ret
      }
  }
  ```

### SEMA-0012: IncompatibleBinaryOp
**Message**: `Operator '{op}' is not defined for types '{left}' and '{right}'`
* **Incorrect**:
  ```rzn
  res := true + 5 // Error: binary operator + is not defined for bool and i32
  ```
* **Correct**:
  ```rzn
  res := 5 + 5
  ```

### SEMA-0013: IncompatibleUnaryOp
**Message**: `Operator '{op}' is not defined for type '{type}'`
* **Incorrect**:
  ```rzn
  res := -true // Error: unary operator - is not defined for bool
  ```
* **Correct**:
  ```rzn
  res := -5
  ```

### SEMA-0014: NonCallable
**Message**: `Cannot call non-callable type '{type}'`
* **Incorrect**:
  ```rzn
  x := 10
  x(5) // Error: i32 is not callable
  ```
* **Correct**:
  ```rzn
  add :: fn(x: i32) -> i32 { ret x }
  add(5)
  ```

### SEMA-0015: FieldAccessError
**Message**: `Field '{field}' does not exist on type '{type}' or is private`
* **Incorrect**:
  ```rzn
  Player :: struct {
      score: i32,
  }
  p := Player{ score: 10 }
  name := p.name // Error: field 'name' does not exist on Player
  ```
* **Correct**:
  ```rzn
  Player :: struct {
      pub score: i32,
  }
  p := Player{ score: 10 }
  s := p.score
  ```

### SEMA-0016: MatchNotExhaustive
**Message**: `Match patterns are not exhaustive. Wildcard pattern '_' is required`
* **Incorrect**:
  ```rzn
  match x {
      1 => ret,
      // Error: missing wildcard pattern _
  }
  ```
* **Correct**:
  ```rzn
  match x {
      1 => ret,
      _ => ret,
  }
  ```

### SEMA-0017: InvalidBuiltinCall
**Message**: `Builtin '{builtin}' call invalid: {reason}`
* **Incorrect**:
  ```rzn
  val := @as(i32) // Error: expected 2 arguments, got 1
  ```
* **Correct**:
  ```rzn
  val := @as(i32, 5)
  ```

### SEMA-0018: DereferenceError
**Message**: `Cannot dereference non-pointer, non-reference type '{type}'`
* **Incorrect**:
  ```rzn
  x := 10
  y := x.* // Error: cannot dereference i32
  ```
* **Correct**:
  ```rzn
  x := 10
  ptr := &x
  y := ptr.*
  ```

### SEMA-0019: CaptureMismatch
**Message**: `Cannot capture optional value from non-optional type '{type}'`
* **Incorrect**:
  ```rzn
  val := 42
  if val |x| { // Error: val is not optional
  }
  ```
* **Correct**:
  ```rzn
  mut val: ?i32 = 42
  if val |x| {
      // x is i32
  }
  ```

### SEMA-0020: GenericParamMismatch
**Message**: `Generic parameters mismatch: expected {expected} type arguments, got {found}`
* **Incorrect**:
  ```rzn
  identity<T> :: fn(x: T) -> T { ret x }
  identity(5) // Error: missing generic parameters (needs explicit instantiations or inference)
  ```
* **Correct**:
  ```rzn
  identity<T> :: fn(x: T) -> T { ret x }
  identity<i32>(5)
  ```
