# AGENTS.md — Ifá-Lang

Only facts verifiable from `crates/` source code are included.

## Entrypoint & CLI

Binary `ifa` defined in `crates/ifa-cli/Cargo.toml:9-11` (`[[bin]]`), entry at `src/main.rs`.

Commands (all from `crates/ifa-cli/src/main.rs`):

| Command | Notes |
|---------|-------|
| `ifa run <file>` | Interpreter. Bare filename auto-inserts `run` (line 374-383). |
| `ifa bytecode <file>` | Compile to `.ifab`. |
| `ifa runb <file.ifab>` | **Requires matching `.ifa` source** for Babalawo verification (lines 611-632). |
| `ifa check <file>` | Parses + runs Babalawo static analysis. |
| `ifa fmt <file> --unstable` | **Requires `--unstable`** or hard-errors (lines 1041-1045). |
| `ifa test [path]` | Matches `*_test.ifa` / `test_*.ifa` (lines 1346-1353). |
| `ifa babalawo <path>` | `--strict`, `--format json/compact/verbose`, `--fast`. |
| `ifa lsp` | LSP server (stdin/stdout). |
| `ifa repl` | Interactive REPL. |
| `ifa build <file>` | Transpile → Rust → native binary. |
| `ifa oja <cmd>` | Package manager (init/add/remove/build/run/test/search/publish/audit/upgrade). |
| `ifa deploy` | Zero-config deployment scanner. |
| `ifa debug --file <path>` | Debug Adapter Protocol. |
| `ifa doc <input> <output>` | HTML docs from `.ifa` sources. |

## Test structure (within `crates/`)

| Crate | Tests |
|-------|-------|
| `ifa-vm` | `tests/` — conformance (AST + VM + transpiler), closures, tailcalls, safety, async, okanran, handlers, proptest. Bench: `opcode_dispatch`. |
| `ifa-cli` | `tests/conformance_tier1.rs` — runs `ifa` binary against `.ifa` files with `# expect:` comments. |
| `ifa-embedded` | `tests/cross_runtime.rs`, `semantic_parity.rs`, `yield_tests.rs`, `mmio_tests.rs`, `embedded_ptr_tests.rs`. |
| `ifa-babalawo` | `tests/type_tests.rs`, `wisdom_tests.rs`. |
| `ifa-sandbox` | `tests/security_tests.rs`, `capability_tests.rs`. |
| `ifa-std` | `tests/crypto_tests.rs`. |
| `ifa-bytecode` | `tests/stack_effects.rs`. |
| Property-based | proptest dev-dep in `ifa-vm/Cargo.toml` (line 75). |

## Crate architecture

**Foundation** (no_std compatible):
- `ifa-bytecode` — zero-deps `#![no_std]` `#![forbid(unsafe_code)]`; opcode instruction set + `.ifab` format.
- `ifa-types` — `IfaValue` (Arc<Mutex>, thread-local) and `IfaShared` (Arc<RwLock/DashMap>, Send+Sync). Depends on `ifa-bytecode`.

**Compilation pipeline**: `ifa-parser` (logos + pest) → `ifa-compiler` (AST → bytecode) → `ifa-bytecode`.

**Single execution path**: bytecode `ifa-vm` (`IfaVM`). The tree-walking `ifa-interpreter` crate has been archived — all execution goes through the compilation pipeline above.

**Standard library** (`ifa-std`): 16 Odù domains, feature-gated: `async_runtime`, `network`, `tui`, `crypto`, `backend`, `frontend`, `game`, `iot`, `ml`, `fusion`. Always-available: ogbe, oyeku, iwori, odi, irosu, owonrin, obara, okanran, ogunda, ika, oturupon, ofun. Gated: osa (`async_runtime`), otura (`network`), ose (`tui`), irete (`crypto`). Also: stacks/{crypto,backend,frontend,gamedev,ml,iot,fusion}, ffi (polyglot), infra/{cpu,gpu,storage,kernel,shaders,runtime}.

**Capability security** (`ifa-sandbox`): `Ofun` enum (ReadFiles, WriteFiles, Network, Execute, Environment, Time, Random, Stdio, Bridge). Passed as `CapabilitySet` to `ifa-vm`.

**Embedded** (`ifa-embedded`): no_std, heapless. Target features: `esp32`, `stm32`, `rp2040`. `ifa-vm` as dep with `default-features = false`.

**WASM** (`ifa-wasm`): browser bindings via wasm-bindgen.

**Other crates**: `ifa-transpiler` (AST→Rust), `ifa-macros` (proc macros), `ifa-fmt` (formatter), `ifa-installer-core`, `ifa-installer-gui`.

## Feature flags

Granular: `native`, `parallel` (rayon), `sysinfo`, `gpu` (wgpu), `persistence`, `network` (ureq), `audio` (rodio), `crypto`, `ml`, `js`, `python`, `native_ffi`, `wasm`. See individual `Cargo.toml` files.

## Conventions & traps

- `ifa fmt` requires `--unstable` flag — will hard-error otherwise.
- `.ifab` bytecode cannot run standalone: `ifa runb` re-parses the `.ifa` source for Babalawo integrity check before executing.
- `ifa-bytecode` uses edition 2021 (not workspace edition 2024) and its own version 0.1.0.
- `IfaValue` uses `Arc<Mutex>` / `Arc<str>` — not `Rc<RefCell>` despite what ARCHITECTURE.md claims. `IfaShared` uses `Arc<RwLock>` / `DashMap`.
