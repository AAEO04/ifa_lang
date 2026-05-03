# Ifa-Lang Release Checklist

This checklist is derived from:

- [IFA_LANG_RUNTIME_SPEC.md](./IFA_LANG_RUNTIME_SPEC.md)
- [ROADMAP.md](./ROADMAP.md)
- Current implementation state under `crates/`

It is intended for version wrap-up, not long-term research planning.

## Blockers

- `VM canonicalization gate`: `ifa runb` must satisfy the VM gate in `IFA_LANG_RUNTIME_SPEC.md §21.2`.
- `Tier 1 conformance automation`: verify there is a real automated Tier 1 harness and that `ifa runb` passes it.
- `Result payload rewrite`: `IfaValue::Result` is still the old shape in `crates/ifa-types/src/value_union.rs`.
- `VM closure de-mutexing`: `UpvalueCell` is still `Arc<Mutex<IfaValue>>` in `crates/ifa-types/src/value_union.rs` and still used in `crates/ifa-core/src/vm.rs`.
- `Backend parity decision`: the spec positions `ifa runb` as canonical, but `ifa run` still executes the AST interpreter in `crates/ifa-cli/src/main.rs`. Either finish parity or explicitly scope the release as dual-runtime with documented deviations.
- `Source location threading in VM errors`: the spec marks this as a known gap and Tier 1 requires source location in errors.
- `Bytecode format header/versioning`: the spec flags the missing bytecode version header as a required safety gap.
- `fmt command gating mismatch`: the spec says `ifa fmt` must be behind `--unstable` until verified, but the CLI currently exposes it as normal behavior.

## Should Fix Before Release

- `String-based VM protocol dispatch`: the compiler still emits method strings and the VM still resolves them dynamically.
- `Document the new sys.args contract`: CLI injection now exists for `run` and `runb`, but the spec/docs do not yet define it.
- `Interpreter vs VM catch binding mismatch`: the interpreter catch path binds a structured error map, while the VM catch path binds the raw thrown value or a stringified runtime error.
- `Registry/import gaps`: `OduRegistry::import` still defaults to "not implemented" in the core registry trait, and prior work logs already note VM import-path issues.
- `Stubbed VM stdlib areas`: some VM registry domains and stack integrations still expose stubs, placeholders, or simulation-only behavior.
- `REPL reality mismatch`: keep the docs aligned with the currently AST-backed REPL.
- `Docs encoding cleanup`: roadmap/spec mojibake should be cleaned before shipping externally.

## Defer

- Full AST walker removal after backend parity.
- Full `func_id` / integer-only POP dispatch after conformance and parity.
- Flat-buffer embedded redesign for `ifa-embedded` and `ifa-sandbox`.
- VM-based REPL persistent-state design.
- Advanced proof systems, scheduler redesign, and post-release research items already deferred in `ROADMAP.md`.

## Immediate Release Tasks

1. Run the real conformance and runtime test commands.
2. Finish `Result` layout rewrite.
3. Finish VM upvalue de-mutexing.
4. Decide and document whether this release promises canonical `runb` or a scoped subset.
5. Resolve the `fmt` spec/CLI mismatch.
6. Add spec/docs language for `sys.args`.
7. Produce a short runtime deviations note for anything intentionally incomplete.

## Pre-Tag Commands

```powershell
cargo test -p ifa-core --test conformance_vm_tests
cargo test -p ifa-core --test conformance_ast_tests
cargo test -p ifa-cli --test conformance_tier1
cargo check -p ifa-cli
cargo test -p ifa-babalawo
cargo test -p ifa-embedded
cargo test -p ifa-sandbox
```

## Suggested Tagging Gate

Do not tag the release until:

- all blocker items are either fixed or explicitly removed from the release promise,
- the conformance/test commands above are run and recorded,
- the spec, roadmap, and shipped CLI/runtime behavior tell the same story.
