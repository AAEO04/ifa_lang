Here's the corrected gap analysis after verifying every claim against actual source code. I've marked which claims were CONFIRMED vs REFUTED by the codebase.
Critical — CONFIRMED
Claim	Verdict	Evidence
No static type inference/checking	CONFIRMED	Variables are purely dynamic at runtime
Half of Odù domains feature-gated	CONFIRMED	4 of 16 gated: Osa (async_runtime), Otura (full), Irete (crypto), Ose (tui)
6 domains have stub methods	CONFIRMED	Okanran: "not yet implemented"; Irete: "zstd feature missing"; Ika: "DOM parse handles are not implemented"; CPU/Osa/Otura: only subsets of metadata methods implemented
Critical — REFUTED (my original claims were wrong)
My Claim	Actual	Evidence
Spawn/Send/Recv opcodes exist but undispatchable	Opcodes don't exist	No Spawn, Send, or Recv in OpCode enum. Spawning is via Osa.ise native domain call using tokio + std::sync::mpsc
Yield is a no-op	Returns Err(Yielded)	OpCode::Yield => return Err(IfaError::Yielded) — aborts execution, must be caught by caller
M:N fibers unimplemented	Refuted — spec only exists as DRAFT; actual impl uses 1:1 OS threads via tokio::spawn_blocking	No M:N scheduler anywhere in source
6+ dead opcodes in bytecode	Only Yield exists	Assert, Debug, TypeOf, Spawn, Send, Recv are not in the OpCode enum at all. They don't exist anywhere.
Set not GC-traced	CONFIRMED but re-scoped	Uses Arc<HashSet<IfaValue>> — correct, it's not GC-traced
execute() is 2,644 lines	REFUTED — execute() is 26 lines (thin wrapper); resume_execution() is 87 lines (dispatch loop). The large function is step() at ~1,413 lines.	Lines 548-573 and 944-1030 in vm.rs
Major — CONFIRMED
Claim	Evidence
No borrow checker (OponView)	Entire subsystem designed but zero implementation
No cross-platform targets	WASM path explicitly prints "not yet implemented"
No incremental compilation	Every run recompiles from scratch
Taboo/Abo compile to zero bytes	Empty {} arm in compiler for Abo; let _ = (source, target) for Taboo
Result<_, String> in 5+ files	embedded.rs, gpu.rs, cpu.rs, irosu.rs — stringly-typed errors
No #[non_exhaustive] on any public enum	Zero occurrences across entire crates/
Only i64 and f64 numeric types	IfaValue has no other numeric variants
No Tuple/Struct variants	No variants beyond List, Map, Set
panic!() in library code	18 locations across vm.rs, value.rs, bytecode.rs, odi.rs, okanran.rs, ika.rs, oracle.rs, actor.rs
Major — REFUTED (my claims were wrong)
My Claim	Actual	Evidence
No compiler optimization	Constant folding exists	fold_expression() called inline during compilation; handles arithmetic, comparison, logical, unary folding, plus "Constant Divination" (E6) for Odù calls
Opon allocator not wired	Is wired	Used in epoch begin/end, load/store ops, MMIO recording
--unstable required for fmt	Flag is ignored	unstable: _ — discard pattern. Formatter runs unconditionally
Git deps not implemented	Implemented	oja.rs:739-757 does actual git clone
19 .expect() in vm.rs	1 in production (line 2540), 15 in test code	 
Grammar missing set/pipeline/safe-nav	All exist	set_literal rule, pipeline_expr with |>, optional_chain_op with ?.
execute() single 2,644-line function	step() is 1,413 lines — still large but not the doomsday figure I quoted	 
Missing syntax — CORRECTED TABLE
Construct	Actually missing?
spawn keyword	✅ Missing
with expression	✅ Missing
when guard clause	✅ Missing
triple-quote strings	✅ Missing
assert keyword	❌ Exists (aliases to ewo)
Set literals {1,2,3}	❌ Exists (Set { ... } syntax)
Pipeline |>	❌ Exists
Safe nav ?.	❌ Exists
& reference operator	❌ Exists
.. range	❌ Exists (in match patterns)
Anonymous closures	❌ Exists (ese/fn lambda syntax)
Summary: what's actually missing as a language
The language has working: parser → compiler → VM pipeline for ~10 domains, constant folding, set/pipeline/range/lambda syntax, git clone dependencies, and a 93-opcode bytecode format.
It's actually missing:
1. Concurrency at runtime — Yield kills execution, Spawn/Send/Recv have no bytecode. Actors exist only via Osa native calls (OS threads). M:N fibers are design-only.
2. Type system — No static checking, no borrow checker, no inference, no #[non_exhaustive], no structured types (tuples, records), only i64/f64.
3. Platform targets — WASM and embedded are stubs. Host-only.
4. Tooling — LSP, debugger, package registry all absent. Formatter is unconditional regardless of --unstable.
5. Half the standard library is behind feature flags or returns "not implemented" — Osa, Otura, Irete, Ose require gating; CPU, hardware domains are stub-thin; Okanran mocking and Ika HTML are stubs.
6. Compilation model — No incremental compilation, no build cache. step() at 1,413 lines is the biggest single maintenance burden.