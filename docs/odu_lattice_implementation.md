# Odù Lattice Implementation (Grounded Plan)

This document describes how to implement the Odù lattice/type foundation using what exists *today* in the repo, and what must be made canonical so different crates stop “inventing” their own encodings.

Scope: **representation + encoding + invariants + tests**. Not a full gradual type system, not inference, not “meaning tables”.

---

## 0) What We Have On The Ground (Current Code)

### A. `ifa-std` already contains an Odù model

File: `crates/ifa-std/src/opele.rs`

- `PrincipalOdu` enum: 16 principals, discriminants `0..15`.
  - It also has a `binary_str()` method returning a 4-bit string per principal (e.g. Ogbe → `"1111"`).
  - **Important:** the enum discriminant is an *ordinal index*, not the Teetee nibble itself.
- `Odu { right: PrincipalOdu, left: PrincipalOdu }`: “256 patterns”.
  - `Odu::from_byte(value)` decodes **low nibble = right**, **high nibble = left**.
  - `Odu::to_byte()` encodes as `(right_ordinal) | (left_ordinal << 4)`.
- `CompoundOdu { ancestors: Vec<PrincipalOdu> }`: variable-depth ancestry chain.
  - The docs call depth=3 “GPC (Grandparent-Parent-Child)” and compute sizes as `16^n` (e.g. 3 → 4096).

### B. Canonical 16 Odù Domains (from Runtime Spec v0.2)

The runtime specification defines the canonical 16 Odù domains with their binary patterns and purposes:

| Binary | Odù (Yoruba) | Domain | Purpose |
|--------|---------------|---------|---------|
| `1111` | Oggbè | System | Process control, environment variables, command-line args |
| `0000` | Òyèkú | Random | Random number generation, entropy, non-determinism |
| `0001` | Ìwòrì | Time | Timestamps, durations, scheduling, calendars |
| `0010` | Òdí | Errors | Error types, exception handling, result types |
| `0011` | Ìròsú | Console | Print, read, colored output, terminal control |
| `0100` | Ọ̀wọ́nrín | Collections | Lists, maps, sets, iterators, comprehensions |
| `0101` | Ọ̀bàrà | Math | Arithmetic, trig, statistics |
| `0110` | Ọ̀kànràn | Assertions / Debug | Assert, debug output, panic |
| `0111` | Ògúndá | Collections / Process | Array ops, map/filter, child processes |
| `1000` | Ọ̀sá | Concurrency | Async tasks, channels, sleep |
| `1001` | Ìká | Strings | String manipulation, regex |
| `1010` | Òtúúrúpọ̀n | Math (inverse) | Subtraction, division, rounding |
| `1011` | Òtúrá | Networking | HTTP, TCP/UDP, sockets |
| `1100` | Ìrẹtẹ̀ | Crypto / Compression | SHA-256, HMAC, base64, zlib |
| `1101` | Ọ̀ṣẹ́ | Graphics / Canvas | Drawing, GUI, canvas |
| `1110` | Òfún | Capabilities / Reflection | Type inspection, permission queries |

**Note:** The runtime specification ratifies the protocol-oriented design where these 16 domains act as protocols rather than class hierarchies.

### B. `ifa-core` uses “GPC” for interpreter scoping, not Odù

Files:
- `crates/ifa-core/src/interpreter/mod.rs`
- `crates/ifa-core/src/interpreter/environment.rs`

Here **GPC = Grandparent→Parent→Child scope chain** for lexical variable resolution, implemented as:

- `Environment { values, consts, parent: Option<Box<Environment>> }`
- Resolution walks `self -> parent -> parent -> ...`.

This is unrelated to Odù lattice “GPC”. The acronym collision is real but harmless unless we start mixing the two concepts in docs/specs.

---

## 1) The Hard Requirement: Pick Canonical Encodings (No More “Whatever Works”)

If the lattice is going to be used for typing, serialization, or inter-crate APIs, we must lock down:

1) **Principal identity encoding**
2) **Odu(256) encoding**
3) **Which encoding is used where** (human names vs machine IDs)

Right now `ifa-std` implicitly uses:

- **Ordinal (0..15)** as the enum discriminant.
- A separate “Teetee nibble string” via `binary_str()`.

That is a workable split, but only if it is made explicit and tested.

### Recommendation (breaking changes allowed)

Adopt two explicit encodings for principals:

- `ordinal`: `0..15` (stable index order, tables, display ordering, spec numbering)
- `teetee_bits`: `0..15` (the 4-bit pattern used for the algebra/XOR group operation)

Then define `Odu256`’s canonical ID in one place:

- `id_u8 = (right_ordinal << 4) | left_ordinal` **OR**
- `id_u8 = (left_ordinal << 4) | right_ordinal`

Pick one, publish it, and enforce it with tests and a spec-check script.

Because your spec draft text elsewhere uses “`id = right * 16 + left`”, the least ambiguous encoding is:

> `id_u8 = (right_ordinal << 4) | left_ordinal`  (right in high nibble)

This is the opposite of what `ifa-std` currently does. If we choose this, we must treat the current `Odu::from_byte/to_byte` as **legacy** and migrate.

---

## 2) Where The Canonical Types Should Live

### Don’t leave lattice core in `ifa-std`

`ifa-std` is a library crate with domain flavor (divination chain, proverbs). The lattice core is a **language-wide primitive** used by:

- parser/type annotations
- babalawo/type checker
- bytecode metadata / diagnostics
- tooling and docs

So the canonical definitions should move to (or be duplicated in) a shared crate:

Preferred home:
- `crates/ifa-types` (since it already houses canonical runtime value types and shared structures)

`ifa-std` should then:
- re-export the canonical types, and/or
- add narrative/divination helpers on top (names, proverbs, chain utilities)

Deliverable: a single source of truth for encoding and conversion logic.

---

## 3) Minimal “Phase 1” Lattice API (Mechanically Useful)

### A. Principal type (`OjuOdu` / `PrincipalOdu`)

Provide total conversions and *zero panics*:

- `fn ordinal(self) -> u8` (0..15)
- `fn from_ordinal(u8) -> Option<Self>`
- `fn teetee_bits(self) -> u8` (0..15)
- `fn from_teetee_bits(u8) -> Option<Self>`
- `fn xor(self, other) -> Self` (implemented over `teetee_bits`)

The `xor()` operation must not `unwrap()` internally; enforce closure with tests (see §5).

### B. Odu(256) type (`Odu256`)

Represent the 256 two-leg sign with explicit semantics:

- `right: Principal`
- `left: Principal`
- `fn id_u8(self) -> u8` (canonical encoding)
- `fn from_id_u8(u8) -> Self`

And *separate* these from any “CompoundOdu” lineage types; don’t pretend those are the same space.

### C. Variable-depth lineage (`CompoundOdu`)

Keep `CompoundOdu` as an independent “ancestry list” type, but make it explicit that:

- it enumerates `16^n` combinations of principals
- it is **not** the same thing as the 256 `Odu256` space

Consider renaming the public docs to avoid “GPC” confusion with interpreter scoping:

- “3-level lineage” instead of “GPC”, or
- “OduLineage” instead of “CompoundOdu” (optional refactor)

---

## 4) Migration Plan (If We Flip `Odu` Byte Encoding)

If we adopt `id = right*16 + left` (right in high nibble), `ifa-std`’s current encoding becomes wrong.

Since breaking changes are acceptable, simplest path:

1) Introduce canonical `Odu256` in `ifa-types`.
2) Update `ifa-std` to use it (or to match its encoding).
3) Update tests in `crates/ifa-std/src/opele.rs` to the new encoding.
4) If any serialized artifacts exist (unlikely), either:
   - drop compatibility, or
   - add `from_legacy_u8()` + a one-time migration script.

The repo already has a culture of “spec enforced by executable checks”; treat Odù encoding the same way.

---

## 5) Executable Invariants (The Only Way This Stays Correct)

Add tests that make it impossible to “interpret bits differently” without failing CI.

### A. Principal mapping is total and stable

- `for ordinal in 0..16`: `from_ordinal(ordinal).unwrap().ordinal() == ordinal`
- `for bits in 0..16`: `from_teetee_bits(bits).is_some()` **only if** all 16 patterns are used; otherwise define explicitly which bits are valid.

### B. XOR closure over principals

Exhaustive closure test:

- For every pair `(a,b)` of principals:
  - `a.xor(b)` returns `Some(principal)` with no panic.
  - Optionally check algebraic laws you actually rely on (commutative, identity element) **only if** they are part of language semantics, not folklore.

### C. Odu(256) encoding is bijective

- `for id in 0..=255`: `Odu256::from_id_u8(id).id_u8() == id`

This must live in the canonical crate, not only in `ifa-std`.

### D. Spec-sync check (optional but consistent with current direction)

If the runtime spec contains an Odù encoding section, add a small check similar to opcode sync:

- parse the spec’s declared encoding rule (“high nibble = right”)
- compare against the canonical implementation constants/tests

---

## 6) How This Hooks Into The Interpreter (Today) Without Overreach

The AST interpreter currently uses `Environment` for lexical scoping (“GPC scope chain”) and has no business depending on Odù lattice details yet.

The lowest-risk integration points:

- If the language gains `as (Ogbe + Odi)` style annotations, the parser/type checker can use `Odu256` / principal ops.
- The interpreter can remain dynamically typed and ignore the lattice initially.
- The VM/bytecode can carry optional Odù metadata later (debug info, checks), but shouldn’t block “make programs run”.

---

## 7) Concrete Implementation Checklist (What To Do Next)

1) Add canonical lattice core to `crates/ifa-types`:
   - `Principal` (with ordinal + teetee bits)
   - `Odu256` (two legs, canonical `u8` encoding)
2) Refactor `crates/ifa-std/src/opele.rs` to:
   - reuse canonical types, or
   - match canonical encoding if it keeps its own copy
3) Update `Odu::from_byte/to_byte` tests to reflect the canonical encoding.
4) Add exhaustive invariants tests (16×16 XOR + 0..255 roundtrip).
5) Only after the above is locked: discuss deeper lattices (65,536, etc.) with a real compiler feature that needs them.

---

## 8) Non-Goals (For Now)

- No metaphysical meaning tables in core code.
- No “65,536 child lattice” until a real compiler/typechecker needs it.
- No runtime behavior changes based on Odù until semantics are test-locked.

