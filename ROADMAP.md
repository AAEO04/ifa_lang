# IfáLang Roadmap (Completeness)

This roadmap exists to prevent a common confusion:

- Making the specification internally consistent makes the **spec correct**.
- A “complete language” requires **spec completeness + implementation completeness + ecosystem completeness**.

For the normative definition of behavior, see `IFA_LANG_RUNTIME_SPEC.md`. For current runtime support, use the Capability Matrix (§20) and conformance gates (§21).

## What “Complete Language” Means Here

IfáLang is “complete” (in the practical, shippable sense) when a developer can:

1. Write a real multi-file program end-to-end in IfáLang,
2. Run it with predictable semantics (no silent wrong behavior),
3. Ship an artifact (bytecode/native/wasm as applicable),
4. Using the IfáLang toolchain without “not implemented” workarounds.

That requires three axes to all be in a good state:

- **Specification completeness**: required features have no `[OPEN]` gaps.
- **Implementation completeness**: at least one runtime passes the conformance suite for the claimed tier.
- **Ecosystem completeness**: docs, distribution, editor support, libraries, and release process are real and maintained.

## Sequenced Work (Dependency-Ordered)

### Phase 0 — Spec correctness (ongoing prerequisite)

- Eliminate internal inconsistencies and ambiguous semantics.
- Convert `[OPEN]` items into ratified decisions (or explicitly scope them out of the current language version).
- Keep the Capability Matrix (§20) aligned with reality.

Exit criteria:
- No known contradictions in core semantics; every deviation is documented with a conformance note and tests.

### Phase 1 — VM viability (first “real programs” milestone)

Goal: make `ifa runb` a dependable execution runtime that can plausibly become the semantic oracle.

Exit criteria (high level; details in §21):
- Tier 1 conformance suite exists and is automated.
- `ifa runb` passes Tier 1 conformance (or a clearly defined “core subset” with a published list of exclusions).
- The VM canonicalization gate in `IFA_LANG_RUNTIME_SPEC.md` §21.2 is satisfied.

### Phase 2 — Language surface completeness (core ergonomics)

Goal: cover the surface area needed for everyday programming without “paper features”.

Work themes:
- Functions/closures/lambdas working across runtimes.
- A real module resolver and multi-file execution story.
- Error handling semantics implemented consistently (`gbiyanju`/`gba`/`ta`, propagation, and `nipari`/finally where specified).
- Correct control-flow semantics (truthiness, short-circuiting, matches, etc.).

#### Protocol-Oriented Design (RATIFIED 2026-04-07)

Class-based OOP is formally removed from Ifá-Lang. The data model is:

- **Data** is a `Map` literal.
- **Behaviour** is an `ese` function that takes a Map.
- **Polymorphism** is validated by `ifa-babalawo` structural shape-checking.
- **Encapsulation** is enforced by `#opon` sandbox boundaries.

The compiler now emits a hard `IfaError` on any `class` syntax with a migration guide. The VM rejects stale `DefineClass` opcodes loudly. See `IFA_LANG_RUNTIME_SPEC.md §10` for the full specification.

Exit criteria:
- Multi-file programs run consistently across `ifa run` and `ifa runb` for the supported subset.
- No `class` syntax in any `.ifa` file in the conformance suite.


### Phase 3 — Toolchain completeness (developer experience)

Goal: make the tooling trustworthy and boring.

Work themes:
- `ifa fmt` correctness + idempotence backed by tests.
- `ifa lsp`/static analysis integration (Babalawo) aligned with the spec.
- Debugging workflow (DAP) backed by an implementation, not just a reference.
- REPL that reflects real runtime semantics (not a toy interpreter).

Exit criteria:
- Tooling commands have stable behavior and test coverage for the supported language subset.

### Phase 4 — Ecosystem foundation (sharing and adoption)

Goal: make it possible to build and share useful programs and libraries.

Work themes:
- A real package workflow (`oja`) with well-defined version solving semantics.
- Standard packages beyond the core stdlib (e.g., json/http/cli/crypto), with compatibility guarantees.
- Documentation site that matches the implementation reality (no “coming soon” surprises).
- Clear contribution guide, release cadence, and a published compatibility policy.

Exit criteria:
- A developer can start a project, add deps, build, and ship with repeatable results.

### Phase 5 — Multi-Paradigm Zero-Cost Integration (Long-term design)

Ifá-Lang will not adopt "architectural astronaut" features at the cost of VM performance or binary bloat. Any integration of advanced paradigms MUST adhere to the **Zero-Cost Abstraction Rule**: Hand off 100% of the cognitive overhead to `ifa-babalawo` (Static Analysis) and `ifa-core` (Parsing/Compilation), keeping `ifa runb` (VM) fundamentally "stupid", fast, and stripped of runtime reflection.

The approved integration paths for future paradigms:

#### 1. Generics (`Orí` / `Àlùpúpù`) via Compile-Time Monomorphization
- **Rule:** No Type Erasure or runtime generic dispatch.
- **Mechanism:** `ifa-babalawo` statically verifies bounds. `ifa build` monomorphizes implementations (e.g. producing separate concrete binary functions for `List_Int` and `List_Str` constrained within identical `#opon` GC shapes).

#### 2. Actor Concurrency (`Ẹgbẹ́`) via Affine Ownership
- **Rule:** No deep-copying bottlenecks or Global Interpreter Locks (GIL).
- **Mechanism:** Implement Rust-style Move Semantics. When an actor sends an `Ẹbọ` (payload) via `Osa.ebo()`, `ifa-babalawo` statically proves the sender lost ownership. The VM passes a raw memory pointer (Zero-Copy) to the receiving actor without mutex locking.

#### 3. Metaprogramming / Macros (`Awo`) via Procedural Flattening
- **Rule:** No Lisp-style runtime `eval()` that destroys predictability.
- **Mechanism:** Macros execute strictly during the `ifa-core/parser.rs` transpilation phase. They generate primitive, flattened imperative AST nodes. The bytecode compiler emits raw instructions; the VM executes without observing the macro.

#### 4. Declarative Logic (`Dáfá`) via State-Machine Lowering
- **Rule:** No magical Prolog-unification engines embedded in the VM.
- **Mechanism:** Introduced via standard `comptime` (compile-time) evaluations where declarative rules are flattened into highly optimized unconditional jump tables (raw switch statements).

#### 5. Flat-Buffer Embedded Runtime Direction (`ifa-embedded`, `ifa-sandbox`)

This is the only approved way to borrow anything useful from esoteric-language discussions such as Brainfuck or MOO: strip away the branding, keep the memory and dispatch lesson, and implement it as ordinary systems engineering.

- **Rule:** Do not introduce a "Brainfuck VM", "MOO architecture", or other novelty runtime inside IfÃ¡-Lang.
- **Mechanism:** For `ifa-embedded` and `ifa-sandbox`, favor pre-allocated contiguous buffers, integer-indexed state, bounds-checked offsets, and tight jump-table dispatch loops over heap-scattered runtime cells and allocation-heavy object graphs.
- **Babalawo Constraint:** Preserve semantic meaning at the analysis layer. Do not lower the whole language into tape-motion primitives that destroy type/effect/lifecycle information.

Reference note (kept here because it correctly frames the systems-level lesson):

```text
From: Linus Torvalds <torvalds@linux-foundation.org>
To: Ifá-Lang Runtime Integration Team
Subject: Re: Adding Brainfuck and Moo (COW) concepts to ifa-embedded

Look, I've read this proposal and I'm honestly trying to figure out if this is a joke from somebody who spent too much time reading esoteric programming wikis instead of actually looking at what a CPU does.

Let's get one thing straight immediately: "Moo" and "Brainfuck" are not architectural foundations. They are toys. They are academic circle-jerks designed to prove Turing completeness in the absolute most painful way possible. If I see anyone formally referring to a "Moo-inspired architecture" in a commit message, I will revert the patch and revoke your push access.

Now, if we scrape off the layer of pretentious esoteric bullshit you've layered on top of this, what are you actually proposing?

You're proposing a pre-allocated, flat, linear array of bytes with a single integer index pointer moving across it, rather than passing around `Rc<RefCell<IfaValue>>` boxed garbage scattered randomly across the heap.

Congra-fucking-lations. You've discovered contiguous memory buffers. C has had this since 1972. My grandmother knows about contiguous memory buffers.

Is it a good idea for `ifa-embedded` and `ifa-sandbox`? **Yes. Absolutely yes.** But let's call it what it is: a goddamn flat buffer.

Here is why it actually works, and it has nothing to do with esoteric minimalism:

1. **Cache Locality is King:** Your `Opon` heap logic with `RefCell` is fine for user-space REPLs on a modern desktop chip that can hide your pointer-chasing sins behind massive L3 caches. On an ESP32 or an STM32? Chasing a pointer to resolve an `IfaValue` type tag means you miss the cache, stall the pipeline, and sit there twiddling your thumbs for 100 cycles while memory fetches. A linear tape means the hardware prefetcher actually works.

2. **The Micro-VM Dispatch:** Stripping your opcode set down to integers and bounds-checked array offsets is exactly what you should be doing for raw I/O (like your SPI/I2C streams). Don't you dare build a "Brainfuck interpreter" to do this. Just build a tight, integer-based jump table `switch` statement loop. Keep the bytecode simple, keep the state within a single cache line, and the CPU will eat it alive.

3. **Sandboxing:** Yes, a purely isolated array where the VM cannot physically address anything outside the `[u8; N]` tape is secure. Why? Because you statically defined the boundaries. It's not a "tarpit sandbox", it's basic bounds checking. Don't overcomplicate it.

4. **Formal Verification (Babalawo):** This is where you people lose your minds. Do not try to compile your entire language down into a Brainfuck IR just to run a math proof on it. The state explosion will be massive. Keep the IR simple, sure, but if you reduce a high-level `Ebo` lifecycle to primitive tape shifts, you'll lose the semantic meaning of the code, and your static analyzer will die heat-death trying to trace it back.

Stop trying to make architecture sound mystical. It's just memory management. Use the flat buffer. Build the tight jump tables. Get rid of the allocations. And please, for the love of god, stop talking about cows.

Linus
```

## Version Wrap-Up (Deferred Beyond This Release)

This version is being wrapped with a bias toward shipping the current core cleanly. The following ideas are explicitly deferred and are not release blockers for the current line:

- Domain-bounded isolation as a full interprocedural proof system.
- Character-lattice proofs / 256-state symbolic verification.
- Extended `ÃŒwÃ ` integrity metrics beyond the current lifecycle and borrow tracking model.
- Opon-aware dual-mode sequential/parallel scheduling in the VM.
- `Ã€á¹£áº¹` sovereignty as a formal compiler conflict-resolution system.
- Hierarchical verification beyond the current modular lint/static-analysis passes.

Scope note:
- Some foundations for these exist already in `ifa-babalawo`, `ifa-core`, and `#opon`, but they remain future design work rather than committed deliverables for this version.
- The only acceptable path for any of these features is the same zero-cost rule stated above: push complexity into compile-time analysis and lowering, not VM reflection or runtime dynamism.

Release posture for the current version:
- Prioritize conformance, runtime correctness, docs/spec alignment, and tooling stability.
- Treat advanced proof systems and scheduler redesign as post-release research and architecture work.
