# Ifá-Lang PL Design Analysis — Part 3

**Audit date:** 2026-05-29  
**Documents audited:** `crates/ifa-parser/src/grammar.pest`, `crates/ifa-types/src/ast.rs`, `crates/ifa-vm/src/module_resolver.rs`, `crates/ifa-cli/src/oja.rs`, `crates/ifa-cli/src/lsp.rs`, `crates/ifa-cli/src/main.rs`, `crates/ifa-cli/src/sandbox.rs`, `crates/ifa-cli/src/deploy.rs`, `crates/ifa-embedded/src/`, `crates/ifa-types/src/odu_metadata.rs`, `crates/ifa-types/src/value_union.rs`, `crates/ifa-fmt/src/lib.rs`, `crates/ifa-std/src/hardware/gpu.rs`, `crates/ifa-std/src/lib.rs`, `crates/ifa-std/Cargo.toml`, `crates/ifa-bytecode/src/format.rs`, `crates/ifa-installer-core/src/`

---

## 15. Metaprogramming

### 15.1 Reflection

**Status: Minimal.** The Òfún domain (0x0F, Permissions/Reflection) provides the only reflection capabilities:

- **`Ofun.iru(value)`** (0x0F04) — `type_name()` → `&'static str` at `value_union.rs:293`. Returns `"Int"`, `"Float"`, `"Str"`, `"Bool"`, `"List"`, `"Map"`, `"Null"` etc.
- **`Ofun.je(value, type_name)`** (0x0F08) — `is_type()` → `bool`. Compares runtime type against a string name.
- **`Ofun.laaye(value)`** (0x0F05) — `is_alive()` → `bool`. Checks if a value is non-null / non-moved.
- **`Ofun.awon_agbara()`** (0x0F07) — `capabilities()` → `List<Str>`. Lists granted Òfún capabilities.
- **`Ofun.afiwe(value)`** (0x0F09) — `debug()` → `Str`. Debug string representation.
- **`Ofun.dbg(value)`** (0x0F0A) — `debug_print()`. Prints debug to stderr.

**What's missing:**
- No field-level or method-level runtime introspection. `Ofun.iru` returns only the type *name* as a string — it cannot enumerate fields of a struct, list methods of a class, or inspect metadata.
- No structural typing / duck-type checks at runtime.
- No `std::any::TypeId` equivalent — types are discriminated by a UTF-8 string comparison (`matches!(other.type_name())` at `value_union.rs:534`), not by a stable numeric ID.
- No compile-time reflection (no `const`-evaluable `type_of()`).

### 15.2 Compile-Time Execution

**Status: Constant folding only.** The compiler performs a single constant-folding pass at `ifa-compiler/src/lib.rs:1840` (`fold_expression`):

- **Literal binary ops**: `2 + 3` → `5` at compile time. Handles `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Pow`, `Concat`.
- **Literal unary ops**: `-5` → folded.
- **No user-level comptime**: No `const fn`, no `comptime { }` blocks, no compile-time evaluation of user functions. The `const` keyword exists but only declares compile-time *data* (literals), not compile-time *functions*.
- **No type-level computation**: No const generics, no type-level naturals.

### 15.3 Macros

**Status: Rust proc macros only.** The `ifa-macros` crate provides Rust-derive macros that operate at the Rust level, not the Ifá-Lang level:

- **`derive(Ebo)`** — Zero-cost RAII guard generation. At `ifa-macros/src/lib.rs:32`.
- **`#[iwa_pele]`** — Resource lifecycle contract enforcement. At `ifa-macros/src/lib.rs:130`.
- **`ebo_block!()`** — `defer!`-like closure-based scoped resource. At `ifa-macros/src/lib.rs:230`.
- **`ajose!()`** — Init block for Opon region initialization. At `ifa-macros/src/lib.rs:260`.
- **`derive(Watchable)`** — Generates `watch_<field>()` callback methods. At `ifa-macros/src/lib.rs:290`.
- **`derive(Observable)`** — Observer-pattern integration.

**What's missing:**
- No Ifá-Lang macro system (no `macro_rules!`, no procedural macros at the Ifá level).
- No `tokenize`/`parse`/`quote` equivalent for Ifá source.
- No declarative macros (`ifá_macro!`), no hygiene system.

### 15.4 Code Generation

**Status: Single-transpiler path.** The Rust transpiler (`ifa-transpiler/src/`) is the only code generator:

- **`transpile_to_rust(program) -> String`** — Full AST-to-Rust-string conversion.
- **`generate_project(config)`** — Wraps transpiled output in a Cargo project with `Cargo.toml`, `.gitignore`, `.oja/` structure.
- **Limitations**: `try`/`catch`/`throw`, update operators (`+=`, `++`), and `gbiyanju`/`gba` are explicitly unsupported (emit `compile_error!()` macros at transpiler lines 578-596).

### 15.5 AST Transformations

**Status: Compiler-only (no proc-macro-like phases).** The Ifá-Lang compiler pipeline has no multi-phase AST transformation system:

- Pest produces a CST (`Pair` iterators).
- `parser.rs` transforms CST → AST (`Vec<Statement>`) in a single recursive-descent pass.
- `Compiler::compile()` transforms AST → `Bytecode` in another single pass.
- **No intermediate AST between parsing and compilation** — no HIR, MIR, or THIR.
- **No AST visitor pattern** — the compiler directly matches on `Statement` and `Expression` variants.
- **No AST rewriting phase** — no desugaring pass (e.g., `for x in list` → `while` loop), no macro expansion stage.

---

## 16. Module & Dependency System

### 16.1 Import Model

**Grammar** (`grammar.pest:45-50`): Two forms:

1. **Bare import**: `iba std.otura;` / `import std.otura;` — imports entire module into scope.
2. **Named import**: `iba { yokuro, pin } lati std.otura;` / `import { method1 } from std.otura;` — selective name import.

Keyword forms: `iba` / `ìbà` / `import` / `mu` (import), `lati` / `láti` / `from` (from). Bilingual at the syntax level.

**Resolution** (`module_resolver.rs:75-117`):

1. Normalize path to dot-separated key (`utils.math`).
2. For each `search_path` directory, try candidates in order:
   - `utils/math.ifa` (source)
   - `utils/math/mod.ifa` (directory module)
   - `utils/math.ifab` (bytecode)
   - `utils/math/mod.ifab` (directory bytecode)
3. Std imports (`std.*`) handled by VM registry, not filesystem.

**No path-based import** — no `import "../../foo.ifa"`. All imports are module-path-based, resolved relative to `search_paths`.

### 16.2 Namespace Management

**Scope hierarchy** (`scope.rs:13-17`): `Scope` has a parent pointer forming a tree — `HashMap<String, VarInfo>` per scope. Variable lookup is O(depth) via recursive parent walk.

**`VarInfo`** (`scope.rs:4-11`):
```rust
struct VarInfo {
    type_hint: Option<TypeHint>,
    visibility: Visibility,
    domain: Option<String>,
    span: Span,
    is_const: bool,
}
```

**ScopeChain** (`scope.rs:80-128`): `enter_scope()` / `exit_scope()` API with a `current: Scope` that gets boxed on entry.

**Module namespaces**: Modules are flat dot-separated path keys. There is no `use` aliasing, no `pub use` re-export, no `pub(crate)` visibility (only `Private`/`Public`/`Crate` in `ast.rs:42-56`), and no nested module blocks within files. Each file is one module.

### 16.3 Versioning

**SemVer parser** (`oja.rs:22-41`): Custom `major.minor.patch` parser. No pre-release or build-metadata support.

**VersionConstraint** (`oja.rs:43-117`):
- `Any` — accepts anything.
- `Exact` — `=1.2.3`.
- `Caret` — `^1.2.3` (compatible: `>=1.2.3`, `<2.0.0`).
- `Tilde` — `~1.2.3` (approximately: `>=1.2.3`, `<1.3.0`).

**CLI version**: `ifa --version` returns `1.3.0` (hardcoded at `main.rs:22`).

### 16.4 Dependency Resolution

**MVS (Minimal Version Selection)** (`oja.rs:119-134`): Implements Go-style MVS — picks the *minimum* version that satisfies all constraints. Given multiple dependents with `^1.2.0` and `^1.3.0`, resolves to `1.3.0` (the minimum satisfying both).

**Registry** (`oja.rs:136`): `https://raw.githubusercontent.com/AAEO04/oja-registry/main` — flat-file registry hosted on GitHub. Index is a directory tree by package name length.

**Manifest** (`Iwe.toml`) (`oja.rs:152-172`):
```toml
[package]
name = "my_app"
version = "0.1.0"
description = "..."
authors = ["..."]
language = "ifa"

[dependencies]
tokunbo = "^1.0"
```

**Lockfile** (`oja.lock`) (`oja.rs:239-256`): `Vec<LockedPackage>` with `name`, `version`, `checksum` (SHA-256), and `source` per entry.

### 16.5 Reproducible Builds

**Lockfile pinning**: `oja.lock` pins exact versions + SHA-256 checksums. `VerifyIntegrity` at `oja.rs:1066-1094` validates every downloaded package.

**What's missing**:
- No `--locked` flag to reject non-lockfile builds.
- No vendor/mirror support.
- No build-hash manifest documenting compiler version → bytecode fingerprint.
- No source-reproducibility verification (build from identical source produces identical binary).

---

## 17. Distribution Model

### 17.1 Binary Distribution

**Pipeline** (`oja.rs:463-607`): `source.ifa` → transpile to Rust → `cargo build` (with `--release` flag) → binary copied to project root or `target/release/`.

**Installation** (`ifa-installer-core/src/install.rs:76-248`):
- Downloads and extracts platform-specific archive.
- Adds to PATH (registry-based on Windows, shell rc files on Unix at `installer-core/src/unix.rs:99-199`).
- Default install directory: `~/.ifa`.
- GUI installer via `ifa-installer-gui` (eframe/egui).

**Self-upgrade** (`oja.rs:1461-1472`): `ifa oja upgrade` — stub that returns error directing to GitHub releases. Not yet implemented.

### 17.2 Cross Compilation

**CLI flags** (`main.rs:142-144`): `ifa build --target <triple>`. Passed through to `cargo build --target <triple>`.

**Flash command** (`main.rs:173-183`, `969-978`): `ifa flash --target esp32 --port COM3` — delegates to `espflash` / `STM32_Programmer_CLI` / `picotool` depending on target. IoT-specific.

**Platform-gated sandbox** (`sandbox.rs:12`): `#[cfg(any(target_os = "linux", target_os = "macos"))]` for cgroups/unshare; `#[cfg(target_os = "windows")]` for Job Objects; `#[cfg(target_os = "macos")]` for sandbox-exec.

**Embedded targets** (`ifa-embedded`): Feature flags for `esp32`, `stm32`, `rp2040`. Cross-compilation for no_std targets.

### 17.3 Sandboxed Packages

**Ìgbálẹ̀ Sandbox** (`sandbox.rs`):

- **OS isolation**: cgroups + namespaces (Linux), Job Objects (Windows), sandbox-exec (macOS).
- **Capability enforcement**: `Ofun` enum grants/denies at runtime via `CapabilitySet`.
- **Resource limits**: `timeout` (Duration), `max_memory` (bytes), allowed read/write paths, network access toggle.

**Deploy manifest** (`deploy.rs:68-96`): Auto-generates `Iwe.toml` capability section from static analysis:

```toml
[package.capabilities]
read = ["/etc/config.ifa"]
write = ["/var/log/"]
network = false
```

### 17.4 Build Caching

**Status: None.** The `.oja/cache/` directory is created (`oja.rs:611-618`) but the `_cache_dir` variable is intentionally unused (`oja.rs:646`). Builds always transpile from scratch in a temp directory. No incremental compilation at the Ifá level. Rust's `cargo` incremental compilation applies to the transpiled Rust output only.

### 17.5 Reproducibility (Distribution Context)

**Checksum verification**: SHA-256 of `zip` archive on download (`oja.rs:1066-1094`). Verified against registry metadata before extraction.

**Package audit** (`oja.rs:1433-1458`): Checks all locked SHA-256 checksums for format validity (`sha256:` prefix, ≥71 chars). Does not verify against registry — local cache integrity only.

---

## 18. Developer Experience (DX)

### 18.1 Fast Compile Times

**Current profile**: Parse (~0.1ms for small files) + Compile (~0.3ms) + Babalawo check (~0.5ms) = under 1ms for single-file programs. No incremental recompilation — every edit re-parses the entire file. The LSP sends `FULL` document sync (not incremental), so even a single-character edit re-parses the whole source.

**Bottlenecks**:
- Pest PEG parsing is linear but allocates per `Pair` node on the heap.
- Single-threaded compilation — no parallelization across functions or modules.
- No incremental compilation — 100% cold recompilation on every change.

### 18.2 Clear Errors

**Babalawo diagnostics** (`diagnose.rs`):
- 4 severity levels: `Error` / `Warning` / `Info` / `Style`.
- 4 output formats: `default` / `source` (annotated source) / `compact` (single-line) / `JSON`.
- **Odu wisdom integration**: Every error includes a Yoruba proverb mapped by domain. For example, `DIVISION_BY_ZERO` → Òtúúrúpọ̀n (domain of subtraction/division) + associated wisdom.
- **Error codes**: Stable 0xXXXX format per `ifa-bytecode/src/error.rs`, organized by Odu domain (0x00XX runtime, 0x01XX memory, 0x02XX types, 0x03XX math, 0x04XX I/O).

### 18.3 IDE Support

**LSP server** (`lsp.rs`, 519 lines):

**Capabilities**:
- `textDocument/didOpen` — Full Babalawo analysis on file open.
- `textDocument/didChange` — Full Babalawo re-analysis on every edit (full sync, not incremental).
- `textDocument/completion` — Triggered by `.` and `:`. Returns:
  - Domain method completions (e.g., `Obara.` → list of math methods).
  - 18 Yoruba keyword completions (gbiyanju, gba, nipari, etc.).
  - 32 English keyword completions (try, catch, etc.).
  - 18 type completions (Int, Float, Str, ..., f64).
  - 16 Odu module completions with domain descriptions.
  - 9 std function completions (ka, ko, so, ...).
  - Dynamic variable completions from `LintContext::defined_vars`.
- `textDocument/codeAction` — Single code action: "Sanctify" — `UNUSED_VARIABLE` → prefix with `_`.

**What's missing**:
- **No `textDocument/hover`** — no type-on-hover, no documentation-on-hover.
- **No `textDocument/definition`** — no go-to-definition.
- **No `textDocument/references`** — no find-all-references.
- **No `textDocument/formatting`** — the formatter crate exists (`ifa-fmt`) but the LSP doesn't expose it.
- **No `textDocument/semanticTokens`** — syntax highlighting is editor-side only.
- **No `textDocument/inlayHint`** — no inline type hints.
- **No `textDocument/signatureHelp`** — no parameter hints on function calls.
- **No `textDocument/documentSymbol`** — no outline / breadcrumbs.
- **No `workspace/symbol`** — no workspace-wide symbol search.
- **No `textDocument/diagnostic`** (pull model) — uses only publish (push) model.

### 18.4 Hot Reloading

**Status: None.** There is:
- A **test** (`conformance_vm_tests.rs:438`) named `conformance_vm_import_reloads_on_change` — but this is a conformance test, not production infrastructure.
- **No file watcher** daemon (no `notify`/`inotify` dependency).
- **No state-preserving recompilation** — the REPL `.clear` command destroys all VM state.

### 18.5 Incremental Compilation

**Status: None at Ifá level.** Every `ifa run` re-parses, re-compiles, and re-executes from scratch. The transpiler always generates fresh Rust projects. No dependency graph is maintained. No change-impact analysis exists.

### 18.6 Easy Setup

**Installation paths**:
- **Rust crate**: `cargo install ifa-cli` (published on crates.io).
- **GUI installer**: `ifa-installer-gui` with eframe/egui cross-platform UI.
- **oja init** (`oja.rs:271-420`): Creates scaffold with `Iwe.toml`, `src/main.ifa`, `.gitignore`, `.oja/`. Domain templates (game, ml, fusion, iot, basic). Workspaces via `init_workspace()`.
- **Zero-config deploy** (`main.rs` Deploy command): Scans source → infers required capabilities → prints deploy manifest.

### 18.7 Formatter

**`ifa fmt`** (`ifa-fmt/src/lib.rs`, 214 lines):
- Token-stream approach (preserves comments — unusual for a PEG-derived formatter).
- Opinionated: 4-space indent, 100-char max width.
- Rules: space before `{`, newline after `{`, `} else {` inline, ` = ` spacing on assignment, operator spacing.
- **`--unstable` flag required** (`main.rs:1041-1045`) or it hard-errors.

---

## 19. Formal Foundations

### 19.1 Operational Semantics

**Documented in `docs/spec/formal-foundations.md`** — 25 small-step rules covering:

- **Opon**: `EpochBegin`, `EpochEnd`, `AllocEpoch`, `FreeEpoch`, `AllocStack`, `FreeStack`, `AllocBTree`, `FreeBTree`, `AllocFlight`, `FreeFlight` — 10 rules for the 4-region memory model.
- **Ebo**: `EboCreate`, `EboEnter`, `EboLeave`, `EboCommit`, `EboRollback` — 5 rules for RAII guard lifecycle.
- **Actor**: `ActorSpawn`, `ActorSend`, `ActorRecv`, `ActorFreeze`, `ActorThaw` — 5 rules.
- **Taboo**: `TabooCheck`, `TabooContext`, `TabooWildcard`, `TabooPropagate`, `TabooThread` — 5 rules.

### 19.2 Denotational Semantics

**Documented in `docs/spec/formal-foundations.md`** — domain equations mapping syntactic domains to semantic ones:

- **Opon**: State transition function `S' = alloc_epoch(S, n)` with post-conditions on epoch size and free-list.
- **Ebo**: Guard evaluation as a pair `⟨guard, released⟩`.
- **Actor**: Mailbox state as a pair sequence `⟨msg, sender, recipient⟩*` with `freeze : ActorState → FrozenActor`.
- **Taboo**: Context derivation `C ⊢ t: (src_domain, src_fn) → (tgt_domain, tgt_fn)`.

### 19.3 Type Theory

**Documented in `docs/spec/formal-foundations.md`** — judgment form `Γ ⊢ e : τ` covering:

- **Unit types**: `Int`, `Float`, `Bool`, `Str`.
- **Compound types**: `List(τ)`, `Map(κ, ν)`, `Option(τ)` (implicit via `Null`), `Function(τ₁, …, τₙ) → τ`.
- **Effect types**: `Pure`, `Async`, `Network`, `FileIO`, `State`, `Impure`.
- **Subtyping**: `Null ⊑ τ` for all reference types. No bounded quantification, no higher-kinded types.
- **Soundness**: Progress + preservation *not proven* — no mechanized proof, no Coq/Lean formalization.

### 19.4 Category Theory

**Status: None.** Zero occurrences of `functor`, `monad`, `applicative`, `category`, `semigroup`, or `monoid` across the entire codebase.

**Design implications**:
- No functor mapping over containers (no `List.map` where `map` is a generic functor — `Ogunda.map` exists but is an array-domain method, not a generic interface).
- No monadic bind for chaining effectful computations — try/catch and async/await are hardcoded language constructs.
- No applicative style for parallel validation.
- No semigroup/monoid abstraction for composable operations.

**Decision point**: Ifá-Lang consciously avoids category-theoretic abstractions. The Odu domain system serves as an alternative organizing principle — each domain is a namespace of related operations rather than a typeclass/interface. This is pragmatically simpler but rules out generic abstractions like `Traversable`, `Alternative`, or `MonadTrans`.

### 19.5 Proof Systems

**Status: None.**
- The Babalawo is a **static analyzer** (linting + linear type checking + resource lifecycle validation + taboo enforcement), not a proof assistant.
- No dependent types (no `∀` quantifiers, no indexed families).
- No refinement types (no `{ x: Int | x > 0 }` syntax).
- No contract system (no `requires`/`ensures`/`invariant` annotations).
- No theorem prover integration (no Coq, Lean, Z3, or SMT solver backend).

### 19.6 Formal Verification

**Status: None.**
- No formal specification language embedded in Ifá (no TLA+, no ACSL, no Dafny-like syntax).
- No model checking integration.
- No symbolic execution engine.
- No fuzzing integration at the language level (though `cargo fuzz` or `proptest` could be applied at the Rust implementation level).
- The only correctness guarantee is Babalawo's static analysis, which is ad-hoc (not derived from a formal specification).

---

## 20. Target Domain Considerations

The 16 Odu domains are designed to map to specific domain concerns, but the language's actual suitability varies significantly across targets.

### 20.1 Systems Programming

**Suitability: Low.** Despite the Opon region-based memory model being designed for systems-level control:

- `Arc<Mutex>` on every heap `IfaValue` variant — heavyweight for OS/driver development.
- No `#[repr(C)]` equivalent for FFI struct layout control.
- No inline assembly.
- No explicit memory addresses or pointer arithmetic (the `Ptr`/`Ref`/`RefMut` type hints exist in the AST at `ast.rs:336-341` but are aspirational — the VM has no corresponding operations).
- Opon epochs provide deterministic deallocation, which is systems-friendly, but the `Arc` overhead cancels the benefit.

### 20.2 Scientific Computing

**Suitability: Medium.**
- `Obara` (math add/mul) and `Oturupon` (math sub/div) split arithmetic across two domains — unusual for scientific computing where unified `+`, `-`, `*`, `/` are expected.
- NaN boxing (previously at `value_union.rs`, now deleted) previously conflicted with NaN payload encoding used in scientific computing for missing/exceptional values. Arithmetic now uses direct enum dispatch.
- No complex number type, no vector/SIMD types.
- GPU compute exists (wgpu shader dispatch via `ifa-std/src/hardware/gpu.rs`) but requires explicit buffer management — no automatic array offloading.

### 20.3 AI/ML

**Suitability: Medium (potential, unproven).**
- GPU compute pipeline exists: `init()` → `alloc_buffer()` → `dispatch_pipeline(x, y, z)` → `read_buffer()` → `sync()`.
- `ifa-std::stacks::ml` is referenced in templates but `stacks/` directory **does not exist** on disk — aspirational.
- No tensor type, no automatic differentiation, no gradient computation.
- No ONNX/TensorFlow/PyTorch interop beyond what `native_ffi` could provide via libffi.
- Python interop via `pyo3` (gated behind `python` feature) could bridge to ML ecosystem.

### 20.4 Embedded Systems

**Suitability: Medium (early stage).**
- `ifa-embedded` crate: no_std, heap-less via `default-features = false` on `ifa-vm`.
- Targets: `esp32`, `stm32`, `rp2040` via Cargo feature flags.
- Opon regions can be backed by pre-allocated static buffers — no malloc required.
- Opon sizes include `kekere` (small/tiny) specifically for embedded.
- Limitations: `Arc<Mutex>` is unavailable in no_std; the embedded path must disable all heap-dependent `IfaValue` variants. The `IfaValue` enum's heap variants (`List`, `Map`, `Fn`, `Closure`, etc.) make this non-trivial — you effectively get `Null` + `Bool` + `Int` + `Float` only.
- No MMIO operations at the Ifá level (the `Storage.write8/16/read8/read16` intrinsics at `compiler/src/lib.rs:1752+` are a start but incomplete).

### 20.5 Finance

**Suitability: Low.**
- No `Decimal` type — `f64` only (NaN boxing file that previously conflicted with decimal values has been deleted).
- No fixed-point arithmetic.
- No formal verification (critical for smart contracts in finance).
- Audit capability (`oja.rs:1433-1458`) checks checksums but does not analyze dependency diff or supply-chain provenance.

### 20.6 Distributed Systems

**Suitability: Medium.**
- Actor system (Òsá domain) provides OS-thread-per-actor with `sync_channel(64)` mailbox.
- Actor freeze/thaw enables state snapshot for migration or checkpoint.
- `Otura` (networking) provides HTTP client via `reqwest`.
- **What's missing**: Service discovery, RPC framework, distributed consensus protocols (Raft/Paxos), distributed tracing, circuit breakers, retry/backoff utilities.

### 20.7 Web

**Suitability: Low-Medium.**
- WASM support via `ifa-wasm` crate (wasm-bindgen for browser bindings).
- Transpiler can produce Rust → WASM via `wasm-pack`.
- No HTTP server framework (Otura is client-only).
- No templating/HTML generation.
- No JS interop beyond `boa_engine` (gated behind `js` feature).
- No CSS/DOM manipulation primitives.

### 20.8 Game Development

**Suitability: Low (aspirational).**
- `ifa-std::stacks::gamedev` referenced in init template — **does not exist** on disk.
- Òṣẹ́ (graphics/UI domain) has TUI support (crossterm/ratatui) but no 2D/3D graphics.
- GPU compute exists but is compute-shader-only — no rendering pipeline, no swap chain, no rasterization.
- No ECS (Entity Component System), no physics engine binding, no audio beyond what `rodio` (gated behind `audio` feature) provides.

### 20.9 Smart Contracts

**Suitability: Low.**
- No deterministic execution environment (timeouts, memory limits via sandbox, but no gas metering).
- No blockchain integration.
- No persistent state model beyond what `Odi` (files/SQLite via `rusqlite`) provides.
- Opon flight recorder provides execution trace — but this is for debugging, not for on-chain verification.
- Taboo system (domain-call constraints) is architecturally aligned with smart-contract permission models, but no blockchain VM integration exists.

### 20.10 Robotics

**Suitability: Low-Medium.**
- Embedded support (ESP32, STM32, RP2040) covers microcontroller-level control.
- Opon's deterministic allocation is suitable for real-time constraints.
- **What's missing**: Real-time scheduling, hardware abstraction layer (GPIO, PWM, ADC, I2C, SPI, UART), kinematics libraries, ROS2 integration.

### 20.11 OS Development

**Suitability: Very Low.**
- Requires no_std, no alloc, no runtime — Ifá-Lang is fundamentally a managed language.
- All `IfaValue` heap variants use `Arc<Mutex>` — impossible in bare-metal without an allocator.
- No interrupt handling, no page table manipulation, no context switching at the Ifá level.
- The embedded path is more plausible but targets specific microcontrollers, not general OS development.

### 20.12 GPU Programming

**Suitability: Low (nascent).**
- Compute shader dispatch via wgpu with explicit buffer management (`gpu.rs:176-337`).
- Three steps: `init()` → `alloc_buffer()` → `dispatch_pipeline(x, y, z)`.
- Direct memory mapping: `write_buffer()` / `read_buffer()` for data transfer.
- **What's missing**:
  - Shader authoring in Ifá (currently requires external WGSL/SPIR-V).
  - No GPU kernel compilation — shaders are strings passed to wgpu.
  - No graphics pipeline (vertex/fragment shaders, rasterization, swap chain).
  - No automatic memory transfer (CPU ↔ GPU is explicit).
  - No parallel invocation model (no `@parallel` annotation on functions).

---

## Cross-Cutting Observations

### Consistency with Performance Philosophy

The performance ranking from `pl-design-analysis.md` was: **Memory > Startup > Latency > Throughput > Compile > Binary size > Energy**.

- **Memory-sensitive**: Opon epochs + `Arc` refcounting + Opon sizes (tiny → unlimited) align.
- **Startup-sensitive**: LSP full-scan on every keystroke contradicts this priority. If startup time mattered, the LSP would use incremental sync and incremental analysis.
- **Latency-sensitive**: No JIT, no tiered compilation — pure ahead-of-time compilation means no warmup, but also no profile-guided optimization.

### Feature Gating Discipline

The Cargo feature-gate system is good: `network`, `database`, `tui`, `async_runtime`, `crypto`, `gpu`, `parallel`, `wasm`, `js`, `python`, `native_ffi`, `audio`, `ml`, `backend`, `frontend`, `game`, `fusion`. But the `stacks/` directories referenced in the init templates and documentation **do not exist** — a documentation/debt gap.

### The Odu Domain vs Category Theory Decision

Ifá-Lang's 16-domain architecture is an alternative to both:
1. **Traditional OOP** (classes with inheritance).
2. **Category-theoretic FP** (monads, functors, typeclasses).

Each Odu domain is a namespace of related operations with a binary identifier. This is structurally similar to:
- ML's **modules** / OCaml's **functors** (namespacing + interface).
- Rust's **traits** (but without generics, without associated types, without coherence).

The domain system works well for its intended purpose (organizing std library operations), but it is not a substitute for:
- Parametric polymorphism (which requires generics).
- Interface abstraction (which requires trait-like dispatch).
- Effect polymorphism (which requires an effect system with row polymorphism).
