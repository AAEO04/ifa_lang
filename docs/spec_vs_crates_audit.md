# Spec ↔ Crates Cross-Check (March 2026)

Scope: `IFA_LANG_RUNTIME_SPEC.md` checked against `crates/` (notably `ifa-core`, `ifa-bytecode`, `ifa-types`, `ifa-embedded`).

This document is descriptive and meant to keep the spec’s status notes, capability matrix, and conformance notes aligned with actual code.

## High-Signal Findings

### 0) Opcode table is now byte-aligned with `ifa-bytecode`

- The spec’s §17.2 opcode table contains **79** opcodes.
- `crates/ifa-bytecode/src/lib.rs` defines **79** `OpCode` variants.
- A sync check confirms:
  - no missing rows in either direction
  - no byte-value mismatches

Automation: `tools/check_spec_opcode_sync.py` validates this and exits non-zero on drift.

### 1) Spec conformance notes can drift faster than semantics

Several “conformance gap” notes are about implementation status, not semantics, and should be updated periodically against crates to avoid misleading readers.

Suggested practice: keep the normative semantics stable, but maintain a small, per-runtime “status & deviations” section that is explicitly versioned.

### 2) Remaining VM blockers relative to the spec’s “canonical VM” bar

Even with §17.2 now complete, multiple opcodes/paths required by the spec are still not implemented end-to-end:

- **Tail calls:** `TailCall` is specified and assigned a byte; `ifa-core` now has initial `TailCall` emission + frame-reuse execution, but it still needs conformance coverage beyond “byte emitted”.
- **Closures:** opcode-level closure mechanics now exist in the VM (`MakeClosure` + upvalue load/store), but the source-level compiler/frontend still does not emit these opcodes because lambda/closure syntax + capture analysis are incomplete.
- **Objects:** `GetField` / `SetField` are specified, but object/class property access must be wired into compiler + VM.
- **Imports in VM:** `Import` exists as a bytecode instruction, but `StdRegistry` still needs a concrete module-loader implementation for VM execution.

### 3) Known semantic mismatch hotspots (worth pinning with tests)

These are the kinds of issues that should immediately become conformance tests:

- **Comparisons:** on incompatible types, does VM raise `TypeError` (spec) or return a value (common “dynamic language” behavior)?
- **Logical operators:** are `&&`/`||` purely syntactic sugar over jumps (short-circuit + operand-return), or real bytecode ops returning Bool?
- **Truthiness:** empty collections/strings and `NaN` falsiness rules need to be identical across all runtimes.

## Minimal “keep it honest” checklist

- Run `python tools/check_spec_opcode_sync.py` after any bytecode change.
- For any spec section marked `[DEFINED]`, add at least one executable test (bytecode or source-level) that fails if behavior drifts.
