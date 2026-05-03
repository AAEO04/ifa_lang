# Ifá-Lang Hardening Patch: Strategic Blueprint

This manifest documents the technical debt and architectural flaws in Ifá-Lang v0.2.

---

## 1. Hot-Path Optimization: Arithmetic vs. Strings
**Status**: FIXED
**Fix**: OpCode::Add is now PURE NUMERIC (Int/Float only). String concatenation
uses OpCode::Concat (0x27), which is strict Str + Str only. The compiler emits
Concat for string literals in += paths. The VM Add handler returns a typed error
directing users to += for strings.

---

## 2. Operator Deprecation: ++ and --
**Status**: FIXED  
**Fix**: Removed inc_dec_op from grammar.pest. Removed Increment/Decrement
from UpdateOp in ast.rs. Purged all handling from parser.rs, compiler.rs, and
interpreter/core.rs. The sole mutation paths are now += / -= / *= / /=.
apply_update_add() handles dual int/string += semantics at the interpreter level.

---

## 3. Babalawo: Ghost Verification Engine
**Status**: FIXED
**Checks implemented**:
  - AWAIT_OUTSIDE_ASYNC: reti (await) now errors when used outside a daro function.
  - NON_ITERABLE: For loop warns statically if iterable type is not List/Map/Str/Array.
  - Params warning resolved: function params are now registered as used.
  - in_async_function flag tracks context across nested function definitions.
  - Expression::Get, Call, UnaryOp, InterpolatedString now fully traversed.

---

## 4. The Shield of Okànràn: FFI Security Layer
**Status**: PENDING
**Issue**: Polyglot bridges (Python/JS) lack capability-matrix hooks and secure
library verification.
**Remediation**: Eliminate set_var (BUG-018), implement Guest-Level Audit Hooks
(BUG-019), and Secure Library Verification (BUG-021).

---

## 5. CLI Subcommand Restriction
**Status**: FIXED
**Issue**: ifa <file> is broken; mandatory subcommands required.
**Remediation**: Re-implemented positional argument support by intercepting `std::env::args()` and dynamically injecting the `run` default subcommand.

---

## 6. LSP Type and Ownership Debt
**Status**: FIXED
**Fix**: lsp.rs uses CodeActionRequest and clones new_name to avoid illegal moves.

---

**Current Project Status**: RECOVERY IN PROGRESS (5 of 6 milestones complete)
**Linus Verdict**: The compiler is finally telling the truth about what it checks.
Two fewer lies. One to go.
