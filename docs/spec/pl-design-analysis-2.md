# PL Design Analysis — Ifá-Lang (Part 2)

*Expression Orientation · Macros · DSL · Compiler Design · Error Handling · Safety*

---

## 1. Expression-Oriented vs Statement-Oriented

### Current State

Ifá-Lang is **primarily statement-oriented with expression-oriented features**.

**Grammar structure** (`grammar.pest:12-42`):
```
program  = { SOI ~ statement* ~ EOI }
statement = { var_decl | if_stmt | while_stmt | return_stmt | ... }
```

The top-level unit is always a `statement`. Every statement ends with `;` (except blocks `{ }`).

**Expression terminals** (`grammar.pest:209-227`):
```
atom = {
    lambda_expr | odu_call | method_call | property_access
    | function_call | index_access | await_expr | move_expr
    | list_literal | map_literal | number | interpolated_string
    | string | boolean | nil | ident | "(" ~ expression ~ ")"
}
```

**Expression-oriented features that exist**:

| Feature | Evidence | Location |
|---------|----------|----------|
| Lambda expressions | `lambda_expr = { ese_kw ~ "(" ~ params? ~ ")" ~ "{" ~ statement* ~ "}" }` | Grammar line 230 |
| Blocks as expressions | `{ stmt* }` in grammar for if/while/for — but not first-class | Grammar lines 112-122 |
| Pipeline operator | `pipeline_expr = { null_coalesce_expr ~ ("|>" ~ null_coalesce_expr)* }` | Grammar line 195 |
| Ternary via `yan` | `yan (cond) { true => expr, false => expr }` — match as expression | Grammar line 149 |

**Statement-oriented features**:

| Feature | Evidence |
|---------|----------|
| All control flow is statements | `if`, `while`, `for`, `match` are statements, not expressions |
| Return is a statement | `return_stmt = { return_kw ~ expression? ~ ";" }` — no implicit tail expressions |
| Block `{ }` returns nothing | The last expression in a block is NOT automatically returned |
| Semicolons required | All statements end with `;` |
| Assignment is a statement | `assignment_stmt` — not an expression, so `x = y = 5` is impossible |

### Semantic Gap

The AST reflects this: `Statement` and `Expression` are **separate enums** (`ifa-types/src/ast.rs`). A `Statement::If { condition, then_body, else_body }` cannot be assigned to a variable. A `Match` statement cannot produce a value.

The compiler (`ifa-compiler/src/lib.rs:114-122`) iterates statements:
```rust
pub fn compile(mut self, program: &Program) -> IfaResult<Bytecode> {
    for stmt in &program.statements {
        self.compile_statement(stmt)?;
    }
    self.emit(OpCode::Halt);
}
```

No last-expression-is-return-value logic.

### Recommendation

Make `if`, `match`, and blocks **expression-producing**. The simplest approach:

1. Allow `if`/`match` as expressions when their arms end in expressions (not statements).
2. In the compiler, emit the last expression's value on the stack.
3. The VM already has the semantics: after a block, the stack contains the last value.

The grammar change is minimal — the `statement*` in `if_stmt` already allows expression statements (`instruction` = `expression ;`). The compiler change: after compiling an `if` body that ends with an expression, leave the value on the stack.

Currently blocked by: `Statement::If` and `Statement::Match` have `Vec<Statement>` bodies, not `Expression`. The AST needs a new `ExprIf` / `ExprMatch` variant, or the existing variants must carry a flag `is_expression: bool`.

---

## 2. Macro Systems

### Current State

**Rust procedural macros** (`ifa-macros/src/lib.rs`):

| Macro | Type | Purpose | Lines |
|-------|------|---------|-------|
| `#[derive(Ebo)]` | Derive | Auto-impl RAII `Drop` with optional `#[ebo(cleanup = "method")]` | 31-75 |
| `#[iwa_pele]` | Attribute | Compile-time balance checking: tracks so/pa, si/ti, mu/fi, bere/da pairs | 96-178 |
| `ebo_block!` | Function-like | Scoped RAII block with `_EboGuard` | 191-208 |
| `ajose!` | Function-like | Reactive binding: `ajose!(source => target)` or `ajose!(source => #freeze target)` | 251-278 |
| `#[derive(Observable)]` | Derive | Generates `watch_field()` methods for reactive update callbacks | 292-328 |

These are **host (Rust) macros** for users embedding Ifá-Lang via Rust. There is **no user-facing macro system** in Ifá-Lang itself — no `macro_rules!`, no procedural macros at the .ifa level.

**Ifá-Lang compiler macros** (`ifa-compiler/src/lib.rs:310-317`):

```rust
// Store constant expression for inlining
// ... Binary Ops in constants not yet fully folded ...
// And I inline it. `x = CONST`. `compile_expr(1+1)`.
```

The compiler has a compile-time constant evaluation (`fold_expression` at line 1840) that evaluates `1 + 2 → 3`, `"a" ++ "b" → "ab"`, etc. This is a very simple constant folding pass, not a macro system.

### Gap

| Feature | Status | Evidence |
|---------|--------|----------|
| Rust proc macros for Ifá host embedding | ✅ | `ifa-macros/src/lib.rs` |
| Compile-time constant folding | ✅ | `fold_expression` at `compiler/src/lib.rs:1840` |
| Declarative macros (`macro_rules!`) | ❌ | |
| Procedural macros at .ifa level | ❌ | |
| `#[attribute]` annotations on functions | ❌ | |
| Compile-time code generation | ❌ | |
| Token-based macro expansion | ❌ | |
| Hygiene | ❌ | |

### Recommendation

Add a `macro` declaration:

```
macro when(cond, body) {
    yan (cond) {
        true => body,
        _ => ofo,
    }
}
```

This requires:
1. A `MacroDef` AST node storing name, params, and template body.
2. A `MacroInvoke` expression that expands at compile time (before Babalawo analysis).
3. Hygiene: the macro body captures identifiers from the call site (not definition site) for variables, but captures domain names from the definition site — like Rust's `macro_rules!`.

Keep it simple — no procedural macros at the Ifá-Lang level. Declarative hygiene is sufficient for 95% of use cases (DSL embedding, boilerplate reduction, pattern templating).

---

## 3. DSL Embedding

### Current State

Ifá-Lang already supports **four forms of DSL embedding**:

**1. Odu domain calls** (`grammar.pest:97`):
```
odu_call = { odu_name ~ chain_op ~ ident ~ "(" ~ arguments? ~ ")" }
```
`Obara.fikun(10)` — domain-qualified method calls. Each Odu domain is a DSL. `Otura` is a networking DSL, `Odi` is a file I/O DSL, `Ogbe` is a system lifecycle DSL.

**2. Interpolated strings** (`grammar.pest:233-236`):
```
interpolated_string = { "$" ~ "\"" ~ (interp_part)* ~ "\"" }
interp_expr = { "{" ~ expression ~ "}" }
```
`$"Hello {name}"` — lightweight embedded expression language inside strings.

**3. Ebo blocks** (`grammar.pest:64`):
```
ebo_stmt = { ebo_kw ~ expression ~ ("{" ~ statement* ~ "}")? ~ ";"? }
```
Scoped resource regions — a DSL for memory lifecycle.

**4. Taboo declarations** (`grammar.pest:134`):
```
taboo_stmt = { taboo_kw ~ ":" ~ odu_name ~ "->" ~ odu_name ~ ";" }
```
`èèwọ̀: Ose -> Odi;` — an architectural constraint DSL.

### Gap

| Feature | Status |
|---------|--------|
| Domain-specific method calls | ✅ Odu system |
| String interpolation | ✅ `$"..."` |
| Scoped resource DSL | ✅ Ebo |
| Architectural constraint DSL | ✅ Taboo |
| Custom operators | ❌ |
| User-defined syntax extensions | ❌ |
| Compile-time DSL transformation | ❌ |

### Recommendation

Ifá-Lang already has the **core mechanism** for DSL embedding: the Odu domain system. A domain is a DSL. The missing piece is **user-defined domains with custom syntax**. The most practical approach:

1. Allow `odu` definitions to declare custom parser hooks (e.g., `odu Html { ... }` with inline HTML parsing).
2. Use the macro system (see §2) to define DSL-like syntax without extending the grammar.

Don't add a full Lisp-style macro system. The Odu DSL pattern (domain.method(args)) is extensible enough for most use cases. Focus on making domain creation easier.

---

## 4. Compiler Design

### 4.1 Full Pipeline

```
Source (.ifa)
    │
    ▼
┌──────────────────┐
│ Lexer (logos)    │  Token stream  ─── ifa-parser/src/lexer.rs
└──────────────────┘
    │
    ▼
┌──────────────────┐
│ Parser (pest PEG) │  AST (Program) ─── ifa-parser/src/parser.rs
└──────────────────┘
    │
    ▼
┌──────────────────┐
│ Babalawo (SA)    │  Semantic analysis ─── ifa-babalawo/
│  ├ Type checking  │  (scope, types, effects, taboos, moves)
│  ├ Effect check   │
│  ├ Taboo check    │
│  ├ Move tracking  │
│  └ Iwa (balance)  │
└──────────────────┘
    │
    ▼
┌──────────────────┐
│ Compiler          │  Bytecode (.ifab) ─── ifa-compiler/src/lib.rs
│  ├ Const folding  │
│  ├ Tail-call opt  │
│  └ Bytecode emit  │
└──────────────────┘
    │
    ▼
┌──────────────────┐
│ VM (bytecode)     │  Execution ─── ifa-vm/src/vm.rs
│  OR               │
│ Transpiler (Rust) │  Native binary ─── ifa-transpiler/src/lib.rs
│  OR               │
│ Sandbox (Wasmtime)│  Wasm ─── ifa-sandbox/src/omnibox.rs
└──────────────────┘
```

### 4.2 Lexer

**Logos-based** (`ifa-parser/src/lexer.rs:494` lines):

- `#[derive(Logos)]` on `Token` enum — generates a fast DFA-based lexer.
- `normalize_yoruba()` normalizes diacritics for keyword matching.
- `classify_domain()` maps identifier text to `OduDomain` (16 principal + infrastructure).
- `tokenize()` returns `Vec<Spanned<Token>>` — used by the formatter and LSP.

**Token count**: 40+ variants including keywords, operators, literals, domains, comments.

### 4.3 Parser

**Pest PEG parser** (`ifa-parser/src/parser.rs:1472+` lines):

- Grammar file: `ifa-parser/src/grammar.pest` (369 lines)
- `#[derive(Parser)]` + `#[grammar = "grammar.pest"]` — pest generates the parser.
- Entry: `Rule::program` → `SOI ~ statement* ~ EOI`
- Expression precedence (low to high, line 194):
  ```
  expression > pipeline > null-coalesce > or > and > not
  > comparison > arith > term > pow > factor > atom
  ```
- ~20 statement types, ~15 expression types, 7 binary operators, 4 unary operators.
- Bilingual keywords: `let` | `ayanmo`, `if` | `ti`, `match` | `yan`, etc.
- Diacritic variants: `àyànmọ́`, `gbìyànjú`, `nípàrí`, etc.
- Comments: `//`, `///`, `#`, `/* */`.

**Production rules**: ~70 rules total.

### 4.4 AST Design

**`ifa-types/src/ast.rs`** (598 lines):

Two top-level enums:

```rust
pub enum Statement {
    VarDecl, Assignment, Import, Const, Instruction, OduDef,
    EseDef, If, While, For, Return, Ase, Abo, Taboo, Ewo,
    Opon, Ebo, Defer, Update, Match, Expr, Ailewu, Yield,
    Try, Break, Continue, Throw,
}

pub enum Expression {
    Int, Float, String, Bool, Nil, Identifier,
    BinaryOp, UnaryOp, OduCall, MethodCall, Get,
    Call, Await, List, Map, Index, Try,
    InterpolatedString, Lambda, MoveExpr,
}
```

**Design characteristics**:
- Span info on every node (line, column, start, end).
- `TypeHint` enum for optional type annotations (29 variants).
- `Effect` enum for side-effect declarations (6 variants).
- `OduCall` with `resolved_domain`/`resolved_method_id` for fast dispatch.
- `MoveExpr` — reserved for future move semantics.

### 4.5 Semantic Analysis (Babalawo)

**`ifa-babalawo/`** — modular checker (~1500 lines total):

| Module | Function | Lines |
|--------|----------|-------|
| `checks.rs` | Top-level traverse + dispatch | ~750 |
| `diagnose.rs` | Error formatting, JSON, source annotations | ~450 |
| `effects.rs` | Effect constraint checking | ~60 |
| `iwa.rs` | Resource lifecycle balance + borrow tracking | ~420 |
| `movement.rs` | Linear move tracking | ~340 |
| `taboo.rs` | Domain call constraint enforcement | ~290 |
| `infer.rs` | Capability inference | ~50 |
| `inference.rs` | Type inference | ~50 |
| `wisdom.rs` | Error code → Odu proverb mapping | ~30 |
| `scope.rs` | Variable scope chain | ~80 |

**Two-pass analysis** (`checks.rs:210-220`):
1. First pass: collect definitions (functions, variables, odu, taboos).
2. Second pass: check each statement with full scope context.

### 4.6 Intermediate Representation (IR)

**None.** The compiler goes directly from AST to bytecode:

```rust
pub fn compile(mut self, program: &Program) -> IfaResult<Bytecode> {
    for stmt in &program.statements {
        self.compile_statement(stmt)?;  // AST → opcodes, one pass
    }
    self.emit(OpCode::Halt);
}
```

No IR means:
- No optimization between analysis and codegen.
- No separate SSA, CFG, or control-flow representation.
- No cross-function optimization.
- No type-based specialization at the bytecode level.

### 4.7 Optimization Pipeline

**Current passes** (all in `ifa-compiler/src/lib.rs`):

| Pass | Location | Description |
|------|----------|-------------|
| Constant folding | `fold_expression` (line 1840) | `1 + 2 → 3`, `"a" ++ "b" → "ab"` |
| Domain method folding | `fold_expression` (line 1986-2109) | `Ika.upper("hi") → "HI"` (when const args) |
| Tail-call optimization | `compile_statement` (line 709) | `return func()` → `TailCall` |
| String deduplication | `string_index` (line 31) | Each unique string emitted once |
| Deferred cleanup reordering | `end_scope` | Defer bodies compiled at scope exit |

**Missing passes**:

| Pass | Impact |
|------|--------|
| Dead code elimination | Unused `const` and unreachable code after `return` |
| Inlining | Small constant functions expanded at call site |
| Loop invariant hoisting | Move constant computations out of loops |
| Stack slot reuse | Reuse dead local slots instead of allocating new ones |
| Effect erasure | Pure functions could skip OduRegistry dispatch overhead |
| Constant propagation | Tracking constants across statements |

### 4.8 Code Generation

**Bytecode targets**:

| Target | Mechanism | Status |
|--------|-----------|--------|
| `.ifab` bytecode | `ifa-compiler` → `Bytecode` struct | ✅ Primary target |
| Rust source | `ifa-transpiler` → `transpile_to_rust()` | ✅ `ifa build` |
| Wasm | `ifa-sandbox` (Omnibox: Wasmtime AOT) | ✅ Sandbox target |
| Native binary | Transpile to Rust + cargo build | ✅ `ifa build` |
| x86/ARM | Via Rust compiler | ❌ Not direct (via transpiler → Rust → LLVM) |
| GPU | Opened as future direction (wgpu feature) | ❌ Not integrated |

### Recommendation

1. **Add a lightweight IR**: A `Vec<IfaInstr>` with basic blocks (labels, jumps, ops). This replaces direct bytecode emission and enables:
   - Dead code elimination (remove blocks after `Halt`)
   - Jump threading (eliminate jump-to-jump chains)
   - Simple register allocation (reuse slots)
   
2. **Keep the pipeline shallow**: Don't add LLVM or MLIR. A single IR pass before bytecode emission is sufficient for Ifá-Lang's complexity level.

3. **Compile to SSA for the optimizer**: If performance becomes critical, add a SSA-based optimization layer between AST and bytecode. But this is a long-term investment.

---

## 5. Error Handling Philosophy

### 5.1 Result Types (Primary Mechanism)

**`IfaResult<T>`** (`ifa-types/src/error.rs:10`):
```rust
pub type IfaResult<T> = Result<T, IfaError>;
```

**`IfaValue::Result`** (`value_union.rs:126-130`):
```rust
pub enum ResultPayload {
    Ok(IfaValue),
    Err(IfaValue),
}
```

The `?` error propagation operator (`grammar.pest:206-207`):
```
factor = { unary_op* ~ atom ~ try_op? }
try_op = { "?" }
```
Compiles to `OpCode::PropagateError` (0xA5).

### 5.2 Exceptions (Secondary Mechanism)

**Try/Catch/Finally** (`grammar.pest:175-182`):
```
try_stmt = { try_kw ~ "{" ~ statement* ~ "}" ~ catch_clause ~ nipari_clause? }
catch_clause = { catch_kw ~ "(" ~ ident ~ ")" ~ "{" ~ statement* ~ "}" }
nipari_clause = { nipari_kw ~ "{" ~ statement* ~ "}" }
```

**Throw** (`grammar.pest:185`):
```
throw_stmt = { throw_kw ~ expression ~ ";" }
```

VM implementation:
- `RecoveryFrame` saved on try entry: stack depth, call depth, catch IP, finally IP.
- `Throw` triggers `attempt_recovery()` which unwinds stacks to the recovery point.
- `FinallyEnd` resumes the suspended continuation (return value or error propagation).

**Three modes of error flow**:

| Mode | Mechanism | Finally execution |
|------|-----------|-------------------|
| Result propagation | `?` operator / `PropagateError` | No (no try block) |
| Caught exception | `try`/`catch` + `throw` | Yes, if `nipari` exists |
| Uncaught exception | No matching `RecoveryFrame` | Yes, unwinding through nested frames with finally |

### 5.3 Panic/Fatal Errors

The `IfaError` variants that represent **unrecoverable** conditions:

| Error | Cause | Result |
|-------|-------|--------|
| `StackUnderflow` | VM bug — popped with empty stack | Halts |
| `UnknownOpcode(u8)` | Bytecode corruption or version mismatch | Halts |
| `OponExhausted` | Memory limit exceeded | Halts with hint |
| `LoopBreak` / `LoopContinue` | Break/continue outside loop | Internal signal, never user-visible |
| `Exit(i32)` | `ase;` directive with exit code | Process exit |

### 5.4 Recoverable vs Nonrecoverable

| Classification | Recoverable | Nonrecoverable |
|---------------|-------------|----------------|
| User errors | `DivisionByZero`, `TypeError`, `IndexOutOfBounds`, `FileNotFound` | — |
| System errors | `Timeout`, `ConnectionFailed`, `PermissionDenied` | `OponExhausted` |
| Programming errors | `UndefinedVariable`, `UndefinedFunction`, `ArityMismatch` | `StackUnderflow`, `UnknownOpcode` |
| Control flow | `Yielded` (signals cooperative yield) | — |

Recoverable errors can be caught with `try`/`catch`. Nonrecoverable errors halt the VM.

### 5.5 Stack Traces

**`SpannedError`** (`ifa-types/src/error.rs:261-289`):
```rust
pub struct SpannedError {
    pub error: IfaError,
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
    pub source_line: Option<String>,
}
```

Output format:
```
ERROR at file.ifa:10:5: Division by zero - Ọ̀bàrà rejects
  10 | let x = 5 / 0;
     |         ^
  Hint: A stone does not ask the river its depth.
```

**VM stack trace generation** (`vm.rs`):

The `resume_execution` method enriches errors with source location:
```rust
// After step() returns an error:
// Enrich with bytecode source mapping (line number from debug info)
```

Each `CallFrame` has a `return_addr` that can be mapped to source lines via bytecode debug info.

### 5.6 Compiler Diagnostics (Babalawo)

**Severity levels** (`diagnose.rs:10-16`):
```
Error   — Aṣiṣe (must fix)
Warning — Ìkìlọ̀ (should fix)
Info    — Ìmọ̀ràn (suggestion)
Style   — Style recommendation
```

**Output formats**:

| Format | Method | Example |
|--------|--------|---------|
| Default | `format()` | `error[Ogbè] file.ifa:10:5\n  Variable 'x' not defined\n  Wisdom: ...` |
| Source | `format_with_source(source)` | Full code listing + carets + colors + line numbers + notes |
| Compact | `format_compact()` | `file.ifa:10:5: error: Variable 'x' not defined` |
| JSON | `format_json()` | `{"severity":"error","code":"UNDEFINED_VARIABLE","line":10,...}` |

**Wisdom integration**: Each error code maps to an Odu with a proverb:
```rust
let odu_key = ERROR_TO_ODU.get(code).copied().unwrap_or("OKANRAN");
let wisdom = ODU_WISDOM.get(odu_key).map(|w| w.advice);
```

**LSP integration** (`ifa-cli/src/lsp.rs`):
```rust
connection.sender.send(Notification::new(
    NotificationMethod::PublishDiagnostics,
    PublishDiagnosticsParams { uri, diagnostics, version: None },
));
```

### 5.7 Philosophical Alignment

Ifá-Lang's error handling is aligned with **Iwa Pele** (gentle character):

| Principle | Implementation |
|-----------|---------------|
| Graceful degradation | `try`/`catch` / `?` operator — errors are values |
| Clear guidance | Proverbs + wisdom per error code |
| No silent failures | Every error produces a diagnostic with location |
| Deterministic cleanup | `finally` / `Ebo` RAII — resources released on all exit paths |
| Non-panic by default | `DivisionByZero` returns an error, not a panic |

### Recommendation

1. **Add `--explain <CODE>`**: Print the full wisdom text + Odu context + examples for each error code.
2. **Add `#[allow]` annotations**: Let users suppress specific warnings per scope:
   ```
   #![allow(unused_variable)]
   ```
3. **Multi-span diagnostics**: Each error should be able to point to multiple locations (e.g., "variable `x` first defined here, then used after move here").

---

## 6. Safety & Security

### 6.1 Memory Safety

| Property | Status | Mechanism |
|----------|--------|-----------|
| No dangling pointers | ✅ | Opon uses `Vec<IfaValue>`, `truncate` on epoch end |
| No buffer overflow | ✅ | `get()`/`try_set()` bounds-checked |
| No use-after-free | ✅ | Heap values via `Arc` — Opon truncation doesn't affect Arc'd data |
| No double-free | ✅ | `Ebo` guards via `ManuallyDrop` + `mem::forget` |
| No uninitialized memory | ✅ | `Vec::resize` fills with `IfaValue::Null` |
| Stack overflow prevention | ✅ | Configurable `stack_limit` (default 4096) + `frame_limit` (default 512) |

### 6.2 Type Safety

| Property | Status | Evidence |
|----------|--------|----------|
| Static type checking | ✅ | Babalawo's `checks.rs` |
| Type inference | ✅ | `inference.rs` (`infer_expression_type`) |
| Type annotations | ✅ | `TypeHint` (29 variants) |
| Match exhaustiveness | ❌ | No check that all patterns are covered |
| Parametricity | ❌ | No generics (see Part 2 §1) |
| Null safety | ❌ | Null is a first-class value, no `Option<T>` |

### 6.3 Thread Safety

| Property | Status | Evidence |
|----------|--------|----------|
| Send bound on messages | ⚠️ `unsafe impl Send` | `actor.rs:86` |
| Freeze/thaw deep copy | ✅ | `value_union.rs:392-419` |
| Mutex for shared state | ✅ | `UpvalueCell = Arc<Mutex<IfaValue>>` |
| Cross-actor isolation | ✅ | Fresh `IfaVM` per actor |
| Thread-unsafe type rejection | ✅ | `freeze()` returns `Err` for closures/futures |
| Taboo thread-safety check | ✅ | `taboo.rs:142-171` (naming-convention based) |

### 6.4 Sandboxing

**Omnibox** (`ifa-sandbox/src/omnibox.rs`, 358 lines):

- **Engine**: Wasmtime-based with AOT compilation.
- **Pooling allocator** ("Linus Optimization"): pre-allocates memory pages for fast instantiation.
- **Epoch interruption**: execution timeouts via wasmtime's epoch-based interruption.
- **Capability query ABI**: the sandbox exposes `ewo.can_read`, `ewo.can_write`, `ewo.can_network` host functions.

**Sandbox flow**:
```
Source → parse → compile → wasm (via wasmtime) → execute
                                    ↓
                              CapabilitySet → WASI P1 context → store limits
```

### 6.5 Capability-Based Security

**`Ofun` enum** (`ifa-types/src/capability.rs:6-27`):

```rust
pub enum Ofun {
    ReadFiles { root: PathBuf },
    WriteFiles { root: PathBuf },
    Network { domains: Vec<String> },
    Execute { programs: Vec<String> },
    Environment { keys: Vec<String> },
    Time, Random, Stdio, Crypto,
    Bridge { language: String },
}
```

**`CapabilitySet`** (`capability.rs:60-178`):

- `grant(cap)`: Add capability (blocked if covered by sacrificed).
- `check(required)`: Is the required capability covered by granted or sacrificed?
- `revoke(cap)`: Move from granted to sacrificed (irreversible).
- `remove_matching(f)`: Revoke all matching capabilities.
- `inherit_from(parent)`: Child inherits parent's sacrificed list first.
- `covers(granted, required)`: Path-prefix matching (filesystem), domain-subset matching (network).

**Sacrifice semantics**: A revoked capability is added to the `sacrificed` list. It is permanently denied — even if another grant attempt covers it. Children inherit the sacrificed list: `inherit_from()` adds parent's sacrificed caps before adding parent's granted caps.

### 6.6 Undefined Behavior Policy

**`#![forbid(unsafe_code)]`** enforced in:
- `ifa-bytecode/src/lib.rs:15`
- `ifa-babalawo/src/lib.rs:6`

**No UB in the core bytecode crate** — this is the most critical crate for correctness.

**Known safe-unsafe boundary crossings**:

| Location | `unsafe` block | Soundness justification |
|----------|---------------|------------------------|
| `ebo.rs:42-44` | `ManuallyDrop::drop` + `mem::forget` | `forget(self)` prevents double-free; standard pattern |
| `ebo.rs:53` | `ManuallyDrop::take` | Only called before `forget(self)` |
| `ebo.rs:65` | `ManuallyDrop::take` in `Drop` | Only called once; `forget` prevents prior calls from reaching here |
| `actor.rs:86` | `unsafe impl Send for ActorMsg` | `IfaValue` variants are all `Send` (documented, not enforced by compiler) |

**UB that does NOT exist**:
- No raw pointer dereference in user code.
- No type punning (NaN boxing uses `f64::to_bits()` — safe).
- No inline assembly.
- No transmute of invalid bit patterns.

### 6.7 Safety Architecture Summary

```
Source (.ifa)
    │
    ▼
┌─────────────────────┐
│ Babalawo            │  Static safety
│  ├ Type checker     │    - Type mismatches → compile error
│  ├ Move tracker     │    - Use-after-move → compile error
│  ├ Effect checker   │    - Missing effect decl → compile error
│  ├ Iwa balance      │    - Unclosed resources → compile error
│  └ Taboo enforcer   │    - Forbidden domain calls → compile error
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ Virtual Machine     │  Runtime safety
│  ├ Bounds checking  │    - Opon get/try_set bounds-checked
│  ├ Type checking    │    - Add/Float/Str type errors at runtime
│  ├ Stack limits     │    - Configurable stack_depth + frame_limit
│  └ Fuel budget      │    - Execution fuel for sandboxed runs
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ Capability System   │  Security
│  ├ Ofun checks      │    - Every I/O call checks CapabilitySet
│  ├ Sacrifice        │    - Irreversible revocation
│  ├ Inheritance      │    - Child scopes inherit parent sacrifices
│  └ Sandbox (wasm)   │    - Omnibox: wasmtime with WASI + epoch interrupt
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ Actor Isolation     │  Concurrency safety
│  ├ Freeze/thaw      │    - Deep copy across actor boundaries
│  ├ Resource xfer    │    - ResourceTokens move, not copy
│  └ Fresh VM         │    - No shared globals/opon/registry
└─────────────────────┘
```

### Recommendation

1. **Document the `unsafe` invariant**: Add a `SAFETY.md` in `ifa-vm/src/` listing every `unsafe` block, its invariants, and what would make it unsound (e.g., "adding a non-`Send` variant to `IfaValue` would make `ActorMsg::Send` unsound"). This makes future refactors safer.

2. **Add `#[deny(unsafe_code)]`** to all crates except the three that need it (`ifa-vm`, `ifa-std`, `ifa-types` for NaN boxing). Currently only `ifa-bytecode` and `ifa-babalawo` have it.

3. **Extend the capability system** to cover more resource types: GPU allocation, file descriptor limits, network bandwidth quotas. The `Ofun` enum is extensible by design.

4. **Formalize the safety hierarchy** in `docs/spec/safety-model.md` — memory safety, type safety, thread safety, capability safety, and sandbox safety as separate layers with their own invariants and verification strategies.

---

*Specification v0.1. All claims reference specific code locations in crates/. 
The compiler pipeline document at docs/spec/compiler-pipeline.md contains the full 
pass structure. The safety architecture at docs/spec/safety-model.md is a planned 
document derived from this analysis.*
