# IfáLang Language Runtime Specification
 
**Version:** 1.3 — Canonical Baseline  
**Status:** Ratified  
**Date:** 2026-04-11  
**Authority:** This document supersedes any runtime behavior. Where a runtime disagrees with this specification, the runtime is wrong.

---

> *Ìmọ̀ ni ipilẹ̀ ṣe gbogbo ohun — Knowledge is the foundation of all things.*

---

## Table of Contents

1. [Document Conventions](#1-document-conventions)
2. [Three-Tier Runtime Model](#2-three-tier-runtime-model)
3. [Lexical Structure](#3-lexical-structure)
4. [Value System](#4-value-system)
5. [Truthiness](#5-truthiness)
6. [Operators and Expressions](#6-operators-and-expressions)
7. [Variables and Scope](#7-variables-and-scope)
8. [Functions and Closures](#8-functions-and-closures)
9. [Classes](#9-classes)
10. [Control Flow](#10-control-flow)
11. [Pattern Matching](#11-pattern-matching)
12. [Error Handling](#12-error-handling)
13. [The 16 Odù Standard Domains](#13-the-16-odù-standard-domains)
14. [Module System](#14-module-system)
15. [Concurrency](#15-concurrency)
16. [The Embedded Tier](#16-the-embedded-tier)
17. [Bytecode VM Specification](#17-bytecode-vm-specification)
18. [Memory Model](#18-memory-model)
19. [Error Taxonomy](#19-error-taxonomy)
20. [Capability Matrix](#20-capability-matrix)
21. [Conformance Requirements](#21-conformance-requirements)
22. [The Babalawo — Static Analysis System](#22-the-babalawo--static-analysis-system)
23. [The Ìwà Engine — Resource Lifecycle Validation](#23-the-ìwà-engine--resource-lifecycle-validation)
24. [The Èèwọ̀ System — Architectural Taboos](#24-the-èèwọ̀-system--architectural-taboos)
25. [The Àjọṣe Reactive System](#25-the-àjọṣe-reactive-system)
26. [The Ẹbọ Type System — Resource Obligations](#26-the-ẹbọ-type-system--resource-obligations)
27. [The Multiparadigm Design](#27-the-multiparadigm-design)
28. [The Three Execution Tiers](#28-the-three-execution-tiers)
29. [The Babalawo Capability Inferencer](#29-the-babalawo-capability-inferencer)
30. [The OpeleChain — Temporal Types](#30-the-opelechain--temporal-types)
31. [The 256 Odù — Long-Term Type Lattice Vision](#31-the-256-odù--long-term-type-lattice-vision)
32. [The WASM Playground](#32-the-wasm-playground)
33. [Ọjà — The Package Manager](#33-ọjà--the-package-manager)
34. [The Debug Adapter — DAP Integration](#34-the-debug-adapter--dap-integration)
35. [The Deployment Manager](#35-the-deployment-manager)
36. [The Language Server — `ifa lsp`](#36-the-language-server--ifa-lsp)
37. [The Debug Adapter — `ifa debug`](#37-the-debug-adapter--ifa-debug)
38. [The Documentation Generator — `ifa doc`](#38-the-documentation-generator--ifa-doc)
39. [Zero-Config Deployment — `ifa deploy` and `Iwe.toml`](#39-zero-config-deployment--ifa-deploy-and-iwetoml)
40. [The Formatter — `ifa fmt`](#40-the-formatter--ifa-fmt)

**Appendices**
- [Appendix A — Embedded Tier Restrictions](#appendix-a--embedded-tier-restrictions)
- [Appendix B — Implementation Status Matrix](#appendix-b--implementation-status-matrix)
- [Appendix C — Static Analysis Rule Summary](#appendix-c--static-analysis-rule-summary)
- [Appendix D — Lifecycle Rules Quick Reference](#appendix-d--lifecycle-rules-quick-reference)
- [Appendix E — TypeHint System](#appendix-e--typehint-system)
- [Appendix F — Ọjà Package Manifest Reference](#appendix-f--ọjà-package-manifest-reference)
- [Appendix G — DAP Request/Response Reference](#appendix-g--dap-requestresponse-reference)
- [Appendix H — Canonical Toolchain Summary](#appendix-h--canonical-toolchain-summary)

---

## 1. Document Conventions

### 1.1 Requirement Keywords

The keywords **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **MAY** follow RFC 2119. A runtime that violates a **MUST** requirement is **non-conforming**. A runtime that violates a **SHOULD** requirement is conforming but suboptimal.

### 1.2 Status Markers

Each section carries one of three implementation status markers:

| Marker | Meaning |
|--------|---------|
| `[DEFINED]` | Fully specified. All runtimes must implement this exactly. |
| `[PARTIAL]` | Specified here, but not yet fully implemented. The spec is normative. |
| `[OPEN]` | Decision not yet ratified. Behavior is undefined until resolved. |

### 1.3 Terminology

- **Program** — one or more `.ifa` source files forming a logical unit.
- **Value** — a runtime entity that can be stored, passed, or returned.
- **Binding** — the association of a name to a value in a scope.
- **Environment** — the full set of active bindings at a given point of execution.
- **Runtime** — any of the three conforming implementations: AST interpreter, bytecode VM, or transpiler.
- **Conforming program** — a program that contains no constructs marked unsupported in the capability matrix for the chosen runtime.

---

## 2. Three-Tier Runtime Model

### 2.1 The Tiers

IfáLang has three execution paths. They serve different purposes and have different performance characteristics. They are **not** interchangeable — each is optimized for a distinct phase of the development lifecycle.

```
ifa run    →  AST Interpreter   —  Development tier
ifa runb   →  Bytecode VM       —  Execution tier  (canonical semantics)
ifa build  →  Transpiler        —  Deployment tier
```

### 2.2 Tier Roles

#### `ifa run` — Development Tier

**Purpose:** Fast iteration during development.

- Parses source → walks the AST directly. No bytecode compilation.
- Starts instantly. Zero compilation latency.
- Best error messages: every error **MUST** include source file, line number, and column.
- Optimized for correctness and clarity, not performance.
- Supports the Tier 1 feature set (see §20).

**What it is not:** The semantic authority. If `ifa run` and `ifa runb` produce different results for a conforming program, `ifa run` has a bug.

#### `ifa runb` — Execution Tier

**Purpose:** Canonical program execution. Defines what IfáLang means.

- Parses source → compiles to bytecode → executes in the bytecode VM.
- The **semantic oracle**: when runtimes disagree, this runtime defines correct behavior, subject to the spec.
- Supports the full Tier 1 and Tier 2 feature sets.
- Reports errors with source location (requires source location threading through opcodes — see §17.4).

**What it is not:** "Canonical" by declaration. The VM earns canonical status by passing the conformance suite (see §21). Until that milestone is reached, both runtimes have equal authority over the spec.

#### `ifa build` — Deployment Tier

**Purpose:** Ahead-of-time compilation to native Rust for production deployment.

- Parses source → emits Rust source → invokes `cargo build`.
- Zero interpreter overhead at runtime.
- Output is a standalone native binary or library.
- Supports the Tier 1 and Tier 3 feature sets.
- Any construct not in its supported feature set **MUST** produce `IfaError::NotImplemented` — never a `panic!()`, never a silently dropped no-op, never an emitted comment.

**What it is not:** A semantic validator. The transpiler targeting Rust validates that the emitted Rust compiles, not that the IfáLang semantics are correct. Rust's integer overflow behavior is not IfáLang's integer overflow behavior.

### 2.3 The Shared Pipeline

All three runtimes share a single frontend. No runtime does its own parsing or name resolution.

```
Source (.ifa)
    │
    ▼
  Lexer
    │
    ▼
  Parser  →  AST
    │
    ▼
  Resolver Pass  ←── shared, runs once before any backend
  (annotated AST)
    │
    ├─────────────────────────────────────────────┐
    │                                             │
    ▼                                             ▼
ifa run                                      ifa runb
AST Walker                               Bytecode Compiler
(eval annotated AST directly)            (annotated AST → opcodes)
                                                  │
                                                  ▼
                                             Bytecode VM
    │
    ▼
ifa build
Transpiler
(annotated AST → Rust source)
```

### 2.4 The Resolver Pass

The resolver is a mandatory shared pass that annotates every identifier in the AST before any backend consumes it. It resolves each identifier to one of four binding kinds:

| Binding Kind | Description |
|---|---|
| `Local(depth, slot)` | Local variable in a specific scope frame |
| `Upvalue(index)` | Variable captured from an enclosing scope (closure) |
| `Global(name)` | Module-level or program-level variable |
| `Domain(domain, method)` | Standard library domain call |

Backends **MUST NOT** perform their own name resolution. If an identifier cannot be resolved, the resolver **MUST** produce a `ReferenceError` before any backend executes.

### 2.5 Semantic Authority

The semantics specification (this document) is the canonical authority. The VM is the canonical *runtime* implementation of this specification.

- A disagreement between `ifa run` and `ifa runb` → `ifa run` is wrong.
- A disagreement between `ifa runb` and this spec → `ifa runb` is wrong.
- A disagreement between this spec and common sense → raise a spec issue, do not silently match the runtime.

---

## 3. Lexical Structure `[DEFINED]`

### 3.1 Encoding

Source files **MUST** be UTF-8 encoded. A BOM (`U+FEFF`) at the start of a file **MUST** be silently ignored. IfáLang identifiers **MAY** contain Yoruba Unicode characters with diacritics (e.g., `ìwà`, `ọ̀pẹ̀lẹ̀`).

### 3.2 Comments

```
# This is a line comment — extends to end of line

#{
  This is a block comment.
  Block comments may be nested: #{ nested }#
}#
```

### 3.3 Whitespace

Spaces, tabs, and newlines are whitespace. Whitespace is insignificant — it separates tokens but has no syntactic meaning. IfáLang does **not** use significant indentation.

### 3.4 Keywords

The following are reserved. They cannot be used as identifiers. The Yoruba form is canonical; English aliases are accepted.

| Yoruba | English Alias | Category |
|--------|--------------|----------|
| `ayanmo` | `let` | Variable declaration (mutable) |
| `ayanfe` | `const` | Variable declaration (immutable) |
| `ese` | `fn` | Function definition |
| (LEGACY) | (LEGACY) | `odu` / `class` hard-rejected in v1.3 |
| `pada` | `return` | Return from function |
| `ti` | `if` | Conditional |
| `bibẹkọ` | `else` | Else branch |
| `nigba` | `while` | While loop |
| `fun` | `for` | For loop |
| `duro` | `break` | Break from loop |
| `tesiwaju` | `continue` | Continue to next iteration |
| `ode` | `match` | Pattern match |
| `gbiyanju` | `try` | Error handling block |
| `gba` | `catch` | Catch block |
| `ta` | `throw` | Raise error |
| `ailewu` | `unsafe` | Unsafe pointer block (embedded/VM only) |
| `jowo` | `yield` | Cooperative yield |
| `mu` | `import` | Import module |
| `fi` | `export` | Export symbol |
| `otito` | `true` | Boolean true |
| `eke` | `false` | Boolean false |
| `ofo` | `null` | Null value |
| `ati` | `and` | Logical AND (word form) |
| `tabi` | `or` | Logical OR (word form) |
| `ko` | `not` | Logical NOT (word form) |
| `ara` | `self` | Current instance |
| `iya` | `super` | Parent class |
| `gbangba` | `pub` | Public visibility |
| `aladani` | `private` | Private visibility |
| `daro` | `async` | Async function |
| `reti` | `await` | Await async value |

> **Lexical note — `duro fun` disambiguation:** `duro` is also the keyword for `break` (§10.4), and `fun` is the keyword for `for` (§10.3). The two-token sequence `duro fun` means `await` only when it appears as an expression inside a `daro` (async) function body, preceding an expression that produces a `Future`. A bare `duro` at the end of a loop iteration (break) is never followed by `fun` in grammatically valid code. Implementations **MUST** resolve the ambiguity by context, not by lookahead alone: `duro fun <expr>` in an async context is an await expression; `duro;` or `duro <value>;` in a loop context is a break statement.
>
> ⚠️ **`[OPEN]` — Legacy `duro fun` syntax.** Older versions used the two-word sequence `duro fun` for await. This has been replaced by the single keyword `reti` to avoid ambiguity with `break` (`duro`) and `for` (`fun`). Implementations **MAY** still emit a deprecation hint if the old syntax is encountered in legacy code.
 
### 3.5 Identifiers

An identifier begins with a Unicode letter or `_`, followed by any combination of Unicode letters, digits, `_`, or `'` (apostrophe, for Yoruba orthography). Identifiers are case-sensitive.

```
x             # valid
_count        # valid
ìwà_pẹ̀lẹ́      # valid — Yoruba with diacritics
count2        # valid
2count        # invalid — starts with digit
my-var        # invalid — hyphen not permitted
```

### 3.6 Literals

#### Integer Literals

```
42            # decimal
0xFF          # hexadecimal
0o77          # octal
0b1010        # binary
1_000_000     # underscore as digit separator
```

#### Float Literals

```
3.14
2.0e10
1.5e-3
.5            # leading dot is permitted
```

#### String Literals

Strings are delimited by double quotes. Escape sequences: `\n`, `\t`, `\\`, `\"`, `\uXXXX`.

String interpolation uses `$"..."` with `{expression}` inside:

```
ayanmo name = "Àdé";
Irosu.fo($"Hello, {name}!");     # Hello, Àdé!
Irosu.fo($"2 + 2 = {2 + 2}");   # 2 + 2 = 4
```

#### Boolean and Null

```
otito    # true
eke      # false
ofo      # null
```

---

## 4. Value System `[DEFINED]`

### 4.1 Type Hierarchy

IfáLang is **dynamically typed**. Values carry their type at runtime. There is no compile-time type annotation syntax in the base language.

| Type | Description | Yoruba Name |
|------|-------------|-------------|
| `Int` | 64-bit signed integer | `Nọ́mbà` |
| `Float` | 64-bit IEEE 754 double | `Ìpínpọ̀` |
| `Bool` | `otito` or `eke` | `Òtítọ́` |
| `String` | Immutable UTF-8 sequence | `Ọ̀rọ̀` |
| `Null` | Absence of value (`ofo`) | `Òfò` |
| `List` | Ordered mutable sequence | `Àkójọ` |
| `Map` | Unordered key-value store | `Àpótí` |
| `Function` | Callable value | `Iṣẹ́` |
| `Ptr` | Raw pointer (embedded/unsafe only) | `Ìtọ́kasí` |

### 4.2 Runtime Type Inspection

```
Ofun.iru(42)           # => "Int"
Ofun.iru(3.14)         # => "Float"
Ofun.iru("hello")      # => "String"
Ofun.iru(otito)        # => "Bool"
Ofun.iru(ofo)          # => "Null"
Ofun.iru([1,2,3])      # => "List"
Ofun.iru({a: 1})       # => "Map"
Ofun.iru(ese(){})      # => "Function"
```

### 4.3 Null (`ofo`) `[DEFINED]`

`ofo` is its own type with a single value. A variable declared without initialization holds `ofo`. Accessing a field or method on `ofo` **MUST** produce a `NullReferenceError` unless the optional chaining operator `?.` is used.

```
ayanmo x;              # x is ofo
x.length               # NullReferenceError
x?.length              # => ofo  (safe — no error)
x?.foo?.bar            # => ofo  (chains safely)
```

### 4.4 Integers `[DEFINED]`

The `Int` type is a 64-bit signed integer. Range: `−2^63` to `2^63 − 1`.

**Overflow behavior:**
- **Debug builds:** overflow **MUST** produce an `OverflowError`.
- **Release builds:** overflow **MUST** wrap using two's complement.
- Implementations **MUST** document which mode is active.

Integer division truncates toward zero:

```
9 / 4      # => 2
-9 / 4     # => -2
```

The `%` operator's result has the **sign of the dividend**:

```
-7 % 3     # => -1
7 % -3     # =>  1
```

### 4.5 Floats `[DEFINED]`

The `Float` type is IEEE 754 double precision. `NaN` and `Infinity` are valid `Float` values.

```
1.0 / 0.0    # => Infinity
-1.0 / 0.0   # => -Infinity
0.0 / 0.0    # => NaN
```

Integer `0 / 0` (integer division) **MUST** produce `DivisionByZeroError`.  
Float `0.0 / 0.0` **MUST** produce `NaN`, never an error.

### 4.6 Mixed Arithmetic `[DEFINED]`

When `Int` and `Float` appear together in arithmetic, the `Int` is promoted to `Float`. The result is `Float`.

```
3 + 1.5    # => 4.5  (Float)
```

### 4.7 Strings `[DEFINED]`

Strings are **immutable** sequences of Unicode code points, stored as UTF-8. Length and indexing are in **code points**, not bytes.

```
"Ifá".length     # => 3  (not 4 bytes)
"Ifá"[0]         # => "I"
"Ifá"[-1]        # => "á"  (negative index counts from end)
"Ifá"[10]        # => ofo  (out of bounds returns ofo, never an error)
```

String concatenation with `+`: if **either** operand of `+` is a `String`, the other is coerced to its string representation.

```
"count: " + 5      # => "count: 5"
otito + "!"        # => "otito!"
```

### 4.8 Lists `[DEFINED]`

Lists are **ordered, mutable** sequences. Elements may be of mixed types. Zero-based indexing. Negative indices count from the end.

```
ayanmo nums = [1, 2, 3];
nums[0]             # => 1
nums[-1]            # => 3
nums[99]            # => ofo      (out of bounds: ofo, not an error)
nums.push(4);       # mutates in place
nums.pop();         # returns and removes last element
nums.length         # => 3
```

**Reference semantics:** assignment copies the reference, not the value.

```
ayanmo a = [1, 2, 3];
ayanmo b = a;          # b and a point to the same list
b.push(4);
Irosu.fo(a);           # [1, 2, 3, 4]  — a was mutated via b
```

### 4.9 Maps `[DEFINED]`

Maps are **unordered** collections of key-value pairs. Keys **MUST** be `String`s. Values may be any type.

```
ayanmo p = { name: "Àdé", age: 30 };
p.name               # => "Àdé"     (dot access)
p["age"]             # => 30        (bracket access)
p.missing            # => ofo       (missing key returns ofo, never an error)
p.job = "dev";       # adds new key — valid
```

**Map equality** is structural: two maps are equal if they have the same keys with pairwise equal values (deep recursive comparison).

---

## 5. Truthiness `[DEFINED]`

Every value in IfáLang has a truthiness. This governs `ti`, `nigba`, `!`, `&&`, `||`, and the `??` operator. The complete, exhaustive truthiness table:

| Value | Truthy? | Notes |
|-------|---------|-------|
| `ofo` | **false** | The only null value. Always falsy. |
| `eke` | **false** | Boolean false. |
| `otito` | **true** | Boolean true. |
| `Int: 0` | **false** | Zero is falsy. |
| `Int: non-zero` | **true** | Any non-zero integer. |
| `Float: 0.0` | **false** | Zero float is falsy. |
| `Float: NaN` | **false** | NaN is falsy. Not a number, not a truth. |
| `Float: Infinity` | **true** | Infinity is truthy. |
| `Float: other` | **true** | Any non-zero, non-NaN float. |
| `String: ""` | **false** | Empty string is falsy. |
| `String: non-empty` | **true** | Any string with at least one character. Including `"0"` and `"false"`. |
| `List: []` | **false** | Empty list is falsy. |
| `List: non-empty` | **true** | Any list with at least one element. |
| `Map: {}` | **false** | Empty map is falsy. |
| `Map: non-empty` | **true** | Any map with at least one key. |
| `Function` | **true** | All function values are truthy. |
| `Ptr` | **true** | All pointer values are truthy (embedded only). |

> **Note:** `"0"` is truthy. `"false"` is truthy. Only the empty string `""` is falsy. This is intentional — stringly-typed truthiness is a footgun.

---

## 6. Operators and Expressions `[DEFINED]`

### 6.1 Statement Termination

Statements end with `;`. Semicolons are required after expression statements, declarations, and `pada`. They are **not** required after blocks (`{ }`).

```
ayanmo x = 5;         # required
Irosu.fo(x);          # required
ti x > 0 {            # not required after {
  Irosu.fo("pos");    # required
}                     # not required after }
```

### 6.2 Operator Precedence

From lowest (1) to highest (13). Implementations **MUST** match this table exactly.

| Precedence | Operator(s) | Associativity | Notes |
|-----------|-------------|---------------|-------|
| 1 | `=` `+=` `-=` `*=` `/=` `%=` | Right | Assignment and compound assignment |
| 2 | `\|\|` `tabi` | Left | Short-circuit OR |
| 3 | `&&` `ati` | Left | Short-circuit AND |
| 4 | `==` `!=` | Left | Structural equality |
| 5 | `<` `<=` `>` `>=` | Left | Comparison |
| 6 | `??` | Right | Null coalescing |
| 7 | `+` `-` | Left | Addition, subtraction, string concat |
| 8 | `*` `/` `%` | Left | Multiplication, division, modulo |
| 9 | `**` | Right | Exponentiation — right-associative |
| 10 | `!` `ko` `-` (unary) | Prefix | Logical NOT, unary minus |
| 11 | `&` `*` (unsafe) | Prefix | Address-of, dereference (ailewu only) |
| 12 | `.` `?.` | Left | Member access, optional chaining |
| 13 | `()` `[]` | Left | Call, index |

### 6.3 Short-Circuit Evaluation `[DEFINED]`

`&&` (`ati`): if the left operand is falsy, the right operand is **not evaluated**. Returns the left operand (not `eke`).

`||` (`tabi`): if the left operand is truthy, the right operand is **not evaluated**. Returns the left operand (not `otito`).

```
eke && side_effect()    # side_effect() is never called
otito || side_effect()  # side_effect() is never called

# These return the actual values, not booleans:
ofo || "default"        # => "default"
5 || "default"          # => 5
```

### 6.4 Null Coalescing `??` `[DEFINED]`

Returns the left operand if it is not `ofo`; otherwise evaluates and returns the right operand.

```
ayanmo name = user.name ?? "anonymous";
```

This is different from `||`: `0 ?? "default"` returns `0`. `0 || "default"` returns `"default"`.

### 6.5 Equality Semantics `== !=` `[DEFINED]`

Equality is **structural** (deep value comparison), not reference equality, for all types except `Object` and `Function`.

| Types compared | Rule |
|----------------|------|
| Two `Int` | Equal if same numeric value |
| `Int` and `Float` | `Int` promoted to `Float`; compare as floats. `3 == 3.0` is `otito`. |
| Two `Float` | IEEE 754. `NaN == NaN` is **`eke`**. `-0.0 == 0.0` is `otito`. |
| Two `String` | Equal if same code point sequence |
| Two `Bool` | Equal if both `otito` or both `eke` |
| `ofo == ofo` | Always `otito` |
| `ofo == anything` | Always `eke` |
| Two `List` | Equal if same length and pairwise `==` elements (recursive) |
| Two `Map` | Equal if same keys and pairwise `==` values (recursive) |
| Two `Function` | Always `eke` — functions are never equal |

> ⚠️ **NaN is never equal to anything, including itself.** `NaN == NaN` is `eke`. Implementations **MUST NOT** normalize NaN for equality.

### 6.6 Comparison Semantics `< <= > >=` `[DEFINED]`

Defined for `Int`, `Float`, and `String`. Comparing incompatible types **MUST** produce a `TypeError`.

| Types | Rule |
|-------|------|
| `Int` and `Int` | Numeric |
| `Float` and `Float` | IEEE 754 numeric |
| `Int` and `Float` | `Int` promoted to `Float` |
| `String` and `String` | Lexicographic by Unicode code point value |
| Any other combination | `TypeError` |

### 6.7 Exponentiation `**` `[DEFINED]`

Right-associative. `2 ** 3 ** 2` is `2 ** (3 ** 2)` = `2 ** 9` = `512`.

Negative exponents return `Float`: `2 ** -1` => `0.5`.

---

## 7. Variables and Scope `[DEFINED]`

### 7.1 Declaration

```
ayanmo x = 10;          # mutable — x can be reassigned
ayanfe PI = 3.14159;    # immutable — PI cannot be reassigned

PI = 3;                 # TypeError: cannot assign to constant
```

A variable declared without an initializer holds `ofo`:

```
ayanmo x;               # x = ofo
```

### 7.2 Dynamic Typing

A variable may hold any type at any time. The declaration does not fix the type.

```
ayanmo x = 5;           # x: Int
x = "hello";            # x: String — valid
x = [1, 2, 3];          # x: List — valid
```

### 7.3 Scoping Rules

IfáLang uses **lexical (static) scoping**. Every `{ }` creates a new scope. A binding is visible from its declaration point to the end of its enclosing block.

```
ti otito {
  ayanmo x = 5;
}
Irosu.fo(x);            # ReferenceError: x is not defined here
```

### 7.4 Shadowing `[DEFINED]`

An inner scope **MAY** shadow an outer binding with the same name. The outer binding is unmodified — it is hidden within the inner scope's lifetime only.

```
ayanmo x = 1;
ti otito {
  ayanmo x = 2;         # shadows outer x
  Irosu.fo(x);          # 2
}
Irosu.fo(x);            # 1  — outer x unchanged
```

### 7.5 No Hoisting `[DEFINED]`

Variables are **not** hoisted. Accessing a variable before its declaration **MUST** produce a `ReferenceError`.

```
Irosu.fo(x);            # ReferenceError
ayanmo x = 5;
```

### 7.6 Assignment Semantics `[DEFINED]`

The right-hand side is evaluated completely before the binding is updated.

**Primitive types** (Int, Float, Bool, String) have **value semantics**. Assignment copies the value.

**Compound types** (List, Map, Object) have **reference semantics**. Assignment copies the reference.

```
ayanmo a = [1, 2, 3];
ayanmo b = a;            # both point to the same list
b.push(4);
Irosu.fo(a);             # [1, 2, 3, 4]
```

---

## 8. Functions and Closures `[DEFINED]`

### 8.1 Named Functions

```
ese add(a, b) {
  pada a + b;
}
```

Functions are **first-class values** — they can be assigned, passed, and returned.

```
ayanmo myAdd = add;
Irosu.fo(myAdd(3, 4));    # 7
```

### 8.2 Anonymous Functions (Lambdas)

```
ayanmo double = |x| { x * 2; };
ayanmo triple = |x| x * 3;      # shorthand: single expression, no braces
```

### 8.3 Parameters

#### Required Parameters

Parameters are positional and required by default. Too few arguments: `TypeError`. Too many arguments: the extra arguments are silently ignored.

#### Default Parameters

```
ese greet(name, greeting = "Ẹ káàbọ̀") {
  Irosu.fo($"{greeting}, {name}!");
}

greet("Àdé");                    # Ẹ káàbọ̀, Àdé!
greet("Àdé", "Hello");           # Hello, Àdé!
```

Default values are evaluated **at call time**, not at definition time. Each call gets a fresh evaluation of the default expression.

#### Variadic Parameters

The last parameter may be prefixed with `...` to collect remaining arguments into a `List`.

```
ese sum(...nums) {
  pada nums.apapo();
}
sum(1, 2, 3, 4)     # => 10
```

### 8.4 Return Values `[DEFINED]`

A function returns:
1. The value of an explicit `pada` statement, or
2. The value of the last expression in its body (implicit return), or
3. `ofo` if the body ends with a statement that produces no value.

```
ese square(x) { x * x; }         # implicit return: x*x
ese nothing() { Irosu.fo("hi"); } # implicit return: ofo
```

### 8.5 Closures `[DEFINED]`

A function that references a variable from an enclosing scope creates a **closure**. The closure captures the **binding** (by reference), not the value at the time of capture. Mutations to the captured variable are visible to the closure, and mutations by the closure are visible to the outer scope.

```
ese make_counter() {
  ayanmo count = 0;
  pada || {
    count = count + 1;
    pada count;
  };
}

ayanmo counter = make_counter();
counter()    # => 1
counter()    # => 2
counter()    # => 3
```

The outer `count` is shared between `make_counter`'s scope and the returned closure. The closure holds a reference to the upvalue slot, not a snapshot.

#### Upvalue Lifetime

Upvalues outlive their defining scope when a closure escapes. The captured variable's memory is kept alive as long as any closure that captured it is alive.

### 8.6 Recursion `[DEFINED]`

Named functions may call themselves. With TCO, self-recursive calls in tail position consume no additional stack — they execute in constant stack space. Without TCO, the stack limit is at least 500 frames before `StackOverflowError`.

### 8.7 Tail Call Optimization (TCO) `[DEFINED]`

TCO is **required** for self-recursive tail calls. Mutual recursion between two functions annotated `#[tco]` is also required. Any other form of tail call (non-recursive, cross-module) **MAY** be optimized but is not required.

**Formal definition of tail position:** An expression is in tail position in a function body if and only if:
1. It is the last expression in the function body (implicit return), **or**
2. It is the expression argument of a `pada` statement, **and**
3. No `jowo`, `gbiyanju`, or resource-opening call appears between it and the function's return point.

```
ese factorial_acc(n, acc) {
  ti n <= 1 { pada acc; }
  pada factorial_acc(n - 1, n * acc);    # tail position — TCO applies
}

ese count_down(n) {
  ti n == 0 { pada "done"; }
  Irosu.fo(n);                            # side effect — NOT tail position
  pada count_down(n - 1);                 # still tail position (side effect was before)
}

ese bad(n) {
  pada bad(n - 1) + 1;                   # NOT tail position — addition after the call
}
```

**The `TailCall` opcode** (§17.2, byte `0x54`): replaces the current call frame with a new one rather than pushing a new frame onto the stack. The compiler **MUST** emit `TailCall` instead of `Call` for any call proven to be in tail position. The VM **MUST** implement this by reusing the current frame slot.

**The `#[tco]` attribute for mutual recursion:**

```
#[tco]
ese is_even(n) {
  ti n == 0 { pada otito; }
  pada is_odd(n - 1);       # tail call to a different function
}

#[tco]
ese is_odd(n) {
  ti n == 0 { pada eke; }
  pada is_even(n - 1);
}
```

When `#[tco]` is present, the compiler **MUST** verify that every `pada` in the function is either a tail call or a return of a non-call value. If any non-tail `pada` exists in a `#[tco]` function, the Babalawo **MUST** produce a compile error:

```
error[Ọ̀sá] main.ifa:8:3
  #[tco] function 'process' has a non-tail call at line 8.
  TCO cannot be guaranteed. Remove #[tco] or restructure the function.
```

**Transpiler behavior:** `ifa build` **MUST** emit a Rust `loop { ... continue }` pattern for self-recursive tail calls, not a recursive Rust function call. This ensures the generated code also has O(1) stack usage.

**Recursion depth without TCO:** For functions that are not TCO-eligible, the maximum stack depth is at least 500 frames. Exceeding this **MUST** produce a `StackOverflowError`.

### 8.7 Higher-Order Functions

```
ese apply(f, x) { pada f(x); }
apply(|x| x * 2, 5)             # => 10

ayanmo pipeline = [|x| x+1, |x| x*2];
pipeline[1](5)                   # => 10
```

---

## 9. Protocol-Oriented Data Model (Ratified) `[DEFINED]`
 
Ifá-Lang does not have classes, inheritance, or object-oriented hierarchies. This is a permanent language design decision ratified on 2026-04-07. Polymorphism and data grouping are achieved through **structural subtyping** (Map shape checking) and **Domain Protocols**.
 
### 9.1 Data Grouping (Maps)
 
In place of classes, data is grouped in Map literals. Methods are functions that take a Map as their first argument by convention.
 
```
# Data definition
ayanmo circle = {
  radius: 5,
  color:  "pupa"
};
 
# Method definition (functional/procedural)
ese area(c) {
  pada Obara.pi() * c.radius ** 2;
}
 
Irosu.fo(area(circle)); # => 78.539...
```
 
### 9.2 Structural Subtyping (`ifa-babalawo`)
 
The Babalawo performs structural shape checking. A function that requires a specific field on a map will accept any map that possesses that field.
 
```
ese say_name(p) {
  Irosu.fo(p.name);
}
 
ayanmo person = { name: "Àdé", age: 30 };
ayanmo dog    = { name: "Ajá", breed: "Local" };
 
say_name(person); # Valid
say_name(dog);    # Valid
```
 
### 9.3 Domain Protocols
 
The 16 Odù Domains act as protocols. A value "implements" a domain protocol if it satisfies the structural requirements expected by that domain's methods.
 
### 9.4 Rationale for Removal
 
- **Philosophical Alignment:** The 16 Odù are sibling domains, not hierarchical. Inheritance contradicts the Ifá model.
- **Zero-Cost:** Class vtables require runtime dynamic dispatch. Maps use static structural analysis.
- **Complexity Reduction:** Removing OOP eliminates `ara` (self) ambiguity, `iya` (super) complexity, and fragile inheritance chains.

---

## 10. Control Flow `[DEFINED]`

### 10.1 Conditionals

```
ti condition {
  # truthy branch
} bibẹkọ ti other {
  # chained condition
} bibẹkọ {
  # fallback
}
```

The condition is evaluated for truthiness (§5). Any number of `bibẹkọ ti` arms. `bibẹkọ` is optional.

### 10.2 While Loop

```
ayanmo i = 0;
nigba i < 10 {
  Irosu.fo(i);
  i = i + 1;
}
```

If the condition is falsy on first evaluation, the body never executes.

### 10.3 For Loop

#### Range Form

```
fun i in 0..10 {     # 0, 1, 2, ..., 9  (exclusive end)
  Irosu.fo(i);
}

fun i in 0..=10 {    # 0, 1, 2, ..., 10 (inclusive end)
  Irosu.fo(i);
}
```

#### Collection Form

```
fun item in my_list {
  Irosu.fo(item);
}

fun key, value in my_map {
  Irosu.fo($"{key}: {value}");
}
```

### 10.4 Loop Control

`duro` exits the **innermost** enclosing loop immediately. `tesiwaju` skips to the **next iteration** of the innermost loop.

```
fun i in 0..20 {
  ti i % 2 == 0 { tesiwaju; }     # skip even numbers
  ti i > 15 { duro; }             # stop when i > 15
  Irosu.fo(i);
}
```

`duro` with a value (loop as expression):

```
ayanmo result = nigba otito {
  ayanmo x = compute();
  ti x > 100 { duro x; }
};
```

---

## 11. Pattern Matching `[DEFINED]`

### 11.1 Syntax

```
ayanmo result = ode value {
  0           => "zero",
  1 | 2       => "one or two",
  3..10       => "three to nine",
  Int         => "some other integer",
  String      => "a string",
  [a, b]      => $"list with two elements: {a}, {b}",
  {name: n}   => $"map with name: {n}",
  _           => "anything else"
};
```

### 11.2 Pattern Types

| Pattern | Syntax | Matches |
|---------|--------|---------|
| Literal | `0`, `"hello"`, `otito` | Exact value (structural equality) |
| Or | `A \| B` | Either A or B |
| Range | `3..10` | Integers in `[3, 9]` (exclusive end) |
| Inclusive range | `3..=10` | Integers in `[3, 10]` |
| Type | `Int`, `Float`, `String`, `Bool`, `List`, `Map` | Value of that type |
| List destructure | `[a, b, ...rest]` | List with at least 2 elements |
| Map destructure | `{key: binding}` | Map containing that key |
| Binding | `name @ pattern` | Matches pattern, binds value to `name` |
| Wildcard | `_` | Anything; discards value |

### 11.3 Exhaustiveness `[DEFINED]`

An `ode` expression **MUST** be exhaustive. A non-exhaustive match is a **compile-time error**, not a runtime warning. The Babalawo **MUST** reject any `ode` that cannot be proven to cover all possible values of the matched expression.

```
error[Òfún] main.ifa:12:3
  Match on Int is not exhaustive. The Creator demands completeness.
  Wisdom: A kì í fi ọ̀nà kan ṣoṣo sọ̀rọ̀ — one never speaks with only one path.
  Help: Add a wildcard arm  _  =>  or cover all cases explicitly.
```

An `ode` is proven exhaustive when **any** of the following hold:

1. **Wildcard arm present:** `_` as the last arm always satisfies exhaustiveness.
2. **Bool coverage:** Both `otito` and `eke` arms are present.
3. **Type-level coverage:** A type arm (`Int`, `Float`, `String`, `Bool`, `List`, `Map`) that covers all values of the matched expression's static type.
4. **Full range coverage:** Arms whose ranges tile the entire value space of a sized integer type — e.g., `0..=255` for `u8`, `-128..=127` for `i8`. This applies to the embedded tier's sized types only. Dynamic `Int` is unbounded and cannot be exhausted by ranges alone.
5. **Null coverage:** If the expression may be `ofo`, either an explicit `ofo =>` arm or `_` is required.

Arms that do **not** constitute exhaustiveness on their own:
- Any number of integer literals on a dynamic `Int` (there are always more integers)
- Any number of string literals on a `String`
- A list or map destructure arm without a wildcard for unmatched shapes

```
# Compile error — non-exhaustive on Int:
ode x {
  0 => "zero",
  1 => "one"
}

# Correct — type arm covers remaining Int:
ode x {
  0 => "zero",
  1 => "one",
  Int => "other"
}

# Correct — wildcard:
ode x { 0 => "zero", _ => "other" }

# Correct — Bool, both arms:
ode flag { otito => "yes", eke => "no" }

# Correct — u8 exhaustive range (embedded tier):
ode byte_val { 0..=127 => "low", 128..=255 => "high" }
```

**Runtime behavior:** Because exhaustiveness is a compile-time guarantee, a conforming runtime **MUST NOT** produce a `MatchError` for a program that passed the Babalawo. A `MatchError` at runtime indicates a Babalawo bug or a non-conforming implementation. The error kind is retained in §19 for non-conforming runtimes only.

---

## 12. Error Handling `[DEFINED]`

### 12.1 Try/Catch

```
gbiyanju {
  ayanmo data = Odi.ka("/etc/passwd");
  Irosu.fo(data);
} gba e {
  Irosu.fo($"Error: {e.message}  kind: {e.kind}");
}
```

The `gba` block receives a structured error value (see §19.1). The identifier `e` is scoped to the `gba` block.

### 12.2 Throw

```
ese divide(a, b) {
  ti b == 0 { ta "Division by zero"; }
  pada a / b;
}
```

`ta` raises a `UserError`. The string argument becomes the `message` field. The current source location becomes the `line` field.

### 12.3 Error Propagation `[PARTIAL]`

The `?` operator propagates errors upward:

```
ese read_config(path) {
  ayanmo data = Odi.ka(path)?;     # if Odi.ka fails, return the error immediately
  pada parse_config(data);
}
```

`?` is syntactic sugar for: if the expression produced an error, `pada` that error from the current function.

---

## 13. The 16 Odù Standard Domains `[DEFINED]`

Standard domains are always available. No import is required. The call syntax is `DomainName.method(args)`.

Each domain corresponds to a principal Odù of Ifá. The binary encoding of the Odù (Teeté pattern) determines the domain's ID byte in `CallOdu` opcodes.

### Domain Reference

| Binary | Yoruba Name | Domain | Core Responsibility |
|--------|------------|--------|---------------------|
| `1111` | Ọ̀gbè (Ogbe) | System / Lifecycle | CPU info, env vars, process args |
| `0000` | Ọ̀yẹ̀kú (Oyeku) | Exit / Sleep | Process exit, sleep, timer |
| `0110` | Ìwòrì (Iwori) | Time / Iteration | Clock, date, ranges, iterators |
| `1001` | Òdí (Odi) | Files / Database | File I/O, SQLite, key-value store |
| `1100` | Ìrosù (Irosu) | Console I/O | Print, read input, terminal colors |
| `0011` | Ọ̀wọ́nrín (Owonrin) | Randomness | PRNG, UUID, shuffle |
| `1000` | Ọ̀bàrà (Obara) | Math | Arithmetic, trig, statistics |
| `0001` | Ọ̀kànràn (Okanran) | Assertions / Debug | Assert, debug output, panic |
| `1110` | Ògúndá (Ogunda) | Collections / Process | Array ops, map/filter, child processes |
| `0111` | Ọ̀sá (Osa) | Concurrency | Async tasks, channels, sleep |
| `0100` | Ìká (Ika) | Strings | String manipulation, regex |
| `0010` | Òtúúrúpọ̀n (Oturupon) | Math (inverse) | Subtraction, division, rounding |
| `1011` | Òtúrá (Otura) | Networking | HTTP, TCP/UDP, sockets |
| `1101` | Ìrẹtẹ̀ (Irete) | Crypto / Compression | SHA-256, HMAC, base64, zlib |
| `1010` | Ọ̀ṣẹ́ (Ose) | Graphics / Canvas | Drawing, GUI, canvas |
| `0101` | Òfún (Ofun) | Capabilities / Reflection | Type inspection, permission queries |

---

### 13.1 Irosu — Console I/O

```
Irosu.fo(value)              # print value with newline
Irosu.fo_inline(value)       # print value without newline
Irosu.ka(prompt)             # read line from stdin, returns String
Irosu.ka_airi(prompt)        # read password (no echo)
Irosu.awọ(text, color)       # colored terminal output
                             # color: "pupa"(red), "ewe"(green), "ofeefee"(yellow)
                             #        "buluu"(blue), "funfun"(white)
```

### 13.2 Obara — Mathematics

```
Obara.kun(a, b)              # a + b
Obara.isodipupo(a, b)        # a * b
Obara.agbara(base, exp)      # base ** exp
Obara.gbongbo(x)             # sqrt(x)
Obara.sin(x)                 # sine (radians)
Obara.cos(x)                 # cosine (radians)
Obara.tan(x)                 # tangent (radians)
Obara.pi()                   # π (3.14159...)
Obara.e()                    # Euler's number
Obara.apapo(list)            # sum of list
Obara.agbedemeji(list)       # mean of list
Obara.iyatọ(list)            # standard deviation
Obara.log(x)                 # natural log
Obara.log10(x)               # base-10 log
Obara.min(a, b)              # minimum of two values
Obara.max(a, b)              # maximum of two values
Obara.abs(x)                 # absolute value
```

### 13.3 Ika — Strings

```
Ika.gigun(s)                 # string length in code points
Ika.oke(s)                   # uppercase
Ika.isale(s)                 # lowercase
Ika.pin(s, sep)              # split string by separator → List
Ika.so(parts, sep)           # join list of strings with separator
Ika.ropo(s, from, to)        # replace all occurrences
Ika.ni(s, sub)               # contains substring → Bool
Ika.bere(s, prefix)          # starts with → Bool
Ika.pari(s, suffix)          # ends with → Bool
Ika.ge(s, start, end)        # slice [start, end)
Ika.dara(s)                  # trim whitespace
Ika.ṣe_eeka(s)               # parse to Int → Int | ofo
Ika.ṣe_owo(s)                # parse to Float → Float | ofo
Ika.ṣe_ọrọ(val)             # convert any value to String
```

### 13.4 Odi — Files and Database

```
Odi.ka(path)                 # read entire file → String | Error  (auto-closes)
Odi.si(path)                 # open file handle for streaming → Handle (MUST call Odi.pa)
Odi.pa(handle)               # close a file handle opened with Odi.si
Odi.kọ(path, data)           # write file (overwrite, auto-closes)
Odi.fi_kun(path, data)       # append to file (auto-closes)
Odi.wa(path)                 # file exists → Bool
Odi.paarẹ(path)              # delete file
Odi.ṣẹda(path)               # create directory (and parents)
Odi.akojọ(path)              # list directory → List<String>
Odi.ṣii_db(path)             # open SQLite database
```

> **Lifecycle note:** `Odi.ka()`, `Odi.kọ()`, and `Odi.fi_kun()` open, operate, and close internally — they carry no Ìwà lifecycle obligation. `Odi.si()` returns a handle that **MUST** be closed with `Odi.pa()`. Omitting `Odi.pa()` is an `UNCLOSED_RESOURCE` error flagged by the Ìwà Engine.

### 13.5 Oyeku — Exit and Sleep

```
Oyeku.jade(code)             # exit process with code
Oyeku.sun(ms)                # sleep for milliseconds (blocking)
Oyeku.sun_ìsẹ́jú(sec)        # sleep for seconds
Oyeku.duro()                 # halt — lifecycle closer for Ogbe.bi() and Ogbe.bere()
```

### 13.6 Iwori — Time and Ranges

```
Iwori.bayi()                 # current Unix timestamp (ms) → Int
Iwori.epoch()                # current Unix timestamp (seconds) → Float
Iwori.odun()                 # current year → Int
Iwori.osu()                  # current month (1-12) → Int
Iwori.ọjọ()                  # current day of month → Int
Iwori.akoko(fmt)             # formatted datetime string
Iwori.iwọn(start, end)       # range iterator [start, end)
Iwori.iwọn_igbese(s, e, step)# range with step
Iwori.yipo()                 # begin a tracked loop (Ìwà lifecycle opener)
Iwori.pada()                 # end a tracked loop — lifecycle closer for Iwori.yipo()
```

### 13.7 Owonrin — Randomness

```
Owonrin.nọ́mbà(min, max)      # random Int in [min, max]
Owonrin.ìpínpọ̀()             # random Float in [0.0, 1.0)
Owonrin.bẹ́ẹ̀_bẹ́ẹ̀(prob)       # random Bool with given probability
Owonrin.yàn(list)            # random element from list
Owonrin.daru(list)           # shuffle list in place
Owonrin.uuid()               # random UUID v4 string
```

> ⚠️ `Owonrin` uses a PRNG seeded from system time. **Do not use for security-sensitive randomness.** Use `Irete.random_bytes(n)` instead.

### 13.8 Ogunda — Collections and Processes

```
Ogunda.fi_kun(list, val)     # push to list
Ogunda.yọ(list)              # pop from list
Ogunda.to_run(list)          # sort list in place
Ogunda.sare(list, fn)        # filter list: keep elements where fn returns truthy
Ogunda.yipada(list, fn)      # map list: transform each element
Ogunda.din(list, fn, init)   # reduce list to a single value
Ogunda.wa(list, fn)          # find first matching element | ofo
Ogunda.gbogbo(list, fn)      # all elements match fn → Bool
Ogunda.kan(list, fn)         # any element matches fn → Bool
Ogunda.so(a, b)              # concatenate two lists → new List
Ogunda.ise(cmd, args)        # spawn child process (absolute path required)
Ogunda.ge(size)              # allocate a buffer of `size` bytes → Buffer (MUST call Irete.tu)
Ogunda.da(type_name, args)   # create a managed resource object (MUST call Irete.tu)
```

### 13.9 Osa — Concurrency

```
Osa.ise(task)                # spawn async task → Future
Osa.ọ̀nà(buf_size)           # create channel → {send, recv}
Osa.duro(ms)                 # async sleep (use inside daro functions)
Osa.akoko(task, ms)          # run task with timeout → result | TimeoutError
Osa.gbogbo(futures)          # await all futures → List of results
Osa.eyikeyi(futures)         # await first completed future
```

### 13.10 Otura — Networking

```
Otura.gba(url)               # HTTP GET → {status, body, headers}
Otura.fi(url, body)          # HTTP POST → {status, body, headers}
Otura.fi_json(url, obj)      # HTTP POST with JSON body
Otura.so(addr, port)         # TCP connect → Connection (MUST call Otura.pa)
Otura.de(addr, port)         # TCP bind/listen → Listener (MUST call Otura.pa)
Otura.gba_de(addr, port)     # TCP listen (convenience alias for Otura.de)
Otura.pa(conn)               # close a Connection or Listener opened with so/de
```

### 13.11 Irete — Crypto and Compression

```
Irete.sha256(data)           # SHA-256 hash → hex String
Irete.sha512(data)           # SHA-512 hash → hex String
Irete.hmac(key, data)        # HMAC-SHA256 → hex String
Irete.random_bytes(n)        # cryptographically random bytes → List<Int>
Irete.base64_encode(data)    # base64 encode → String
Irete.base64_decode(s)       # base64 decode → String
Irete.compress(data, level)  # zlib compress → String (binary)
Irete.decompress(data)       # zlib decompress → String
Irete.tu(resource)           # free a resource allocated with Ogunda.ge() or Ogunda.da()
```

### 13.12 Okanran — Assertions and Debug

```
Okanran.ẹri(cond, msg)       # assert condition; throws AssertionFailed if false
Okanran.ẹri_dọgba(a, b)      # assert a == b
Okanran.ẹri_iyatọ(a, b)      # assert a != b
Okanran.ta(msg)              # throw UserError (equivalent to `ta` keyword)
Okanran.ṣayẹwo(label, val)   # debug-print with label, return val unchanged
Okanran.kilo_ibi()           # return current source location string
```

### 13.13 Ogbe — System and Lifecycle

```
Ogbe.akokoro()               # CPU core count → Int
Ogbe.iranti()                # memory stats → {total, used, free} in bytes
Ogbe.ayika(key)              # get environment variable → String | ofo
Ogbe.ariyanjiyan()           # command-line args → List<String>
Ogbe.orukọ_eto()             # OS name → String ("linux", "macos", "windows")
Ogbe.akoko_eto()             # system uptime in seconds → Int
Ogbe.bi()                    # initialize a system resource (auto-closed at program end)
Ogbe.bere()                  # start a long-running system service (auto-closed at program end)
```

### 13.14 Ofun — Capabilities and Reflection

```
Ofun.iru(val)                # type name → String
Ofun.ni_iru(val, name)       # value is of type name → Bool
Ofun.le(capability)          # capability check → Bool
Ofun.ju_silẹ(capability)     # drop capability (cannot be re-acquired)
Ofun.gbogbo_iru()            # list all runtime type names → List<String>
Ofun.da(type_name, args)     # create a managed object (MUST call Ofun.pa)
Ofun.pa(obj)                 # destroy a managed object created with Ofun.da
```

Capabilities follow deny-by-default semantics. A program **MUST** explicitly be granted a capability to use it. Capabilities **MUST** be checked at every domain call boundary.

### 13.15 Ose — Graphics and Canvas

```
Ose.bẹ̀rẹ̀()                  # initialize graphics context
Ose.fa_apoti(title, w, h)    # draw a box with title
Ose.awọ(binary_code)         # set color from Odù binary (e.g. "1111" = Ogbe)
Ose.fa_ọrọ(x, y, text)      # draw text at position
Ose.fa_ila(x1, y1, x2, y2)  # draw line
Ose.bẹ̀rẹ̀_titẹ()             # poll for input events → Event | ofo
```

### 13.16 Oturupon — Inverse Mathematics

```
Oturupon.yọkuro(a, b)        # a - b
Oturupon.pin(a, b)           # a / b (float division)
Oturupon.iyoku(a, b)         # a % b
Oturupon.yipada_ami(x)       # negate x
Oturupon.ilẹ̀(x)              # floor(x) → Int
Oturupon.oke(x)              # ceil(x) → Int
Oturupon.yika(x, n)          # round to n decimal places
Oturupon.gige(x, lo, hi)     # clamp x to [lo, hi]
```

---

## 14. Module System `[PARTIAL]`

### 14.1 Module Identity

A module is a single `.ifa` file. Its identity is its path relative to the project root. Modules do not declare their own name — their path is their name.

### 14.2 Import Semantics — Namespace Isolation Model

IfáLang uses the **namespace isolation** model. Importing a module makes its exported symbols available under a name. It does **not** execute the module's top-level statements into the importing scope.

```
mu Circle from "./shapes";              # import default export
mu { Circle, Square } from "./shapes"; # named imports
mu * as Shapes from "./shapes";        # namespace import
mu Ika from "std/ika";                 # import standard module alias
```

### 14.3 Exports

```
fi ese helper(x) { ... }         # export a function
fi ayanfe MAX = 100;             # export a constant
fi odu Circle { ... }            # export a class
```

Everything not marked `fi` is private to the module.

### 14.4 Circular Import Detection

The module resolver **MUST** detect circular imports and produce a `CircularImportError` before any module code executes.

---

## 15. Concurrency `[PARTIAL]`

### 15.1 Async Functions

```
daro ese fetch(url) {
  ayanmo response = reti Otura.gba(url);
  pada response.body;
}
```
 
An `async` function returns a `Future`. `reti` inside an async function suspends until the future resolves.

### 15.2 Cooperative Yield — `jowo`

```
jowo 500000;      # yield for 500ms (duration in microseconds)
jowo 0;           # yield immediately with no duration hint
```

In the **hosted tier**: `jowo` is a scheduler hint. The runtime may sleep the current task for approximately the given duration.  
In the **embedded tier**: `jowo` is a hardware sleep. The MCU enters a low-power state for the given duration.

The transpiler **MUST** emit `std::thread::sleep(Duration::from_micros(N))` for `jowo N` in hosted Rust output.

---

## 16. The Embedded Tier `[DEFINED]`

The embedded runtime (`ifa-embedded`) is a restricted Tier 1 implementation for bare-metal targets.

### 16.1 Supported Features

| Feature | Supported |
|---------|-----------|
| Primitives: Int, Float, Bool | ✓ |
| Sized integers: u8, i8, u16, i16, u32, i32, u64, i64 | ✓ |
| Control flow: ti, nigba, fun, ode | ✓ |
| Functions (ese) | ✓ |
| Pointer operations (ailewu, &, *) | ✓ |
| MMIO bus access | ✓ |
| jowo (hardware sleep) | ✓ |

### 16.2 Restricted Features

| Feature | Restriction |
|---------|-------------|
| Classes (odu) | Not available — no heap, no vtable |
| Heap-allocated List / Map | Not available — static arrays only |
| String concatenation (allocating) | Not available |
| File I/O (Odi domain) | Not available — no filesystem |
| Networking (Otura domain) | Not available |
| Async/await (Osa domain) | Not available |
| Module system | Not available |

### 16.3 Sized Integer Types

The default `Int` type maps to `i32` on 32-bit targets and `i64` on 64-bit targets. Use explicit sized types when the width must be exact:

```
ailewu {
  ayanmo reg: u32 = 0x4000_1000;
  *reg = 1u32;
}
```

### 16.4 MMIO and Pointers

```
ailewu {
  ayanmo GPIO_OUT: *u32 = &0x4000_1000;
  *GPIO_OUT = 1u32;
  ayanmo val: u32 = *GPIO_OUT;
}
```

MMIO addresses at or above the configured `mmio_base` are routed to hardware via the attached HAL. Below `mmio_base`, addresses are normal memory.

### 16.5 Bytecode Format (Embedded)

The embedded bytecode format includes a version header absent from the main format:

```
Magic:   4 bytes  "IFAS"
Version: 1 byte   format version (currently 0x01)
Flags:   1 byte   feature flags
Payload: remainder
```

> ⚠️ **The main bytecode format (ifa runb) does not yet have a version header.** This is a known gap (issue A2 in the overlooked issues document). Adding a magic + version header to the main format is required before .ibc files can be safely distributed or cached.

---

## 17. Bytecode VM Specification `[DEFINED]`

### 17.1 Bytecode Format

All multi-byte numeric operands are **little-endian**. This is the universal encoding convention; any big-endian emission is a bug.

```
Bytecode file structure:
  [opcode: u8] [operands...]

String pool:
  Referenced by u16 index (little-endian)
  All opcodes that reference strings use: [opcode] [idx_lo: u8] [idx_hi: u8]

Integer constant pool:
  Referenced by u16 index (little-endian)
  Stored as i64 (8 bytes, little-endian)

Float constant pool:
  Referenced by u16 index (little-endian)
  Stored as f64 (8 bytes, IEEE 754, little-endian)
```

### 17.2 Complete Opcode Table

| Opcode | Byte | Operands | Stack Effect | Description |
|--------|------|----------|--------------|-------------|
| `PushInt` | 0x01 | `u16` const pool idx | → Int | Push integer from constant pool |
| `PushFloat` | 0x02 | `u16` const pool idx | → Float | Push float from constant pool |
| `PushStr` | 0x03 | `u16` string pool idx | → String | Push string from pool |
| `PushBool` | 0x04 | `u8` (0=false, 1=true) | → Bool | Push boolean literal |
| `PushNull` | 0x05 | — | → Null | Push `ofo` |
| `PushFn` | 0x06 | `u16` fn registry idx | → Function | Push function reference |
| `Pop` | 0x07 | — | val → | Discard top of stack |
| `Dup` | 0x08 | — | val → val val | Duplicate top of stack |
| `LoadLocal` | 0x10 | `u16` slot | → val | Load local variable by slot index |
| `StoreLocal` | 0x11 | `u16` slot | val → | Store to local variable slot |
| `LoadGlobal` | 0x12 | `u16` string pool idx | → val | Load global by name |
| `StoreGlobal` | 0x13 | `u16` string pool idx | val → | Store to global by name |
| `LoadUpvalue` | 0x14 | `u16` upvalue idx | → val | Load captured variable |
| `StoreUpvalue` | 0x15 | `u16` upvalue idx | val → | Store to captured variable |
| `MakeClosure` | 0x16 | `u16` fn idx, `u8` upvalue count, then upvalue descriptors | → Function | Create closure capturing upvalues |
| `Add` | 0x20 | — | a b → result | Arithmetic add or string concat |
| `Sub` | 0x21 | — | a b → result | Subtract |
| `Mul` | 0x22 | — | a b → result | Multiply |
| `Div` | 0x23 | — | a b → result | Divide |
| `Mod` | 0x24 | — | a b → result | Modulo |
| `Pow` | 0x25 | — | a b → result | Exponentiate |
| `Neg` | 0x26 | — | a → -a | Unary negate |
| `Eq` | 0x30 | — | a b → Bool | Structural equality |
| `Ne` | 0x31 | — | a b → Bool | Structural inequality |
| `Lt` | 0x32 | — | a b → Bool | Less than |
| `Le` | 0x33 | — | a b → Bool | Less than or equal |
| `Gt` | 0x34 | — | a b → Bool | Greater than |
| `Ge` | 0x35 | — | a b → Bool | Greater than or equal |
| `Not` | 0x36 | — | a → Bool | Logical NOT (via truthiness) |

> **Note on `&&` and `\|\|`:** These operators are **not** compiled to stack opcodes. They are compiled to conditional jump sequences using `JumpIfFalse`/`JumpIfTrue` that preserve the operand value on the stack. There are no `And` or `Or` opcodes. Any implementation of `&&`/`\|\|` as a binary stack operation that eagerly evaluates both operands and returns Bool is non-conforming — it violates the short-circuit and operand-return semantics defined in §6.3.
| `Jump` | 0x40 | `i32` signed offset | — | Unconditional jump relative to next instruction |
| `JumpIfFalse` | 0x41 | `i32` signed offset | val → val | Jump if top is falsy (leave on stack) |
| `JumpIfTrue` | 0x42 | `i32` signed offset | val → val | Jump if top is truthy (leave on stack) |
| `PopJumpIfFalse` | 0x43 | `i32` signed offset | val → | Jump if top is falsy and pop |
| `PopJumpIfTrue` | 0x44 | `i32` signed offset | val → | Jump if top is truthy and pop |
| `Call` | 0x50 | `u8` arg count | fn arg... → result | Call function |
| `CallMethod` | 0x51 | `u16` string pool idx, `u8` arg count | obj arg... → result | Call method on object |
| `CallOdu` | 0x52 | `u8` domain byte, `u16` string pool idx, `u8` arg count | arg... → result | Call standard domain method |
| `Return` | 0x53 | — | val → | Return from function, restore frame |
| `TailCall` | 0x54 | `u8` arg count | fn arg... → (reuses frame) | Replace current frame — no stack growth. Only valid for TCO-eligible calls. |
| `MakeList` | 0x60 | `u16` element count | val... → List | Collect N stack values into a List |
| `MakeMap` | 0x61 | `u16` pair count | k v... → Map | Collect N key-value pairs into a Map |
| `GetField` | 0x62 | `u16` string pool idx | obj → val | Get field by name |
| `SetField` | 0x63 | `u16` string pool idx | obj val → | Set field by name |
| `GetIndex` | 0x64 | — | coll idx → val | Index into List or Map |
| `SetIndex` | 0x65 | — | coll idx val → | Set index in List or Map |
| `REJECTED` | 0x70 | — | — | Legacy `MakeClass` (rejected) |
| `REJECTED` | 0x71 | — | — | Legacy `NewInstance` (rejected) |
| `REJECTED` | 0x72 | — | — | Legacy `Inherit` (rejected) |
| `REJECTED` | 0x73 | — | — | Legacy `GetSuper` (rejected) |
| `REJECTED` | 0x74 | — | — | Legacy `LoadSelf` (rejected) |
| `Throw` | 0x80 | — | val → | Raise error |
| `TryBegin` | 0x81 | `i32` catch offset | — | Mark start of try block |
| `TryEnd` | 0x82 | — | — | Mark end of try block |
| `Yield` | 0x90 | — | duration → | Cooperative yield |
| `Halt` | 0xFF | — | — | Stop execution, return top of stack |

### 17.3 Call Frame Layout

```
Frame:
  fn_id:      u32      — function registry index
  ip:         usize    — instruction pointer into bytecode
  base_slot:  usize    — index of first local in the value stack
  upvalues:   Vec<UpvalueRef>  — references to captured variables
```

On `Call`:
1. Pop `arg_count` arguments from the value stack.
2. Push a new frame: `base_slot = stack.len()`.
3. Push argument values into local slots `0..arg_count`.
4. Set `ip` to the start of the called function's bytecode.

On `Return`:
1. Pop the return value.
2. Pop all locals down to `base_slot`.
3. Restore the previous frame.
4. Push the return value onto the restored stack.

### 17.4 Source Location Threading `[PARTIAL]`

Every opcode that can produce a runtime error **SHOULD** carry an associated source location. The implementation **SHOULD** store a parallel array of `(line: u32, col: u16)` entries indexed by bytecode offset. Runtime errors **MUST** include the source location if it is available.

> ⚠️ Source location threading is not yet implemented. Until it is, runtime errors from `ifa runb` will not include line numbers. This is a known gap (issue A3).

---

## 18. Memory Model `[DEFINED]`

### 18.1 Automatic Memory Management

IfáLang uses automatic memory management via reference counting (`Arc`). The programmer does not allocate or free memory.

### 18.2 Reference Cycles

Reference cycles (a `Map` or `Object` that contains a reference to itself) will **leak memory** under pure reference counting. Programs **MUST NOT** intentionally create reference cycles.

The Babalawo (§22.2) detects **obvious self-referential literals** at compile time and emits a `REFERENCE_CYCLE` warning. Cycles created through mutation or indirect references are not detectable at compile time and will leak until a cycle-detection pass (mark-and-sweep collector) is added.

```
ayanmo a = {self: a};    # warning[Ìrẹtẹ̀] REFERENCE_CYCLE: 'a' references itself
```

A mark-and-sweep collector is a planned future addition.

### 18.3 String Interning `[PARTIAL]`

Identifiers and string literals **SHOULD** be interned. Interning means identical strings share the same backing allocation and compare by pointer. This is a performance optimization and **MUST NOT** change observable semantics. Currently unimplemented — all strings are cloned.

### 18.4 Performance Characteristics and Known Costs

IfáLang's hosted tier makes deliberate design tradeoffs that favor correctness, safety, and expressiveness over raw performance. The following are known, acknowledged costs — not bugs to be silently fixed:

| Subsystem | Cost | Mitigation |
|-----------|------|-----------|
| `Arc` reference counting | Atomic increment/decrement on every value copy | CoW via `Arc::make_mut` for mutation-heavy code; future optimization pass |
| Babalawo on every `ifa run`/`ifa runb` | Full AST walk before execution | `--no-check` flag bypasses Babalawo for trusted code in production; `BabalawoConfig::fast()` disables wisdom lookup |
| Àjọṣe reactive registry | Global subscription table traversal on every reactive source change | Subscription count is bounded by program structure; O(subscribers) per change |
| String cloning (no interning yet) | Every string comparison and domain method call clones | String interning (§18.3) will fix this; tracked as a known issue |
| All strings are `Arc<String>` | Two-level indirection for every string value | Symbol table with integer IDs planned for identifiers |

**The embedded tier is not affected by any of the above.** The embedded tier has no `Arc`, no Babalawo at runtime, no Àjọṣe registry, no string heap. Its performance envelope is determined entirely by the const-generic `EmbeddedVm<OPON_SIZE, STACK_SIZE>` parameters chosen at compile time.

**The `ifa build` (transpiler) target produces native Rust code** with none of the hosted interpreter overhead. For performance-critical production workloads, `ifa build` is the correct deployment path. The interpretation overhead of `ifa run`/`ifa runb` is a development-time cost, not a production-time cost.

> *Ẹni tó bá fẹ́ jẹun kíákíá kò gbọdọ̀ máa ṣe oúnjẹ rẹ̀ pẹ̀lẹ́. — One who wants to eat quickly must not cook slowly.*
> The hosted interpreter is cooked slowly, by design. For fast eating, use `ifa build`.

---

## 19. Error Taxonomy `[DEFINED]`

### 19.1 Error Value Structure

Every runtime error is a structured value with the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `message` | `String` | Human-readable description |
| `kind` | `String` | Error kind (see table below) |
| `line` | `Int \| ofo` | Source line number if available |
| `col` | `Int \| ofo` | Source column if available |
| `cause` | `Error \| ofo` | Original error if this was re-thrown |

### 19.2 Error Kinds

| Kind | Trigger |
|------|---------|
| `TypeError` | Operation on incompatible types |
| `ReferenceError` | Undefined variable or identifier |
| `OverflowError` | Integer overflow in debug mode |
| `NullReferenceError` | Field/method access on `ofo` |
| `IndexError` | (Reserved — IfáLang returns ofo instead) |
| `DivisionByZeroError` | Integer division by zero |
| `MatchError` | Non-exhaustive `ode` match |
| `StackOverflowError` | Recursion depth exceeded |
| `UserError` | Raised by `ta` keyword |
| `AssertionFailed` | Failed `Okanran.ẹri()` |
| `PermissionError` | Missing capability |
| `IoError` | File or network operation failure |
| `NetworkError` | Network-specific failure |
| `TimeoutError` | `Osa.akoko()` deadline exceeded |
| `CircularImportError` | Circular module dependency detected |
| `ParseError` | Malformed numeric literal or source |
| `NotImplemented` | Feature not supported by this runtime |

---

## 20. Capability Matrix `[DEFINED]`

This table defines the current implementation status across all three runtimes. **This is not aspirational — it reflects the actual state as of March 2026.** It is updated with every conformance milestone.

| Feature | `ifa run` (AST) | `ifa runb` (VM) | `ifa build` (Transpiler) | Embedded |
|---------|:-:|:-:|:-:|:-:|
| Integer arithmetic | ✓ | ✓ | ✓ | ✓ |
| Float arithmetic | ✓ | ✓ | ✓ | ✓ |
| String literals | ✓ | ✓ | ✓ | ✗ |
| String interpolation `$"..."` | ⚠ | ✗ | ✓ | ✗ |
| Boolean logic | ✓ | ⚠ | ✓ | ✓ |
| Comparison operators `< > <= >=` | ✓ | ✗ | ✓ | ✓ |
| Variable declaration (ayanmo/ayanfe) | ✓ | ✓ | ✓ | ✓ |
| Variable shadowing | ✓ | ✓ | ✓ | ✓ |
| Conditionals (ti/bibẹkọ) | ✓ | ✗ | ✓ | ✓ |
| While loop (nigba) | ✓ | ✗ | ✓ | ✓ |
| For loop (fun) | ⚠ | ✗ | ✓ | ✓ |
| Break/continue (duro/tesiwaju) | ✓ | ✗ | ✓ | ✓ |
| Pattern matching (ode) | ⚠ | ✗ | ⚠ | ✗ |
| User-defined functions (ese) | ✗ | ✗ | ✓ | ✓ |
| Default parameters | ✗ | ✗ | ✓ | ✗ |
| Variadic parameters (...) | ✗ | ✗ | ✓ | ✗ |
| Anonymous functions / lambdas | ✗ | ✗ | ✓ | ✗ |
| Closures with upvalue capture | ✗ | ✗ | ⚠ | ✗ |
| Recursion | ✗ | ✗ | ✓ | ✓ |
| Classes (odu) + methods | ✗ | ✗ | ✓ | ✗ |
| Inheritance (:) | ✗ | ✗ | ✓ | ✗ |
| ara (self) | ✗ | ✗ | ✓ | ✗ |
| List literals | ⚠ | ⚠ | ✓ | ✗ |
| Map literals | ⚠ | ⚠ | ✓ | ✗ |
| Optional chaining (?.) | ✗ | ✗ | ⚠ | ✗ |
| Null coalescing (??) | ✗ | ✗ | ⚠ | ✗ |
| Standard domain calls | ⚠ | ✗ | ✓ | ✓ |
| Error handling (gbiyanju/gba/ta) | ✗ | ✗ | ✗ | ✗ |
| Error propagation (?) | ✗ | ✗ | ✗ | ✗ |
| Module system (mu/fi) | ✗ | ✗ | ✗ | ✗ |
| Async/await (daro/duro fun) | ✗ | ✗ | ⚠ | ✗ |
| Channels (Osa.ọ̀nà) | ✗ | ✗ | ✗ | ✗ |
| Pointer ops (ailewu/&/*) | ✗ | ✗ | ✗ | ✓ |
| Yield (jowo) | ✗ | ✗ | ✗ | ✓ |
| Sized integer types | ✗ | ✗ | ✗ | ✓ |
| Tail call optimization | ✗ | ⚠ | ⚠ | ✗ |
| REPL | ✗ | ✗ | N/A | N/A |

**Legend:** ✓ Working · ⚠ Partial/buggy · ✗ Not implemented

---

## 21. Conformance Requirements `[DEFINED]`

### 21.1 Tier 1 — Minimum Conformance

A Tier 1 conforming implementation **MUST** correctly implement:

- All primitive types (Int, Float, Bool, String, Null)
- All arithmetic operators with specified overflow behavior
- All comparison operators with specified type promotion
- Truthiness rules (exact match to §5)
- Equality semantics (exact match to §6.5)
- Short-circuit evaluation for `&&` and `||`
- Variable declaration, scoping, and shadowing rules
- Function definition, call, and return (including closures)
- Control flow: `ti`, `nigba`, `fun`, `duro`, `tesiwaju`
- Pattern matching with `_` wildcard arm
- Error handling: `gbiyanju`/`gba`/`ta`
- All 16 Odù domain method signatures
- Source location in error messages

### 21.2 VM Canonicalization Gate

The VM (`ifa runb`) earns canonical status by satisfying all of the following:

1. Passes 100% of the Tier 1 conformance suite.
2. Comparison opcodes (`Lt`, `Le`, `Gt`, `Ge`) are implemented and tested.
3. `PushFn` + call frame protocol is implemented.
4. `StoreGlobal`/`LoadGlobal` use consistent little-endian encoding.
5. Closure upvalue capture via `MakeClosure` + `LoadUpvalue`/`StoreUpvalue` is implemented.
6. A single `IfaValue` type is used across the entire codebase (no split between `value.rs` and `value_union.rs`).
7. Source location is threaded through at least the most common error-producing opcodes.

Until this gate is passed, both `ifa run` and `ifa runb` are equally authoritative against the spec.

### 21.3 Transpiler Conformance

The transpiler (`ifa build`) is conforming when:

1. Every unsupported language construct produces `IfaError::NotImplemented` before emitting any code.
2. No `panic!()` calls exist in any `Statement` or `Expression` match arm.
3. Generated Rust output passes `cargo clippy --deny warnings`.
4. Try/catch (`gbiyanju`/`gba`) emits correct Rust `Result` unwinding, not a comment.
5. `jowo N` emits `std::thread::sleep(Duration::from_micros(N))`.

### 21.4 Conformance Test Protocol

A conformance test is a `.ifa` source file with a corresponding `.expected` file containing exact stdout output.

```
tests/
  conformance/
    arithmetic/
      integer_ops.ifa
      integer_ops.expected
      float_nan.ifa
      float_nan.expected
    truthiness/
      ...
    closures/
      ...
```

The harness runs each test against all runtimes that declare support for the feature. Any output mismatch is a failure. The spec (this document) determines the correct output when runtimes disagree.

---

---

## 22. The Babalawo — Static Analysis System `[DEFINED]`

> *Baba-n-ṣe-awo: The father who reads the mysteries.*

The Babalawo (`ifa-babalawo`) is IfáLang's compile-time static analysis pass. It runs after parsing and before any backend executes. It produces structured diagnostics — errors, warnings, and suggestions — each associated with the Odù domain that governs the class of error. It is not optional. All three runtimes **MUST** invoke the Babalawo on every program before execution begins.

### 22.1 Invocation

```
ifa run program.ifa      →  parse → Babalawo → AST walker
ifa runb program.ifa     →  parse → Babalawo → compiler → VM
ifa build program.ifa    →  parse → Babalawo → transpiler
```

If the Babalawo produces any `Error`-severity diagnostic, execution **MUST NOT** proceed. The program is rejected with the diagnostic output.

### 22.2 Analysis Passes

The Babalawo runs two passes over the AST:

**Pass 1 — Definition Collection**  
Walks all statements to build the complete set of declared names before any use-checking begins. This allows forward references to function names (an `ese` defined later in the file can be called earlier).

**Pass 2 — Issue Detection**  
Walks all statements checking for the issues in §22.3. This pass also invokes the Ìwà Engine (§22.5) and the Èèwọ̀ Enforcer (§22.6).

**Pass 2 — Self-Referential Literal Detection**  
During Pass 2, when a variable declaration `ayanmo name = <expr>` is processed, the Babalawo checks whether the initializer expression directly references `name` at the top level of a `Map` or `List` literal. This is the Babalawo's detection of obvious reference cycles that would cause memory leaks under `Arc` reference counting.

Detected patterns (examples):
```
ayanmo a = {self: a};              # REFERENCE_CYCLE: a contains itself
ayanmo b = [1, 2, b];             # REFERENCE_CYCLE: b contains itself
ayanmo c = {child: {parent: c}};  # not detected (nested — runtime only)
```

The detection is **syntactic and shallow** — it only catches cases where the variable name appears directly inside the literal being assigned to it, not inside nested lambdas or function calls. Deeper cycles (created through mutation or function calls) are not detectable at compile time and remain a runtime leak risk until a mark-and-sweep collector is added (§18.2).

**Final checks**  
After both passes: unused variable warnings, unclosed resource errors from the Ìwà Engine, taboo violations from the Èèwọ̀ Enforcer.

### 22.3 Static Checks

| Check | Code | Severity | Description |
|-------|------|----------|-------------|
| Undefined variable | `UNDEFINED_VARIABLE` | Error | Variable used before declaration |
| Unused variable | `UNUSED_VARIABLE` | Warning | Declared but never read. Suppressed for names starting with `_`. |
| Self-referencing init | `UNINITIALIZED` | Error | `ayanmo x = x + 1` — x used in its own initialization |
| Self-referential literal | `REFERENCE_CYCLE` | Warning | `ayanmo a = {self: a}` — obvious cycle at construction site |
| Division by zero | `DIVISION_BY_ZERO` | Error | Literal `0` as divisor in `/` or `%` |
| Type mismatch (decl) | `TYPE_MISMATCH` | Error | Declared type `i32`, assigned `String` |
| Type mismatch (assign) | `TYPE_MISMATCH` | Error | Statically typed variable assigned incompatible value |
| Unsafe outside ailewu | `UNSAFE_OUTSIDE_AILEWU` | Error | Pointer type used outside `ailewu` block |
| Unclosed resource | `UNCLOSED_RESOURCE` | Error | Resource opened but never closed (Ìwà Engine) |
| Missing return | `MISSING_RETURN` | Warning | Function may not return on all paths |
| Taboo violation | `TABOO_VIOLATION` | Error | Forbidden dependency (Èèwọ̀ Enforcer) |
| Infinite loop | `INFINITE_LOOP` | Warning | Loop with no exit condition |
| Unreachable code | `UNREACHABLE_CODE` | Warning | Code after unconditional `pada` or `duro` |
| Private access | `PRIVATE_ACCESS` | Error | Accessing `aladani` member from outside the class |
| Capability undeclared | `CAPABILITY_UNDECLARED` | Warning | Domain call uses capability not in `ifa.toml [capabilities]` |
| Dynamic network target | `DYNAMIC_NETWORK` | Warning | `Otura` call with dynamic URL — inferred `Network { domains: ["*"] }`. Declare explicitly. |
| Dynamic env key | `DYNAMIC_ENV` | Warning | `Ogbe.ayika()` with dynamic key — inferred `Environment { keys: ["*"] }`. Declare explicitly. |

**Warning deduplication:** The same warning code at the same source location is emitted at most once per Babalawo pass. If the same warning code fires at ten different locations, all ten are shown. If it fires at the same location twice (e.g., inside a loop body that is analyzed twice due to branching), only the first occurrence is shown. Implementations **MUST NOT** suppress distinct-location warnings of the same code.

**Warning cap:** When warnings exceed 50 in a single pass, the Babalawo stops emitting new warnings and instead prints: `... N more warnings suppressed. Fix the above warnings first or run with --all-warnings.` Errors are never capped.

### 22.4 Diagnostic Format

Every diagnostic has the form:

```
severity[OduName] file:line:col
  message text
  Wisdom: proverb from the Odù domain
```

Example:

```
error[Ogbè] main.ifa:12:5
  Variable 'count' used before declaration
  Wisdom: Check your initialization. All things must have a proper beginning.

warning[Ọ̀sá] main.ifa:34:1
  Function 'compute' may not return on all paths
  Wisdom: Verify your conditionals. Flow must have logic.

2 errors, 1 warning. Àṣẹ!
```

Three output formats are available:
- **Default** — full format with Odù name, location, message, and wisdom (for errors)
- **Compact** — `file:line:col: severity: message` — for IDE integration, no Odù name, no wisdom
- **JSON** — structured output for tooling

The `--wisdom` flag controls wisdom verbosity independently of output format:

| Flag | Behavior |
|------|----------|
| `--wisdom=full` (default) | Wisdom shown for all errors; hidden for warnings unless `--verbose` |
| `--wisdom=brief` | Wisdom shown only for the first occurrence of each error code in a session |
| `--wisdom=none` | No wisdom output. Error codes and messages only. |

> **Encoding contract:** Error codes (`UNDEFINED_VARIABLE`, `TYPE_MISMATCH`, etc.) and error kind names (`TypeError`, `ReferenceError`, etc.) are always 7-bit ASCII. They are safe to embed in CI logs, grep pipelines, and JSON without Unicode handling. Odù names and proverbs use full Yoruba Unicode with diacritics. If your terminal does not render Yoruba diacritics correctly, use `--wisdom=none` or the **Compact** format, both of which are ASCII-only. The diacritics are not decorative — they are correct Yoruba orthography — but they are confined to the human-facing wisdom layer and never appear in machine-readable fields.

### 22.5 Error-to-Odù Mapping

Every error code maps to one of the 16 Odù domains. The Odù is not cosmetic — it encodes the *character* of the error, grounding it in the metaphysical domain that governs that class of problem.

| Error Class | Odù | Why |
|-------------|-----|-----|
| Uninitialized variables, undefined names | Ogbè (The Light) | Ogbè governs beginnings. An undefined name has no beginning. |
| Unclosed resources, orphan processes | Ọ̀yẹ̀kú (The Darkness) | Ọ̀yẹ̀kú governs endings. An unclosed resource has no ending. |
| Infinite loops, exhausted iterators | Ìwòrì (The Mirror) | Ìwòrì governs cycles. A loop with no exit has lost its reflection. |
| File errors, permission denied | Òdí (The Vessel) | Òdí governs containment. File errors violate the vessel. |
| Format errors, output overflow | Ìrosù (The Speaker) | Ìrosù governs expression. Malformed output is broken speech. |
| Seed errors, non-determinism failures | Ọ̀wọ́nrín (The Chaotic) | Ọ̀wọ́nrín governs chance. A seed error is uncontrolled chaos. |
| Overflow, arithmetic errors | Ọ̀bàrà (The King) | Ọ̀bàrà governs expansion. Overflow is unchecked growth. |
| Division by zero, underflow | Òtúúrúpọ̀n (The Bearer) | Òtúúrúpọ̀n governs reduction. Division by zero is reduction without substance. |
| Exceptions, assertions, unused vars | Ọ̀kànràn (The Troublemaker) | Ọ̀kànràn governs disruption. Unhandled errors are ignored troubles. |
| Array out of bounds | Ògúndá (The Cutter) | Ògúndá governs separation. Cutting past the boundary is imprecision. |
| Missing return, invalid jump | Ọ̀sá (The Wind) | Ọ̀sá governs control flow. Missing return is directionless wind. |
| String errors, encoding | Ìká (The Constrictor) | Ìká governs binding. A malformed string is a broken bond. |
| Network errors, timeouts | Òtúrá (The Messenger) | Òtúrá governs transmission. Network failure is a silenced messenger. |
| Memory leaks, double free | Ìrẹtẹ̀ (The Crusher) | Ìrẹtẹ̀ governs compression. A leak is memory that refuses to be released. |
| Graphics, coordinate errors | Ọ̀ṣẹ́ (The Beautifier) | Ọ̀ṣẹ́ governs form. Wrong coordinates are misplaced beauty. |
| Type errors, inheritance | Òfún (The Creator) | Òfún governs creation. A type error is misidentified essence. |

---

## 23. The Ìwà Engine — Resource Lifecycle Validation `[DEFINED]`

> *Ohun tí a ṣí, a gbọdọ̀ pa. — What we open, we must close.*

The Ìwà Engine enforces resource lifecycle balance at compile time. Every domain method that opens a resource (a file, a connection, a memory allocation, a loop) must have a corresponding closing call. This is not a style check — it is a compile error.

### 23.1 Lifecycle Rules

| Opener | Required Closer | Domain | Meaning |
|--------|----------------|--------|---------|
| `Odi.si()` | `Odi.pa()` | Òdí | File open → close |
| `Odi.kọ()` | `Odi.pa()` | Òdí | File write-open → close |
| `Otura.de()` | `Otura.pa()` | Òtúrá | TCP bind → close |
| `Otura.so()` | `Otura.pa()` | Òtúrá | TCP connect → close |
| `Ogunda.ge()` | `Irete.tu()` | Ògúndá/Ìrẹtẹ̀ | Allocate → free |
| `Ogunda.da()` | `Irete.tu()` | Ògúndá/Ìrẹtẹ̀ | Create → free |
| `Ofun.da()` | `Ofun.pa()` | Òfún | Create object → delete |
| `Ogbe.bi()` | `Oyeku.duro()` | Ògbè/Ọ̀yẹ̀kú | Init → halt (auto-close) |
| `Ogbe.bere()` | `Oyeku.duro()` | Ògbè/Ọ̀yẹ̀kú | Start → stop (auto-close) |
| `ebo.begin` | `ebo.sacrifice` | Ẹbọ | Resource block start → sacrifice |
| `Iwori.yipo()` | `Iwori.pada()` | Ìwòrì | Loop begin → return |

**Auto-close resources**: `Ogbe.bi` and `Ogbe.bere` are automatically closed at program end. The Ìwà Engine does not flag these as errors even without an explicit close.

### 23.2 Borrow Checking

The Ìwà Engine also performs a simplified borrow check for reference types (`Ref`, `RefMut`). The rules mirror Rust's borrow checker:

- Multiple simultaneous **immutable** borrows of the same variable: **permitted**
- One **mutable** borrow while any other borrow exists: **error** (`AlreadyMutablyBorrowed` or `ImmutableBorrowExists`)
- Borrows are released when their scope exits

```
ayanmo x = [1, 2, 3];
ayanmo ref_a = &x;         # immutable borrow
ayanmo ref_b = &x;         # ok — multiple immutable borrows allowed
ayanmo ref_mut = &mut x;   # ERROR: ImmutableBorrowExists
```

### 23.3 The `#[iwa_pele]` Attribute

The `#[iwa_pele]` attribute enforces lifecycle balance at the function level for paired method calls. It is a compile-time check — a function that opens something without closing it **will not compile**.

```
#[iwa_pele]
ese network_task() {
  ayanmo conn = Otura.so("api.example.com", 443);
  # ... work ...
  conn.pa();    # REQUIRED — or compile error:
                # Ìwà Pẹ̀lẹ́ violation: 1 'so' calls but only 0 'pa' calls.
                # Proverb: Ohun tí a ṣí, a gbọdọ̀ pa.
}
```

Pairs checked by `#[iwa_pele]`:

| Open method | Close method |
|-------------|-------------|
| `so` | `pa` |
| `si` | `pa` |
| `mu` | `fi` |
| `bere` | `da` |

---

## 24. The Èèwọ̀ System — Architectural Taboos `[DEFINED]`

> *Ẹni tó bá fọwọ́ kan èèwọ̀, yóò rí àṣèdá. — Whoever touches a taboo will see the consequences.*

Èèwọ̀ (taboo) in Ifá are not merely prohibitions — they are structural laws that maintain cosmic order. In IfáLang, architectural taboos are **enforced at compile time**. They prevent forbidden dependencies between components.

### 24.1 Declaring Taboos

```
# In source: declare that component Ose (UI) cannot call Odi (Files) directly
taboo Ose -> Odi;

# Wildcard: block all calls to a domain
taboo * -> Otura;     # No network access allowed anywhere in this module
```

### 24.2 Common Architectural Taboos

| Taboo | Meaning |
|-------|---------|
| `Ose -> Odi` | UI layer cannot call file system directly (must go through a domain service) |
| `Ose -> Otura` | UI layer cannot make network calls directly |
| `* -> Ailewu` | No unsafe pointer operations except in designated modules |

### 24.3 Thread Safety Taboo — The Hut and the Market

A specific built-in taboo governs thread safety. Non-thread-safe values cannot cross thread boundaries:

> *The Hut cannot go to the Market without a ritual (Freeze)*

The following types are **Hut types** — they live in a single-threaded space:

- `IfaValue` (uncloned)
- `Rc<T>`
- `RefCell<T>`
- `GcPtr<T>`

The following contexts are **Market contexts** — shared, multi-threaded:

- `Osa.ise()` task closures
- `Thread` spawning
- `Spawn` operations

Passing a Hut type into a Market context **MUST** produce a `TABOO_VIOLATION` error. The fix is to use the `#freeze` modifier (which converts to an `Arc`-backed thread-safe representation) or to restructure the code so no Hut type crosses the boundary.

```
ayanmo counter = 0;

# ERROR: IfaValue cannot cross to Osa task (Hut → Market)
Osa.ise(|| { counter = counter + 1; });

# CORRECT: use freeze modifier to make it Market-safe
ajose!(counter => #freeze shared_counter);
Osa.ise(|| { shared_counter = shared_counter + 1; });
```

---

## 25. The Àjọṣe Reactive System `[DEFINED]`

> *Àjọṣe: A sacred covenant relationship — binding that creates mutual obligation.*

Reactive bindings in IfáLang model the Ifá concept of àjọṣe: a covenant between two parties. When the source changes, the target is automatically updated — not by polling, but because they are bound in covenant.

### 25.1 Syntax

```
ajose!(source.field => target.field);               # standard binding
ajose!(source.field => #freeze shared.field);       # cross-thread (freeze modifier)
```

### 25.2 Semantics

A reactive binding is a subscription: when `source.field` changes, a callback fires that updates `target.field`. The binding is **one-directional** by default — source drives target, not the reverse.

The `#freeze` modifier produces a thread-safe binding. The value is converted to an `Arc`-backed representation before being written to the target. This allows bindings to cross thread boundaries (Market contexts).

### 25.3 The Global Àjọṣe Registry `[DEFINED]`

The global Àjọṣe registry is the runtime data structure that holds all active subscriptions and dispatches updates when sources change. It is **part of the language runtime** (`ifa-core`), not a user library.

#### 25.3.1 Registry Structure

```rust
// In ifa-core — the canonical Àjọṣe runtime
pub struct AjoseRegistry {
    // All active subscriptions, keyed by source object ID + field name
    subscriptions: Arc<Mutex<HashMap<SubscriptionKey, Vec<Callback>>>>,
    // Epoch counter for batched update detection (prevents re-entrant loops)
    epoch: AtomicU64,
}

pub struct SubscriptionKey {
    source_id: usize,   // object identity (pointer-based)
    field: Arc<str>,    // interned field name
}

pub type Callback = Arc<dyn Fn(IfaValue) + Send + Sync>;
```

A single process-global registry instance is initialized before `main()` executes and cleared by `Oyeku.duro()` on program exit.

#### 25.3.2 Registration — `ajose!` at Runtime

When `ajose!(source.field => target.field)` is evaluated:

1. The source object's ID and field name form a `SubscriptionKey`.
2. A callback closure is created: `move |new_val| { target.field = new_val; }`.
3. The callback is registered in the registry under the key.
4. The target is immediately synchronized to the current source value (initial sync).

The `_subscription` binding that was previously dropped is now unnecessary — the registry owns the subscription lifetime. The `ajose!` expression evaluates to `()`.

#### 25.3.3 Update Dispatch

When any assignment `source.field = new_value` executes at runtime, the VM **MUST** check the registry for subscriptions keyed to `(source_id, "field")` and invoke all registered callbacks with `new_value`. This check is O(subscribers) and happens on every field assignment to an observable object.

**Batching:** All callbacks triggered by a single assignment are fired in registration order within the same epoch. A callback that itself triggers another assignment does **not** re-enter dispatch in the same epoch — it is deferred to the next epoch. This prevents infinite reactive loops.

```
ajose!(a.x => b.y);
ajose!(b.y => a.x);    # circular binding — safe due to epoch batching
a.x = 5;               # fires b.y = 5 in epoch N
                        # b.y = 5 would fire a.x = 5 — deferred to epoch N+1
                        # epoch N+1: a.x already == 5 → no-op → stabilizes
```

#### 25.3.4 The `#freeze` Modifier

For cross-thread bindings, the `#freeze` modifier wraps the new value in `Arc` before calling the callback, enabling the callback to run in a `Market` (Osa task) context:

```
ajose!(sensor.temp => #freeze display.reading);
# callback: move |v| { display.reading = Arc::new(v); }
# weak back-reference prevents the display from keeping sensor alive
```

The registry uses **weak references** (`Weak<T>`) to the target object. If the target is dropped, the subscription is silently removed on the next dispatch cycle. This prevents the reactive registry from being a source of memory leaks.

#### 25.3.5 Registry Lifetime

| Event | Registry action |
|-------|----------------|
| Program start | Registry initialized (empty) |
| `ajose!(...)` | Subscription registered |
| Target object dropped | Weak ref expires; subscription removed on next dispatch |
| `Oyeku.duro()` or normal exit | Registry cleared; all callbacks dropped |
| Panic/signal | Registry cleared in the `Drop` impl |

### 25.4 The `#[derive(Observable)]` Attribute

Structs can be made observable, generating `watch_field()` methods for each field. These methods accept a callback that fires when the field changes.

```
#[derive(Observable)]
odu Counter {
  ayanmo value: Int = 0;
}

ayanmo c = Counter.new();
c.watch_value(|v| { Irosu.fo($"Counter changed: {v}"); });
c.value = 5;     # fires the callback: "Counter changed: 5"
```

---

## 26. The Ẹbọ Type System — Resource Obligations `[PARTIAL]`

> *Ẹbọ: The sacrifice that transforms — the offering that fulfills an obligation.*

In Ifá, Ẹbọ is the act of sacrifice that resolves a debt between a person and the spiritual forces governing their situation. Every resource in IfáLang has an Ẹbọ obligation: the obligation to be released. The Ẹbọ type system encodes these obligations in the type itself, making unfulfilled obligations a compile error.

### 26.1 The `#[derive(Ebo)]` Attribute

```
#[derive(Ebo)]
#[ebo(cleanup = "close")]
odu FileHandle {
  aladani handle: Int;

  ese close() {
    Odi.pa(ara.handle);
  }
}
```

`#[derive(Ebo)]` generates a `Drop` implementation that calls the specified cleanup method when the value goes out of scope. If no cleanup method is specified, a default drop message is emitted.

The Ẹbọ obligation is the type-level encoding of the Ìwà Engine's lifecycle rules: both enforce the principle that what is opened must be closed. The Ìwà Engine catches failures at the function level; the Ẹbọ type system catches them at the scope level.

### 26.2 The `ebo_block!` Macro

```
ebo_block! {
  ayanmo file = Odi.si("/tmp/data.txt");
  # file is automatically closed when this block exits
  # even if an error is thrown
}
```

An `ebo_block!` is a scope with guaranteed cleanup. Any resource opened inside the block is released when the block exits, regardless of whether exit is normal or via error propagation.

### 26.3 Ẹbọ and the Ìdájọ́ Principle

Ìdájọ́ means judgment — the consequence of unfulfilled obligation. In IfáLang, an unfulfilled Ẹbọ obligation at compile time is a compile error. At runtime, it is a resource leak. The Babalawo catches compile-time violations; the Ìwà Engine catches runtime ones. No obligation may be left unfulfilled.

---

## 27. The Multiparadigm Design `[DEFINED]`

> *Ìmọ̀ kì í wọ inú àpò. — Knowledge does not fit inside a single bag.*

IfáLang is multiparadigm not by accumulation — not by bolting features onto a single-paradigm core — but by design. Each paradigm maps to a principle of Ifá practice. The paradigms are not modes you switch between. They are co-present dimensions of every program.

### 27.1 The Seven Paradigms

| Paradigm | Ifá Concept | Mechanism | Example |
|----------|-------------|-----------|---------|
| **Procedural** | Ìgbésẹ̀ (The Step) | `ese` definitions, sequential ops | `ese add(a, b) { a + b; }` |
| **Functional** | Odù casting (Input yields result) | Immutable values, map/filter/reduce | `Ogunda.sare(list, \|x\| x > 0)` |
| **Domain-Oriented** | Odù (Named sacred space) | 16 Odù structs, method dispatch | `Ika.gigun(s)` |
| **Reactive** | Àjọṣe (Sacred covenant) | `ajose!` binding macro | `ajose!(counter => label)` |
| **Capability-Based** | Àṣẹ (Authority to act) | Ofun/CapabilitySet | `Ofun.le("network")` |
| **Systems/Embedded** | Ilẹ̀ (The sacred ground) | `no_std` EmbeddedVm, MMIO | `ailewu { *GPIO = 1; }` |
| **Concurrent** | Ọ̀sá (The Runner) | Async tasks, channels | `Osa.ise(daro ese () { ... })` |

### 27.2 Paradigm 1 — Procedural: Ìgbésẹ̀ (The Step)

Every Ifá consultation follows a sequence. The Babalawo casts the chain, reads the Odù, recites the verse, prescribes the Ẹbọ. Order matters. Procedural code is the ground of all other paradigms — the sequence that gives everything else something to stand on.

```
ese bake_bread(flour, water, yeast) {
  ayanmo dough = Ika.so([flour, water, yeast]);
  Oyeku.sun(3600);        # wait 1 hour
  dough.shape();
  dough.bake(200);
}
```

### 27.3 Paradigm 2 — Functional: The Odù as Pure Oracle

In Ifá, the oracle does not change when consulted. The Odù are eternal — they reveal truth but do not alter it. Functional programming encodes this: pure functions take values and return values without side effects. The 16 Odù domains are zero-state structs by design — `Obara`, `Ika`, `Oturupon` hold no mutable state. You consult them; they do not remember being consulted.

> *Ifá kò ṣẹ́kú, kò ṣẹ́ àárọ̀. — Ifá does not grow old, it does not grow young — it simply is.*

```
# Pure functional chain — no state, no side effects
ayanmo result = Ogunda
  .sare(sales_data, |s| s.region == "Lagos")
  .yipada(|s| s.total * 1.075)
  .din(0, |acc, x| acc + x);
```

### 27.4 Paradigm 3 — Domain-Oriented: The Odù as Ontological Namespace

The 16 principal Odù are not merely module names. Each carries a metaphysical identity that constrains what belongs inside it. `Odi` (the Seal, `1001`) governs containment — files, databases, closures. `Otura` (the Messenger, `1011`) governs transmission — networks, protocols, signals. The domain is not organizational; it is **ontological**. When you add a method to `Odi`, you are making a claim that this operation belongs to the character of containment.

This is why the Babalawo's error-to-Odù mapping is not cosmetic: `FILE_NOT_FOUND → Odi`, `DIVISION_BY_ZERO → Oturupon`, `NETWORK_TIMEOUT → Otura`. Every error has a domain because every error has a character.

### 27.5 Paradigm 4 — Reactive: Àjọṣe (Sacred Relationship)

Àjọṣe in Yoruba means a sacred covenant relationship — the binding between two parties that creates mutual obligation. When `counter` changes, `label` changes — not because `label` polls `counter`, but because they are bound in covenant. See §25 for the complete reactive specification.

### 27.6 Paradigm 5 — Capability-Based: Àṣẹ (The Power to Act)

Àṣẹ is the divine authority to make things happen — the power that flows through all living things. In IfáLang, no operation has inherent permission. Permission flows from explicit capability grants, just as àṣẹ flows from deliberate spiritual authorization.

The sandbox shim stub that grants all permissions when `ifa-sandbox` is absent is philosophically incorrect: **the absence of a priest does not grant everyone priestly authority.** Capabilities are deny-by-default in all contexts, in all runtimes, at all times.

```
# Explicit àṣẹ — authority must be named before it can be exercised
@aṣẹ(read: "/tmp", network: ["api.example.com"])
ese main() {
  ayanmo data = Odi.ka("/tmp/input.json");
  ayanmo result = Otura.gba("https://api.example.com/process");
}
```

### 27.7 Paradigm 6 — Systems: Ilẹ̀ (The Sacred Ground)

Ilẹ̀ is the earth — the ground of all existence. Without the physical world, thought has nowhere to land. IfáLang's embedded tier is its ilẹ̀: the layer where the language touches physical reality, where values are bytes in registers, where timing is microseconds. See §16 for the embedded specification.

```
ailewu {
  ayanmo GPIO_LED: *u32 = &0x4000_1000;
  *GPIO_LED = 1u32;       # the light obeys
  jowo 500000;             # yield 500ms — let the world breathe
  *GPIO_LED = 0u32;
}
```

### 27.8 Paradigm 7 — Concurrent: Ọ̀sá (The Runner Who Never Stops)

In Ifá, Ọ̀sá is associated with sudden movement and energy that cannot be contained. Concurrency in IfáLang inherits this character: tasks that run alongside each other, channels that carry messages, and the TaskGraph that enforces dependency order — Ọ̀sá constrained by precedent.

The TaskGraph's cycle detection (`CycleDetected` error) is philosophically sound: in Ifá, circular dependency between Odù is impossible by design. No Odù can be its own ancestor.

```
ayanmo results = duro fun Osa.gbogbo([
  Osa.ise(daro ese () { pada fetch_prices(); }),
  Osa.ise(daro ese () { pada fetch_volumes(); }),
  Osa.ise(daro ese () { pada fetch_news(); }),
]);
# All three tasks run in parallel; results arrive when all complete
```

### 27.9 Ifarapọ — The Unification Principle

> *Bí a bá fẹ́ mọ ẹni tó dára, a wo ihà rẹ̀. — If you want to know a good person, look at their character from all sides.*

Ifarapọ means unification — bringing together. The multiparadigm vision is not about giving programmers many tools. It is about giving programs many dimensions. A single IfáLang program can simultaneously be:

- A capability-secured system (Àṣẹ enforcement at compile time)
- A domain-oriented specification (Odù as ontological namespace)
- A functional pipeline (pure transforms on immutable values)
- A reactive covenant (Àjọṣe bindings)
- A concurrent orchestration (Ọ̀sá TaskGraph)
- A bare-metal controller (Ilẹ̀ EmbeddedVm)

These are not modes. They coexist in the same program, the same function, the same expression.

---

## 28. The Three Execution Tiers `[DEFINED]`

Every IfáLang program lives at one of three execution tiers. The language surface adapts to the tier.

| Tier | Yoruba Name | Character | Ifá Concept | IfáLang Surface |
|------|------------|-----------|-------------|-----------------|
| Ground | Ilẹ̀ | No OS, bare metal, microseconds | Physical world, earthly constraints | `no_std` EmbeddedVm, MMIO, `const`-generic sizing, sized integers |
| Person | Ẹni | OS present, filesystem, network | Human world, social contracts | Full `std`, all 16 Odù, async runtime, modules |
| Sky | Ọ̀run | Distributed, cloud, WASM sandbox | Spirit world, capabilities enforced | WASM target, Ofun sandbox, Wasmtime OmniBox |

### 28.1 The Ilẹ̀ Tier (Embedded)

Described fully in §16. Key properties: no heap allocation, const-generic VM sizing (`EmbeddedVm<OPON_SIZE, STACK_SIZE>`), MMIO via HAL, `jowo` as hardware sleep, sized integer types.

### 28.2 The Ẹni Tier (Hosted)

The full language as described throughout this spec. All 16 Odù domains available. Async runtime. Module system. REPL (planned). All three runtimes (`ifa run`, `ifa runb`, `ifa build`) target this tier.

### 28.3 The Ọ̀run Tier (WASM)

The WASM tier runs IfáLang programs inside a Wasmtime sandbox. Key properties:

- **OmniBox**: Wasmtime engine with pooling allocator and epoch interruption
- **AOT path**: Programs compiled with `compile_artifact()` to native `.cwasm` files; loaded at runtime with `deserialize_artifact()` for sub-2ms startup
- **JIT path**: `run_wasm_file()` for development use
- **Security profiles**: `Untrusted` (5s, 64MB), `Standard` (30s, 256MB), `Development` (5min, 2GB)
- **EWO host functions**: WASM modules query capabilities via `ewo.can_read()`, `ewo.can_write()`, `ewo.can_network()`, `ewo.is_secure()`
- **WASM bindings**: `run_code(source) → String` for browser playground; `cast_opele() → String` for Odù selection

The Ọ̀run tier enforces the strictest capability model. All capabilities are deny-by-default. The WASI preopened-directory model handles filesystem capability enforcement; the EWO functions handle network and permission queries.

> ⚠️ **Version mismatch risk**: `deserialize_artifact()` uses `unsafe`. Artifacts compiled with a different Wasmtime version will produce undefined behavior. No version check is currently implemented. Until a version header and signature verification are added, `.cwasm` artifacts **MUST** only be loaded from the same engine version that compiled them.

---

## 29. The Babalawo Capability Inferencer `[PARTIAL]`

The Babalawo includes a static capability inference pass (`infer_capabilities()`) that walks the AST before execution and determines which capabilities the program requires. This enables:

1. **Zero-config execution**: `ifa run` automatically grants the inferred capabilities without a manifest file.
2. **Manifest generation**: `ifa build --manifest` emits the inferred capability set as a lockfile.
3. **Overprivilege detection**: If the declared capabilities exceed the inferred set, the Babalawo warns.

### 29.1 Inference Rules

| AST pattern | Inferred capability |
|-------------|---------------------|
| `Odi.ka(path)` with literal path | `ReadFiles { root: path }` |
| `Odi.ka(dynamic_expr)` | **None inferred — requires explicit declaration (see §29.2)** |
| `Odi.kọ(path)` with literal path | `WriteFiles { root: path }` |
| `Odi.kọ(dynamic_expr)` | **None inferred — requires explicit declaration** |
| `Otura.gba(url)` with literal URL | `Network { domains: [parsed_host] }` |
| `Otura.gba(dynamic_expr)` | `Network { domains: ["*"] }` with a `DYNAMIC_NETWORK` warning |
| `Ogunda.ise(cmd, args)` with literal cmd | `Execute { programs: [cmd] }` |
| `Ogunda.ise(dynamic_expr, args)` | **None inferred — requires explicit declaration** |
| `Iwori.bayi()` / `Iwori.epoch()` | `Time` |
| `Owonrin.*` | `Random` |
| `Ogbe.ayika(key)` with literal key | `Environment { keys: [key] }` |
| `Ogbe.ayika(dynamic_expr)` | `Environment { keys: ["*"] }` with a `DYNAMIC_ENV` warning |
| `Irosu.fo()` / `Irosu.ka()` | `Stdio` |

### 29.2 Dynamic Path Policy — Deny, Not Broad Grant

The previous behavior of granting `ReadFiles { root: "/" }` for any dynamic `Odi.ka(path)` call is a **security anti-pattern**. Granting filesystem root access because the path is computed at runtime inverts the principle of least privilege: it rewards programs for hiding their access patterns behind dynamic expressions.

**The correct policy:** when a file or process path cannot be statically inferred, the inferencer infers **no capability** for that call. The program **MUST** declare the capability explicitly in `ifa.toml [capabilities]`. At runtime, the sandbox enforces the declared capability. If the declared capability is `read = ["/"]`, that is the user's explicit decision — not an automatic grant.

```
# A dynamic path produces no inference and a CAPABILITY_UNDECLARED warning:
ayanmo config_path = Ogbe.ayika("CONFIG_PATH") ?? "./config.json";
ayanmo data = Odi.ka(config_path);
```

```
warning[Òdí] main.ifa:2:18
  Dynamic path in Odi.ka() — no capability inferred.
  Declare this capability explicitly in ifa.toml [capabilities]:
    read = ["./", "/etc/myapp"]   # or the specific paths your program needs
  Running without this declaration will produce a PermissionError at runtime.
  Wisdom: Guard well what you store.
```

At runtime, if the declared `read` paths do not cover the actual path used, a `PermissionError` is raised. The error message includes the attempted path and the declared paths, making the gap easy to diagnose.

> *Ẹni tó bá ń pa àṣírí rẹ̀ mọ́, kì í pè ní àṣá. — One who hides their intentions does not call it custom.* Dynamic paths that hide their targets from the static analyzer do not earn broad trust.

---

## 30. The OpeleChain — Temporal Types `[PARTIAL]`

> *Ọ̀pẹlẹ tó bá fà, a máa ń sọ̀rọ̀ Ifá. — When the divination chain is cast, it speaks the words of Ifá.*

The OpeleChain (`Opele.chain()`) is IfáLang's most philosophically grounded data structure. In Ifá divination, the chain is cast and the pattern is permanent — it cannot be uncasted. The OpeleChain enforces this: you can only `cast()` new entries, never modify past ones.

### 30.1 Semantics

```
ayanmo chain = Opele.yera();                        # create empty chain
chain.ta(Transaction { amount: 1000, to: "Àdé" }); # append-only cast
chain.ta(Transaction { amount: 500, to: "Títí" });

Okanran.ẹri(chain.ṣayẹwo());                       # verify integrity: always true
                                                     # if no past entries were modified
```

Every `ta()` call computes a hash of `(previous_hash, new_entry)` and appends it. The `ṣayẹwo()` (verify) method walks the entire chain confirming that every hash is consistent. Modification of any past entry makes `ṣayẹwo()` return `eke`.

> ⚠️ **Non-crypto hash mode**: When using FNV-1a (default), OpeleChain provides tamper-evidence but not cryptographic security. FNV-1a is not collision-resistant — a determined adversary can craft collisions. For audit-critical use cases (financial ledgers, medical records, identity systems), use SHA-256 mode: `Opele.yera_aabo()`.

> ⚠️ **Timestamp monotonicity**: OpeleChain uses `SystemTime::now()` for entry timestamps. On systems where the clock goes backward (NTP adjustment, VM snapshot restore), two consecutive entries may have the same or decreasing timestamp. The chain's integrity check verifies hashes, not timestamp ordering. Do not rely on OpeleChain timestamps for causal ordering.

### 30.2 Long-Term Vision: Temporal Types

The OpeleChain points toward a future direction in IfáLang's type system: **temporal types** — values that carry their history as part of their type. A version-tracked value would be opaque unless you also present its chain of custody.

```
# Future syntax — not yet implemented
ayanmo ledger: OpeleChain<Transaction> = Opele.yera_aabo();
ledger.ta(Transaction { amount: 1000, to: "Alice" });

# Type system guarantees: ledger.ṣayẹwo() is always true
# Past entries are immutable — modification is a compile error
ledger[0].amount = 500;   # compile error: OpeleChain entries are immutable
```

This would make IfáLang uniquely suited for audit-critical systems. The type is not just a container — it is a witness.

---

## 31. The 256 Odù — Long-Term Type Lattice Vision `[OPEN]`

> *A kì í wo ọ̀nà kan ṣoṣo. — One never looks in only one direction.*

The 16 principal Odù and their 256 pairwise combinations form a complete binary lattice (every combination of two 4-bit patterns). This lattice has mathematical structure that maps naturally to a type hierarchy.

### 31.1 Current State

Today, the 16 Odù are domain identifiers — the `u8` byte in `CallOdu` opcodes, the domain routing key in the handler registry, the philosophical character of error messages. This is the foundation.

### 31.2 The Vision

In the long-term design, the 256 Odù combinations become the 256 fundamental type categories. Each combination represents a specific character of program behavior:

- **Ogbe + Ogbe** (`1111 + 1111`): The fully illuminated — complete, self-consistent values
- **Oyeku + Ogbe** (`0000 + 1111`): The transition from darkness to light — generator types, lazy values
- **Ogbe + Oyeku** (`1111 + 0000`): The full-to-empty transformation — compression, hashing, encryption
- **Odi + Otura** (`1001 + 1011`): Containment + transmission — serializable types
- **Irete + Osa** (`1101 + 0111`): Compression + concurrency — parallel-safe immutable values

### 31.3 Types as Character (Ìwà)

The deepest insight: in this vision, types in IfáLang encode **character** (ìwà), not just structure. The question is not "what fields does this struct have?" but "what is the character of this value in the world?"

A `FileHandle` is not just a file descriptor — it is a value with an obligation (Ẹbọ: it must be closed). An `OpeleChain<T>` is not just a list — it is a value with immutable history. A `CapabilitySet` is not just a set of strings — it is a value that carries authority.

> *Ìwà l'ẹwà. — Character is beauty.*

The 256 Odù as a type lattice is the horizon this language is moving toward. It is not the current state. It is the direction.

---

## 32. The WASM Playground `[DEFINED]`

The `ifa-wasm` crate provides WebAssembly bindings for running IfáLang in the browser.

### 32.1 Public API

```typescript
// Run IfáLang source code, return stdout + canvas output as string
run_code(source: string): string

// Return version string: "Ifá-Lang vX.Y.Z (WASM Core)"
get_version(): string

// Cast the Opele — return a random Odù name from the 16 principal Odù
// Uses JS Date.now() as seed (not cryptographically random)
cast_opele(): string
```

### 32.2 Output Model

`run_code()` returns a single string containing:
1. All stdout output from `Irosu.fo()` calls, newline-separated
2. If the canvas has non-blank content: a `═══ Canvas Output ═══` separator followed by the canvas render

### 32.3 Error Handling

Parse errors return `"Parse Error: <message>"`. Runtime errors return `"Runtime Error: <message>"`. Neither throws a JavaScript exception.

### 32.4 Limitations in WASM Context

- No filesystem access (`Odi` domain unavailable)
- No network access (`Otura` domain unavailable)
- No process spawning (`Ogunda.ise()` unavailable)
- No async runtime (`Osa` domain unavailable)
- `Owonrin` uses `Date.now()` as entropy source — not cryptographically random

---

## 33. Ọjà — The Package Manager `[PARTIAL]`

> *Ọjà ni ibi tí gbogbo ènìyàn ń pàdé. — The market is where all people meet.*

Ọjà (market) is IfáLang's package manager. Like the Yoruba market — a place of exchange, trust, and cultural order — Ọjà governs how IfáLang programs share, acquire, and declare their dependencies. It is not a standalone tool bolted onto the language. It is integrated with the capability system, the Babalawo static analyzer, and the deployment manager: a package's declared capabilities constrain what it may do at runtime, and the Babalawo verifies that the program's actual calls never exceed what was declared.

### 33.1 The Manifest — `ifa.toml`

Every IfáLang project has an `ifa.toml` manifest at its root. This is the single source of truth for the project's identity, dependencies, capabilities, and deployment targets.

```toml
[project]
name        = "my-app"
version     = "1.0.0"
description = "An example IfáLang application"
authors     = ["Àdé Olú <ade@example.com>"]
license     = "MIT"
edition     = "2026"

[dependencies]
ifa-http    = "2.1"
ifa-json    = "1.4.0"
ifa-crypto  = { version = "3.0", features = ["argon2"] }

[dev-dependencies]
ifa-test    = "1.0"

[capabilities]
# Explicit àṣẹ — the program's declared authority
read  = ["/tmp", "./data"]
write = ["/tmp"]
network = ["api.example.com", "cdn.example.com"]
# No process spawning declared → Ogunda.ise() will fail at runtime

[workspace]
members = ["./core", "./cli", "./wasm"]

[deploy]
# See §35 for deployment configuration
targets = ["hosted", "wasm"]
```

### 33.2 The Lockfile — `oja.lock`

`oja.lock` records the exact resolved version and SHA-256 checksum of every dependency, direct and transitive. It **MUST** be committed to version control for applications and **SHOULD NOT** be committed for libraries. The lockfile format is:

```toml
# oja.lock — generated by Oja, do not edit manually

[[package]]
name     = "ifa-http"
version  = "2.1.3"
source   = "registry+https://oja.ifá.dev/index"
checksum = "sha256:e3b0c44298fc1c149afbf4c8996fb924..."

[[package]]
name     = "ifa-json"
version  = "1.4.0"
source   = "registry+https://oja.ifá.dev/index"
checksum = "sha256:a87ff679a2f3e71d9181a67b7542122c..."
```

Every install or update operation verifies checksums against the lockfile. A checksum mismatch **MUST** halt installation with an `OjaIntegrityError`.

### 33.3 CLI Commands

```
oja add <package>[@version]    # add a dependency to ifa.toml and install it
oja remove <package>           # remove a dependency
oja install                    # install all dependencies in ifa.toml
oja update [package]           # update to latest allowed version
oja publish                    # publish the current package to the registry
oja audit                      # check all dependencies for known vulnerabilities
oja list                       # list installed packages and versions
oja tree                       # show full dependency tree
oja search <query>             # search the registry
oja init                       # create a new ifa.toml in the current directory
oja new <name>                 # create a new project directory with scaffold
```

### 33.4 Version Semantics

Ọjà uses semantic versioning (`MAJOR.MINOR.PATCH`). Version constraints in `ifa.toml` follow the Cargo convention:

| Constraint | Meaning |
|------------|---------|
| `"1.4.0"` | Exactly version 1.4.0 |
| `"^1.4"` | Compatible with 1.4 — allows 1.4.x and 1.5+ but not 2.0 |
| `"~1.4"` | Patch-level compatible — allows 1.4.x only |
| `">=1.4, <2.0"` | Range constraint |
| `"*"` | Any version (not recommended) |

### 33.5 The Registry

The official registry is hosted at `oja.ifá.dev`. It is organized by **Odù namespace**: packages that belong to a domain's character are published under that domain's name.

```
odi/ifa-sqlite          # file/database (Òdí domain)
otura/ifa-http          # networking (Òtúrá domain)
irete/ifa-argon2        # cryptography (Ìrẹtẹ̀ domain)
osa/ifa-tokio           # concurrency (Ọ̀sá domain)
obara/ifa-ndarray       # mathematics (Ọ̀bàrà domain)
```

A package published under a domain namespace makes a philosophical claim — that this package belongs to the character of that domain. The Ọjà registry enforces that packages in a domain namespace only use capabilities appropriate to that domain. A package under `odi/` that makes network calls will be rejected at publish time.

### 33.6 Capability Integration

The `[capabilities]` section of `ifa.toml` declares the program's àṣẹ at the project level. This declaration flows through the entire toolchain:

1. **At install time:** Ọjà verifies that no dependency's declared capabilities exceed the project's declared capabilities. A dependency that requires `network = ["*"]` cannot be used in a project that declares no network capability.
2. **At build time:** The Babalawo's `infer_capabilities()` pass (§29) compares the statically inferred capability set against the declared set. Undeclared capabilities produce a `CAPABILITY_UNDECLARED` warning.
3. **At runtime:** The sandbox enforces the declared capabilities as hard limits. No code in the program — not even a dependency — may exceed what `ifa.toml` declares.

> *Àṣẹ kò sí níbẹ̀ láì jẹ́ pé a fún. — Authority does not exist there without being granted.*

### 33.7 Workspace Support

A workspace is a collection of related packages that share a single `oja.lock` and build output directory. The root `ifa.toml` declares the workspace members:

```toml
[workspace]
members = ["core", "cli", "wasm"]

[workspace.dependencies]
# Shared dependencies available to all members
ifa-json = "1.4"
```

Individual member `ifa.toml` files reference workspace dependencies:

```toml
[dependencies]
ifa-json = { workspace = true }
```

### 33.8 Security and Supply Chain

All packages downloaded from the registry are verified against their SHA-256 checksum before installation. The `oja audit` command cross-references all installed package versions against a vulnerability database maintained at `oja.ifá.dev/advisories`.

A package that introduces a supply-chain dependency with capabilities broader than the project declares **MUST** be rejected. The transitivity of trust flows downward: a dependency cannot grant capabilities that the root project has not declared.

> ⚠️ **Relationship to `ifa-installer-core`**: `ifa-installer-core` handles the installation of the IfáLang toolchain itself (the `ifa` binary, stdlib, etc.). Ọjà handles project-level package dependencies. They use the same checksum verification logic but serve different purposes. `ifa-installer-core` bootstraps the tool; Ọjà manages what the tool builds.

---

## 34. The Debug Adapter — DAP Integration `[PARTIAL]`

> *Ẹni tó bá fẹ́ mọ ibi tó gbà, kò gbọdọ̀ dúró ní ibi kan ṣoṣo. — One who wants to know where they are must not stand in only one place.*

IfáLang implements the [Debug Adapter Protocol (DAP)](https://microsoft.github.io/debug-adapter-protocol/) — the JSON-RPC standard that allows any conforming IDE (VS Code, Neovim with nvim-dap, Emacs with dap-mode) to debug IfáLang programs without IDE-specific plugins. The debug adapter is not a separate process — it is a mode of the existing runtimes, activated by a flag.

### 34.1 Activating the Debug Adapter

```bash
ifa run   --dap [--port 4711] program.ifa    # AST interpreter in DAP mode
ifa runb  --dap [--port 4711] program.ifa    # VM in DAP mode
```

When started with `--dap`, the runtime:
1. Listens for a DAP client connection on the specified port (default: 4711)
2. Does not begin execution until the client sends an `initialize` request
3. Sends all output through the DAP `output` event rather than directly to stdout
4. Responds to all standard DAP requests

### 34.2 Supported DAP Capabilities

| DAP Feature | Support | Notes |
|-------------|---------|-------|
| `setBreakpoints` | ✓ | Line breakpoints in `.ifa` source files |
| `setFunctionBreakpoints` | ✓ | Break on entry to a named `ese` |
| `setExceptionBreakpoints` | ✓ | Break on any error or specific error kinds (§19.2) |
| `continue` | ✓ | Resume execution until next breakpoint |
| `next` (step over) | ✓ | Execute current statement, stop at next |
| `stepIn` | ✓ | Step into a function call |
| `stepOut` | ✓ | Execute until current function returns |
| `pause` | ✓ | Pause a running program |
| `stackTrace` | ✓ | Full call stack with source locations |
| `scopes` | ✓ | Local, upvalue, global, and domain scopes |
| `variables` | ✓ | Inspect any `IfaValue` with full type display |
| `evaluate` | ✓ | Evaluate an expression in the current scope |
| `reverseContinue` | ✓ | Time-travel: rewind to previous state (see §34.3) |
| `stepBack` | ✓ | Step backward one statement (see §34.3) |
| `restartFrame` | ⚠ | Planned — restart current call frame |
| `completions` | ⚠ | Partial — keyword and domain method completion |

### 34.3 Time-Travel Debugging — The StateHistoryBuffer

The Babalawo (`ifa-babalawo`) includes a `StateHistoryBuffer`: a 32-step circular buffer of execution snapshots. When the debug adapter is active, the runtime records a snapshot at every statement boundary, enabling backward execution.

Each snapshot contains:
- The current source line and column
- All local variable bindings and their values
- The call stack at that moment
- The Ìwà Engine state: open resources and active borrows

DAP `reverseContinue` steps backward through the buffer until the previous breakpoint. DAP `stepBack` moves one snapshot backward. When the buffer is exhausted (more than 32 steps back), the runtime reports `cannotStepBack` and the client must restart the session.

```
# Example: DAP stackTrace response for an IfáLang call
{
  "stackFrames": [
    {
      "id": 1,
      "name": "area",
      "source": { "path": "shapes.ifa" },
      "line": 12, "column": 3,
      "presentationHint": "normal"
    },
    {
      "id": 2,
      "name": "main",
      "source": { "path": "main.ifa" },
      "line": 28, "column": 1,
      "presentationHint": "normal"
    }
  ]
}
```

### 34.4 Odù-Aware Scope Display

The DAP `scopes` response for an IfáLang stack frame returns four named scopes:

| Scope name | Contents |
|------------|---------|
| `Locals` | Variables declared with `ayanmo`/`ayanfe` in the current function |
| `Upvalues` | Captured closure variables visible in the current function |
| `Globals` | Module-level and program-level bindings |
| `Domains` | The 16 active Odù domain handles (read-only, shows availability) |

When the Ìwà Engine has open resources or active borrows, an additional scope appears:

| Scope name | Contents |
|------------|---------|
| `Ìwà State` | Open resources (with opener location) and active borrow ledger |

This makes resource leaks visible during interactive debugging without requiring a separate tool.

### 34.5 Babalawo Diagnostics in DAP

When the Babalawo produces diagnostics during a debug session (e.g., on a hot-reload or evaluate request), they are sent to the client as DAP `output` events with the full Odù-tagged format:

```json
{
  "type": "output",
  "body": {
    "category": "stderr",
    "output": "warning[Ọ̀kànràn] shapes.ifa:15:5\n  Variable 'temp' is defined but never used\n  Wisdom: Problems are opportunities in disguise.\n"
  }
}
```

IDEs that support custom DAP output categories **MAY** display these with Odù-specific icons or colors.

### 34.6 Embedded Tier DAP Extensions

When debugging an embedded program (`ifa runb --embedded --dap`), two additional scopes appear:

| Scope name | Contents |
|------------|---------|
| `MMIO Registers` | Mapped memory-mapped I/O addresses with current values |
| `VM State` | `OPON_SIZE`, `STACK_SIZE`, current stack depth, free stack space |

These allow hardware register inspection during embedded debugging without leaving the IDE.

### 34.7 REPL Integration

When the REPL (§21, planned) is active, it operates as a persistent DAP session. Each REPL input is a single-statement DAP `evaluate` request. The StateHistoryBuffer persists across REPL inputs, enabling `stepBack` within an interactive session.

---

## 35. The Deployment Manager `[PARTIAL]`

> *Ọ̀nà tó dára kò ní jẹ́ kí ẹsẹ̀ rẹ jẹ. — A good road will not hurt your feet.*

The deployment manager (`ifa deploy`) orchestrates the complete path from IfáLang source to a running, capability-constrained artifact in a specific target environment. It is not a wrapper around `ifa build` — it is the system that answers: *where does this program run, with what authority, and how is its health verified?*

### 35.1 The Three Deployment Targets

Every IfáLang program deploys to one of the three execution tiers (§28). The deployment target determines the output artifact, the capability enforcement model, and the health-check mechanism.

| Target | Tier | Artifact | Capability Enforcement | Runtime |
|--------|------|----------|----------------------|---------|
| `embedded` | Ilẹ̀ | ELF/binary for MCU | Compile-time only (no OS) | `ifa-embedded` VM |
| `hosted` | Ẹni | Native binary (Linux/macOS/Windows) | `ifa-sandbox` NativeRuntime | `ifa runb` or transpiled binary |
| `wasm` | Ọ̀run | `.wasm` module + `.cwasm` AOT artifact | Wasmtime OmniBox + EWO host functions | `ifa-wasm` / `ifa-sandbox` |

### 35.2 Deployment Configuration in `ifa.toml`

```toml
[deploy]
targets = ["hosted", "wasm"]      # build for both

[deploy.hosted]
profile     = "release"           # release | debug | development
security    = "standard"          # untrusted | standard | development
entry       = "src/main.ifa"
output_dir  = "./dist/hosted"
health_check = "http://localhost:8080/health"

[deploy.wasm]
profile    = "release"
entry      = "src/lib.ifa"
output_dir = "./dist/wasm"
aot        = true                 # pre-compile to .cwasm for fast startup

[deploy.embedded]
profile    = "release"
target     = "thumbv7m-none-eabi" # Rust target triple for cross-compilation
entry      = "src/main.ifa"
output_dir = "./dist/embedded"
mmio_base  = "0x40000000"        # MMIO address boundary for the HAL
```

### 35.3 CLI Commands

```
ifa deploy                          # deploy all targets in ifa.toml
ifa deploy --target hosted          # deploy a specific target
ifa deploy --target wasm --dry-run  # validate without building or deploying
ifa deploy --env production         # deploy to a named environment
ifa deploy --rollback               # revert to the previous deployed artifact
ifa deploy status                   # show current deployment state for all targets
ifa deploy history                  # show deployment history (uses OpeleChain log)
```

### 35.4 The Deployment Pipeline

For each target, `ifa deploy` executes this pipeline in order. Any step failure halts the pipeline and no deployment occurs.

```
1. Parse ifa.toml and resolve deployment configuration
2. Run Babalawo static analysis (mandatory — §22)
   → Reject if any Error-severity diagnostic exists
3. Verify capability declarations match Babalawo inferred capabilities
   → Warn on excess declared capabilities
   → Error on undeclared capabilities
4. Run conformance tests (if test suite exists — §21)
   → Reject if any conformance test fails
5. Build artifact:
   → hosted:   ifa build → cargo build --release → native binary
   → wasm:     ifa build → wasm-pack build → .wasm → (if aot=true) .cwasm
   → embedded: ifa build → cross build --target <triple> → ELF binary
6. Sign artifact with project key (if signing key configured)
7. Record deployment in OpeleChain log (append-only, tamper-evident)
8. Deploy artifact to target environment
9. Run health check (if configured)
   → hosted: HTTP GET to health_check URL, expect 200
   → wasm:   instantiate and call _health() export if present
   → embedded: not applicable
10. Promote environment marker (dev → staging → production)
```

### 35.5 The Deployment Log — OpeleChain Integration

Every deployment is recorded as an immutable entry in the project's deployment OpeleChain (stored at `.oja/deploy.chain`):

```
ifa deploy history

Deployment history for my-app:
────────────────────────────────────────────────────────────────────────
#4  2026-03-16 14:32:01  hosted   v1.2.0  sha256:a3f8...  ✓ healthy
#3  2026-03-15 09:11:44  wasm     v1.1.0  sha256:b9d2...  ✓ healthy
#2  2026-03-14 16:05:22  hosted   v1.1.0  sha256:b9d2...  ✗ health check failed
#1  2026-03-13 11:00:09  hosted   v1.0.0  sha256:c41e...  ✓ healthy
────────────────────────────────────────────────────────────────────────
Chain integrity: ✓ verified (all 4 entries consistent)
```

Because the log is an OpeleChain (§30), past entries are immutable. A deployment cannot be retroactively edited or deleted. `ifa deploy --rollback` does not modify past entries — it creates a new entry that points to a previous artifact.

### 35.6 Environment Promotion

Named environments allow controlled promotion from development through to production:

```toml
[deploy.environments]
development = { auto_deploy = true,  requires_approval = false }
staging     = { auto_deploy = false, requires_approval = false }
production  = { auto_deploy = false, requires_approval = true  }
```

```bash
ifa deploy --env development    # deploy immediately (auto)
ifa deploy --env staging        # deploy after confirmation
ifa deploy --env production     # deploy after approval + health check pass
```

Promotion between environments always creates a new OpeleChain entry. The entry records which environment marker was promoted from and to, who approved, and the SHA-256 of the artifact.

### 35.7 Capability Enforcement at Deployment

The capability declarations in `ifa.toml` become hard constraints at deployment time — not just advisory. For the `wasm` and `hosted` targets, the deployment manager configures the sandbox before the artifact is executed:

**Hosted target** — `ifa-sandbox` `NativeRuntime` is configured from `ifa.toml [capabilities]`:

```rust
// Generated from ifa.toml [capabilities] at deploy time
let caps = CapabilitySet::new()
    .grant(Ofun::ReadFiles { root: "/tmp".into() })
    .grant(Ofun::WriteFiles { root: "/tmp".into() })
    .grant(Ofun::Network { domains: vec!["api.example.com".into()] });

let runtime = NativeRuntime::new(caps);
// Every domain call is checked against runtime before execution
```

**WASM target** — `SandboxConfig` is built from `ifa.toml [capabilities]` and passed to `OmniBox`:

```rust
let config = SandboxConfig::new(SecurityProfile::Standard)
    .with_capability(Ofun::ReadFiles { root: "./data".into() })
    .with_capability(Ofun::Network { domains: vec!["cdn.example.com".into()] });

let omnibox = OmniBox::new(config)?;
omnibox.run_module(&module)?;
```

**Embedded target** — capabilities are enforced at compile time only. The `[capabilities]` section for embedded targets serves as documentation and is verified by the Babalawo but is not enforced at runtime (no OS, no sandbox).

### 35.8 Ẹbọ Contracts at Deployment

When the deployment target includes Ẹbọ-annotated types (`#[derive(Ebo)]`), the deployment manager verifies that the artifact's entry point returns without any outstanding Ẹbọ obligations. This is checked by the Babalawo as part of step 2 of the pipeline.

A program that opens a file with `Odi.si()` in its `main()` but exits without calling `Odi.pa()` **MUST** be rejected at deployment, not just warned. The Ìwà Engine (§23) is the enforcement mechanism; the deployment manager adds the policy that this check is a hard gate.

### 35.9 Rollback

```bash
ifa deploy --rollback [--to <entry_number>]
```

Rollback retrieves the artifact recorded in the specified (or most recent previous) OpeleChain entry and re-deploys it to the current environment. It does not modify the deployment log — it creates a new entry:

```
#5  2026-03-16 15:00:00  hosted   v1.1.0  sha256:b9d2...  ↩ rollback from #4
```

The `sha256` of the rolled-back artifact **MUST** match the checksum recorded in the original entry. If the artifact cannot be found or its checksum has changed, the rollback **MUST** fail with an `OjaIntegrityError`.

---

## Appendix F — Ọjà Package Manifest Reference

Complete `ifa.toml` field reference.

### `[project]` — Required

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Package name. Lowercase, hyphens allowed. |
| `version` | String | Yes | Semantic version: `MAJOR.MINOR.PATCH` |
| `edition` | String | Yes | IfáLang edition year (e.g., `"2026"`) |
| `description` | String | No | Short description for registry listing |
| `authors` | `[String]` | No | `["Name <email>"]` format |
| `license` | String | No | SPDX license identifier |
| `repository` | String | No | Source repository URL |
| `readme` | String | No | Path to README file |
| `keywords` | `[String]` | No | Up to 5 registry search keywords |
| `odu_namespace` | String | No | Registry namespace (e.g., `"odi"`, `"otura"`) |

### `[dependencies]` and `[dev-dependencies]`

Values are either a version string or a table:

```toml
[dependencies]
ifa-http = "2.1"                                    # version constraint
ifa-crypto = { version = "3.0", features = ["argon2"] }  # with features
ifa-local  = { path = "../local-crate" }            # local path dependency
ifa-git    = { git = "https://...", branch = "main" }  # git dependency
```

### `[capabilities]`

| Field | Type | Description |
|-------|------|-------------|
| `read` | `[String]` | Filesystem paths allowed for reading |
| `write` | `[String]` | Filesystem paths allowed for writing |
| `network` | `[String]` | Allowed network domains (`"*"` for all) |
| `execute` | `[String]` | Allowed absolute paths for process spawning |
| `env` | `[String]` | Allowed environment variable keys (`"*"` for all) |
| `time` | Bool | Allow high-resolution time access |
| `random` | Bool | Allow random number generation |

### `[deploy.<target>]`

| Field | Type | Description |
|-------|------|-------------|
| `profile` | String | `"release"` \| `"debug"` \| `"development"` |
| `security` | String | `"untrusted"` \| `"standard"` \| `"development"` |
| `entry` | String | Path to program entry point `.ifa` file |
| `output_dir` | String | Directory for build artifacts |
| `aot` | Bool | (WASM only) Pre-compile to `.cwasm` |
| `target` | String | (Embedded only) Rust target triple |
| `mmio_base` | String | (Embedded only) MMIO address boundary (hex) |
| `health_check` | String | (Hosted only) URL for health check GET request |

---

## Appendix G — DAP Request/Response Reference

Standard DAP requests supported by IfáLang runtimes. All use JSON-RPC 2.0 over a TCP socket (default port 4711).

### Initialize

```json
// Request
{ "command": "initialize", "arguments": {
    "clientName": "VS Code",
    "adapterID": "ifalang",
    "linesStartAt1": true,
    "columnsStartAt1": true,
    "supportsRunInTerminalRequest": true,
    "supportsStepBack": true
}}

// Response — IfáLang DAP capabilities
{ "body": {
    "supportsConfigurationDoneRequest": true,
    "supportsFunctionBreakpoints": true,
    "supportsConditionalBreakpoints": true,
    "supportsStepBack": true,
    "supportsRestartFrame": false,
    "supportsGotoTargetsRequest": false,
    "supportsCompletionsRequest": true,
    "supportsExceptionOptions": true,
    "supportsValueFormattingOptions": true,
    "supportsExceptionInfoRequest": true,
    "supportTerminateDebuggee": true,
    "supportsSetVariable": true
}}
```

### IfáLang-Specific DAP Extensions

IfáLang adds the following non-standard fields to standard DAP responses:

**`stackFrames` extension:** Each frame includes an `odu` field if the frame is a domain call:

```json
{
  "id": 3,
  "name": "Ika.gigun",
  "odu": { "name": "Ìká", "binary": "0100", "title": "The Constrictor" },
  "source": { "path": "main.ifa" },
  "line": 7, "column": 12
}
```

**`variables` extension:** IfáLang values include a `ifa_type` field alongside the standard `type`:

```json
{
  "name": "counter",
  "value": "5",
  "type": "Int",
  "ifa_type": "Int (i64, mutable)",
  "variablesReference": 0
}
```

**`output` event for Babalawo diagnostics:** Uses category `"ifa-babalawo"`:

```json
{
  "type": "event",
  "event": "output",
  "body": {
    "category": "ifa-babalawo",
    "output": "warning[Ọ̀kànràn] main.ifa:15:5\n  Variable 'temp' is unused\n",
    "source": { "path": "main.ifa" },
    "line": 15, "column": 5
  }
}
```

---

## Appendix C — Static Analysis Rule Summary

Quick reference for the complete Babalawo rule set, indexed by Odù domain.

| Odù | Title | Error Codes Governed |
|-----|-------|---------------------|
| Ogbè | The Light | `UNDEFINED_VARIABLE`, `UNINITIALIZED`, `NULL_REFERENCE` |
| Ọ̀yẹ̀kú | The Darkness | `UNCLOSED_RESOURCE`, `ORPHAN_PROCESS`, `INCOMPLETE_SHUTDOWN` |
| Ìwòrì | The Mirror | `INFINITE_LOOP`, `ITERATOR_EXHAUSTED`, `LOOP_INVARIANT_VIOLATED` |
| Òdí | The Vessel | `FILE_NOT_FOUND`, `FILE_NOT_CLOSED`, `PERMISSION_DENIED`, `PRIVATE_ACCESS` |
| Ìrosù | The Speaker | `FORMAT_ERROR`, `OUTPUT_OVERFLOW` |
| Ọ̀wọ́nrín | The Chaotic | `SEED_ERROR` |
| Ọ̀bàrà | The King | `OVERFLOW`, `ARITHMETIC_ERROR` |
| Ọ̀kànràn | The Troublemaker | `UNHANDLED_EXCEPTION`, `ASSERTION_FAILED`, `UNUSED_VARIABLE` |
| Ògúndá | The Cutter | `INDEX_OUT_OF_BOUNDS`, `ARRAY_EMPTY` |
| Ọ̀sá | The Wind | `UNREACHABLE_CODE`, `INVALID_JUMP`, `MISSING_RETURN` |
| Ìká | The Constrictor | `INVALID_ENCODING`, `STRING_OVERFLOW` |
| Òtúúrúpọ̀n | The Bearer | `UNDERFLOW`, `DIVISION_BY_ZERO` |
| Òtúrá | The Messenger | `CONNECTION_REFUSED`, `TIMEOUT`, `NETWORK_UNREACHABLE` |
| Ìrẹtẹ̀ | The Crusher | `MEMORY_LEAK`, `DOUBLE_FREE`, `OUT_OF_MEMORY` |
| Ọ̀ṣẹ́ | The Beautifier | `INVALID_COORDINATES`, `BUFFER_OVERFLOW` |
| Òfún | The Creator | `TYPE_ERROR`, `INHERITANCE_ERROR`, `OBJECT_NOT_FOUND`, `TYPE_MISMATCH` |

---

## Appendix D — Lifecycle Rules Quick Reference

| Opener | Closer | Domain | Auto-Close? |
|--------|--------|--------|-------------|
| `Odi.si()` | `Odi.pa()` | Òdí | No |
| `Odi.kọ()` | `Odi.pa()` | Òdí | No |
| `Otura.de()` | `Otura.pa()` | Òtúrá | No |
| `Otura.so()` | `Otura.pa()` | Òtúrá | No |
| `Ogunda.ge()` | `Irete.tu()` | Ògúndá/Ìrẹtẹ̀ | No |
| `Ogunda.da()` | `Irete.tu()` | Ògúndá/Ìrẹtẹ̀ | No |
| `Ofun.da()` | `Ofun.pa()` | Òfún | No |
| `Iwori.yipo()` | `Iwori.pada()` | Ìwòrì | No |
| `Ogbe.bi()` | `Oyeku.duro()` | Ògbè/Ọ̀yẹ̀kú | **Yes** |
| `Ogbe.bere()` | `Oyeku.duro()` | Ògbè/Ọ̀yẹ̀kú | **Yes** |
| `ebo.begin` | `ebo.sacrifice` | Ẹbọ | No |

---

## Appendix E — TypeHint System

IfáLang supports optional static type annotations. When a type hint is present on a declaration, the Babalawo performs type checking at both declaration and assignment points.

| TypeHint | Description | Requires ailewu? |
|----------|-------------|-----------------|
| `Int` | Dynamic integer (i64-range) | No |
| `Float` | Dynamic float (f64) | No |
| `Str` | Dynamic string | No |
| `Bool` | Boolean | No |
| `List` | Dynamic list | No |
| `Map` | Dynamic map | No |
| `Any` | No type checking (default) | No |
| `i8` / `i16` / `i32` / `i64` | Sized signed integers | No (embedded: `i32`/`i64` ok; smaller sizes need `ailewu`) |
| `u8` / `u16` / `u32` / `u64` | Sized unsigned integers | No (same as above) |
| `f32` / `f64` | Sized floats | No |
| `*T` (pointer) | Raw pointer to T | **Yes** |
| `&T` (reference) | Immutable reference | Tracked by Ìwà Engine |
| `&mut T` (mut reference) | Mutable reference | Tracked by Ìwà Engine |

Type compatibility rules:
- `Any` is compatible with all types
- Sized integer types are compatible with the dynamic `Int` type
- `Float` / `f64` are compatible with `Int` / integer literals (promotion)
- `ofo` (null) is compatible with any type (null can be assigned to any typed variable)

---


---


---


---

## 36. The Language Server — `ifa lsp` `[DEFINED]`

> *Ẹni tó bá ń gbọ́ ọ̀rọ̀, yóò mọ ohun tí a ń sọ. — One who listens will understand what is being said.*

The IfáLang Language Server implements the [Language Server Protocol (LSP)](https://microsoft.github.io/language-server-protocol/) — the JSON-RPC 2.0 standard for editor-agnostic language intelligence. It is a standalone process started by editors and communicated with over stdin/stdout or a TCP socket. It is not a plugin. Any editor that speaks LSP (VS Code, Neovim, Emacs, Helix, Zed, IntelliJ) gets full IfáLang intelligence without bespoke integration.

### 36.1 Starting the Language Server

```bash
ifa lsp                          # stdio transport (default — for editors)
ifa lsp --tcp [--port 2087]      # TCP transport (for remote/container use)
ifa lsp --version                # print LSP server version and exit
```

The server announces its capabilities on `initialize` and begins serving requests. It is stateful: it maintains an in-memory representation of all open `.ifa` files, re-running the Babalawo incrementally on every change.

### 36.2 LSP Capabilities

| LSP Feature | Support | Backed by |
|-------------|---------|-----------|
| `textDocument/publishDiagnostics` | ✓ | Babalawo §22 |
| `textDocument/completion` | ✓ | Odù domain registry + scope resolver |
| `textDocument/hover` | ✓ | Babalawo wisdom + domain method signatures |
| `textDocument/definition` | ✓ | Resolver pass binding annotations |
| `textDocument/references` | ✓ | Resolver pass |
| `textDocument/rename` | ✓ | Resolver pass + text edit generation |
| `textDocument/formatting` | ✓ | `ifa fmt` engine §40 |
| `textDocument/rangeFormatting` | ✓ | `ifa fmt` engine §40 |
| `textDocument/codeAction` | ✓ | Babalawo fix suggestions (see §36.4) |
| `textDocument/inlayHint` | ✓ | Inferred types on `ayanmo`, Odù domain on calls |
| `textDocument/semanticTokens` | ✓ | Keyword, domain, Odù-domain-specific token types |
| `workspace/symbol` | ✓ | All `ese`, `odu`, `ayanfe` declarations |
| `workspace/executeCommand` | ✓ | `ifa.castOpele`, `ifa.showDomainWisdom` |

### 36.3 Odù-Aware Completion

Completion in IfáLang is domain-aware. When a developer types `Ika.`, the completion list shows all `Ika` domain methods with their signatures, return types, and the Odù's philosophical domain:

```
Ika.gigun(s)       → Int      [Ìká: The Constrictor — string length]
Ika.oke(s)         → String   [Ìká: uppercase]
Ika.isale(s)       → String   [Ìká: lowercase]
Ika.pin(s, sep)    → List     [Ìká: split by separator]
...
```

Domain methods are grouped under their Odù name in the completion UI. Methods from outside the current Odù appear in a separate "Other Domains" group to signal the ontological boundary.

For user-defined variables and functions, completion uses the resolver pass binding annotations (§2.4) to show only names that are in scope at the cursor position. Shadowed names are shown with a strikethrough.

### 36.4 Wisdom Injection on Hover

When the cursor hovers over an error or warning diagnostic, the LSP sends a `textDocument/hover` response that includes the full Babalawo diagnostic with Odù name, proverb, and fix suggestion:

```markdown
**error[Òdí]** — UNCLOSED_RESOURCE

`Odi.si()` opened at line 14 was never closed.
Call `Odi.pa(handle)` before the function returns.

---
*Òdí — The Vessel*
> Ṣàpẹẹrẹ ohun tí a tọ́jú, kó mọ́ rẹ — Guard well what you store,
> that it may be found clean.

**Quick fix:** Insert `Odi.pa(handle);` before return →
```

The proverb is always shown on hover regardless of the `--wisdom` flag — hover is interactive and opt-in, unlike terminal output which can be overwhelming at scale.

### 36.5 Code Actions

The LSP exposes Babalawo diagnostic fixes as LSP code actions (the "💡 Quick Fix" lightbulb in VS Code):

| Diagnostic | Code Action |
|------------|-------------|
| `UNCLOSED_RESOURCE` | Insert closer call at end of scope |
| `UNUSED_VARIABLE` | Prefix name with `_` to suppress |
| `MISSING_RETURN` | Insert `pada ofo;` at end of function |
| `TYPE_MISMATCH` | Insert explicit cast or remove type annotation |
| `CAPABILITY_UNDECLARED` | Add capability to `ifa.toml [capabilities]` |
| Non-exhaustive `ode` | Add `_ => ofo` wildcard arm |
| `REFERENCE_CYCLE` | Break cycle with `Irete.tu()` or restructure |

### 36.6 Custom LSP Commands

```
ifa.castOpele         # randomly select an Odù and show its wisdom in the editor
ifa.showDomainWisdom  # show the wisdom for the Odù domain at the cursor
ifa.runBabalawo       # force a full Babalawo re-analysis of the current file
ifa.explainError      # open a side panel with full error explanation + examples
```

### 36.7 Semantic Token Types

The LSP emits custom semantic token types for Odù-aware syntax highlighting:

| Token type | Applies to |
|------------|-----------|
| `odu-domain` | Domain names (`Ika`, `Obara`, `Otura`, ...) |
| `odu-method` | Domain method calls (`Ika.gigun`, `Otura.gba`, ...) |
| `keyword-yoruba` | Yoruba-form keywords (`ayanmo`, `ese`, `ti`, ...) |
| `keyword-english` | English-alias keywords (`let`, `fn`, `if`, ...) |
| `upvalue` | Variables captured from enclosing scope |
| `lifecycle-opener` | Calls that open a resource (`Odi.si`, `Otura.so`, ...) |
| `lifecycle-closer` | Calls that close a resource (`Odi.pa`, `Otura.pa`, ...) |
| `iwa-balanced` | A function decorated with `#[iwa_pele]` that is balanced |
| `iwa-violation` | A function decorated with `#[iwa_pele]` that has a violation |

---

## 37. The Debug Adapter — `ifa debug` `[DEFINED]`

> *Ẹni tó bá fẹ́ rí ohun tó farapamọ́ gbọdọ̀ wo inú. — One who wants to see what is hidden must look inside.*

`ifa debug` is the canonical DAP entry point — a first-class CLI command that starts a dedicated debug session without requiring the program to be started separately with a `--dap` flag. It uses the same DAP protocol as §34 but provides a cleaner user experience and exposes the `Debugger` trait for runtime extensibility.

### 37.1 CLI Interface

```bash
ifa debug program.ifa              # debug with AST interpreter (default)
ifa debug --vm program.ifa         # debug with bytecode VM
ifa debug --embedded program.ifa   # debug embedded program (MMIO inspection)
ifa debug --port 4711 program.ifa  # listen on specific port
ifa debug --no-dap program.ifa     # TUI debugger mode (no IDE required)
```

When started without `--no-dap`, `ifa debug` starts a DAP server, prints the port, and waits for a client connection before executing the program. When started with `--no-dap`, it launches a built-in terminal UI debugger.

### 37.2 The `Debugger` Trait

Every runtime that participates in `ifa debug` **MUST** implement the `Debugger` trait. This trait is the interface between the DAP server and the execution engine:

```rust
pub trait Debugger {
    // Breakpoints
    fn set_breakpoint(&mut self, file: &str, line: u32) -> BreakpointId;
    fn remove_breakpoint(&mut self, id: BreakpointId);
    fn set_function_breakpoint(&mut self, name: &str) -> BreakpointId;

    // Execution control
    fn continue_execution(&mut self) -> DebugEvent;
    fn step_over(&mut self) -> DebugEvent;
    fn step_into(&mut self) -> DebugEvent;
    fn step_out(&mut self) -> DebugEvent;
    fn step_back(&mut self) -> Result<DebugEvent, CannotStepBack>;  // StateHistoryBuffer
    fn pause(&mut self);

    // State inspection
    fn stack_trace(&self) -> Vec<StackFrame>;
    fn scopes(&self, frame_id: usize) -> Vec<Scope>;    // Locals, Upvalues, Globals, Domains, Ìwà State
    fn variables(&self, scope_ref: usize) -> Vec<Variable>;
    fn evaluate(&mut self, expr: &str, frame_id: usize) -> Result<IfaValue, IfaError>;

    // IfáLang extensions
    fn iwa_state(&self) -> IwaEngineSnapshot;           // open resources + borrow ledger
    fn odù_scopes(&self) -> Vec<OduDomainStatus>;       // all 16 domains + availability
}
```

The AST interpreter and bytecode VM each implement `Debugger`. The `ifa debug` command accepts any `Box<dyn Debugger>` and drives it via the DAP protocol or TUI.

### 37.3 The TUI Debugger (`--no-dap`)

When `--no-dap` is specified, `ifa debug` renders a terminal UI with four panes:

```
┌─ Source ─────────────────────────────────┬─ Variables ──────────────────┐
│  12    ayanmo conn = Otura.so("...", 443);│ Locals:                      │
│▶ 13    ayanmo data = conn.recv(1024);     │   conn  : Connection         │
│  14    conn.pa();                         │   data  : ofo                │
│  15  }                                    │ Ìwà State:                   │
│                                           │   ⚠ Otura.so (line 12) open  │
├─ Call Stack ─────────────────────────────┼─ Babalawo ───────────────────┤
│▶ fetch_data     main.ifa:13              │ warning[Ọ̀sá] line 15         │
│  main           main.ifa:28             │   Function may not return     │
│                                          │   on all paths               │
└──────────────────────────────────────────┴──────────────────────────────┘
[n]ext  [s]tep  [o]ut  [c]ontinue  [b]reak  [r]ewind  [q]uit
```

The Ìwà State pane shows all open resources and their opener locations, updating in real time as the program executes.

### 37.4 Relationship to §34

§34 documents the DAP protocol details (request/response format, opcode-level integration, StateHistoryBuffer). §37 documents the CLI command and the `Debugger` trait. They are complementary — §37 is the user-facing contract, §34 is the protocol-level contract.

---

## 38. The Documentation Generator — `ifa doc` `[DEFINED]`

> *Ọ̀rọ̀ tó dára jẹ́ fún gbogbo ènìyàn. — Good words are a gift to everyone.*

`ifa doc` generates HTML documentation from IfáLang source files. It is not a Markdown-to-HTML converter — it is a **corpus-style generator** that visually organizes documentation by Odù domain, showing the philosophical character of each public API alongside its technical specification.

### 38.1 CLI Interface

```bash
ifa doc                           # generate docs for the current project
ifa doc --open                    # generate and open in browser
ifa doc --output ./docs           # specify output directory
ifa doc --theme odù               # use the 16-domain visual theme (default)
ifa doc --theme plain             # plain HTML without Odù styling
ifa doc src/lib.ifa               # document a specific file
```

Output is written to `./target/doc/` by default, mirroring the `oja doc` convention.

### 38.2 Doc Comment Syntax

Doc comments use triple-hash `###` for items (functions, classes, constants) and `##` for module-level documentation:

```
## The shapes module provides geometric calculations.
## All area calculations use Float64 precision.

### Compute the area of a circle.
###
### @param radius - the circle's radius (must be > 0)
### @returns Float - the area in square units
### @throws UserError if radius <= 0
### @domain Obara - mathematical expansion
ese circle_area(radius) {
  ti radius <= 0 { ta "radius must be positive"; }
  pada Obara.pi() * radius ** 2;
}
```

### 38.3 The `@domain` Tag

The `@domain` tag is unique to IfáLang documentation. It declares which Odù domain governs the character of this function. The documentation generator uses this tag to:

1. **Group functions** under their governing Odù in the domain-organized index
2. **Render the Odù's visual identity** (binary pattern, title, proverb) alongside the function
3. **Cross-link** to all other functions in the same Odù domain
4. **Warn** (via Babalawo) if the function's actual domain calls contradict the declared `@domain`

If `@domain` is omitted, the generator infers the domain from the first stdlib domain call in the function body. If no stdlib call exists, the function is placed in a "General" category.

### 38.4 Domain-Organized HTML Output

The generated documentation has two views:

**Alphabetical view** — traditional function listing, alphabetical by name.

**Odù view** — the default. Functions are organized into 16 sections, one per Odù domain. Each section opens with the Odù's binary pattern, Yoruba name, English title, and governing proverb, then lists all functions whose character belongs to that domain.

```html
<!-- Example rendered section -->
<section class="odu-domain" data-odu="OBARA" data-binary="1000">
  <header>
    <div class="binary-pattern">│ │ │ │</div>
    <h2>Ọ̀bàrà — The King</h2>
    <p class="proverb">Àgbàdo tó bá gbó, máa ń ní igi rẹ̀ gígùn.
      <em>The corn that grows tall always has deep roots.</em></p>
  </header>
  <div class="functions">
    <article id="circle_area">
      <h3>circle_area(radius) → Float</h3>
      <p class="doc">Compute the area of a circle...</p>
      <pre class="signature">ese circle_area(radius: Float) → Float</pre>
    </article>
  </div>
</section>
```

### 38.5 Doc Test Execution

Code blocks in doc comments are executed as tests when `ifa test --doc` is run. A doc test fails if the code throws an error or if a line ending with `# => value` produces a different value:

```
### @example
### ```
### circle_area(5.0)  # => 78.539816...
### circle_area(-1)   # throws UserError
### ```
```

### 38.6 Babalawo Integration

The doc generator runs the Babalawo on every documented item. If a `@domain` tag contradicts the inferred domain (e.g., a function tagged `@domain Odi` that only makes `Otura` network calls), the generator emits:

```
warning[Òfún] shapes.ifa:8
  @domain Odi declared but function only calls Otura methods.
  The Creator requires accurate self-description.
  Consider: @domain Otura
```

---

## 39. Zero-Config Deployment — `ifa deploy` and `Iwe.toml` `[DEFINED]`

> *Àṣírí tó bá jáde, kò ní padà sẹ́yìn. — A secret that has escaped cannot return.*

`Iwe.toml` (iwe = Yoruba for document/paper) is the **deployment manifest** — a separate file from `ifa.toml` that is generated by `ifa deploy analyze` and committed to version control as the authoritative record of what a deployed program is allowed to do. It is not hand-edited. It is generated by the capability scanner, reviewed by the developer, and enforced by the runtime.

This separates concerns cleanly: `ifa.toml` is the project manifest (what you build), `Iwe.toml` is the deployment contract (what you allow to run).

### 39.1 The Capability Scanner

`ifa deploy analyze` performs a deep static analysis of the program's capability surface. It goes beyond the Babalawo's inference pass (§29) — it does not just look at direct domain calls, it follows the call graph transitively through all dependencies.

```bash
ifa deploy analyze                    # analyze current project, print report
ifa deploy analyze --output Iwe.toml  # generate Iwe.toml
ifa deploy analyze --compare Iwe.toml # compare current code against existing manifest
ifa deploy analyze --strict           # fail if any capability cannot be statically inferred
```

The scanner produces a capability report showing:

```
ifa deploy analyze

Capability Analysis for my-app v1.0.0
══════════════════════════════════════

Direct calls (your code):
  ✓ ReadFiles  { root: "./data" }          — src/main.ifa:14  Odi.ka()
  ✓ WriteFiles { root: "/tmp" }            — src/main.ifa:31  Odi.kọ()
  ✓ Network    { domains: ["api.example.com"] } — src/fetch.ifa:8  Otura.gba()
  ✓ Stdio                                  — (always inferred)

Transitive (from dependencies):
  ✓ Random                                 — ifa-http@2.1.3  (nonce generation)
  ✓ Time                                   — ifa-http@2.1.3  (request timestamps)
  ⚠ Network    { domains: ["*"] }         — ifa-http@2.1.3  (dynamic redirect)
    → ifa-http follows redirects to dynamic domains.
      Declare explicitly if redirect following is intended.

Dynamic paths (cannot be fully inferred):
  ⚠ src/config.ifa:22  Odi.ka(config_path)
    → Path is dynamic. Declare the root explicitly in Iwe.toml.

Suggested Iwe.toml:
  → Run with --output Iwe.toml to generate
```

### 39.2 The `Iwe.toml` Format

```toml
# Iwe.toml — Deployment Manifest
# Generated by: ifa deploy analyze v0.2
# Project: my-app v1.0.0
# Generated: 2026-03-16T14:32:01Z
# SHA-256 of analyzed source: a3f8c...
#
# REVIEW THIS FILE before committing.
# Every entry is a capability your program has at runtime.
# Capabilities not listed here will produce PermissionError.

[manifest]
schema_version = "1"
project        = "my-app"
version        = "1.0.0"
generated_by   = "ifa deploy analyze"

[capabilities]
read    = ["./data", "./config"]
write   = ["/tmp"]
network = ["api.example.com"]
# network = ["*"]   # uncomment if redirect following is required (see analysis warning)
time    = true
random  = true
stdio   = true

[dynamic_paths]
# Paths that could not be fully inferred statically.
# Add the root directories that cover all dynamic paths your program uses.
# src/config.ifa:22 — Odi.ka(config_path)
config_read = ["./", "/etc/my-app"]

[transitive_capabilities]
# Capabilities required by dependencies.
# These are automatically enforced — listed here for audit visibility.
"ifa-http@2.1.3" = ["random", "time", "network:*"]

[audit]
# Capability counts for change detection
total_direct       = 5
total_transitive   = 3
dynamic_path_count = 1
last_verified      = "2026-03-16T14:32:01Z"
```

### 39.3 Runtime Enforcement

When `ifa deploy` uses an `Iwe.toml`, the sandbox is configured from `Iwe.toml`, **not** from `ifa.toml [capabilities]`. The `Iwe.toml` is the authoritative runtime contract. `ifa.toml [capabilities]` is the developer's declaration of intent; `Iwe.toml` is the scanner's verified result. If they differ, `ifa deploy` warns and requires explicit confirmation.

### 39.4 Comparison and Drift Detection

`ifa deploy analyze --compare Iwe.toml` detects capability drift between the current code and the committed manifest:

```
ifa deploy analyze --compare Iwe.toml

Comparing current code against Iwe.toml (last verified 2026-03-15)...

ADDED capabilities (not in Iwe.toml):
  ⚠ Execute { programs: ["/usr/bin/ffmpeg"] }  — src/media.ifa:44  Ogunda.ise()
    → New process spawning call added since last manifest generation.

REMOVED capabilities (in Iwe.toml but no longer called):
  ✓ WriteFiles { root: "/var/log" }  — was in Iwe.toml but no code uses it.
    → Consider removing from Iwe.toml to reduce attack surface.

Recommendation: Re-run  ifa deploy analyze --output Iwe.toml  to update.
```

This check runs automatically as step 3 of the deployment pipeline (§35.4).

### 39.5 Relationship to §35

§35 documents the deployment pipeline, `ifa.toml [deploy]` configuration, and the `ifa deploy` CLI. §39 documents the capability scanner and `Iwe.toml` as the security layer within that pipeline. In the updated pipeline, step 3 reads:

> **3. Generate or verify `Iwe.toml` capability manifest**  
> If `Iwe.toml` does not exist: run `ifa deploy analyze --output Iwe.toml` and pause for developer review.  
> If `Iwe.toml` exists: run `ifa deploy analyze --compare Iwe.toml`. Reject on any ADDED capability without explicit `--allow-new-capabilities` flag.

---

## 40. The Formatter — `ifa fmt` `[DEFINED]`

> *Ọ̀nà tí a gbà mọ ní àtẹ̀lẹwọ́ ni a fi ń rìn. — The path we know from the palm of our hand is the one we walk.*

`ifa fmt` is the canonical IfáLang formatter. It is opinionated — there are no style options beyond a small set of project-level overrides. The formatter operates on the token stream, not the AST, preserving comments exactly as written and only rearranging whitespace and structure. It is idempotent: `ifa fmt (ifa fmt source)` == `ifa fmt source`.

> ⚠️ **Implementation status:** `ifa-fmt` currently has a 30-line unresolved internal monologue in the `Token::Comment` arm and zero test coverage. This section specifies the *intended* behavior. The existing code must be rewritten to conform to this spec before `ifa fmt` can be used in production. Until conformance is verified, `ifa fmt` **MUST** be gated behind `--unstable` and **MUST** fail with a clear error if invoked without that flag.

### 40.1 CLI Interface

```bash
ifa fmt program.ifa               # format file in place
ifa fmt --check program.ifa       # check only — exit 1 if formatting needed
ifa fmt --diff program.ifa        # show diff without modifying
ifa fmt --unstable program.ifa    # required until ifa-fmt passes conformance tests
ifa fmt ./src/                    # format all .ifa files in a directory
ifa fmt --stdin < program.ifa     # format from stdin, write to stdout
```

### 40.2 Formatting Rules

The formatter enforces the following rules. These are **not** configurable — they are the canonical IfáLang style.

#### Indentation
- 2 spaces. No tabs. Ever.
- Continuation lines (wrapped expressions) indent 4 spaces from the opening line.

#### Blank lines
- **2 blank lines** before and after top-level `ese` and `odu` definitions.
- **1 blank line** between methods inside an `odu` body.
- **0 blank lines** at the start or end of any block body.
- No more than **1 consecutive blank line** anywhere in a file.

#### Semicolons
- Semicolons are required as specified in §6.1. The formatter inserts them where missing and removes any duplicate or trailing semicolons.

#### Braces
- Opening brace `{` on the same line as its statement. Never on a new line.
- Closing brace `}` on its own line, at the same indentation as the opening statement.

```
# Correct:
ese area(r) {
  pada Obara.pi() * r ** 2;
}

# Formatter will fix:
ese area(r)
{
    pada Obara.pi() * r ** 2
}
```

#### Operators
- Single space around all binary operators: `a + b`, not `a+b`.
- No space between unary operator and operand: `-x`, not `- x`.
- No space inside parentheses: `(a + b)`, not `( a + b )`.
- Single space after comma: `f(a, b, c)`, not `f(a,b,c)`.

#### Domain calls
- Domain name is always capitalized as defined: `Ika`, `Obara`, never `ika`, `obara`.
- Method name is always as defined (with diacritics): `Ika.gigun`, never `Ika.Gigun`.

#### Line length
- Soft limit: 100 characters. The formatter attempts to keep lines within 100 chars.
- Hard limit: 120 characters. Lines exceeding 120 chars that the formatter cannot break automatically produce a `FMT_LINE_TOO_LONG` warning.

#### Comments
- Line comments (`#`) are preserved exactly, including all whitespace and diacritics.
- A single space is inserted between `#` and the comment text if absent: `# comment`, not `#comment`.
- Block comments (`#{ ... }#`) are preserved exactly — the formatter does not reflow comment text.

#### String literals
- String contents are never modified by the formatter.
- String interpolation expressions (`$"...{expr}..."`) are formatted: the expression inside `{}` follows standard expression formatting.

### 40.3 Idempotence Requirement

The formatter **MUST** be idempotent. Running `ifa fmt` twice on any source file **MUST** produce the same output as running it once. The conformance test for `ifa fmt` is:

```
for every test_file in tests/fmt/:
    assert fmt(fmt(test_file)) == fmt(test_file)
    assert fmt(test_file) == test_file.expected
```

This test **MUST** pass before `ifa fmt` is permitted to run without `--unstable`.

### 40.4 The `ifa.toml [fmt]` Overrides

A small number of project-level overrides are permitted:

```toml
[fmt]
line_length  = 100      # soft limit (default: 100, range: 80-120)
hard_limit   = 120      # hard limit (default: 120, range: 100-160)
# All other style rules are fixed and not configurable.
```

No per-file or per-function overrides exist. Style consistency across a project is non-negotiable in IfáLang — this reflects the Ifá principle of ìwà pẹ̀lẹ́: gentle, consistent character.

### 40.5 LSP Integration

When `ifa lsp` is running, the LSP's `textDocument/formatting` request is served by `ifa fmt`. The formatter runs on the in-memory document buffer and returns text edits. No file is written to disk during LSP formatting — the editor applies the edits.

### 40.6 CI Integration

```yaml
# Example GitHub Actions step
- name: Check IfáLang formatting
  run: ifa fmt --check ./src/
  # Exit code 0: all files formatted correctly
  # Exit code 1: one or more files need formatting
  # Diff is printed to stdout for review
```

---

## Appendix H — Canonical Toolchain Summary

The complete IfáLang toolchain. Every command available after installing `ifa`.

### Core Execution

| Command | Description | Spec |
|---------|-------------|------|
| `ifa run program.ifa` | AST interpreter — development tier | §2 |
| `ifa runb program.ifa` | Bytecode VM — execution tier, canonical semantics | §2, §17 |
| `ifa build program.ifa` | Transpile to Rust — deployment tier | §2 |

### Package Management

| Command | Description | Spec |
|---------|-------------|------|
| `oja add <pkg>` | Add dependency | §33 |
| `oja install` | Install all dependencies | §33 |
| `oja publish` | Publish to registry | §33 |
| `oja audit` | Security audit | §33 |
| `oja new <name>` | Scaffold new project | §33 |

### Developer Tools

| Command | Description | Spec |
|---------|-------------|------|
| `ifa lsp` | Language server (LSP) | §36 |
| `ifa debug program.ifa` | Debug adapter (DAP + TUI) | §37, §34 |
| `ifa fmt program.ifa` | Formatter | §40 |
| `ifa doc` | Documentation generator | §38 |
| `ifa check program.ifa` | Run Babalawo only, no execution | §22 |

### Deployment

| Command | Description | Spec |
|---------|-------------|------|
| `ifa deploy` | Deploy to target | §35, §39 |
| `ifa deploy analyze` | Capability scanner → `Iwe.toml` | §39 |
| `ifa deploy analyze --compare Iwe.toml` | Drift detection | §39 |
| `ifa deploy --rollback` | Revert to previous artifact | §35 |
| `ifa deploy history` | OpeleChain deployment log | §35 |

### Testing

| Command | Description | Spec |
|---------|-------------|------|
| `ifa test` | Run test suite | §21 |
| `ifa test --doc` | Run doc tests | §38 |
| `ifa test --conformance` | Run conformance harness | §21 |

---

## 40.5 The Ìgbálẹ̀ Sandbox — OS-Level Isolation `[DEFINED]`

> *Igbálẹ̀ ni ibi tí gbogbo ìmọ̀ ń bọ̀ wá. — The sacred ground is where all knowledge descends.*

The spec references `NativeRuntime` and `OmniBox` as enforcement mechanisms for the hosted and WASM execution tiers (§35.7). This section specifies what those mechanisms actually do at the operating system level.

The Ìgbálẹ̀ (sacred ground) is the name for the complete two-layer sandboxing architecture:

- **Layer 1 — OS-level process isolation** (`Ìgbálẹ̀` proper): enforced by the operating system kernel using platform-specific primitives. Prevents the process from exceeding its resource and syscall limits regardless of what the IfáLang runtime allows.
- **Layer 2 — Capability enforcement** (`NativeRuntime` / `CapabilitySet`): enforced inside the IfáLang runtime at every domain call boundary. Prevents IfáLang code from using capabilities not declared in `ifa.toml`.

Layer 1 constrains what the process *can* do. Layer 2 constrains what IfáLang code *is allowed to ask for*. Both **MUST** be active for the `hosted` security profile to be meaningful. Layer 2 alone is not sufficient — a bug in the runtime or a malicious native library loaded via FFI could bypass Layer 2. Layer 1 is the hard floor.

### 40.5.1 Layer 1 — OS-Level Isolation

**Linux — cgroups v2 + namespaces + seccomp-bpf:**

For each hosted IfáLang program running under a non-`development` security profile, the runtime **MUST**:

**1. Spawn in a new set of Linux namespaces:**

| Namespace flag | Effect |
|---------------|--------|
| `CLONE_NEWPID` | Process cannot see or signal PIDs outside its namespace |
| `CLONE_NEWNET` | Fresh network stack; no interfaces except loopback unless `network` capability is declared |
| `CLONE_NEWNS` | New mount namespace; program cannot mount or unmount filesystems |
| `CLONE_NEWUSER` | Maps to an unprivileged UID; `setuid` binaries cannot escalate privileges |

**2. Apply cgroup v2 limits before exec:**

```
# Written to the cgroup before execve:
memory.max   = <profile memory limit>
cpu.max      = "<quota> <period>"   # e.g. "100000 100000" = 1 CPU core
pids.max     = 64                   # prevents fork bombs
```

**3. Apply a seccomp-bpf allowlist:**

| Syscall category | Allowed by default | Notes |
|-----------------|-------------------|-------|
| Memory | `mmap`, `munmap`, `mprotect`, `brk` | Always allowed |
| File I/O | `read`, `write`, `open`, `openat`, `close`, `stat`, `fstat`, `lseek`, `readlink` | Path prefix checked by runtime |
| Process | `exit`, `exit_group`, `getpid`, `gettid`, `clock_gettime`, `nanosleep` | Always allowed |
| Threading | `futex`, `clone` (restricted flags only), `set_robust_list` | `CLONE_NEWPID` flag blocked in clone |
| Signals | `rt_sigaction`, `rt_sigprocmask`, `rt_sigreturn` | Always allowed |
| Networking | `socket`, `connect`, `bind`, `listen`, `accept`, `sendto`, `recvfrom` | Blocked unless `network` capability declared |
| Dangerous | `ptrace`, `process_vm_readv`, `process_vm_writev`, `kexec_load`, `perf_event_open` | **Always blocked** |

Any blocked syscall delivers `SIGSYS` to the process. The runtime's `SIGSYS` handler **MUST** convert this to `IfaError::PermissionError` with the syscall name and number in the message, then terminate the program cleanly.

**4. Apply filesystem path restrictions:**

File open calls are validated against the declared `read` and `write` path prefixes from `ifa.toml [capabilities]`. A path not matching any declared prefix causes `EACCES`. This is enforced either via a bind-mount read-only overlay or via a syscall interception filter in the seccomp BPF program.

---

**macOS — Sandbox profiles + resource limits:**

macOS does not provide cgroups or Linux namespaces. The runtime **MUST**:

1. Call `sandbox_init` (or the newer `sandbox_apply`) with a generated sandbox profile that denies:
   - `network*` unless `network` capability declared
   - `file-write*` for paths not in declared `write` list
   - `process-exec*` unless `execute` capability declared
   - `mach-lookup` for services not required by the runtime

2. Apply `setrlimit` before exec:
   - `RLIMIT_AS`: virtual address space limit = profile memory limit × 4
   - `RLIMIT_DATA`: data segment = profile memory limit
   - `RLIMIT_CPU`: CPU seconds = profile CPU time limit
   - `RLIMIT_NPROC`: subprocess count = profile subprocess limit

3. Assign the process to a process group so a watchdog timer's `SIGKILL` reaches all child processes if the CPU time limit is exceeded.

---

**Windows — Job Objects + restricted token:**

1. Create a Job Object with the following limits:

   | Limit | Value |
   |-------|-------|
   | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` | Kills process when job handle closes (cleanup guarantee) |
   | `JOB_OBJECT_LIMIT_ACTIVE_PROCESS` | 8 (or 0 for `untrusted`) |
   | `JOB_OBJECT_LIMIT_JOB_MEMORY` | Profile memory limit in bytes |
   | `JOB_OBJECT_LIMIT_JOB_TIME` | Profile CPU time limit in 100ns units |
   | `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION` | `TRUE` — no crash dialogs |
   | `JOB_OBJECT_LIMIT_BREAKAWAY_OK` | `FALSE` — process cannot escape the job |

2. Create a restricted process token that removes:
   - `SeDebugPrivilege` (cannot attach to other processes)
   - `SeLoadDriverPrivilege` (cannot load kernel drivers)
   - `SeBackupPrivilege`, `SeRestorePrivilege` (cannot bypass file ACLs)
   - `SeTcbPrivilege`, `SeAssignPrimaryTokenPrivilege`

3. Set the process integrity level to `Low` (below `Medium`). Low-integrity processes cannot write to most registry keys, `%APPDATA%`, or `Program Files`.

4. For network restrictions: apply a Windows Filtering Platform (WFP) sublayer that blocks outbound connections from the process unless `network` capability is declared.

### 40.5.2 Layer 2 — CapabilitySet Enforcement

Every Odù domain call in the hosted tier passes through `NativeRuntime.check()` before executing. The check is synchronous and blocks execution:

```rust
// Called at every domain call boundary in the hosted tier
fn check(call: &OduCall, caps: &CapabilitySet) -> Result<(), IfaError> {
    let required = capability_for_call(call.domain, call.method);
    if caps.grants(&required) {
        Ok(())
    } else {
        Err(IfaError::PermissionError {
            message: format!(
                "Capability {:?} required for {}.{}()                  but not declared in ifa.toml [capabilities]",
                required, call.domain, call.method
            ),
            required_capability: required,
            call_site: call.source_location,
            granted: caps.list(),
        })
    }
}
```

A conforming `PermissionError` **MUST** include all four fields: the required capability, the call site (domain + method + source location), and the currently-granted capability list. A bare message string is non-conforming.

### 40.5.3 Security Profiles

The three predefined profiles configure both layers.

| Profile | CPU limit | Memory limit | Max subprocesses | Network | Filesystem |
|---------|-----------|--------------|-----------------|---------|------------|
| `untrusted` | 5 s | 64 MB | 0 | Blocked | Blocked |
| `standard` | 30 s | 256 MB | 8 | Blocked | `./` read-only unless declared |
| `development` | 300 s | 2 GB | 64 | Allowed | `./` read-write |

**The Layer 1 OS limits are a hard floor.** A program running under `untrusted` profile with `network = ["api.example.com"]` declared in `ifa.toml` will still be in a network namespace with no interfaces on Linux — Layer 1 takes precedence. The only way to grant network access under `untrusted` is to change the profile.

### 40.5.4 WASM Execution — OmniBox

For the WASM tier, Wasmtime itself provides memory safety (linear memory, no direct syscall access). The OmniBox adds resource limits and the EWO capability layer on top.

**Resource enforcement in OmniBox:**

Wasmtime's epoch interruption mechanism enforces the CPU time limit. A background thread increments the epoch counter at 1000 Hz. The module is configured with an `epoch_deadline` equal to `profile.cpu_seconds * 1000` epochs. When the deadline is reached, Wasmtime traps the module with `TrapCode::Interrupt`, which the OmniBox converts to `IfaError::TimeoutError`.

Memory is limited by configuring the Wasmtime `Store` with a `fuel` limit proportional to the profile's memory limit. Memory growth beyond the linear memory initial size invokes a growth hook that checks against the limit.

**EWO host functions:**

WASM modules call these host functions (exported under the `"ewo"` namespace) to check their own capabilities before making network or filesystem calls:

```
ewo.can_read(path_ptr: i32, path_len: i32) → i32      # 1 = allowed, 0 = denied
ewo.can_write(path_ptr: i32, path_len: i32) → i32
ewo.can_network(host_ptr: i32, host_len: i32) → i32
ewo.is_secure() → i32                                   # 1 = untrusted or standard profile
```

These are the WASM equivalent of `NativeRuntime.check()`. They operate on WASM linear memory pointers for the path/host strings.

### 40.5.5 The `Igbale` Struct — Public API

The Rust API surface exposed by `ifa-sandbox`:

```rust
pub struct Igbale {
    profile: SecurityProfile,
    caps: CapabilitySet,
}

impl Igbale {
    /// Create a new sandbox with the given profile and capabilities.
    pub fn new(profile: SecurityProfile, caps: CapabilitySet) -> Self;

    /// Spawn a hosted IfáLang program in a sandboxed subprocess.
    /// Applies Layer 1 OS isolation before exec.
    pub fn spawn_hosted(&self, binary_path: &Path) -> Result<SandboxedProcess, SandboxError>;

    /// Run a WASM module inside an OmniBox.
    pub fn run_wasm(&self, module: &[u8]) -> Result<(), SandboxError>;

    /// Check whether an IfáLang capability is granted in this sandbox.
    pub fn check(&self, call: &OduCall) -> Result<(), IfaError>;
}

pub struct SandboxedProcess {
    /// Wait for the process to exit; returns its exit code or error.
    pub fn wait(self) -> Result<i32, SandboxError>;
    /// Kill the sandboxed process immediately.
    pub fn kill(&mut self) -> Result<(), SandboxError>;
    /// Get the sandbox's stdin handle for writing input.
    pub fn stdin(&mut self) -> &mut dyn Write;
    /// Get the sandbox's stdout handle for reading output.
    pub fn stdout(&mut self) -> &mut dyn Read;
}

pub enum SandboxError {
    OsSetupFailed(String),       // cgroup/namespace/Job Object creation failed
    PermissionDenied(IfaError),  // capability check failed
    TimedOut,                    // CPU time limit exceeded
    MemoryExceeded,              // memory limit exceeded
    ProcessLimitExceeded,        // subprocess count exceeded
    SyscallBlocked(String),      // Layer 1 blocked a syscall
}
```

### 40.5.6 Platform Conformance Requirements

| Platform | Layer 1 requirement | Layer 2 requirement |
|----------|--------------------|--------------------|
| Linux (kernel ≥ 5.10) | MUST implement namespaces + cgroups v2 + seccomp-bpf | MUST implement |
| Linux (kernel 4.4–5.9) | MUST implement namespaces + cgroups v1 + seccomp-bpf | MUST implement |
| macOS (12+) | MUST implement sandbox profiles + setrlimit | MUST implement |
| Windows (10+) | MUST implement Job Objects + restricted token | MUST implement |
| Other POSIX | SHOULD implement using platform equivalent; Layer 2 MUST be implemented | MUST implement |
| Embedded / WASM | Layer 1 not applicable | MUST implement (Babalawo enforces at compile time) |

---

## 41. Domain Stacks — Feature-Gated Extensions `[DEFINED]`

> *Ẹni tó bá fẹ́ jẹ ohun tó dára, kò gbọdọ̀ jẹ ohun kan ṣoṣo. — One who wants to eat well must not eat only one thing.*

A **domain stack** is a set of IfáLang packages, standard library extensions, and Babalawo checks that together enable a specific application domain. Stacks are activated by a `[stack]` declaration in `ifa.toml`. Activating a stack makes a set of new types and domain methods available in the program, verified at compile time by the Babalawo.

Stacks are not part of the core language. They are feature-gated: unavailable unless declared.

```toml
[stack]
backend  = true   # HTTP server, database, JSON, router
ml       = true   # Tensor, model loading, training loop
```

### 41.1 Stack Activation and Capability Requirements

Activating a stack implies a set of minimum capability declarations. If the capabilities are absent, the Babalawo produces a `CAPABILITY_UNDECLARED` error at the first stack type usage.

| Stack | Required capabilities |
|-------|----------------------|
| `backend` | `network`, `read`, `write` (for static files / DB) |
| `frontend` | `stdio` (browser environment) |
| `ml` | `read` (model files), optionally GPU |
| `gamedev` | (none — pure computation) |
| `crypto` | `random` |
| `iot` | Embedded tier only (`mmio_base` must be set) |

### 41.2 The Backend Stack

The backend stack provides HTTP server primitives, database access, JSON serialization, and route matching. It layers on top of the `Odi` (files/DB), `Otura` (network), and `Irosu` (output) domains.

**Entry point — `HttpServer`:**

```
HttpServer.new(addr: String) → HttpServer
```

Creates a new HTTP server bound to the given address (`"0.0.0.0:8080"` format). Does not begin accepting until `HttpServer.run()` is called.

```
HttpServer.route(method: String, path: String, handler: daro ese(Request) → Response) → HttpServer
```

Registers a route. `method` is one of `"GET"`, `"POST"`, `"PUT"`, `"DELETE"`, `"PATCH"`. `path` may contain named segments: `"/users/:id"`. Returns the same `HttpServer` for method chaining. The handler **MUST** be a `daro ese` (async function).

```
HttpServer.run() → daro ese() → Null
```

Begins accepting connections. This is an async function and **MUST** be called with `reti`. Runs until the process is terminated or `HttpServer.stop()` is called.

```
HttpServer.middleware(f: daro ese(Request, NextFn) → Response) → HttpServer
```

Registers middleware that wraps every request. `NextFn` is a function `daro ese() → Response` that calls the next handler in the chain.

**Request type — `Request`:**

| Field | Type | Description |
|-------|------|-------------|
| `method` | `String` | HTTP method in uppercase |
| `path` | `String` | Request path without query string |
| `params` | `Map` | Named route parameters (`{id: "42"}` for `/users/:id`) |
| `query` | `Map` | Query string parameters |
| `headers` | `Map` | Request headers (lowercase keys) |
| `body` | `String \| ofo` | Request body as UTF-8 string, or `ofo` if absent |
| `json()` | `daro ese() → Map \| List` | Parse body as JSON |

**Response type — `Response`:**

```
Response.new(status: Int, body: String) → Response
Response.json(data: Map | List, status: Int = 200) → Response
Response.redirect(url: String, status: Int = 302) → Response
Response.status(code: Int) → Response          # set status, returns same Response
Response.header(name: String, val: String) → Response  # add header, chainable
```

**Complete working example:**

```
# ifa.toml: [stack] backend = true
# ifa.toml: [capabilities] network = ["*"], read = ["./static"]

daro ese handle_users(req: Request) → Response {
  ti req.method == "GET" {
    ayanmo users = reti Odi.ka("./data/users.json");
    pada Response.json(Irete.json_parse(users));
  }
  ti req.method == "POST" {
    ayanmo body = reti req.json();
    # persist body...
    pada Response.new(201, "Created");
  }
  pada Response.new(405, "Method Not Allowed");
}

daro ese handle_user_by_id(req: Request) → Response {
  ayanmo id = req.params["id"];
  pada Response.json({ id: id, name: "Àdé" });
}

daro ese main() {
  ayanmo server = HttpServer.new("0.0.0.0:8080")
    .route("GET",  "/users",     handle_users)
    .route("POST", "/users",     handle_users)
    .route("GET",  "/users/:id", handle_user_by_id);

  Irosu.fo("Listening on :8080");
  reti server.run();
}
```

**Database access — `Database`:**

```
Database.open(path: String) → daro ese() → Database
Database.query(sql: String, params: List = []) → daro ese() → List<Map>
Database.exec(sql: String, params: List = []) → daro ese() → Int   # rows affected
Database.close() → daro ese() → Null
```

`Database.open()` supports SQLite (path ending in `.db`) and PostgreSQL (connection string starting with `postgres://`). The `params` list uses positional binding: `?` placeholders in SQL, filled left-to-right from the list.

**JSON — methods on `Irete` domain:**

```
Irete.json_parse(s: String) → Map | List   # parses JSON string → IfáLang value
Irete.json_emit(v: Map | List) → String    # serializes Map or List → JSON string
Irete.json_emit_pretty(v: Map | List) → String  # indented JSON
```

`Irete.json_parse` raises `ParseError` if the input is not valid JSON. `Irete.json_emit` raises `TypeError` if the value contains non-serializable types (`Function`, `Object` without a `to_json()` method).

### 41.3 The ML Stack

The ML stack provides `Tensor`, model loading, and training loop primitives. It targets `f64` precision by default and uses BLAS/LAPACK if available.

**Tensor construction:**

```
Tensor.from_list(data: List, shape: List<Int>) → Tensor
Tensor.zeros(shape: List<Int>) → Tensor
Tensor.ones(shape: List<Int>) → Tensor
Tensor.random(shape: List<Int>) → Tensor          # uniform [0, 1)
Tensor.randn(shape: List<Int>) → Tensor           # normal (mean=0, std=1)
```

**Tensor fields:**

| Field | Type | Description |
|-------|------|-------------|
| `shape` | `List<Int>` | Dimension sizes, e.g. `[3, 4]` |
| `ndim` | `Int` | Number of dimensions |
| `size` | `Int` | Total element count |
| `dtype` | `String` | `"f64"` or `"f32"` |

**Tensor operations:**

```
Tensor.add(other: Tensor | Float) → Tensor    # element-wise add or scalar broadcast
Tensor.sub(other: Tensor | Float) → Tensor
Tensor.mul(other: Tensor | Float) → Tensor    # element-wise or scalar
Tensor.matmul(other: Tensor) → Tensor         # matrix multiply — shapes must be compatible
Tensor.transpose() → Tensor                   # reverses all dimensions
Tensor.reshape(new_shape: List<Int>) → Tensor # total elements must be unchanged
Tensor.sum(axis: Int | ofo = ofo) → Tensor    # sum all or along axis
Tensor.mean(axis: Int | ofo = ofo) → Tensor
Tensor.max(axis: Int | ofo = ofo) → Tensor
Tensor.min(axis: Int | ofo = ofo) → Tensor
Tensor.exp() → Tensor                         # element-wise e^x
Tensor.log() → Tensor                         # element-wise natural log
Tensor.relu() → Tensor                        # element-wise max(0, x)
Tensor.sigmoid() → Tensor                     # element-wise 1/(1+e^-x)
Tensor.softmax(axis: Int) → Tensor
Tensor.get(indices: List<Int>) → Float        # single element access
Tensor.slice(start: List<Int>, end: List<Int>) → Tensor
Tensor.to_list() → List                        # convert to nested IfáLang lists
```

**Model loading:**

```
Model.load(path: String) → daro ese() → Model   # loads ONNX or SafeTensors format
Model.forward(input: Tensor) → daro ese() → Tensor
```

**Complete working example:**

```
# ifa.toml: [stack] ml = true
# ifa.toml: [capabilities] read = ["./models", "./data"]

daro ese main() {
  # Load data
  ayanmo X = Tensor.from_list(
    [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    [3, 2]           # 3 samples, 2 features
  );
  ayanmo y = Tensor.from_list([0.0, 1.0, 1.0], [3, 1]);

  # Simple linear layer: W (2x1) + b (1)
  ayanmo W = Tensor.randn([2, 1]);
  ayanmo b = Tensor.zeros([1]);

  # Forward pass
  ayanmo logits = X.matmul(W).add(b);     # [3, 1]
  ayanmo probs  = logits.sigmoid();        # [3, 1]

  Irosu.fo($"Predictions: {probs.to_list()}");

  # Load pre-trained ONNX model
  ayanmo model = reti Model.load("./models/classifier.onnx");
  ayanmo input = Tensor.from_list([0.5, 0.3], [1, 2]);
  ayanmo output = reti model.forward(input);
  Irosu.fo($"Class probabilities: {output.to_list()}");
}
```

### 41.4 The GameDev Stack

The GameDev stack provides a game loop, 2D sprite management, input handling, and collision detection. It extends the `Ose` (graphics/canvas) domain.

```
World.new(width: Int, height: Int, title: String) → World
World.run(update: ese(delta_ms: Float) → Null, draw: ese() → Null) → Null
  # update fires every frame with time since last frame (ms)
  # draw fires after update — use Ose methods to draw

World.key_pressed(key: String) → Bool   # "ArrowLeft", "Space", "a", etc.
World.key_held(key: String) → Bool
World.mouse_pos() → { x: Int, y: Int }
World.mouse_button(button: Int) → Bool  # 0=left, 1=middle, 2=right
World.width() → Int
World.height() → Int
World.exit() → Null                     # request loop exit

Sprite.new(image_path: String) → Sprite
Sprite.draw(x: Float, y: Float) → Null
Sprite.draw_scaled(x: Float, y: Float, scale: Float) → Null
Sprite.width() → Int
Sprite.height() → Int

# Collision
Rect.new(x: Float, y: Float, w: Float, h: Float) → Rect
Rect.overlaps(other: Rect) → Bool
Rect.contains(x: Float, y: Float) → Bool
```

**Complete working example:**

```
# ifa.toml: [stack] gamedev = true

ayanmo player_x = 100.0;
ayanmo player_y = 100.0;
ayanfe SPEED = 3.0;

ese update(delta: Float) {
  ti World.key_held("ArrowLeft")  { player_x = player_x - SPEED; }
  ti World.key_held("ArrowRight") { player_x = player_x + SPEED; }
  ti World.key_held("ArrowUp")    { player_y = player_y - SPEED; }
  ti World.key_held("ArrowDown")  { player_y = player_y + SPEED; }
  ti World.key_pressed("Escape")  { World.exit(); }
}

ayanmo hero = Sprite.new("./assets/hero.png");

ese draw() {
  Ose.clear(0, 0, 0);                 # black background
  hero.draw(player_x, player_y);
}

ayanmo world = World.new(800, 600, "IfáLang Game");
world.run(update, draw);
```

### 41.5 The Crypto Stack

The crypto stack extends the `Irete` domain with higher-level cryptographic operations beyond hashing.

```
# Symmetric encryption
Irete.aes_encrypt(plaintext: String, key: String) → String  # AES-256-GCM, returns base64
Irete.aes_decrypt(ciphertext: String, key: String) → String | Error
Irete.derive_key(password: String, salt: String, iterations: Int = 100000) → String
  # PBKDF2-SHA256; returns 32-byte key as hex string

# Asymmetric
Irete.keygen_ed25519() → { public: String, private: String }  # base64-encoded
Irete.sign(message: String, private_key: String) → String     # Ed25519 signature, base64
Irete.verify(message: String, signature: String, public_key: String) → Bool

# Password hashing (safe)
Irete.argon2_hash(password: String) → String   # Argon2id, includes salt in output
Irete.argon2_verify(password: String, hash: String) → Bool

# Certificate operations
Irete.cert_load(path: String) → Certificate
Irete.cert_fingerprint(cert: Certificate) → String  # SHA-256 hex fingerprint
```

### 41.6 The IoT Stack

The IoT stack is available only in the embedded tier. It provides sensor abstraction, GPIO, I2C, SPI, and UART primitives over the MMIO HAL.

```
# GPIO
GPIO.init(pin: u8, mode: String) → Null   # mode: "input" | "output" | "input_pullup"
GPIO.write(pin: u8, value: u8) → Null     # 0 or 1
GPIO.read(pin: u8) → u8

# I2C
I2C.init(sda: u8, scl: u8, freq_hz: u32) → Null
I2C.write(addr: u8, data: List<u8>) → Null
I2C.read(addr: u8, n_bytes: u8) → List<u8>

# UART
UART.init(baud: u32) → Null
UART.write(s: String) → Null
UART.read_line() → String

# Timers
Timer.delay_ms(ms: u32) → Null           # blocking delay (calls jowo internally)
Timer.millis() → u32                     # milliseconds since boot
```

---

## 42. Embedded MMIO and Pointer Semantics `[DEFINED]`

> *Ilẹ̀ ni ipilẹ̀ gbogbo ohun. — The earth is the foundation of all things.*

The embedded tier bridges IfáLang to physical hardware through memory-mapped I/O (MMIO). This section defines the complete pointer model, MMIO access, and how to write bare-metal code that maps to physical registers.

### 42.1 The `ailewu` Block

Pointer operations are only available inside an `ailewu` block. The Babalawo produces an `UNSAFE_OUTSIDE_AILEWU` error if pointer syntax is used anywhere else.

```
ailewu {
  # pointer creation, dereference, and MMIO access are available here
}
```

The `ailewu` block is the only place where:
- `&expr` (address-of) is valid
- `*ptr` (dereference) is valid
- Sized integer types (`u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`) are usable as pointer targets
- MMIO reads and writes are performed

### 42.2 Pointer Types

| Syntax | Type | Description |
|--------|------|-------------|
| `*u8`, `*u32` | Raw pointer to sized type | The only valid pointer types |
| `&expr` | Creates a pointer to a variable | Address-of operator |
| `*ptr` | Reads or writes through pointer | Dereference operator |

Pointers are always typed. `*u32` is a pointer to a 32-bit unsigned integer. There are no void pointers. There is no pointer arithmetic. Pointer casting (`*u32` to `*u8`) is **not** permitted in v0.2 — each MMIO address is declared at its natural width.

### 42.3 MMIO Register Access — Flipping a GPIO Pin

The canonical example: toggle a GPIO LED on an ARM Cortex-M microcontroller. The GPIO Output Data Register (ODR) on an STM32 is at `0x4001 0C0C`. Bit 5 controls pin PA5 (the built-in LED on Nucleo boards).

```
# Complete GPIO blink program for STM32 Nucleo-F401RE
# ifa.toml: [deploy.embedded] target = "thumbv7em-none-eabihf", mmio_base = "0x40000000"

# RCC AHB1 Enable register — enables GPIOA clock
ayanfe RCC_AHB1ENR: u32 = 0x40023830;
# GPIOA Mode Register — sets pin direction
ayanfe GPIOA_MODER: u32 = 0x40020000;
# GPIOA Output Data Register — controls output state
ayanfe GPIOA_ODR:   u32 = 0x4002000C;

# Bit masks
ayanfe PA5_ENABLE:  u32 = 0x00000020;  # bit 5 = 1
ayanfe PA5_OUTPUT:  u32 = 0x00000400;  # bits 11:10 = 01 (output mode)
ayanfe PA5_MODE_CLR:u32 = 0xFFFFF3FF;  # clear bits 11:10

ese main() {
  ailewu {
    # Step 1: Enable GPIOA clock via RCC
    ayanmo rcc_ptr: *u32 = &RCC_AHB1ENR;
    *rcc_ptr = *rcc_ptr | PA5_ENABLE;

    # Step 2: Set PA5 to output mode in MODER register
    ayanmo moder_ptr: *u32 = &GPIOA_MODER;
    *moder_ptr = (*moder_ptr & PA5_MODE_CLR) | PA5_OUTPUT;

    # Step 3: Blink loop
    ayanmo odr_ptr: *u32 = &GPIOA_ODR;
    nigba otito {
      *odr_ptr = *odr_ptr | PA5_ENABLE;   # LED on
      jowo 500000;                          # yield 500ms
      *odr_ptr = *odr_ptr & ~PA5_ENABLE;  # LED off
      jowo 500000;
    }
  }
}
```

**Line-by-line:**

1. `ayanfe GPIOA_ODR: u32 = 0x4002000C` — declares a typed constant holding the MMIO address. The `: u32` annotation tells the Babalawo this is a pointer-target type.
2. `ayanmo odr_ptr: *u32 = &GPIOA_ODR` — creates a `*u32` pointer pointing at the constant's address. `&GPIOA_ODR` takes the address of the constant — which is the literal address `0x4002000C` on the target.
3. `*odr_ptr = *odr_ptr | PA5_ENABLE` — reads the current register value (right-hand `*odr_ptr`), ORs in the bit, writes back (left-hand `*odr_ptr`).
4. `jowo 500000` — yields for 500,000 microseconds = 500ms. In embedded tier, this becomes a hardware sleep via the HAL.

### 42.4 HAL Dispatch

MMIO addresses at or above `mmio_base` (configured in `ifa.toml [deploy.embedded]`) are dispatched through the Hardware Abstraction Layer (HAL) at compile time. Below `mmio_base`, addresses are ordinary memory.

The HAL translates IfáLang's typed MMIO read/write into the appropriate memory-barrier and volatile-access pattern for the target architecture:

- **ARMv7-M (Cortex-M3/M4):** Uses `core::ptr::read_volatile` / `core::ptr::write_volatile`
- **RISC-V:** Uses `core::ptr::read_volatile` / `core::ptr::write_volatile` with appropriate fence instructions
- **Other targets:** Implementor-defined

The compiler **MUST** emit volatile reads and writes for all MMIO access. It **MUST NOT** optimize away or reorder MMIO operations within a single `ailewu` block. Cross-block ordering is not guaranteed — use `ailewu` blocks as atomic hardware interaction units.

### 42.5 Sized Integer Arithmetic in `ailewu`

Inside `ailewu`, the default `Int` type (i64) is still available but the sized variants are preferred for hardware code. Sized integer arithmetic uses the natural width of the type — no promotion to Float on overflow. Instead:

- Unsigned types (`u8`, `u16`, `u32`, `u64`): wrap on overflow (two's complement)
- Signed types (`i8`, `i16`, `i32`, `i64`): wrap on overflow in release builds, `UNDEFINED_BEHAVIOR` panic in debug builds

This differs from the hosted tier's Int overflow behavior (§4.4 Float promotion) because MMIO code must have predictable bit patterns. Float promotion is not available inside `ailewu`.

### 42.6 The Bitwise NOT Operator in `ailewu`

The `~` operator is **only available inside `ailewu`** blocks for use in register bitmask operations. Outside `ailewu`, use `ko` (logical NOT).

```
ailewu {
  ayanmo mask: u32 = ~0x00000020u32;   # 0xFFFFFFDF — clears bit 5
}
ko otito    # logical NOT, outside ailewu — evaluates to eke
```

---

## 43. The Ọjà Dependency Resolver — Complete Algorithm `[DEFINED]`

> *Ọjà k̀ò níí gbà olówó àti tálíkà pẹ̀lú ojú ìdánilójú kan náà. — The market does not greet the rich and the poor with the same confidence.*

The Ọjà version resolver determines which specific version of each dependency to install given the constraints declared in `ifa.toml` and all transitive dependencies. This section specifies the algorithm precisely so that all conforming Ọjà implementations produce identical results.

### 43.1 Version Constraint Semantics (Complete)

IfáLang uses semantic versioning. A version `MAJOR.MINOR.PATCH` carries the following compatibility contract, which Ọjà **MUST** enforce:

| Change type | Version bump | Compatibility |
|-------------|-------------|---------------|
| Breaking API change | MAJOR | Incompatible |
| New backward-compatible feature | MINOR | Compatible |
| Bug fix | PATCH | Compatible |

Version `0.x.y` is **unstable**: any `0.x` update may break compatibility. The caret constraint `"^0.3"` only allows `0.3.x` — it does not allow `0.4`. This differs from `"^1.3"` which allows `1.x` for any `x ≥ 3`.

| Constraint | Minimum | Maximum (exclusive) | Stable behavior | Unstable behavior |
|------------|---------|---------------------|-----------------|-------------------|
| `"1.2.3"` | 1.2.3 | 1.2.3 | Exact | Exact |
| `"^1.2.3"` | 1.2.3 | 2.0.0 | `1.x.y` for `x≥2` | N/A |
| `"^1.2"` | 1.2.0 | 2.0.0 | `1.x.y` for `x≥2` | N/A |
| `"^0.3.1"` | 0.3.1 | 0.4.0 | N/A | `0.3.y` for `y≥1` |
| `"^0.3"` | 0.3.0 | 0.4.0 | N/A | `0.3.x` |
| `"~1.2.3"` | 1.2.3 | 1.3.0 | `1.2.y` for `y≥3` | N/A |
| `"~1.2"` | 1.2.0 | 1.3.0 | `1.2.x` | N/A |
| `">=1.2, <2"` | 1.2.0 | 2.0.0 | Any `1.x.y` `x≥2` | N/A |
| `"*"` | 0.0.0 | ∞ | Any | Any |

### 43.2 The Resolution Algorithm — Minimum Version Selection (MVS)

Ọjà uses **Minimum Version Selection (MVS)**, the same algorithm used by Go modules. This choice is intentional and non-negotiable. It is explicitly **not** SAT-based (npm's approach), **not** a backtracking solver (pip's approach), and **not** a latest-first greedy algorithm.

**Why MVS:** MVS is deterministic, fast (linear time), and produces the minimum set of versions that satisfies all constraints. It never silently upgrades a dependency. It makes builds reproducible without a lockfile for pure libraries. It produces exactly one resolution for a given set of constraints.

**The algorithm:**

```
Input:  A dependency graph G where each node is (package, version_constraint)
Output: A map M from package → exact_version

1. Build the build list:
   a. Start with the root package's direct dependencies.
   b. For each dependency D at version constraint C:
      i.  Resolve C to a minimum version V = min_satisfying(C, registry)
      ii. Fetch D@V's own dependencies (its ifa.toml [dependencies])
      iii. Recursively build D@V's list
   c. For each package that appears multiple times with different minimum versions,
      keep the MAXIMUM of the minimums (not the latest available).
      This is the "minimum version selection" step.

2. Detect conflicts:
   - A conflict exists if no version of package P satisfies all constraints
     simultaneously. Specifically: if there exists a constraint C_i on P
     such that max_of_minimums(P) does not satisfy C_i.
   - If a conflict exists, halt with OjaConflictError listing all constraints
     and their sources.

3. Verify capability constraints:
   - For each resolved package P@V, load its declared [capabilities].
   - If P@V declares capabilities not in the root project's [capabilities],
     halt with OjaCapabilityError.

4. Produce the build list M: package → resolved_version

5. Write oja.lock with M plus SHA-256 checksums.
```

**Concrete example:**

```
# Root project ifa.toml:
[dependencies]
A = "^1.2"
B = "^2.0"

# A@1.2.0 ifa.toml:
[dependencies]
C = "^1.0"

# B@2.0.0 ifa.toml:
[dependencies]
C = "^1.3"

# Registry has: C@1.0, C@1.1, C@1.2, C@1.3, C@1.4, C@2.0

Resolution steps:
  Root requires A ≥ 1.2  → A@1.2.0 (minimum satisfying)
  Root requires B ≥ 2.0  → B@2.0.0
  A@1.2.0 requires C ≥ 1.0 → minimum satisfying = C@1.0.0
  B@2.0.0 requires C ≥ 1.3 → minimum satisfying = C@1.3.0
  C appears twice: max(1.0.0, 1.3.0) = C@1.3.0
  Verify: C@1.3.0 satisfies "^1.0" (yes) and "^1.3" (yes) → no conflict

Result: A@1.2.0, B@2.0.0, C@1.3.0
```

### 43.3 Conflict Detection and Error Format

When MVS cannot produce a valid build list, Ọjà **MUST** produce an `OjaConflictError` that names every conflicting constraint and its origin:

```
OjaConflictError: Cannot resolve dependency graph.

Package 'ifa-json' has incompatible constraints:
  ifa-json = "^1.4"  (required by root project in ifa.toml)
  ifa-json = "^2.0"  (required by ifa-http@2.1.3 in oja.lock)

The minimum version satisfying "^2.0" is ifa-json@2.0.0.
The constraint "^1.4" allows ifa-json@1.x only (maximum < 2.0.0).

Resolution:
  → Upgrade root constraint to "^2.0" if ifa-json 2.x is compatible
  → Downgrade ifa-http to a version that uses ifa-json "^1.x"
  → Pin ifa-json to an exact version if you need both (not recommended)
```

### 43.4 `oja update` Behavior

`oja update [package]` does **not** use the resolver. It uses a simpler algorithm:

1. For the specified package (or all packages if none specified), find the latest version satisfying the constraint in `ifa.toml`.
2. Run MVS with that version as the new minimum.
3. Write the new `oja.lock`.

This means `oja update` can only move forward, never backward. If updating `ifa-http` to `3.0.0` causes a conflict with `C`'s constraint, Ọjà reports the conflict and does not update.

### 43.5 Lockfile Guarantees

A project with a committed `oja.lock` **MUST** install exactly the versions in the lockfile, ignoring all constraint solving. The resolver only runs when:
- `oja.lock` does not exist
- `oja add`, `oja remove`, or `oja update` is explicitly called

This is the Cargo/Go modules model: the lockfile is the source of truth for reproducible builds. The version solver is only invoked to produce or update the lockfile, never to produce the installed set from scratch on each install.

---

## 44. The REPL — Interactive Session `[PARTIAL]`

> *Ẹni tó bá fẹ́ mọ ọ̀nà, kò gbọdọ̀ bèrè ní ìgbà tó bá ṣíná. — One who wants to know the road must not ask only when lost.*

The IfáLang REPL (`ifa repl`) provides an interactive session for exploring the language, testing expressions, and iterating on programs without the overhead of writing and running a file. It is not a debugger — for step-by-step execution see §34.

**Status:** The REPL is implemented on top of the AST interpreter (`ifa run`). It does not currently use the bytecode VM. §44.3 specifies the VM-based REPL that becomes available once the VM canonicalization gate (§21.2) is passed.

### 44.1 Activation

```
ifa repl                    # start REPL session
ifa repl --quiet            # suppress banner
ifa repl --import path.ifa  # pre-load a file, then open REPL in its scope
```

### 44.2 Session Semantics

The REPL maintains a persistent **session environment** across inputs. Each input is a single statement or expression. Variable bindings, function definitions, and class definitions survive between inputs.

```
IfáLang v0.2.0 REPL — Ọrúnmìlà knows. Àṣẹ.
Type :quit to exit, :help for commands.

> ayanmo x = 5;
5
> x + 3
8
> ese double(n) { pada n * 2; }
[function double]
> double(x)
10
> ayanmo names = ["Àdé", "Tìtí", "Kọ́lá"];
["Àdé", "Tìtí", "Kọ́lá"]
> names.length
3
```

**Expression evaluation:** If the input is an expression (not a declaration or statement with no value), the REPL prints the result on the next line. If the result is `ofo`, nothing is printed.

**Statement execution:** Declarations, assignments, and function calls that produce `ofo` execute silently.

**Error recovery:** A runtime error in the REPL does not terminate the session. The error is printed with its Odù-tagged diagnostic format (§22.4) and the session continues. The environment is not rolled back — partial mutations that occurred before the error are preserved.

```
> ayanmo y = 10 / 0;
error[Òtúrúpọ̀n] repl:1:14
  Division by zero
  Wisdom: Reduction without substance produces nothing.
> y
ofo          ← y was not assigned; binding is ofo
```

### 44.3 REPL Commands

Commands begin with `:` and are not IfáLang syntax:

| Command | Description |
|---------|-------------|
| `:quit` | Exit the REPL |
| `:help` | Show command list |
| `:env` | List all bindings in the current session environment |
| `:clear` | Reset session environment to empty |
| `:load path.ifa` | Load a file into the current environment |
| `:type expr` | Show the runtime type of an expression without evaluating side effects |
| `:time expr` | Evaluate expression and show execution time |
| `:babalawo` | Run the Babalawo on the entire current session environment |
| `:odu NAME` | Show Odù wisdom and domain methods for the named Odù |

### 44.4 Multi-line Input

The REPL detects incomplete input (an open `{`, unclosed `gbiyanju`, or function body in progress) and shows a continuation prompt:

```
> ese factorial(n) {
... ti n <= 1 { pada 1; }
... pada n * factorial(n - 1);
... }
[function factorial]
> factorial(5)
120
```

The continuation prompt is `...`. The input is submitted when the parser determines the expression is complete (the outermost block is closed).

### 44.5 VM-Based REPL — Persistent State Model `[OPEN]`

The AST-based REPL (§44.2) works because the AST interpreter maintains its environment as a `HashMap<String, IfaValue>` that is simply never reset. The bytecode VM cannot do this directly — its execution model is a call stack that begins from `main()` and is fully unwound on return.

Supporting a REPL on the bytecode VM requires one of the following approaches. This section specifies the design target; the implementation is not yet complete.

**Approach A — Globals Module (recommended):**

Each REPL input is compiled as a standalone function `_repl_N()` that is added to a persistent globals table. The globals table is not cleared between calls. Local variables declared in one input are promoted to globals before compilation so subsequent inputs can access them.

```
# Input 1: `ayanmo x = 5;`
# Compiled as:
_globals["x"] = 5;    # StoreGlobal x

# Input 2: `x + 3`
# Compiled as:
LoadGlobal x           # reads from persistent globals
PushInt 3
Add
```

The globals table (`HashMap<String, IfaValue>`) is allocated before the first REPL input and lives for the duration of the session. Between inputs, the VM returns to the REPL's dispatch loop. The call stack is fully unwound after each input — only globals persist.

**Constraints:**
- Functions defined in one input **MUST** be stored in the function registry and remain available for subsequent inputs.
- Classes defined in one input **MUST** remain in the class registry.
- The Babalawo runs on each input individually, with the accumulated globals visible as externally-defined names.

**Approach B — Long-running top-level coroutine:**

The REPL input is compiled into a coroutine that `jowo`s after each statement, yielding control to the REPL dispatcher. The coroutine's stack frame is preserved between yields. This is the approach used by MicroPython's interactive mode.

This requires the VM to support `jowo` as a full coroutine yield (not just a sleep), which is not specified in v0.2. This approach is deferred to a future version.

**Current requirement:** Any conforming `ifa repl` implementation **MUST** use Approach A or an equivalent persistent-globals model. It **MUST NOT** silently discard bindings between inputs. It **MUST** handle errors without terminating the session.

---

## 45. Ó Yẹlà — The Machine Revelation `[DEFINED]`

> *Ohun tó farapamọ́ máa ń jáde lọjọ́ kan. — What is hidden always reveals itself one day.*

**Ó Yẹlà** (it reveals itself / it unfurls) is IfáLang's hidden easter egg: a four-act ASCII art movie showing how the machine actually executes a program — from source text all the way down to CPU registers — rendered entirely in the terminal, frame by frame, in the aesthetic of Ifá divination.

It is not a debugger. It is not a profiler. It is a movie about computation, told in the language of the oracle. When a developer discovers it, they see something they did not expect: the machine narrating its own hidden life.

### 45.1 Discovery

Ó Yẹlà has exactly one trigger. It is not listed in `ifa --help`. It is not mentioned in the README. It is not documented anywhere visible. The only way to find it is to type it:

```bash
ifa oyela program.ifa
```

`ifa --help` shows: `run`, `runb`, `build`, `repl`, `lsp`, `debug`, `fmt`, `doc`, `deploy`, `test`. The word `oyela` appears nowhere in the help output. A developer who finds it found it themselves.

**The discovery reward.** When `ifa oyela` runs for the first time on a machine — detected by the absence of `~/.ifa/oyela_discovered` — the terminal clears and prints one line before the movie begins, letter by letter, at 30ms per character:

```
Ó yẹlà. The machine has chosen to show itself to you.
```

A two-second pause. Then the movie starts. This message appears exactly once per developer, ever. On all subsequent runs the movie begins immediately.

**No other trigger exists.** Source annotations, REPL commands, and `--flag` variations on `ifa run` or `ifa runb` are not discovery paths. The hidden command is the only path. This is a deliberate constraint: one secret, one door.

### 45.2 The Movie — Structure and Format

Ó Yẹlà is a sequential ASCII art animation that plays in four acts. Each act covers one layer of abstraction, from the highest (human-readable source) to the lowest (CPU hardware). The acts play back-to-back with a brief title card between them. The entire movie plays to completion, then exits.

The output is raw bytes written to stdout. No TUI framework. No alternate screen buffer. No cursor positioning. The movie scrolls — each frame is printed below the last, like a film strip unrolling downward. The terminal scrollback becomes the full record of the program's execution.

Every frame is exactly 72 characters wide (fits an 80-column terminal with margins). Frames are separated by a blank line. The animation speed is set by `--speed slow | normal | fast` (default: `normal`).

#### Frame anatomy

Each frame has the same structure:

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT I · Ìmọ̀ · THE WORD                              frame 003 / 012
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  [act-specific art]

  Babalawo: "A value steps forward. It has not yet found its name."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

The header line shows the act name (in Yoruba and English), and the current frame count. The footer always shows the Babalawo's narration for this step. The art occupies the middle section and changes each frame.

### 45.3 Act I — Ìmọ̀ (The Word) · Compiler Pipeline

**Act I shows how source code transforms into bytecode** — the compiler pipeline from human text to machine-readable opcodes.

The act animates one statement at a time. For each statement, three frames play in sequence:

**Frame A — The source line, isolated:**

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT I · Ìmọ̀ · THE WORD                              frame 001 / 018
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

       SOURCE
       ──────
       ayanmo x = 5 + 3;
       ^^^^^^             ← mutable binding keyword
              ^           ← identifier: "x"
                  ^^^^^   ← expression: 5 + 3

  Babalawo: "The word is spoken. The lexer hears each syllable."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Frame B — The AST node:**

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT I · Ìmọ̀ · THE WORD                              frame 002 / 018
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

       SOURCE                 AST
       ──────                 ───
       ayanmo x = 5 + 3;  →  VarDecl
                               name:    "x"
                               mutable: true
                               init:    Add
                                          left:  Int(5)
                                          right: Int(3)

  Babalawo: "The parser gives shape to sound. A tree grows."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Frame C — The bytecode:**

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT I · Ìmọ̀ · THE WORD                              frame 003 / 018
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

       SOURCE                 AST              BYTECODE
       ──────                 ───              ────────
       ayanmo x = 5 + 3;  →  VarDecl      →   0x08  PushInt   5
                               Add              0x08  PushInt   3
                                                0x20  Add
                                                0x1B  StoreGlobal "x"

  Babalawo: "The compiler breathes life into the tree. Opcodes are born."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

This three-frame sequence repeats for every statement in the program. The `→` arrows appear letter-by-letter in each frame, giving the impression of information flowing rightward from source to machine.

### 45.4 Act II — Ìmọ̀ Inú (The Inner Word) · Bytecode Execution

**Act II shows the bytecode VM running** — the instruction pointer moving, opcodes firing one by one, and the value stack changing with each instruction.

One frame per opcode. The IP arrow (`▶`) advances down the opcode listing with each frame. The stack diagram on the right updates to show the before and after state.

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT II · Ìmọ̀ Inú · THE INNER WORD                   frame 004 / 031
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    BYTECODE                           STACK  (Òrún · The Sky)
    ────────                           ──────────────────────
    0x00  PushInt  5                   ┌─────────────────┐
    0x09  PushInt  3                   │    [ Int(3) ]   │  ← top
  ▶ 0x12  Add                          │    [ Int(5) ]   │
    0x1B  StoreGlobal  "x"             └─────────────────┘
    0x08  PushInt  5                       depth: 2
    0x80  Print                        ──────── after Add ────────
                                       ┌─────────────────┐
    globals:                           │    [ Int(8) ]   │  ← top
      "x" : (unset)                    └─────────────────┘
                                           depth: 1

  Babalawo: "Two truths meet. A third is born."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

When a `CallOdu` opcode fires, Act II pauses and displays a domain call frame for three seconds before continuing:

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT II · Ìmọ̀ Inú · THE INNER WORD           DOMAIN CALL
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    ╔══════════════════════════════════════════════════════════════╗
    ║                                                              ║
    ║    Ìrosù  ·  The Speaker  ·  domain ID 4                    ║
    ║                                                              ║
    ║    method : fo("hello")                                      ║
    ║    capability required : Stdio                               ║
    ║    capability granted  : YES  ✓                              ║
    ║                                                              ║
    ║    pattern:  █ █ ░ ░   (1100 · Ìrosù)                       ║
    ║                                                              ║
    ║    "Ẹni tó bá ń sọ̀rọ̀, kò gbọdọ̀ sọ irọ́."                  ║
    ║    One who speaks must not speak falsehood.                  ║
    ║                                                              ║
    ╚══════════════════════════════════════════════════════════════╝

  Babalawo: "The oracle domain is invoked. Àṣẹ flows through the boundary."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

The capability check is shown explicitly — `granted: YES ✓` or `granted: NO ✗ → PermissionError`. This makes the capability system visible as a real enforcement mechanism, not an abstraction.

The Teeté binary pattern is rendered with `█` for `1` and `░` for `0`. Ìrosù (`1100`) shows as `█ █ ░ ░`.

### 45.5 Act III — Ilẹ̀ (The Ground) · Memory

**Act III shows the program's memory** — globals, locals, call frames, and the heap reference count — as the program runs. This act plays concurrently with Act II: each Act II opcode frame has a corresponding Act III frame that shows what memory looks like at that exact moment.

At the end of Act II, Act III plays a summary sequence showing the full memory state at each major checkpoint: function entry, function return, and program end.

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT III · Ilẹ̀ · THE GROUND                           frame 011 / 031
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    MEMORY at opcode 0x1B (StoreGlobal "x")

    ┌─ Globals ───────────────────────────────────────────┐
    │   "x"  :  Int(8)              ← just written        │
    └─────────────────────────────────────────────────────┘

    ┌─ Call Frames (Òrún base) ───────────────────────────┐
    │   frame 0  [main]  base_slot=0  ip=0x1B             │
    └─────────────────────────────────────────────────────┘

    ┌─ Heap (Arc references) ─────────────────────────────┐
    │   0 active Arc<T> allocations                       │
    │   (no Lists, Maps, or Closures created yet)         │
    └─────────────────────────────────────────────────────┘

  Babalawo: "The binding is made. What was unnamed is now known."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

When a function call occurs, Act III shows the new frame being pushed:

```
    ┌─ Call Frames ───────────────────────────────────────┐
    │   frame 0  [main]       base_slot=0   ip=0x34  wait │
    │   frame 1  [add_nums]   base_slot=2   ip=0x00  ▶    │
    └─────────────────────────────────────────────────────┘
```

The `wait` and `▶` markers show which frame is paused and which is executing.

### 45.6 Act IV — Erín (The Machine Laughs) · CPU Hardware

**Act IV shows what happens at the hardware level** — how the VM's opcodes map to actual CPU behavior: registers, memory addresses, clock cycles, and instruction fetch/decode/execute cycles.

This act is deliberately simplified and approximate. It does not disassemble to actual machine code for the host CPU. Instead it shows a model CPU — the "Erín processor" (erín = elephant, for its memory) — with a small set of named registers and a simplified pipeline.

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ACT IV · Erín · THE MACHINE LAUGHS                    frame 002 / 014
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    IfáLang opcode:   Add  (0x20)
    Rust function:    vm::step() → match OpCode::Add { ... }

    ┌─ CPU PIPELINE ──────────────────────────────────────────────────┐
    │                                                                  │
    │   FETCH         DECODE         EXECUTE        WRITE-BACK        │
    │  ┌───────┐    ┌──────────┐   ┌──────────┐   ┌──────────────┐  │
    │  │ 0x20  │ →  │  "Add"   │ → │  5 + 3   │ → │  stack[top]  │  │
    │  │ (Add) │    │ pop pop  │   │  = 8     │   │  = Int(8)    │  │
    │  └───────┘    │ push     │   └──────────┘   └──────────────┘  │
    │               └──────────┘                                      │
    │                                                                  │
    │   REGISTERS at this cycle:                                       │
    │     IP   0x0000_0012   instruction pointer                       │
    │     SP   0x0000_0002   stack pointer (depth=2 before, 1 after)  │
    │     ACC  0x0000_0008   accumulator (result: 8)                  │
    │     FP   0x0000_0000   frame pointer (main, no locals)          │
    │                                                                  │
    │   CLOCK: cycle ~3-5 (approximate — interpreted, not JIT)        │
    │                                                                  │
    └──────────────────────────────────────────────────────────────────┘

    RAM snapshot (stack region):
    addr  0x00:  [ 00 00 00 00 00 00 00 05 ]  ← Int(5)  (was here)
    addr  0x08:  [ 00 00 00 00 00 00 00 03 ]  ← Int(3)  (was here)
    addr  0x08:  [ 00 00 00 00 00 00 00 08 ]  ← Int(8)  (now here, replaced)

  Babalawo: "The machine does not think. It counts, moves, and repeats."
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Act IV plays for the first five opcodes of a function call, then summarizes the rest with a frame count note: `[ ... 47 more opcodes, same pattern — fetch, decode, execute, write-back ... ]`. This prevents Act IV from becoming overwhelming on programs with hundreds of opcodes.

The RAM snapshot shows values as hex bytes in an 8-byte wide display. Stack values are shown at their conceptual memory addresses (not real host addresses). This is explicitly labeled as a model, not a disassembly.

### 45.7 The Babalawo Narrations — Complete Table

Every opcode class has a canonical one-sentence narration that appears in the frame footer across all four acts.

| Opcode class | Narration |
|---|---|
| `PushInt` / `PushStr` / `PushFloat` | *"A value steps forward. It has not yet found its name."* |
| `PushNull` | *"The void is placed on the stack. Ofo stands ready."* |
| `StoreGlobal` / `StoreLocal` | *"The binding is made. What was unnamed is now known."* |
| `LoadGlobal` / `LoadLocal` | *"The name is called. Its value answers."* |
| `Add` / `Sub` / `Mul` / `Div` | *"Two truths meet. A third is born."* |
| `Mod` | *"What remains after division is not waste — it is remainder."* |
| `Pow` | *"The king multiplies itself. Expansion without end."* |
| `Eq` / `Ne` | *"The oracle is consulted. The answer is otito or eke."* |
| `Lt` / `Le` / `Gt` / `Ge` | *"Order is established. Something is before; something is after."* |
| `JumpIfFalse` / `JumpIfTrue` | *"The path divides. The program chooses."* |
| `Jump` | *"No choice is made. The program moves without condition."* |
| `Call` | *"A new voice speaks. The old voice waits."* |
| `Return` | *"The voice has spoken. The waiting voice resumes."* |
| `TailCall` | *"The voice does not wait — it becomes the call. No debt is created."* |
| `CallOdu` | *"The oracle domain is invoked. Àṣẹ flows through the boundary."* |
| `MakeClosure` | *"A function remembers where it was born. This is the upvalue."* |
| `LoadUpvalue` / `StoreUpvalue` | *"The memory beyond the frame is consulted."* |
| `GetField` / `SetField` | *"The object is asked. The object answers."* |
| `GetIndex` / `SetIndex` | *"A position is named. The collection opens."* |
| `BuildList` | *"Many become one. The list is assembled."* |
| `BuildMap` | *"Keys and values enter covenant. The map is formed."* |
| `TryBegin` | *"The gbiyanju opens. What follows will be caught if it falls."* |
| `TryEnd` | *"The gbiyanju closes. The danger has passed."* |
| `Throw` | *"An error rises. The gba will receive it if one waits."* |
| `Yield` | *"The program steps aside. The world may breathe."* |
| `Halt` | *"The chain is complete. The program has returned to silence."* |
| `Print` / `PrintRaw` | *"The result speaks. Irosu carries the word outward."* |
| `Not` / `And` / `Or` | *"Truth is tested. The logic holds or breaks."* |
| `DefineClass` | *"The odu takes form. Its methods are bound."* |

### 45.8 Speed Control

| Flag | Behavior |
|------|----------|
| `--speed slow` | 1.5 seconds per frame — for careful study of each layer |
| `--speed normal` | 400ms per frame — default; readable without waiting |
| `--speed fast` | 80ms per frame — for overview, opcode blur intentional |

There is no step mode and no pause. Ó Yẹlà is a movie, not an interactive debugger. It plays from start to finish. If the terminal is resized mid-playback, the remaining frames render at the new width. Pressing `q` exits cleanly at any point; the partial output remains in scrollback.

### 45.9 The Closing Sequence

After Act IV completes, the movie ends with a five-frame closing sequence that plays at `--speed slow` regardless of the user's speed setting.

**Frame 1:** Program output — whatever the IfáLang program printed, reproduced exactly.

**Frame 2:** Counts

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  THE NUMBERS
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    source lines         :  12
    AST nodes            :  34
    bytecode opcodes     :  61
    domain calls         :   3   (Irosu × 2,  Obara × 1)
    capability checks    :   3   (all passed)
    stack peak depth     :   4
    Arc allocations      :   0   (no Lists, Maps, or Closures)
    CPU cycles (approx)  :  ~240

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Frame 3:** The koan — printed one line at a time at 40ms per character:

```
  The computer is a divination system.
  You give it a question encoded as source code.
  It casts the oracle — compiling your words into patterns.
  It reads the patterns — executing opcodes one by one.
  It returns an answer — the output of your program.

  The Babalawo does not invent the future.
  The computer does not invent the output.
  Both only reveal what was already implied
  by the question you asked.
```

**Frame 4:** Silence — a blank frame held for two seconds.

**Frame 5:**

```
  Àṣẹ.
```

Then exit with code 0.

### 45.10 No-Arguments Behavior

When `ifa oyela` is run with no source file, it skips all four acts and plays only the closing koan and `Àṣẹ.` — frames 3 through 5 of §45.9. This is the entry point for someone who has heard the name but not yet understood what it is.

### 45.11 Implementation Architecture

Ó Yẹlà is implemented as a post-processing pass over a full execution trace, not as a live hook into the running VM. The VM executes the program normally, capturing every opcode event into a `Vec<OyelaFrame>`. Once execution completes, the renderer walks the frame list and prints the movie to stdout.

```rust
#[cfg(feature = "oyela")]
pub struct OyelaFrame {
    pub op:           OpCode,
    pub ip:           usize,
    pub stack_before: Vec<IfaValue>,
    pub stack_after:  Vec<IfaValue>,
    pub globals:      HashMap<String, IfaValue>,
    pub locals:       Vec<IfaValue>,
    pub source_line:  Option<u32>,
    pub domain_call:  Option<OyelaDomainEvent>,
}

#[cfg(feature = "oyela")]
pub struct OyelaDomainEvent {
    pub domain_id:  u8,
    pub method:     String,
    pub args:       Vec<IfaValue>,
    pub capability: Ofun,
    pub granted:    bool,
}

#[cfg(feature = "oyela")]
pub fn render_movie(
    source: &str,
    ast: &Program,
    bytecode: &Bytecode,
    frames: &[OyelaFrame],
    speed: OyelaSpeed,
    out: &mut dyn Write,
);
```

The VM accumulates `OyelaFrame`s during execution only when the `oyela` feature is compiled in. When the feature is absent, the accumulation code is completely elided at compile time — zero overhead on production builds.

The renderer is a pure function: it takes the captured trace and writes movie frames to any `dyn Write`. This makes it testable without a terminal — tests can capture the rendered output and assert on its content.

### 45.12 Spec Status and Non-goals

**Status:** `[DEFINED]`. Ó Yẹlà is gated behind `#[cfg(feature = "oyela")]` and **MUST** be completely absent from release binaries. The `oyela` feature **MUST NOT** be enabled in the default feature set.

**Non-goals:**
- Ó Yẹlà is not a debugger. There are no breakpoints, no state mutation, no step mode.
- Ó Yẹlà does not work on programs that exceed 500 opcodes. For longer programs, `ifa oyela` exits with: `"This program is too long to watch. Some things must be experienced, not observed."` The 500-opcode limit is intentional — Ó Yẹlà is for small programs that reveal the machine clearly, not large programs that would produce unwatchable noise.
- Ó Yẹlà does not work on the embedded tier. The embedded VM is a different runtime with different execution semantics and no terminal output.
- Act IV (CPU hardware) is explicitly a model — the "Erín processor" — not a disassembly of host machine code. It teaches the concept of fetch/decode/execute, not x86 or ARM specifics. This is a deliberate choice: the point is understanding the idea, not the particular silicon.

---

*Àṣẹ.*

---

*IfáLang Language Runtime Specification v0.2 — March 2026*
*Authority: this document. Implementation: ifa-core, ifa-std, ifa-embedded, ifa-babalawo, ifa-sandbox, ifa-wasm, ifa-oja, ifa-dap, ifa-deploy, ifa-lsp, ifa-fmt, ifa-docgen.*
*IfáLang is not trying to be Python with Yoruba keywords. It is trying to make Ifá philosophy computable.*