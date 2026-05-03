# Workspace Crate Split Implementation Plan

This is the concrete, step-by-step guide to fracturing the `ifa-core` and `ifa-std` God Crates into a flat, high-performance workspace architecture.

> **Rule 0:** Commit after *every single step*. Do not attempt to do this in one massive PR.

---

## Phase 1: Extracting the Front-End (The Parser)

Currently, changing VM execution logic forces a rebuild of the Pest grammar. We extract the parser first to isolate compile times.

### Step 1.1: Create `ifa-parser`
1. Run: `cargo new --lib crates/ifa-parser`
2. Move files:
   - Move `crates/ifa-core/src/grammar.pest` -> `crates/ifa-parser/src/`
   - Move `crates/ifa-core/src/parser.rs` -> `crates/ifa-parser/src/`
   - Move `crates/ifa-core/src/lexer.rs` -> `crates/ifa-parser/src/`
   - Move the AST definitions (likely in `ifa-types` or `ifa-core`) into `ifa-parser` if they aren't already cleanly isolated.
3. Update `crates/ifa-parser/Cargo.toml`:
   - Add dependencies: `pest`, `pest_derive`, `ifa-types`.
4. Update `crates/ifa-core/Cargo.toml`:
   - Remove `pest` dependencies.
   - Add `ifa-parser = { path = "../ifa-parser" }`.
5. Fix imports in `ifa-core` and run `cargo check`. Commit.

---

## Phase 2: Extracting the Middle-End (The Compiler & Transpiler)

The compilation pipeline (AST to Bytecode) is completely separate from execution.

### Step 2.1: Create `ifa-compiler`
1. Run: `cargo new --lib crates/ifa-compiler`
2. Move files:
   - Move `crates/ifa-core/src/compiler.rs` (and any related IR/codegen files) -> `crates/ifa-compiler/src/`
3. Update `crates/ifa-compiler/Cargo.toml`:
   - Add dependencies: `ifa-types`, `ifa-parser`, `ifa-bytecode`.
4. Update `crates/ifa-core/Cargo.toml`:
   - Add `ifa-compiler = { path = "../ifa-compiler" }`.
5. Fix imports and run `cargo check`. Commit.

### Step 2.2: Create `ifa-transpiler`
1. Run: `cargo new --lib crates/ifa-transpiler`
2. Move files:
   - Move `crates/ifa-core/src/transpiler/` (Go/JS/Python generation) -> `crates/ifa-transpiler/src/`
3. Update `Cargo.toml` dependencies (`ifa-parser`, `ifa-types`).
4. Fix imports and run `cargo check`. Commit.

---

## Phase 3: The VM & The Legacy Interpreter

What is left in `ifa-core` is the actual execution engine (VM) and the old AST interpreter.

### Step 3.1: Isolate the Interpreter
1. The AST tree-walking interpreter is being extracted for architectural clarity.
2. Run: `cargo new --lib crates/ifa-interpreter`
3. Move `crates/ifa-core/src/interpreter/` into `crates/ifa-interpreter/src/`.
4. Update `crates/ifa-interpreter/Cargo.toml` with necessary dependencies (`ifa-types`, `ifa-parser`, etc).

### Step 3.2: Rename `ifa-core` to `ifa-vm`
1. Rename the folder: `mv crates/ifa-core crates/ifa-vm`
2. Open `crates/ifa-vm/Cargo.toml` and change `name = "ifa-core"` to `name = "ifa-vm"`.
3. The VM should now contain *only*: `vm.rs`, `opon.rs`, `vm_ikin.rs`, `vm_iroke.rs`, `ebo.rs`, and native interface stubs.
4. Search and replace `ifa_core` with `ifa_vm` across the entire workspace (especially in `ifa-cli` and `ifa-std`).
5. Run `cargo build`. Commit.

---

## Phase 4: Slaying the `ifa-std` Bloat (Deferred)

*Note: Modifications to `ifa-std` and the introduction of heavy feature-gating are deferred for a later phase. We will leave `ifa-std` as-is for now while focusing on flattening `ifa-core`.*

---

## Phase 5: Workspace Cleanup

1. Open the root `Cargo.toml`.
2. Verify the `[workspace.members]` array strictly matches the new directory structure:
```toml
[workspace]
members = [
    "crates/ifa-types",
    "crates/ifa-bytecode",
    "crates/ifa-parser",
    "crates/ifa-compiler",
    "crates/ifa-transpiler",
    "crates/ifa-vm",
    "crates/ifa-interpreter",
    "crates/ifa-std",
    "crates/ifa-babalawo",
    "crates/ifa-embedded",
    "crates/ifa-sandbox",
    "crates/ifa-cli"
]
```
3. Run `cargo clean` and then a full `cargo build --workspace`. 
4. Verify compile times have dropped significantly.
