# Deferred Feature Implementation Plan
## Against the Actual Codebase

> Read before writing anything: `grammar.pest` (331 lines), `ast.rs` (497 lines), `compiler.rs` (1670 lines), `vm.rs` (2861 lines), `ifa-bytecode/src/lib.rs`. All specific line numbers are verified.

---

## Tier 1 — Near-Term (Phase 2 Unblocked)

These are language features already partially present or trivially addable without new architectural work. They block Tier 1 conformance.

---

### 1. `??` Null Coalescing Operator

**What exists:**
- Grammar: **nothing.** No `??` token anywhere in `grammar.pest`. The operator precedence table in the spec (§6.2) lists it at level 6, but the grammar has no rule for it.
- AST: `BinaryOperator` in `ast.rs:446–467` has Add, Sub, Mul, Div, Mod, Eq, NotEq, Lt, LtEq, Gt, GtEq, And, Or. No `NullCoalesce`.
- Compiler: no emit path for it.
- VM: no opcode for it. It does not need one — it's syntactic sugar (same as `||` but checks `is_null()` instead of `is_falsy()`).
- Interpreter: no handler.

**What is needed — exact changes:**

**Step 1 — Grammar (`grammar.pest`)**

The expression grammar currently is:
```
or_expr = { and_expr ~ (or_op ~ and_expr)* }
```

Insert `null_coalesce_expr` between `or_expr` and `and_expr` at precedence level 6:
```pest
expression   = { or_expr }
or_expr      = { null_coalesce_expr ~ (or_op ~ null_coalesce_expr)* }
null_coalesce_expr = { and_expr ~ (null_coalesce_op ~ and_expr)* }
and_expr     = { not_expr ~ (and_op ~ not_expr)* }

null_coalesce_op = { "??" }
```

**Step 2 — AST (`ifa-types/src/ast.rs`)**

Add to `BinaryOperator`:
```rust
/// Null coalescing: left ?? right
/// Returns left if not null, otherwise evaluates and returns right
NullCoalesce,
```

Add to `Display` impl:
```rust
Self::NullCoalesce => write!(f, "??"),
```

**Step 3 — Parser (`ifa-core/src/parser.rs`)**

In `parse_binary_op`, add:
```rust
Rule::null_coalesce_op => Ok(BinaryOperator::NullCoalesce),
```

In the expression parsing branch that handles `or_expr`/`and_expr` groups, add `Rule::null_coalesce_expr` to the match arm alongside `Rule::or_expr`.

**Step 4 — Compiler (`ifa-core/src/compiler.rs`)**

In `compile_expression` → `BinaryOp` → match `op`:
```rust
BinaryOperator::NullCoalesce => {
    // Compile left
    self.compile_expression(left)?;
    // Dup left — we need it both to check and to return
    self.emit(OpCode::Dup);
    // Check is_null: push null, compare
    self.emit(OpCode::PushNull);
    self.emit(OpCode::Eq);
    // If NOT null, jump over the right side
    let end_jump = self.emit_jump(OpCode::JumpIfFalse);
    // Left was null: pop the dup'd left, evaluate right
    self.emit(OpCode::Pop);
    self.compile_expression(right)?;
    self.patch_jump(end_jump);
}
```

**Step 5 — Interpreter (`ifa-core/src/interpreter/core.rs`)**

In the `BinaryOp` evaluation match:
```rust
BinaryOperator::NullCoalesce => {
    let left_val = self.eval_expr(left)?;
    if left_val.is_null() {
        self.eval_expr(right)
    } else {
        Ok(left_val)
    }
}
```

**Step 6 — Conformance test**

```ifa
# expect: "default"
# test: null coalescing returns right on null left
ayanmo x = ofo;
pada x ?? "default";
```

```ifa
# expect: 0
# test: null coalescing returns 0 (not "default") — 0 is not null
ayanmo x = 0;
pada x ?? "default";
```

---

### 2. `**` Exponentiation Operator

**What exists:**
- Grammar: **nothing.** `grammar.pest` has `mul_op = { "*" }` but no `pow_op`. No `**` rule anywhere.
- Bytecode: `OpCode::Pow = 0x26` exists in `ifa-bytecode/src/lib.rs:118`. The VM handler at `vm.rs:2362–2374` is already implemented. Both `Int` and `Float` paths work.
- AST: `BinaryOperator` has no `Pow` variant.
- Compiler: no emit path.
- Interpreter: no handler (but `Obara.agbara(b, e)` handles power via domain call).

The VM opcode already exists and works. This is purely a frontend gap.

**What is needed:**

**Step 1 — Grammar**

The spec says `**` is right-associative (§6.7: `2 ** 3 ** 2` = `2 ** 9`). Right-associativity requires a recursive rule, not a `*` repetition:

```pest
// Insert between term and factor in the precedence chain:
term  = { pow_expr ~ ((mul_op | div_op | mod_op) ~ pow_expr)* }
pow_expr = { factor ~ (pow_op ~ pow_expr)? }   // right-recursive for right-assoc

pow_op = { "**" }
```

**Critical:** `pow_op` must be defined BEFORE `mul_op` in the grammar file order, or PEG will greedily match `*` from `mul_op` before seeing `**`. Currently `mul_op = { "*" }` is defined at line 228. Add `pow_op = { "**" }` BEFORE `mul_op`, and reorder the `term` rule to try `**` before `*`.

**Step 2 — AST**

```rust
/// Exponentiation: base ** exponent (right-associative)
Pow,
```

**Step 3 — Parser**

```rust
Rule::pow_op => Ok(BinaryOperator::Pow),
```

Handle right-associativity in the expression parsing loop: when you see `pow_op`, recursively parse the right side rather than looping.

**Step 4 — Compiler**

```rust
BinaryOperator::Pow => {
    self.compile_expression(left)?;
    self.compile_expression(right)?;
    self.emit(OpCode::Pow);
}
```

**Step 5 — Interpreter**

```rust
BinaryOperator::Pow => {
    let base = self.eval_expr(left)?;
    let exp = self.eval_expr(right)?;
    match (base, exp) {
        (IfaValue::Int(b), IfaValue::Int(e)) if e >= 0 => {
            Ok(IfaValue::Int(b.pow(e as u32)))
        }
        (IfaValue::Int(b), IfaValue::Int(e)) => {
            // Negative int exponent → Float per spec §6.7
            Ok(IfaValue::Float((b as f64).powf(e as f64)))
        }
        (IfaValue::Float(b), IfaValue::Float(e)) => Ok(IfaValue::Float(b.powf(e))),
        (IfaValue::Int(b), IfaValue::Float(e)) => Ok(IfaValue::Float((b as f64).powf(e))),
        (IfaValue::Float(b), IfaValue::Int(e)) => Ok(IfaValue::Float(b.powi(e as i32))),
        _ => Err(IfaError::type_error("** requires numeric operands")),
    }
}
```

**Step 6 — VM fix**

Current VM handler (`vm.rs:2366–2368`) only handles `(Int, Int)` and `(Float, Float)`. The interpreter above handles mixed types. Fix the VM to match.

```rust
OpCode::Pow => {
    let exp = self.pop()?;
    let base = self.pop()?;
    match (base, exp) {
        (IfaValue::Int(b), IfaValue::Int(e)) if e >= 0 => {
            self.push(IfaValue::int(b.pow(e as u32)))?
        }
        (IfaValue::Int(b), IfaValue::Int(e)) => {
            self.push(IfaValue::float((b as f64).powf(e as f64)))?
        }
        (IfaValue::Float(b), IfaValue::Float(e)) => {
            self.push(IfaValue::float(b.powf(e)))?
        }
        (IfaValue::Int(b), IfaValue::Float(e)) => {
            self.push(IfaValue::float((b as f64).powf(e)))?
        }
        (IfaValue::Float(b), IfaValue::Int(e)) => {
            self.push(IfaValue::float(b.powi(e as i32)))?
        }
        _ => return Err(IfaError::Runtime("** requires numeric operands".into())),
    }
}
```

---

### 3. `%=` ModAssign

**What exists:**
- Grammar (`grammar.pest:82`): `update_op = { "+=" | "-=" | "*=" | "/=" }` — no `%=`.
- AST (`ast.rs:225–231`): `UpdateOp` has `AddAssign`, `SubAssign`, `MulAssign`, `DivAssign` — no `ModAssign`.
- Compiler: `compile_update_op` handles the four existing ops with `OpCode::Mod` already available.
- VM: `OpCode::Mod` exists and works.

**What is needed — minimal, mechanical:**

**Step 1 — Grammar**

```pest
update_op = { "+=" | "-=" | "*=" | "/=" | "%=" }
```

**Step 2 — AST**

```rust
pub enum UpdateOp {
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,   // ← add this
}
```

**Step 3 — Parser** (wherever `update_op` is converted to `UpdateOp`)

```rust
"%=" => UpdateOp::ModAssign,
```

**Step 4 — Compiler**

```rust
UpdateOp::ModAssign => self.emit(OpCode::Mod),
```

**Step 5 — Interpreter** (wherever `UpdateOp` is applied)

```rust
UpdateOp::ModAssign => match (current, rhs) {
    (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Int(a % b)),
    _ => Err(IfaError::type_error("%= requires integers")),
},
```

---

### 4. `ayanfe` Const Immutability Enforcement in the VM

**What exists:**
- Interpreter: correctly enforces `ayanfe`. `Environment::is_const` exists at `environment.rs:62`. Three call sites at `core.rs:535, 583, 656` check it before mutation and return `TypeError`.
- Compiler: `is_const_binding` at `compiler.rs:165` exists. Used at lines 258 and 381 to produce compile-time errors.
- VM: `StoreLocal`, `StoreGlobal`, `StoreUpvalue` handlers in `vm.rs` — **no const checking**. The VM does not know if a slot was declared `ayanfe`.

**The gap:** compile-time const checking works (the compiler rejects `ayanfe x = 1; x = 2;` at compile time). But if bytecode is constructed or arrives from an untrusted source, the VM will silently overwrite const slots.

**Decision required:** Is const enforcement a VM invariant or a compiler invariant?

- If **compiler invariant only**: document this in the spec, add a note to §17 that const enforcement is pre-execution. No VM change needed.
- If **VM invariant** (more correct): add a `is_const: bool` flag to the local variable slot. This changes the VM's local variable representation.

**Recommended approach** (minimal cost, correct semantics): Add a `const_slots: HashSet<u16>` to the `CallFrame` struct. When `StoreLocal` fires, check if the slot index is in `const_slots`. Populate `const_slots` during function prologue via a new opcode `MarkConst(slot)` emitted by the compiler after `StoreLocal` for `ayanfe` declarations.

**Step 1 — New opcode**

```rust
// ifa-bytecode/src/lib.rs
MarkConst = 0x5E,   // followed by u16 slot index
```

**Step 2 — Compiler**

After emitting `StoreLocal` for a `Const` statement:
```rust
if it's a const declaration {
    self.emit(OpCode::MarkConst);
    self.emit_u16(slot);
}
```

**Step 3 — VM CallFrame**

```rust
pub struct CallFrame {
    // ... existing fields
    const_slots: HashSet<u16>,
}
```

**Step 4 — VM handlers**

```rust
OpCode::MarkConst => {
    let slot = self.read_u16(bytecode)? as u16;
    self.current_frame_mut().const_slots.insert(slot);
}

OpCode::StoreLocal => {
    let slot = self.read_u16(bytecode)?;
    if self.current_frame().const_slots.contains(&(slot as u16)) {
        return Err(IfaError::Runtime(
            "Cannot assign to constant (ayanfe) binding".into()
        ));
    }
    // ... existing store logic
}
```

---

### 5. String/List Bounds Behavior

**What exists:** Need to verify actual VM behavior for `"Ifá"[-1]` and `nums[99]`.

**Check (run this):**
```powershell
# Create test file and run
echo 'pada "Ifa"[-1];' > test_bounds.ifa
cargo run -p ifa-cli -- runb test_bounds.ifa
```

**Spec requirement (§4.7):**
- String negative index: `"Ifá"[-1]` → `"á"` (last character)
- String out-of-bounds: `"Ifá"[10]` → `ofo`

**Spec requirement (§4.8):**
- List negative index: `nums[-1]` → last element
- List out-of-bounds: `nums[99]` → `ofo`

**Where to fix if broken:** `vm.rs` — the `GetIndex` opcode handler. Find the `OpCode::GetIndex` arm and add:
```rust
// For String
IfaValue::Str(s) => {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let idx = if *i < 0 { len + *i } else { *i };
    if idx < 0 || idx >= len {
        self.push(IfaValue::null())?;
    } else {
        self.push(IfaValue::str(&chars[idx as usize].to_string()))?;
    }
}
// For List
IfaValue::List(items) => {
    let len = items.len() as i64;
    let idx = if *i < 0 { len + *i } else { *i };
    if idx < 0 || idx >= len {
        self.push(IfaValue::null())?;
    } else {
        self.push(items[idx as usize].clone())?;
    }
}
```

Do the same fix in `interpreter/core.rs` for the AST interpreter.

---

## Tier 2 — Long-Term (Phase 5, Do Not Touch Until Phase 2 Complete)

These require new AST nodes, grammar rules, new opcode sequences, and new Babalawo analysis passes. None of this belongs in the current release. This section documents the work so it is not re-invented later.

---

### 6. `iru` — Sum Types / Algebraic Data Types

**What exists:** Nothing. `iru` appears only as a method name (`Ofun.iru()` = runtime type inspection). The ADT meaning is a design document item only.

**The full pipeline needed:**

**Grammar additions:**
```pest
iru_def = {
    public_mod? ~ "iru" ~ ident ~ "{" ~ iru_variant* ~ "}"
}
iru_variant = {
    ident ~ ("(" ~ type_name ~ ("," ~ type_name)* ~ ")")?  ~ ","?
}
```

**AST additions** (`ifa-types/src/ast.rs`):
```rust
Statement::IruDef {
    name: String,
    visibility: Visibility,
    variants: Vec<IruVariant>,
    span: Span,
}

pub struct IruVariant {
    pub name: String,
    pub fields: Vec<TypeHint>,  // empty = unit variant
}
```

**Value representation** (`ifa-types/src/value_union.rs`):

`IfaValue` needs a new variant:
```rust
Variant {
    type_name: Arc<str>,    // "Odu"
    tag: Arc<str>,          // "Eji_Ogbe"
    payload: Vec<IfaValue>, // fields if any
}
```

**Bytecode opcodes** (`ifa-bytecode/src/lib.rs`):
```rust
// Construct a named variant
MakeVariant = 0x5F,  // [type_str_idx: u16][tag_str_idx: u16][field_count: u8]
// Test if a value is a specific variant tag
IsVariant  = 0x60,  // [tag_str_idx: u16] → bool
// Destructure variant fields onto stack
UnpackVariant = 0x61, // [field_count: u8]
```

**Babalawo integration:** Pattern exhaustiveness check (Confluence Rule B2 — see section 8).

**Dependency:** `wo` fold (section 7) is the primary consumer. Implement together.

---

### 7. `wo` — Structural Fold

**What exists:** `okanran.rs` has a method named `wo` for debug output. That is unrelated.

**What is needed:**

`wo` consumes a recursive `iru` value by applying a function to each variant arm. The key design decision: is `wo` a language keyword or a Babalawo-verified pattern? Recommended: **keyword + Babalawo exhaustiveness check**.

```
wo odu {
  Eji_Ogbe    => process(odu),
  Branch(l,r) => wo(l) + wo(r)
}
```

**Grammar:**
```pest
wo_expr = {
    "wo" ~ expression ~ "{" ~ wo_arm* ~ "}"
}
wo_arm = {
    iru_pattern ~ "=>" ~ (expression | ("{" ~ statement* ~ "}"))
}
iru_pattern = {
    ident ~ ("(" ~ ident ~ ("," ~ ident)* ~ ")")?
    | wildcard_pattern
}
```

**AST:**
```rust
Expression::Wo {
    target: Box<Expression>,
    arms: Vec<WoArm>,
    span: Span,
}

pub struct WoArm {
    pub pattern: IruPattern,
    pub body: Box<Expression>,
}
```

**Compiler:** Emits `IsVariant` checks + `UnpackVariant` + jumps to the matching arm body. Similar to `match` compilation but with destructuring.

**Parallel eligibility:** The roadmap mentions independent branches are candidates for parallel execution. This is a Babalawo annotation only — the current VM is single-threaded. Tag it with a `#[parallel_eligible]` attribute in the `WoArm` for future use. Do not implement the parallel scheduler now.

---

### 8. `pade` — Structural Generate

**What is needed:**

```
ayanmo tree = pade(depth = 8) {
  ti depth == 0: Eji_Ogbe
  bibeko: Branch(pade(depth - 1), pade(depth - 1))
}
```

`pade` is syntactic sugar for a recursive function that bottom-folds into an `iru` value. The compiler desugars it into a named recursive function. There is no new VM opcode needed — it compiles to `PushFn` + `TailCall`.

**Grammar:**
```pest
pade_expr = {
    "pade" ~ "(" ~ pade_args? ~ ")" ~ "{" ~ pade_arm* ~ "}"
}
pade_args = { ident ~ "=" ~ expression ~ ("," ~ ident ~ "=" ~ expression)* }
pade_arm  = {
    ("ti" | "if") ~ expression ~ ":" ~ expression
    | ("bibeko" | "else") ~ ":" ~ expression
}
```

**Compiler desugaring:** `pade(depth = 8) { ti depth == 0: A; bibeko: B }` →
```
ayanmo __pade_0 = ese(depth) { ti depth == 0 { pada A; } bibeko { pada B; } };
__pade_0(8)
```

This is a source-level macro expansion in the parser/compiler. No new opcodes.

---

### 9. Babalawo Confluence Rules B1, B2, B3

**What exists:** `ifa-babalawo/src/checks.rs` has existing check infrastructure. `is_in_unsafe`, `is_in_async`, scope depth tracking, and `check_unsafe_ffi_call` are all present.

**B1 — Ebo Exhaustion:** Every opened resource must close on all exit paths.

This extends the existing `Ìwà` lifecycle tracking. The current engine tracks `begin_epoch`/`end_epoch` for Opon scoping. B1 requires tracking resource handles (file handles, network connections, DB connections) opened via Osa/Odi/Backend domain calls and verifying they are closed in every function exit path including error paths.

**Implementation approach:** Add a `resource_stack: Vec<ResourceHandle>` to `LintContext` in `checks.rs`. When Babalawo sees `Odi.open(...)`, push to stack. When it sees `Odi.close(...)` or the block exits, pop. If a function returns while `resource_stack` is non-empty, emit a warning `BW::ResourceLeak`.

**B2 — Yàn Exhaustivity:** Every `match` must cover all arms or use a wildcard.

Currently `match` in the AST has no exhaustiveness checking in Babalawo. Add:
```rust
fn check_match_exhaustiveness(arms: &[MatchArm], baba: &mut Babalawo) {
    let has_wildcard = arms.iter().any(|a| matches!(a.pattern, MatchPattern::Wildcard));
    if !has_wildcard {
        // For iru types: verify all variants are covered
        // For now: require wildcard if no iru type information available
        baba.warn(BabalawoWarn::NonExhaustiveMatch);
    }
}
```

When `iru` types are implemented, extend this to verify variant coverage statically.

**B3 — Purity Violation:** Functions marked pure may not call I/O domains.

This requires a `#[pure]` attribute on function definitions first. The AST `EseDef` does not have an `is_pure` flag. Add it:
```rust
Statement::EseDef {
    is_pure: bool,  // from #[pure] attribute
    // ... rest
}
```

Then in Babalawo, when traversing a pure function body, any call to Odi (fs), Otura (network), Osa (async), Sys domains emits `BabalawoError::PurityViolation`.

---

### 10. `Ẹgbẹ́` Actor Concurrency

**What exists:**
- `ifa-std/src/infra/cpu.rs` — task graph execution exists using Rayon. This is data-parallelism, not actor-model concurrency.
- `Osa` domain handles async operations via Tokio futures.
- The `Egbe` keyword/concept exists in planning documents only. No AST node, no grammar rule, no VM support.

**The scope of work (do not underestimate):**

Actor concurrency requires:

1. **Grammar:** `egbe` keyword for actor definition, `Osa.ebo()` for message send (already has Osa domain but needs `ebo` method to be the send primitive).

2. **Runtime isolation:** Each actor needs its own `Opon` (memory tray), its own stack, its own local variable frame. The VM currently has one `Opon` instance per `IfaVM`. Actors require `N` Opon instances.

3. **Message channel:** A typed mailbox (`mpsc::channel` in Rust terms) per actor. The `Ẹbọ` (payload) sent must be a value type — no shared references. This means `IfaValue` must be `Send` (check: does the current `IfaValue` contain `Rc`? If yes, actors cannot use it directly — need `Arc` or value cloning).

4. **Babalawo ownership verification:** When `Osa.ebo()` sends a value, Babalawo must statically prove the sender no longer holds a reference to that value. This is the hardest part — it requires a lightweight affine type check in Babalawo (not full Rust borrow checker — just "did you use this variable after sending it?").

5. **Scheduler:** Work-stealing scheduler (e.g. `rayon` for CPU-bound, `tokio` for I/O-bound). The current `cpu.rs` task graph uses Rayon — extend it.

**Minimum viable version (do not over-engineer):**

Phase 1 of actors: no ownership verification. Just spawn + message pass with deep copy semantics. Establish the runtime plumbing. Add Babalawo ownership checks in Phase 2 of actors after the runtime is stable.

```
egbe Counter {
    ayanmo count = 0;
    ese handle(msg) {
        ti msg == "inc" { count = count + 1; }
        ti msg == "get" { Osa.ebo(self, count); }
    }
}
ayanmo c = egbe.spawn(Counter);
Osa.send(c, "inc");
ayanmo result = reti Osa.recv(c);
```

**Key architectural constraint from the roadmap:** The actor implementation MUST NOT put a global lock in the VM. Each actor runs in its own `IfaVM` instance in its own thread. The VM is already thread-compatible as long as `IfaValue` is `Send`. Verify this first before any other actor work.

---

## Implementation Sequencing

```
Phase 2 (Start Now)
├── 1. ?? null coalescing          ← grammar + AST + compiler + interpreter
├── 2. ** exponentiation           ← grammar + AST (opcode + VM handler exist)
├── 3. %= ModAssign                ← grammar + AST + 4 lines in compiler/interpreter
├── 4. ayanfe const VM enforcement ← new MarkConst opcode + CallFrame.const_slots
└── 5. String/List bounds          ← fix GetIndex in vm.rs + interpreter

Phase 2 (After Conformance Gate Passes)
├── 6. B2 match exhaustiveness    ← Babalawo checks.rs, no new AST needed
├── 7. B1 resource leak detection ← Babalawo checks.rs, extend Ìwà tracking
└── 8. B3 purity violation        ← Add is_pure to EseDef AST + Babalawo check

Phase 5 (Do Not Start Until Phase 2 Ships)
├── 9.  iru sum types              ← New AST node + new IfaValue variant + 3 new opcodes
├── 10. wo structural fold         ← Depends on iru. New AST node, compiler desugars to jumps.
├── 11. pade structural generate   ← Desugars to recursive ese. Depends on iru.
└── 12. Ẹgbẹ́ actors               ← Runtime isolation, channel plumbing, Rayon/Tokio bridge
```

---

## What Can Be Done in a Day vs a Week

| Feature | Time estimate | Blocker |
|---|---|---|
| `%=` ModAssign | 30 minutes | None |
| String/List negative index | 1 hour | Need to verify current behavior first |
| `**` exponentiation (frontend only, opcode exists) | 2 hours | Grammar PEG ordering with `*` |
| `??` null coalescing | 3 hours | None |
| `ayanfe` VM const enforcement | 4 hours | None |
| B2 match exhaustiveness (wildcard-only) | 2 hours | None |
| B1 resource leak (simple heuristic) | 1 day | None |
| B3 purity violation | 1 day | Needs `is_pure` flag on EseDef first |
| `iru` sum types end-to-end | 1–2 weeks | Design IfaValue::Variant repr first |
| `wo` fold | 1 week | Depends on `iru` |
| `pade` generate | 3 days | Depends on `iru` |
| `Ẹgbẹ́` actors (Phase 1, no ownership check) | 3–4 weeks | Verify IfaValue is Send first |
