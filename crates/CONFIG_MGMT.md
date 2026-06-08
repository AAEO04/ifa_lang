# Configuration Management — Ifá-Lang Crates

Per NPR 7123.1 Technical Configuration Management.

## Configuration Items

| CI ID | Item | Version | Location | Type |
|-------|------|---------|----------|------|
| CI-01 | ifa-bytecode (OpCode enum) | Workspace | `crates/ifa-bytecode/src/lib.rs` | Binary interface |
| CI-02 | ifa-bytecode (`.ifab` format) | Workspace | `crates/ifa-bytecode/src/format.rs` | File format |
| CI-03 | ifa-types (IfaValue, IfaError, AST) | Workspace | `crates/ifa-types/src/` | Shared type contract |
| CI-04 | ifa-parser (grammar) | Workspace | `crates/ifa-parser/src/` | Language grammar |
| CI-05 | ifa-compiler (bytecode emission) | Workspace | `crates/ifa-compiler/src/` | Compilation logic |
| CI-06 | ifa-transpiler (Rust output) | Workspace | `crates/ifa-transpiler/src/` | Code generation |
| CI-07 | ifa-vm (IfaVM bytecode execution) | Workspace | `crates/ifa-vm/src/` | Runtime engine |
| CI-08 | ifa-vm (tree-walking interpreter) | Workspace | `crates/ifa-vm/src/` | Interpreter engine |
| CI-09 | ifa-std (16 Odù domains) | Workspace | `crates/ifa-std/src/odu/` | Standard library |
| CI-10 | ifa-babalawo (static analysis rules) | Workspace | `crates/ifa-babalawo/src/` | Analysis engine |
| CI-11 | ifa-cli (subcommand definitions) | 1.3.0 | `crates/ifa-cli/src/main.rs` | User interface |
| CI-12 | ifa-fmt (formatting rules) | Workspace | `crates/ifa-fmt/src/lib.rs` | Style configuration |
| CI-13 | ifa-sandbox (Ofun capability set) | Workspace | `crates/ifa-sandbox/src/capability.rs` | Security policy |
| CI-14 | ifa-embedded (EmbeddedOpCode subset) | Workspace | `crates/ifa-embedded/src/lib.rs` | Embedded runtime |
| CI-15 | ifa-infra (ComputeBackend) | Workspace | `crates/ifa-infra/src/` | Hardware abstraction |
| CI-16 | ifa-wasm (WASM API surface) | Workspace | `crates/ifa-wasm/src/lib.rs` | Browser API |
| CI-17 | ifa-macros (proc-macro expansion) | Workspace | `crates/ifa-macros/src/lib.rs` | Code generation macros |
| CI-18 | ifa-installer-core (install logic) | Workspace | `crates/ifa-installer-core/src/` | Distribution |
| CI-19 | ifa-installer-gui (GUI wizard) | Workspace | `crates/ifa-installer-gui/src/` | Distribution |
| CI-20 | ifa-docs (documentation artifacts) | Workspace | `crates/ifa-docs/src/` | Documentation |

## Version Management

- **Workspace version**: All crates share the workspace version from root `Cargo.toml` (except `ifa-bytecode` which uses 0.1.0 and edition 2021).
- **Breaking changes**: Any change to OpCode discriminant values, AST node structure, or `IfaValue` representation is breaking.
- **Semantic versioning**: Workspace follows semver. Breaking changes require a minor or major version bump.

| CI | Stability Guarantee | Change Authority |
|----|--------------------|-----------------|
| CI-01 (OpCode values) | **Stable** — values never change | Core team approval |
| CI-02 (.ifab format) | **Stable** — magic, version fields | Core team approval |
| CI-03 (IfaValue) | **Stable** — tagged union layout | Core team approval |
| CI-04–CI-20 | **Unstable** — may change across minor versions | Individual crate maintainer |

## Change Control

| Change Type | Process | Approver |
|-------------|---------|----------|
| New opcode | Add variant, update `from_u8`/`operand_bytes`/`stack_effect`, update all VMs | Core team |
| AST change | Update parser grammar, compiler, transpiler, Babalawo | Core team |
| IfaValue change | Update value_union, all match sites in VM/std/sandbox | Core team |
| New feature (feature flag) | Add to Cargo.toml, update lib.rs cfg-gates | Crate maintainer |
| Test addition | Add to `tests/` or inline `#[cfg(test)]` | Any contributor |

## Baseline Documents

| Document | Location | Status |
|----------|----------|--------|
| Architecture overview | `ARCHITECTURE.md` | ✅ Current |
| Runtime specification | `IFA_LANG_RUNTIME_SPEC.md` | ✅ Current |
| Engineering analysis | `docs/spec/engineering-analysis.md` | ✅ Current |
| Requirements (this registry) | `crates/*/REQUIREMENTS.md` | ✅ Created |
| Verification matrix (this registry) | `crates/*/VERIFICATION_MATRIX.md` | ✅ Created |
| Risk log | `crates/RISK_LOG.md` | ✅ Created |
| Configuration management | `crates/CONFIG_MGMT.md` | ✅ Created |
