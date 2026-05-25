# Keywords & Symbols

## Keywords

* **Control Flow**: `if`, `else`, `match`, `loop`, `stop`, `next`, `ret`
* **Memory & Error**: `mut`, `try`, `catch`, `defer`
* **Modules & Visibility**: `mod`, `use`, `pub`, `ext`
* **Definitions**: `fn`, `struct`, `union`, `enum`, `error`, `behave`, `type`, `test`
* **Literals**: `true`, `false`, `nil`

## Core Sigils

* `:=`       : Inferred variable assignment (Runtime)
* `::`       : Inferred constant assignment (Comptime)
* `: type =` : Explicit type variable assignment (Runtime)
* `: type :` : Explicit type constant assignment (Comptime)
* `~>`       : Implements behaviour/trait
* `->`       : Function return type pointer
* `@`        : Builtin compiler macro/type prefix
* `?`        : Optional/Nullable type prefix (e.g., `?i32`)
* `!`        : Error union separator (e.g., `ErrorUnionName!i32`)
* `&`        : Reference sigil — `&T` immutable, `&mut T` mutable
* `.*`       : Explicit postfix dereference (e.g., `ptr.* = value`)
* `..`       : Exclusive range (e.g., `0..10`)
* `..=`      : Inclusive range (e.g., `0..=10`)

## Operators

* **Math**: `+`, `-`, `*`, `/`, `%`, `+=`, `-=`, `*=`, `/=`, `%=`
* **Logic**: `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`
* **Bitwise**: `&`, `|`, `^`, `~`, `<<`, `>>`
* **Access**: `.`, `_`, `|`
