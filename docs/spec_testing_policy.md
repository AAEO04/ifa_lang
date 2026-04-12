# Spec Testing Policy

Goal: stop treating `IFA_LANG_RUNTIME_SPEC.md` as “prose truth” and make it enforceable via executable checks.

## What is enforced automatically

- **Opcode byte assignments:** `tools/check_spec_opcode_sync.py` ensures §17.2 opcode rows match `crates/ifa-bytecode/src/lib.rs` exactly.

## What must be enforced with conformance tests

Any spec section marked `[DEFINED]` should have at least one test that fails when behavior drifts.

Current minimum:
- VM source-level conformance programs live under `tests/conformance/vm/`.
- `crates/ifa-core/tests/conformance_vm_tests.rs` compiles each `.ifa` file and executes it in the bytecode VM, asserting the `# expect:` directive.
- AST source-level conformance programs live under `tests/conformance/ast/`.
- `crates/ifa-core/tests/conformance_ast_tests.rs` executes each `.ifa` file in the AST interpreter and asserts the `# expect:` directive.

## Adding a new `[DEFINED]` section

1. Add a `.ifa` program under `tests/conformance/vm/` with:
   - a `# spec:` reference to the section (human traceability)
   - a `# expect:` line (machine-checked)
2. If the feature cannot be tested from source (e.g., instruction decoding), add a unit test in the most appropriate crate.
