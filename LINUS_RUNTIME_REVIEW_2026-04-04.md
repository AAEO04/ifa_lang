# Ifa-Lang Runtime Review

Date: 2026-04-04
Scope: `ifa-core`, `ifa-types`, runtime/spec alignment
Status: updated after follow-up fixes

## Fixed In This Pass

### 1. AST `try/catch/finally` cleanup now runs
Files:
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\interpreter\core.rs`

`nipari` is no longer parsed and ignored. The AST interpreter now executes `finally_body`, and there is a regression test for return-through-finally.

### 2. Transpiler file I/O no longer lies with fake success
Files:
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\transpiler\domains.rs`
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\transpiler\mod.rs`

`odi.read` and `odi.write` now surface structured error maps instead of degrading to `Nil` or returning `Bool(true)` after ignored write failures.

### 3. Unlimited Opon growth is bounded
Files:
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\opon.rs`

`Ailopin` is no longer “resize until the process dies”. It now stops at a hard host-safety ceiling and returns a limit error.

### 4. Unsupported value serialization now fails explicitly
Files:
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-types\src\value_union.rs`

Unsupported runtime values no longer silently round-trip as `Null`. Serialization now fails loudly, which is what a runtime should do.

## Findings Still Worth Fixing

### 1. The value layer still has two division semantics
File: `C:\Users\allio\Desktop\ifa_lang\crates\ifa-types\src\value.rs:254`
File: `C:\Users\allio\Desktop\ifa_lang\crates\ifa-types\src\value.rs:290`

`Div` still returns `Null` on divide-by-zero while `checked_div` returns a real error. That is still incoherent.

Verdict: this should still be removed.

### 2. Runtime side effects are still scattered through semantic code
Files:
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\interpreter\core.rs:892`
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\interpreter\core.rs:929`
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\interpreter\core.rs:941`
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\vm.rs:1893`
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\vm.rs:2102`

There is still direct printing and blocking sleep behavior in runtime paths. It is better than before, but still too coupled to stdout and host timing.

Verdict: side effects need to be routed through runtime interfaces, not sprinkled through execution code.

### 3. The VM and interpreter files are still too monolithic
Files:
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\vm.rs`
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\interpreter\core.rs`

The code is more correct now, but still not especially clean. Too much unrelated policy lives in single files.

Verdict: correctness improved faster than structure. Refactoring debt remains.

### 4. Warnings still point to stale code paths
Files:
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\vm.rs`
- `C:\Users\allio\Desktop\ifa_lang\crates\ifa-core\src\transpiler\core.rs`

`call_value`, `read_i16`, and `base_dir` are still producing dead-code noise. That is not severe, but it is still slop.

Verdict: remove or use them.

## Test Coverage

Good now:
- AST conformance passes
- VM conformance passes
- closure opcode regression passes
- AST finally behavior has a dedicated regression test
- transpiler file-I/O error surfacing has a regression test
- Opon hard ceiling has a regression test

Still missing:
- tests that pin stdout/stderr side effects behind interfaces
- direct coverage for the value-layer `Div` semantics mismatch

## Short Version

The runtime is in materially better shape.

The worst semantic lies from the earlier review are fixed.

What remains is mostly cleanup and hardening:
- remove `Null` division semantics
- stop printing and sleeping directly from runtime code
- break up the monolithic execution files
