# Cargo & Crate Architecture — Hygiene Plan

> **Grounded in:** actual `Cargo.toml` files across 14 crates.
> **Rule:** Fix what is broken. Do not reorganize what works. No new crates unless the dependency problem cannot be solved any other way.

---

## Ground Truth: Confirmed Problems

| # | Problem | Location | Severity |
|---|---|---|---|
| 1 | `rayon` version mismatch: `1.8` vs `1.10` | `ifa-std` vs `ifa-core` | High |
| 2 | `ifa-types` hardcodes `version = "0.2.0"` | `ifa-types/Cargo.toml:3` | Medium |
| 3 | `ifa-bytecode` hardcodes `version = "0.1.0"` and `edition = "2021"` | `ifa-bytecode/Cargo.toml:3-4` | Medium |
| 4 | `colored = "2.0"` not in workspace | `ifa-babalawo/Cargo.toml:11` | Low |
| 5 | `sysinfo = "0.30"` duplicated locally in `ifa-cli` | `ifa-cli/Cargo.toml:49` | Low |
| 6 | `once_cell = "1.19"` not in workspace | `ifa-babalawo/Cargo.toml:14`, `ifa-std/Cargo.toml:26` | Low |
| 7 | `ndarray` pulled into `wasm` feature via `rsa_math` | `ifa-std/Cargo.toml:159,166` | High |
| 8 | `pyo3` with `auto-initialize` is a CI footgun | `ifa-std/Cargo.toml:78` | High |
| 9 | No minimal test feature defined in `ifa-std` | `ifa-std/Cargo.toml` | Medium |
| 10 | `logos` + `pest` + `pest_derive` unconditional in `ifa-core` | `ifa-core/Cargo.toml` | Medium |
| 11 | `ifa-babalawo` depends on `ifa-core` to reach AST types | `ifa-babalawo/Cargo.toml:10` | Medium |

### What the Review Got Wrong (Do Not Act On)
- "Circular dependencies" — there are none. Cargo would refuse to build.
- "miette/ariadne" — would destroy the proverb-based diagnostic identity. Rejected.
- "Typed index arenas" — premature for the current AST usage pattern.
- `ifa-embedded`'s custom serde features — intentional `no_std` constraint, correct as-is.

---

## Phase 1 — One-Line Workspace Fixes

**Goal:** Bring stray version pins into workspace control.  
**Risk:** Zero. These are declaration changes with no behavioral effect.  
**All changes in:** `Cargo.toml` (workspace root) + individual crate manifests.

### 1.1 — Add missing workspace dependencies

In `Cargo.toml` (workspace root), add to `[workspace.dependencies]`:

```toml
# Terminal coloring (Ìmọ̀lẹ̀ - The Illuminator)
colored = "2.0"

# Lazy statics
once_cell = "1.19"

# Concurrent HashMap
dashmap = "6.1"
```

Then remove the local pins:
- `ifa-babalawo/Cargo.toml`: replace `colored = "2.0"` → `colored.workspace = true`
- `ifa-babalawo/Cargo.toml`: replace `once_cell = "1.19"` → `once_cell.workspace = true`
- `ifa-std/Cargo.toml`: replace `once_cell = "1.19"` → `once_cell.workspace = true`
- `ifa-std/Cargo.toml`: replace `dashmap = { version = "6.1", optional = true }` → `dashmap = { workspace = true, optional = true }`

### 1.2 — Fix `ifa-types` and `ifa-bytecode` version pins

**`ifa-types/Cargo.toml`:**
```toml
# Before
version = "0.2.0"

# After
version.workspace = true
```

**`ifa-bytecode/Cargo.toml`:**
```toml
# Before
version = "0.1.0"
edition = "2021"

# After
version.workspace = true
edition.workspace = true
```

### 1.3 — Fix `sysinfo` duplicate in `ifa-cli`

`sysinfo` is already a workspace dep (declared in workspace root line 63 area — confirm). If not present, add to workspace:
```toml
sysinfo = { version = "0.30", optional = true }
```

In `ifa-cli/Cargo.toml`, replace:
```toml
# Before (line 49)
sysinfo = "0.30"

# After
sysinfo.workspace = true
```

### 1.4 — Fix `rayon` version mismatch

Currently: `ifa-std/Cargo.toml` pins `rayon = "1.8"`, `ifa-core/Cargo.toml` pins `rayon = "1.10"`.

Add to workspace root:
```toml
rayon = "1.10"
```

Replace in both crates:
```toml
# ifa-core/Cargo.toml
rayon = { version = "1.10", optional = true }
# →
rayon = { workspace = true, optional = true }

# ifa-std/Cargo.toml
[dependencies.rayon]
version = "1.8"
optional = true
# →
rayon = { workspace = true, optional = true }
```

### Exit Criteria for Phase 1

```powershell
# All dependency versions resolve to a single rayon instance:
cargo tree -p ifa-std --edges features | grep rayon
# Should show exactly one version.

# No local version pins remaining (excluding ifa-embedded which is intentional):
rg 'version = "' crates/*/Cargo.toml | grep -v "ifa-embedded"
# Should only show package version declarations, not dependency pins.
```

---

## Phase 2 — `ifa-std` Feature Flag Surgery

**Goal:** Fix the two dangerous feature definitions (`wasm` pulling `ndarray`, `pyo3` CI footgun) and define a minimal test feature for CI.

### 2.1 — Fix `wasm` feature pulling `ndarray`

**Root cause:** `wasm = ["rsa_math", ...]` and `rsa_math = ["ndarray", "rand"]`. `ndarray` has no guaranteed WASM-compatible build.

**Fix in `ifa-std/Cargo.toml`:**
```toml
# Before
wasm = [
    "rsa_math",
]
rsa_math = ["ndarray", "rand"]

# After — decouple wasm from ndarray entirely
wasm = [
    # Pure math/rand only — no native linear algebra in WASM
    "rand",
]
rsa_math = ["ndarray", "rand"]  # rsa_math stays as-is for native builds
```

Add a comment:
```toml
# NOTE: rsa_math (ndarray) is intentionally excluded from wasm.
# ndarray requires native BLAS linkage that is not WASM-compatible.
# For WASM math: use pure-Rust alternatives when needed.
```

### 2.2 — Defang `pyo3` `auto-initialize`

`auto-initialize` spawns a Python interpreter at build time in some configurations. This silently fails on CI environments without Python installed and can produce cryptic linker errors.

**Fix in `ifa-std/Cargo.toml`:**
```toml
# Before
pyo3 = { version = "0.27", features = ["auto-initialize"], optional = true }

# After
pyo3 = { version = "0.27", optional = true }
```

Add to `ifa-std/src/lib.rs` or wherever pyo3 is initialized:
```rust
// If auto-initialize was removed: initialize manually at the call site.
// pyo3::prepare_freethreaded_python() must be called before any Python FFI.
// See: https://pyo3.rs/latest/python-from-rust/calling-existing-code.html
```

### 2.3 — Define a `minimal` CI test feature

Currently `default = ["backend", "parallel", "dashmap"]` pulls tokio, reqwest, rusqlite on every `cargo test`. Add:

```toml
[features]
# ... existing features ...

# CI-safe minimal feature set: no tokio, no reqwest, no sqlite, no GPU.
# Use for: cargo test --no-default-features --features ci
ci = ["dashmap", "parallel"]
```

**In CI configuration** (`.github/workflows/*.yml` or equivalent):
```yaml
- name: Test ifa-std (minimal)
  run: cargo test -p ifa-std --no-default-features --features ci

- name: Test ifa-std (full, integration only)
  run: cargo test -p ifa-std --features full -- --test-threads=1
```

### Exit Criteria for Phase 2

- `cargo build -p ifa-std --features wasm --target wasm32-unknown-unknown` completes without ndarray-related errors.
- `cargo test -p ifa-std --no-default-features --features ci` passes in a Python-free environment.
- `cargo check -p ifa-std --features python` is the only command that requires Python on the host.

---

## Phase 3 — Feature-Gate Parser/Lexer in `ifa-core` for Embedded

**Goal:** `ifa-embedded` depends on `ifa-core` with `default-features = false`. But `logos`, `pest`, and `pest_derive` are **unconditional** dependencies — they compile into every `ifa-core` build, including embedded. A microcontroller binary should not contain a PEG parser.

### 3.1 — Add `compiler` feature to `ifa-core`

**`ifa-core/Cargo.toml`:**
```toml
[features]
default = ["native", "parallel", "sysinfo", "compiler"]
compiler = ["dep:logos", "dep:pest", "dep:pest_derive"]
native = ["ifa-sandbox"]
# ... rest unchanged ...
```

Move the parser/lexer deps to optional:
```toml
# Before (unconditional)
logos = "0.14"
pest = "2.7"
pest_derive = "2.7"

# After
logos = { version = "0.14", optional = true }
pest = { version = "2.7", optional = true }
pest_derive = { version = "2.7", optional = true }
```

### 3.2 — Gate parser/lexer source files

In `ifa-core/src/lib.rs`, wrap the parser/lexer module declarations:

```rust
#[cfg(feature = "compiler")]
pub mod lexer;

#[cfg(feature = "compiler")]
pub mod parser;

#[cfg(feature = "compiler")]
pub mod parser_utils;

#[cfg(feature = "compiler")]
pub mod compiler;

#[cfg(feature = "compiler")]
pub mod grammar {
    // pest_derive proc-macro output
}
```

### 3.3 — Update `ifa-cli` to enable `compiler`

`ifa-cli` needs to parse source files, so it must explicitly enable `compiler`:

```toml
# ifa-cli/Cargo.toml
ifa-core = { path = "../ifa-core", features = ["std", "sysinfo", "parallel", "gpu", "compiler"] }
```

### 3.4 — Verify `ifa-embedded` stays clean

```powershell
# Confirm no parser symbols in embedded binary:
cargo build -p ifa-embedded --no-default-features --features iot
# logos/pest must not appear in the dependency tree:
cargo tree -p ifa-embedded | Select-String "logos|pest"
# Expected: no output
```

### Exit Criteria for Phase 3

- `cargo build -p ifa-embedded --no-default-features` compiles without `logos` or `pest` in the dep tree.
- `cargo build -p ifa-cli` still compiles with parser available.
- `cargo build -p ifa-babalawo` still compiles (it uses AST types from `ifa-types`, not the parser).

---

## Phase 4 — Wire `ifa-babalawo` Off `ifa-core`

**Goal:** `ifa-babalawo` depends on `ifa-core` to access the AST. But `ifa-core/src/ast.rs` is already a **6-line re-export shim**:
```rust
pub use ifa_types::ast::*;
```

The AST lives in `ifa-types/src/ast.rs` (13KB). `ifa-babalawo` forces a full `ifa-core` compile — parser, lexer, VM, compiler, 109KB `vm.rs` — just to walk an AST that's already in `ifa-types`.

### 4.1 — Confirm AST completeness in `ifa-types`

Verify that `ifa-types/src/ast.rs` contains the full AST definition (Program, Statement, Expression, etc.) and nothing is missing that `ifa-babalawo` uses. Run:

```powershell
# Find everything ifa-babalawo imports from ifa-core:
rg "ifa_core::" crates/ifa-babalawo/src/ --type rust
```

If all imports are AST types re-exported through `ifa_core::ast::*`, then the swap is safe.

### 4.2 — Update `ifa-babalawo/Cargo.toml`

```toml
[dependencies]
# Before
ifa-core = { path = "../ifa-core" }

# After — AST types only, no VM, no parser, no compiler
ifa-types = { path = "../ifa-types" }
```

Remove `ifa-core` entirely from `ifa-babalawo`'s dependencies **only if** step 4.1 confirms all used symbols are in `ifa-types`. If `ifa-babalawo` uses anything from `ifa-core` that is not the AST (e.g., `IfaError`, `IfaValue`), those are also already in `ifa-types` — check each import and redirect.

### 4.3 — Update import paths in `ifa-babalawo/src/`

```rust
// Before
use ifa_core::ast::{Program, Statement, Expression};
use ifa_core::error::IfaError;

// After
use ifa_types::ast::{Program, Statement, Expression};
use ifa_types::error::IfaError;
```

### Exit Criteria for Phase 4

```powershell
# ifa-babalawo must not depend on ifa-core:
cargo tree -p ifa-babalawo | Select-String "ifa-core"
# Expected: no output

# ifa-babalawo still compiles:
cargo check -p ifa-babalawo
```

**Benefit:** After Phase 3, `ifa-core --features compiler` is heavy (logos, pest, 109KB vm.rs). `ifa-babalawo` is now free of that weight. Cold compile time for `ifa-babalawo` alone will drop significantly.

---

## Phase 5 — Deferred / Rejected

| Idea | Decision | Reason |
|---|---|---|
| New `ifa-parser` crate | **Deferred** | Phase 3 feature-gating achieves the same isolation at zero structural cost. Revisit only if `ifa-core` itself becomes a problem as a dependency for other new crates. |
| `miette` / `ariadne` diagnostics | **Rejected** | Proverb-based error messages are a core identity feature. `ariadne` would replace them with generic Rust-style output. |
| Typed index arenas for AST | **Deferred** | No compiler passes requiring O(1) node lookup yet. Add when SSA/CFG work begins. |
| Splitting `ifa-std` into domain sub-crates | **Deferred** | High effort, low yield. Fix features first. |
| `wasmtime` version audit | **Noted** | `wasmtime` is not in the workspace manifest or any reviewed Cargo.toml. The original review may have been looking at stale data. Verify before acting. |

---

## Sequencing Summary

```
Phase 1: Workspace dep fixes     ← Zero risk. Do first, commit separately.
    ↓
Phase 2: ifa-std feature surgery ← Depends on Phase 1 (rayon unified).
    ↓
Phase 3: Compiler feature gate   ← Depends on Phase 1 (workspace clean).
    ↓
Phase 4: Babalawo off ifa-core   ← Depends on Phase 3 (ifa-core is lighter after gating).
```

Phases 2 and 3 can be done in parallel after Phase 1.

---

## File Change Map

| Phase | Files Changed |
|---|---|
| 1 | `Cargo.toml`, `ifa-types/Cargo.toml`, `ifa-bytecode/Cargo.toml`, `ifa-babalawo/Cargo.toml`, `ifa-std/Cargo.toml`, `ifa-cli/Cargo.toml` |
| 2 | `ifa-std/Cargo.toml`, CI config |
| 3 | `ifa-core/Cargo.toml`, `ifa-core/src/lib.rs`, `ifa-cli/Cargo.toml` |
| 4 | `ifa-babalawo/Cargo.toml`, `ifa-babalawo/src/*.rs` (import paths only) |
