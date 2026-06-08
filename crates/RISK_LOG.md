# Risk Log — Ifá-Lang Crates (Cross-Cutting)

Per NPR 7123.1 Technical Risk Management.

## Risk Scoring
- **L** = Likelihood (1–5), **I** = Impact (1–5), **RPN** = L × I (Risk Priority Number)

| ID | Risk | Crate(s) | L | I | RPN | Mitigation | Status |
|----|------|-----------|---|---|-----|------------|--------|
| RSK-01 | **No dedicated tests for compilation pipeline**: ifa-compiler, ifa-parser, ifa-transpiler have zero dedicated tests; all coverage is indirect through ifa-vm | ifa-parser, ifa-compiler, ifa-transpiler | 4 | 4 | 16 | Add dedicated unit tests for each crate; conformance test suite must cover compile-level errors | Open |
| RSK-02 | **ifa-cli 94% untested**: 15 of 16 subcommands have no automated tests | ifa-cli | 3 | 5 | 15 | Add CLI integration tests for each subcommand using `assert_cmd` or `trycmd` | Open |
| RSK-03 | **ifa-infra zero tests**: Hardware abstraction (CPU/GPU/Storage/Kernel) completely untested | ifa-infra | 3 | 4 | 12 | Add unit tests for ComputeBackend, fallback tests for GPU (mock wgpu device), storage roundtrip | Open |
| RSK-04 | **ifa-macros zero tests**: Proc macros (Ebo, iwa_pele, ajose, Observable) have no expansion or error tests | ifa-macros | 3 | 3 | 9 | Add trybuild integration tests for macro expansion; add compile-fail tests for error paths | Open |
| RSK-05 | **Opcodes added without updating all consumers**: New opcode in ifa-bytecode may not be handled in ifa-vm, ifa-embedded, or ifa-compiler | ifa-bytecode → all | 2 | 5 | 10 | Add exhaustive switch-coverage compile test; CI must rebuild all consumers on opcode change | Open |
| RSK-06 | **Binary format stability violation**: Accidental change to OpCode discriminant values breaks all existing `.ifab` files | ifa-bytecode | 1 | 5 | 5 | Critical opcode stability test + CI check (existing) | ✅ Mitigated |
| RSK-07 | **Cross-runtime semantic drift**: ifa-embedded and ifa-vm produce different results for the same bytecode | ifa-embedded, ifa-vm | 2 | 4 | 8 | Cross-runtime & semantic parity tests (existing, EM-R08) | ✅ Mitigated |
| RSK-08 | **Capability bypass**: Bug in sandbox allows code to access resources without capability grant | ifa-sandbox | 2 | 5 | 10 | Deny-by-default enforced; security_tests.rs + capability_tests.rs (partial coverage) | Partially Mitigated |
| RSK-09 | **WASM playground untested**: Browser integration has no automated tests | ifa-wasm | 3 | 3 | 9 | Add wasm-bindgen-test or Playwright integration test suite | Open |
| RSK-10 | **Installer fails silently**: Core install/uninstall logic has no tests | ifa-installer-core, ifa-installer-gui | 3 | 4 | 12 | Add integration tests using temp directories; test headless mode | Open |
| RSK-11 | **Regressions in Babalawo diagnostics**: Changes to static analysis produce false positives/negatives | ifa-babalawo | 2 | 3 | 6 | Existing type_tests + wisdom_tests + effects_tests (67% coverage) | Partially Mitigated |
| RSK-12 | **StdLib domain implementation drift**: Domain implementations diverge from spec | ifa-std | 3 | 3 | 9 | Per-domain unit tests needed; only crypto_tests.rs exists | Open |

## Risk Trends
- **Highest risk**: RSK-01 (compilation pipeline) and RSK-02 (CLI) — these are the most user-facing and most untested.
- **Critical safety risks**: RSK-06 (binary format stability) and RSK-08 (capability bypass) — both partially mitigated.
- **Infrastructure risks**: RSK-03 (ifa-infra) and RSK-10 (installer) — no tests but lower user impact.

## Top Risk Reduction Priorities
1. Add ifa-parser + ifa-compiler dedicated tests (RSK-01)
2. Add ifa-cli subcommand integration tests (RSK-02)
3. Add ifa-infra compute backend tests (RSK-03)
4. Add ifa-installer-core integration tests (RSK-10)
