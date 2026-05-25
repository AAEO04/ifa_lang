# Unfinished Features — Verified Against Actual Code

Items from the unified implementation plan that were confirmed as genuinely **not done** (or only partially done) during the multi-agent audit of `crates/` source code. Each entry includes the current state, target behavior, and a concrete implementation plan.

---

## 1. Language Surface (Grammar / AST / Compiler)

---

### 1.1 `**` Exponentiation — VM Handler Missing

**Current state:** `OpCode::Pow = 0x30` exists in the bytecode enum (`ifa-bytecode/src/lib.rs:170`). The grammar has power precedence, and the compiler emits `Pow`. But the VM `step()` in `ifa-vm/src/vm.rs` has **no matching arm** — any program using `**` hits an "unknown opcode" runtime error.

**Target:** `a ** b` evaluates as exponentiation for Int (`x.pow(y)`), Float (`x.powf(y)`), and mixed types.

**Implementation:**
1. Add arm to `step()` in `vm.rs` (around line 1530, near other binary ops):
   ```rust
   OpCode::Pow => {
       let b = self.stack.pop().ok_or(VmError::StackUnderflow)?;
       let a = self.stack.pop().ok_or(VmError::StackUnderflow)?;
       let result = match (a, b) {
           (IfaValue::Int(x), IfaValue::Int(y)) => {
               let y_u32 = u32::try_from(y).map_err(|_| VmError::Overflow)?;
               IfaValue::Int(x.checked_pow(y_u32).ok_or(VmError::Overflow)?)
           }
           (IfaValue::Float(x), IfaValue::Float(y)) => IfaValue::Float(x.powf(y)),
           (IfaValue::Int(x), IfaValue::Float(y)) => IfaValue::Float((x as f64).powf(y)),
           (IfaValue::Float(x), IfaValue::Int(y)) => IfaValue::Float(x.powi(i32::try_from(y).map_err(|_| VmError::Overflow)?)),
           _ => return Err(VmError::TypeError("Pow: expected numeric operands")),
       };
       self.stack.push(result);
   }
   ```
2. Test: `2 ** 3 == 8`, `2.5 ** 2 == 6.25`, `2 ** -1` → overflow error.

**Files touched:** `crates/ifa-vm/src/vm.rs`

---

### 1.2 `ayanfe` / `const` Declarations

**Current state:** No `const` keyword, no `MarkConst` opcode, no `Statement::Const` AST variant. The plan's "5.5 ayanfe/const" item is entirely absent.

**Target:** `const NAME = value;` declares a compile-time constant. Babalawo warns on reassignment. Compiler inlines the value.

**Syntax:**
```
const MAX_SIZE = 1024;
ayanfe PI = 3.14159;
```

**Implementation:**
1. Add AST variant (in `crates/ifa-types/src/ast.rs`):
   ```rust
   Statement::Const { name: String, value: Box<Expression>, span: Span }
   ```
2. Grammar (in `crates/ifa-parser/src/grammar.pest`):
   ```pest
   const_stmt = { const_kw ~ ident ~ "=" ~ expression ~ ";" }
   const_kw = { "const" | "ayanfe" }
   ```
3. Parser: arm for `Rule::const_stmt`
4. Compiler: evaluate initializer at compile time if it's a literal expression; emit value directly (not via `StoreLocal`). Store in constant pool for reuse.
5. Babalawo: `ConstDecl` enters name in scope as read-only. Warn on `StoreLocal` to a const name.
6. No runtime changes.

**Files touched:** `crates/ifa-types/src/ast.rs`, `crates/ifa-parser/src/grammar.pest`, `crates/ifa-parser/src/parser.rs`, `crates/ifa-compiler/src/lib.rs`, `crates/ifa-babalawo/src/checks.rs`

---

### 1.3 Alias / Rename Syntax

**Current state:** No `alias` keyword, no `Statement::Alias` variant. Plan item "5.5 alias" is absent.

**Target:** `alias NewName = ExistingName;` creates a compile-time alias for types or functions.

**Syntax:**
```
alias OgbeStr = Ogbe;
alias transform = Ose.map;
```

**Implementation:**
1. Add AST variant:
   ```rust
   Statement::Alias { name: String, target: Box<Expression>, span: Span }
   ```
2. Grammar:
   ```pest
   alias_stmt = { alias_kw ~ ident ~ "=" ~ expression ~ ";" }
   alias_kw = { "alias" | "oruko" }
   ```
3. Parser: arm for `Rule::alias_stmt`
4. Compiler: resolve alias at compile time, substituting target everywhere the alias is referenced
5. Babalawo: add alias to scope with a `Resolved::Alias` indirection. On name lookup, follow alias chain.

**Files touched:** Same set as `const` above.

---

### 1.4 Set Type `Set<T>`

**Current state:** No `Set` variant in `IfaValue`, no `BuildSet`/`SetAdd`/`SetHas`/`SetRemove` opcodes. Plan item "K4" is entirely absent.

**Target:** `Set{1, 2, 3}` creates an unordered collection of unique values.

**Syntax:**
```
let s = Set{1, 2, 3};
s |> Set.add(4);
s |> Set.has(2);   // true
s |> Set.remove(1);
for x in s { ... }
```

**Implementation:**
1. Add `IfaValue::Set(Vec<IfaValue>)` to `crates/ifa-types/src/value_union.rs`. Back with `Vec` for small sets (inline), or a `HashSet` behind the `native` feature flag.
2. Add opcodes to `crates/ifa-bytecode/src/lib.rs`:
   ```
   BuildSet = 0x60   // pop N values, push Set
   SetAdd   = 0x61   // pop set, pop value, push updated set
   SetHas   = 0x62   // pop set, pop value, push Bool
   SetRemove= 0x63   // pop set, pop value, push updated set
   SetLen   = 0x64   // pop set, push Int(len)
   ```
3. Wire opcodes in bytecode's `from_u8`, `mnemonic`, `operand_bytes`, `stack_effect`.
4. Compiler: `Expression::SetLit { values }` → compile each value → `BuildSet(N)`. Method calls `Set.add`, `Set.has`, etc. desugar to opcodes.
5. VM handlers in `step()` for each opcode.
6. Babalawo: verify all elements in `SetLit` have the same type.

**Files touched:** `crates/ifa-types/src/value_union.rs`, `crates/ifa-bytecode/src/lib.rs`, `crates/ifa-types/src/ast.rs`, `crates/ifa-parser/src/grammar.pest`, `crates/ifa-parser/src/parser.rs`, `crates/ifa-compiler/src/lib.rs`, `crates/ifa-vm/src/vm.rs`, `crates/ifa-babalawo/src/checks.rs`

---

### 1.5 Default Parameter Values

**Current state:** `Param` struct has only `name` and `type_hint`. No `default_value` field. Grammar param rule has no `= expr` syntax. Plan item "K6" is absent.

**Target:** `fn greet(name: Str, greeting: Str = "Hello")` allows omitting `greeting` at call site.

**Syntax:**
```
fun greet(name: Str, greeting: Str = "Hello") {
    print(greeting, " ", name);
}
greet("Alice");           // "Hello Alice"
greet("Bob", "Hi");       // "Hi Bob"
```

**Implementation:**
1. Extend `Param` in `ast.rs`:
   ```rust
   pub struct Param {
       pub name: String,
       pub type_hint: Option<TypeHint>,
       pub default_value: Option<Expression>,
   }
   ```
2. Grammar:
   ```pest
   param = { ident ~ (":" ~ type_name)? ~ ("=" ~ expression)? }
   ```
3. Parser: parse optional `= expr` after type hint in params
4. Compiler: count optional params. At call site, if fewer args than params, push default expressions for missing optional params before function body executes.
5. Babalawo: required params must precede optional params; default value type must match `type_hint`.

**Files touched:** `crates/ifa-types/src/ast.rs`, `crates/ifa-parser/src/grammar.pest`, `crates/ifa-parser/src/parser.rs`, `crates/ifa-compiler/src/lib.rs`, `crates/ifa-babalawo/src/checks.rs`

---

### 1.6 Ìpa Side-Effect Tags (`pelu Ipa`)

**Current state:** No `has_effect` field on `EseDef`. No `pelu Ipa` syntax in grammar. Plan item "K8" is absent.

**Target:** Functions can declare side effects with `pelu Ipa`. Babalawo warns when a pure function calls an effectful one.

**Syntax:**
```
fun pure_add(a: Int, b: Int) -> Int {
    a + b                    // no side effects, OK
}

fun log_message(msg: Str) pelu Ipa {
    print(msg)               // side effect, declared
}

fun bad_pure() {
    log_message("hi");       // Babalawo warning: pure function calls effectful 'log_message'
}
```

**Implementation:**
1. Add `has_effect: bool` to `EseDef` in AST:
   ```rust
   pub struct EseDef {
       pub name: String,
       pub params: Vec<Param>,
       pub body: Vec<Statement>,
       pub return_type: Option<TypeHint>,
       pub has_effect: bool,     // NEW
       pub span: Span,
   }
   ```
2. Grammar:
   ```pest
   effect_modifier = { "pelu" ~ "Ipa" }
   ese_def = { ese_kw ~ ident ~ "(" ~ params? ~ ")" ~ return_type? ~ effect_modifier? ~ "{" ~ statement* ~ "}" }
   ```
3. Parser: set `has_effect: true` when `effect_modifier` tokens are present
4. Babalawo: propagate effect tracking through `LintContext`. When inside a function with `has_effect: false`, flag any call to a function with `has_effect: true`.
5. Compiler: no change (compile-time annotation only).

**Files touched:** `crates/ifa-types/src/ast.rs`, `crates/ifa-parser/src/grammar.pest`, `crates/ifa-parser/src/parser.rs`, `crates/ifa-babalawo/src/checks.rs`

---

### 1.7 AssertType Opcode

**Current state:** No `AssertType` opcode. Plan item "K11" is absent.

**Target:** `assert_type(x, Int)` verifies a value's type at runtime, panicking on mismatch. Useful for FFI and dynamic dispatch boundaries.

**Syntax:**
```
assert_type(x, Int);
assert_type(y, Str);
```

**Implementation:**
1. Add `OpCode::AssertType = 0x65` to bytecode — pops value and type tag, checks match, errors on mismatch
2. Define `TypeTag` encoding as u8: `0=Int, 1=Float, 2=Str, 3=Bool, 4=List, 5=Map, 6=Set, 7=Closure, 8=Odu`
3. Add `Statement::AssertType { value: Box<Expression>, type_tag: TypeTag, span: Span }` to AST
4. Grammar:
   ```pest
   assert_type_stmt = { "assert_type" ~ "(" ~ expression ~ "," ~ type_name ~ ")" ~ ";" }
   ```
5. Parser: parse as a statement
6. Compiler: compile value → push type_tag → `AssertType`
7. VM: match on `IfaValue` discriminant against tag, `Err(VmError::TypeError("assert_type failed"))` on mismatch

**Files touched:** `crates/ifa-bytecode/src/lib.rs`, `crates/ifa-types/src/ast.rs`, `crates/ifa-parser/src/grammar.pest`, `crates/ifa-parser/src/parser.rs`, `crates/ifa-compiler/src/lib.rs`, `crates/ifa-vm/src/vm.rs`

---

## 2. Static Analysis (Babalawo)

---

### 2.1 Match Exhaustiveness Checking

**Current state:** `Expression::Match` exists in AST (line 458). Babalawo's `check_expression` handles `Match` (line 786+) but does not check whether all possible patterns are covered. Plan item "G6" is absent.

**Target:** Babalawo warns on non-exhaustive match: missing pattern variants produce a `LintError` with suggestions.

**Implementation:**
1. In `crates/ifa-babalawo/src/checks.rs`, extend the `Match` handler:
   - Infer the type of the matched value
   - For integer matches: check if a `default`/`else` arm exists; if not, warn
   - For enum-like types (future): verify all variants covered
   - For string matches: check that `default`/`else` exists
2. Pattern types to check: `Pattern::Wildcard` (always exhaustive), `Pattern::Literal` (partial), `Pattern::Range` (partial), `Pattern::Destructure` (partial)
3. Emit `LintError` with `"non-exhaustive match"` message and a note listing uncovered cases

**Files touched:** `crates/ifa-babalawo/src/checks.rs`

---

### 2.2 Resource Leak Detection

**Current state:** No tracking of resource lifetimes. `defer` and `ebo` epoch regions exist but Babalawo doesn't verify that resources are properly closed. Plan item "G7" is absent.

**Target:** Babalawo warns when a resource-returning call's result is neither assigned to a variable, passed to a close function, nor wrapped in a `defer`/`ebo`.

**Implementation:**
1. Define a list of methods that return resources: `Ogbe.open`, `Otura.connect`, `Ogbe.create_file`, etc.
2. In `LintContext`, add `resources: Vec<ResourceInfo>` where each resource tracks:
   - Variable it's bound to (if any)
   - Span of creation
   - Whether it has been closed or passed to a close function
3. Known close functions: `Ogbe.close`, `Ogbe.sink`, `Otura.disconnect`, etc.
4. On scope exit (`end_scope`), emit warnings for any `resources` still open
5. `defer` and `ebo` bodies that close the resource count as valid cleanup

**Files touched:** `crates/ifa-babalawo/src/checks.rs`, `crates/ifa-babalawo/src/scope.rs`

---

### 2.3 Purity Enforcement

**Current state:** No purity tracking. The `fun` keyword exists but Babalawo never verifies that a (non-`pelu Ipa`) function body is actually pure. Plan item "G8" is absent.

**Target:** Babalawo flags impure operations inside functions that don't have `pelu Ipa`.

**Implementation:**
1. In `LintContext`, add `pure_scope: bool` — set to `true` when entering a function without `pelu Ipa`
2. Define impure operations: `print`, `Ofun` calls (`dbg`, `read`), `Oyeku` (random), `Otura` (network), `Ogbe` (file I/O), `Ebo` statements, `defer` (if body is impure)
3. When `pure_scope && impure_operation` is detected, emit `LintError`: `"impure operation in pure function"`
4. Domain methods are pure by default; mark specific methods as impure in `ODU_METHODS`

**Files touched:** `crates/ifa-babalawo/src/checks.rs`, `crates/ifa-types/src/odu_metadata.rs`

---

### 2.4 `abo` / `strict` Mode

**Current state:** No `abo` keyword or strict mode. Plan item "G8.5" is absent.

**Target:** `abo;` at file top promotes all babalawo warnings to errors and enables additional strict checks.

**Syntax:**
```
abo;           // at top of file
```

**Additional strict checks:**
- Unused variables → error (not warning)
- Missing return type annotation on functions → error
- Unused function parameters → error
- Implicit Any type → error (type annotation required)
- Shadowing → error

**Implementation:**
1. Add `abo_kw` token to grammar: `"abo" | "strict"`
2. Parser: detect `abo;` at statement level, set `Program::strict: bool`
3. Babalawo: read `program.strict`. In strict mode, change warning `LintError` severity to error. Enable additional checks listed above.
4. Compiler: no change.

**Files touched:** `crates/ifa-types/src/ast.rs`, `crates/ifa-parser/src/grammar.pest`, `crates/ifa-parser/src/parser.rs`, `crates/ifa-babalawo/src/checks.rs`

---

## 3. VM Internals

---

### 3.1 Unified Call Dispatch

**Current state:** `vm.rs` has 4 separate copies of Fn/Closure/NativeFunction dispatch logic — at `Call` (line 1473), `TailCall` (line 1477), `Return` (line 1481), and `CallMethod` (line 1555). Each unwraps the callee value, matches on variant, pushes frames, etc. Plan item "E1" is absent.

**Target:** A single `call_value(&mut self, callee: IfaValue, args: Vec<IfaValue>) -> IfaResult<()>` method that all dispatch sites call.

**Implementation:**
1. Extract into `impl IfaVM`:
   ```rust
   fn call_value(&mut self, callee: IfaValue, args: Vec<IfaValue>) -> IfaResult<()> {
       match callee {
           IfaValue::Closure(closure) => {
               // push frame with closure.ip, closure.captures, args
               self.frames.push(Frame {
                   ip: self.ip,
                   stack_base: self.stack.len() - args.len(),
                   return_ip: self.ip + 1, // or from caller
                   // ...
               });
               self.ip = closure.ip;
           }
           IfaValue::NativeFunction(func) => {
               let result = func(args)?;
               self.stack.push(result);
           }
           IfaValue::DomainMethod(method) => {
               let result = dispatch_domain(method.domain, method.method, args, &self.opon)?;
               self.stack.push(result);
           }
           IfaValue::OduCall(odu) => {
               let result = dispatch_odu(odu.domain, odu.method, args)?;
               self.stack.push(result);
           }
           _ => return Err(VmError::TypeError("not callable")),
       }
       Ok(())
   }
   ```
2. Replace each dispatch site with `self.call_value(callee, args)?`
3. Test: `ifa test crates/ifa-vm/tests/` passes (all call patterns still work)

**Files touched:** `crates/ifa-vm/src/vm.rs`

---

### 3.2 Opcode Dispatcher Split

**Current state:** `step()` is ~1118 lines (lines 1214–2331) with most opcode handlers inline. Only 5 dispatch helpers have been extracted: `dispatch_call`, `dispatch_tail_call`, `dispatch_return`, `dispatch_parallel_for`, `dispatch_call_method`. Plan item "I2" is partially done.

**Target:** Each opcode (or family) has a `#[inline]` helper method. `step()` is a thin match dispatcher.

**Implementation:**
1. For each of the ~80 opcodes, extract inline match-arm code into methods:
   ```rust
   fn dispatch_push_int(&mut self, value: i64) { self.stack.push(IfaValue::Int(value)); }
   fn dispatch_push_str(&mut self, id: usize) -> IfaResult<()> { /* ... */ }
   fn dispatch_add(&mut self) -> IfaResult<()> { /* pop 2, push sum */ }
   fn dispatch_sub(&mut self) -> IfaResult<()> { /* pop 2, push diff */ }
   // ... 50-60 more
   ```
2. Group related opcodes into shared handlers:
   ```rust
   fn dispatch_binary_int_op(&mut self, op: IntOp) -> IfaResult<()> {
       let b = self.pop_int()?;
       let a = self.pop_int()?;
       self.stack.push(IfaValue::Int(match op {
           IntOp::Add => a + b,
           IntOp::Sub => a - b,
           IntOp::Mul => a * b,
           IntOp::Div => a / b,
           // ...
       }));
       Ok(())
   }
   ```
3. Result: `step()` becomes ~20 lines of `match` → `self.dispatch_*(...)`
4. Test: byte-for-byte identical execution for all existing test suites

**Files touched:** `crates/ifa-vm/src/vm.rs` (major refactor, single file)

---

## 4. Architecture Refactoring

---

### 4.1 Feature-gate Parser Dependencies

**Current state:** `ifa-parser/Cargo.toml` lists `logos`, `pest`, `pest_derive`, `pest_consume` as unconditional dependencies. Any crate depending on `ifa-parser` (even for type definitions) pulls in the full parser toolchain. Plan item "C3" is absent.

**Target:** Parser dependencies are optional, gated behind a `compiler` feature. Programs using only `ifa-types` or the bytecode crate don't link pest/logos.

**Implementation:**
1. In `crates/ifa-parser/Cargo.toml`:
   ```toml
   [features]
   default = ["compiler"]
   compiler = ["logos", "pest", "pest_derive", "pest_consume"]
   
   [dependencies]
   logos = { version = "...", optional = true }
   pest = { version = "...", optional = true }
   pest_derive = { version = "...", optional = true }
   pest_consume = { version = "...", optional = true }
   ```
2. Gate parser implementation body behind `#[cfg(feature = "compiler")]`
3. When `compiler` is disabled, export stub types only (AST, token types) — the `parse()` function returns `Err("parser not available")`
4. Update `crates/ifa-vm/Cargo.toml` to add a `compiler` feature:
   ```toml
   [features]
   compiler = ["ifa-parser/compiler"]
   ```
5. Update downstream dependents that need parsing to enable the feature

**Files touched:** `crates/ifa-parser/Cargo.toml`, `crates/ifa-parser/src/lib.rs`, `crates/ifa-vm/Cargo.toml`, other crates' Cargo.toml as needed

---

### 4.2 Decouple Babalawo from ifa-std

**Current state:** `ifa-babalawo/Cargo.toml` lists `ifa-std` as a dependency. This pulls all domain implementations (network, crypto, hardware, etc.) into babalawo's dependency tree, even though babalawo only needs metadata about domains (names, methods, type signatures). Plan item "C4" is absent.

**Target:** Babalawo depends on `ifa-types` only. Domain metadata lives in `ifa-types/src/odu_metadata.rs`.

**Implementation:**
1. Audit all `use ifa_std::*` imports in `crates/ifa-babalawo/src/`. Common patterns:
   - Domain name lookups → move to `ifa-types/src/odu_metadata.rs`
   - Method type signatures → already in `ODU_METHODS`
   - Capability checks → use `CapabilitySet` from `ifa-types`
2. Move any missing metadata from `ifa-std/src/vm_registry.rs` into `ifa-types/src/odu_metadata.rs`
3. Replace `ifa-std` dep with `ifa-types` in `ifa-babalawo/Cargo.toml`
4. Update import paths throughout babalawo
5. Test: `cargo check -p ifa-babalawo --no-default-features` succeeds without ifa-std
6. Test: `cargo test -p ifa-babalawo` still passes

**Files touched:** `crates/ifa-babalawo/Cargo.toml`, various `crates/ifa-babalawo/src/*.rs`, `crates/ifa-types/src/odu_metadata.rs`, `crates/ifa-std/src/vm_registry.rs`

---

### 4.3 ModuleLoader Struct

**Current state:** `ModuleState` exists but there's no `ModuleLoader` struct with caching, dependency resolution, or cycle detection. Module loading is done inline in the CLI commands. Plan item "E2" is partially done — the state type exists but the loader abstraction doesn't.

**Target:** A `ModuleLoader` that caches compiled modules keyed by file path, detects circular imports, and resolves transitive dependencies.

**Implementation:**
1. Create `crates/ifa-vm/src/loader.rs` (or in `ifa-cli`):
   ```rust
   pub struct ModuleLoader {
       cache: HashMap<PathBuf, CompiledModule>,
       loading: HashSet<PathBuf>,  // cycle detection
   }
   
   pub struct CompiledModule {
       pub ast: Program,
       pub bytecode: Bytecode,
       pub source: String,
       pub modified: SystemTime,
   }
   
   impl ModuleLoader {
       pub fn new() -> Self;
       pub fn load(&mut self, path: &Path) -> IfaResult<&CompiledModule>;
       pub fn load_from_source(&mut self, path: &Path, source: &str) -> IfaResult<&CompiledModule>;
       pub fn invalidate(&mut self, path: &Path);
   }
   ```
2. `load()`: check cache → check file modification time → parse → compile → cache
3. Cycle detection: if `path` is in `loading`, return error (`"circular import detected"`)
4. Replace inline module loading in CLI's `run`, `check`, `build` commands with `ModuleLoader`
5. Expose `invalidate()` for REPL/`--watch` mode

**Files touched:** new file `crates/ifa-vm/src/loader.rs`, `crates/ifa-vm/src/lib.rs`, `crates/ifa-cli/src/main.rs`

---

### 4.4 ExecutionContext Extraction

**Current state:** `stack: Vec<IfaValue>`, `frames: Vec<Frame>`, and `ip: usize` are direct fields on `IfaVM`. Any operation that needs execution state must go through the whole VM. Plan item "E3" is absent.

**Target:** A self-contained `ExecutionContext` struct holding stack, frames, and ip. VM operations take `&mut ExecutionContext` explicitly.

**Implementation:**
1. Create struct:
   ```rust
   pub struct ExecutionContext {
       pub stack: Vec<IfaValue>,
       pub frames: Vec<Frame>,
       pub ip: usize,
       pub fuel: usize,
   }
   ```
2. Move `stack`, `frames`, `ip`, `fuel` from `IfaVM` into `IfaVM::ctx: ExecutionContext`
3. Update all references:
   - `self.stack.pop()` → `self.ctx.stack.pop()`
   - `self.ip` → `self.ctx.ip`
   - `self.frames.push(...)` → `self.ctx.frames.push(...)`
   - `self.fuel` → `self.ctx.fuel`
4. For helper methods that don't need the full VM (e.g., `dispatch_add`), pass `&mut ExecutionContext` instead of `&mut self`
5. Benefit: snapshots for debugger (`ifa debug`), context reset for REPL, future parallel execution

**Files touched:** `crates/ifa-vm/src/vm.rs` (major refactor, single file)

---

## 5. Domain Wiring (Stream F)

---

### 5.1 Remaining Domain Registration

**Current state:** Of the 30 domain slots, domains 0–13 are wired (with gaps: Irosu audio methods missing, Otura HTTP-only). Domains 14–17, 19, 21–22, 28–29 have no dispatch table entries. Plan item "Stream F" is partially done (12/30 wired fully).

**Target:** Every domain from 0–29 has at least a stub dispatch returning "domain not implemented", with real methods for key domains.

**Domains to wire:**

| ID | Name | Purpose | Priority |
|----|------|---------|----------|
| 14 | Eta-Ogunda | Infrastructure (network, storage, config) | Medium |
| 15 | Ofun Nje | Security / permissions supplement | Medium |
| 16 | Coop | Cooperative concurrency, actors | Low |
| 17 | Opele | Extended random / divination chain | Low |
| 19 | — | GPU compute shaders | Low |
| 29 | — | Kernel (process, syscall) | Low |

**Also missing:**
- Domain 3 (Irosu): audio methods `siro` (play), `gbigbọn` (vibrate), `ariwo` (volume)
- Domain 12 (Otura): TCP methods `connect`, `listen`, `send`, `recv` (currently HTTP only)

**Implementation:**
1. For each new domain, create a stub module in `crates/ifa-std/src/domains/` (e.g., `eta_ogunda.rs`):
   ```rust
   pub fn dispatch(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
       match method {
           "ping" => Ok(IfaValue::Str("eta-ogunda: pong".into())),
           _ => Err(VmError::NotImplemented(format!("Eta-Ogunda: {} not implemented", method))),
       }
   }
   ```
2. Register in `crates/ifa-std/src/vm_registry.rs`:
   - Add `mod eta_ogunda;` etc.
   - Add dispatch table entries at lines 107–151
   - Add name mappings at lines 320–340
3. For Irosu audio: add methods to domain 3 dispatch (line ~500 area):
   ```rust
   "siro" | "play" => { /* delegate to audio backend */ }
   "gbigbọn" | "vibrate" => { /* delegate to haptic backend */ }
   ```
4. For Otura TCP: extend domain 12 dispatch with TCP methods alongside existing HTTP
5. Test: each domain responds to at least `ping`; audio methods exist in dispatch

**Files touched:** `crates/ifa-std/src/vm_registry.rs`, 10+ new files in `crates/ifa-std/src/domains/`, `crates/ifa-std/src/lib.rs`

---

## 6. Tooling

---

### 6.1 Build Cache

**Current state:** `ifa build` creates a temporary directory `ifa_build_{pid}` and recompiles from scratch every time. No persistent cache. Plan item "L1" is absent.

**Target:** `ifa build` caches compiled bytecode in `.oja/build_cache/`, keyed by source file hash. Subsequent builds skip unchanged files.

**Implementation:**
1. Add `build_cache` module in `crates/ifa-cli/src/`:
   ```rust
   pub struct BuildCache {
       cache_dir: PathBuf,
   }
   
   impl BuildCache {
       pub fn new(project_root: &Path) -> Self;
       pub fn lookup(&self, source_path: &Path) -> Option<Vec<u8>>;
       pub fn store(&self, source_path: &Path, bytecode: &[u8]) -> IfaResult<()>;
       pub fn clean(&self, older_than: Duration) -> IfaResult<()>;
   }
   ```
2. Cache key: SHA-256 of source file contents
3. Cache entry: `<hash>.ifab` in `.oja/build_cache/`
4. In `ifa build` command: before compilation, check cache; after compilation, store in cache
5. `ifa build --clean` deletes entries older than 30 days
6. Cache invalidation: if source file modification time is newer than cache entry, recompile

**Files touched:** new file `crates/ifa-cli/src/build_cache.rs`, `crates/ifa-cli/src/main.rs`

---

### 6.2 Parallel Babalawo + Compiler

**Current state:** `ifa run` and `ifa check` run babalawo analysis and bytecode compilation **sequentially**. Plan item "L2" is absent.

**Target:** Babalawo analysis and bytecode compilation run in parallel via `rayon::join`, saving wall time on multi-core systems.

**Implementation:**
1. In `crates/ifa-cli/src/main.rs`, restructure the `run` and `check` commands:
   ```rust
   use rayon::join;
   
   let (analysis, compilation) = join(
       || babalawo::analyze_program(&ast, &options),
       || compiler::compile_program(&ast, &options),
   );
   
   let warnings = analysis?;
   let bytecode = compilation?;
   
   // Report warnings from analysis
   for warning in warnings {
       eprintln!("{}", warning);
   }
   
   // Execute bytecode
   vm.run(&bytecode)?;
   ```
2. Handle error cases: if both fail, report both errors; if compilation succeeds but analysis fails (strict mode), report analysis errors
3. The AST must be `Send + Sync` for this to work — verify all AST types implement these traits
4. Benefit: ~30-50% faster `ifa run` and `ifa check` for large files

**Files touched:** `crates/ifa-cli/src/main.rs`
