# Rejected Features — Noise Items

Features evaluated and rejected. Included here so they don't get re-proposed without precedent.

---

## 1. 16-bit Domain Mask Type

**Proposal:** A `u16` bitmask type where each bit corresponds to one Odù domain.

**Rejected because:**
- Odù domains are not flags — they're namespaced capability scopes with methods, not bit positions
- The 16-to-256 expansion (Odu combinations) is a lattice, not a bitfield
- Real use cases (capability sets) already use `CapabilitySet` with named members, not bitmasks
- A bitmask type would imply `AND`/`OR`/`XOR` between domains, which is semantically meaningless

---

## 2. `deploy tighten`

**Proposal:** A CLI flag/command to restrict what a deployment can do.

**Rejected because:**
- This duplicates the existing capability-security model (`Ofun` permissions, `CapabilitySet`)
- The sandbox (`ifa-sandbox`) already handles capability restriction via the `Ofun` enum at the Rust API level
- A CLI flag adds a second capability model that would need to stay in sync with the first
- Users already configure capabilities in their `oja.json` or programmatic API; another flag is cognitive overhead

---

## 3. `#lookback` Directive

**Proposal:** A source-level annotation `#lookback(N)` that shows N previous lines in error messages.

**Rejected because:**
- This is a presentation concern, not a language directive
- Error formatting (including context lines) belongs in the tooling (Babalawo, LSP, CLI flags)
- `ifa check --context=3` or an LSP setting is the right approach
- Baking it into the language grammar couples error presentation to source code

---

## 4. `#verify` Headers

**Proposal:** Source-level `#verify: <predicate>` annotations that run at parse time.

**Rejected because:**
- Overlaps with `ewo` (assertion) statements, which already exist in the AST
- `ewo` is a statement that can appear anywhere in a block; `#verify` would be a directive competing for the same semantic space
- Using `ewo` at module level gives the same effect without a new syntax
- `#` prefixes are used for comments (`#` line comments) — mixing directives into comments is confusing

---

## 5. OpeleChain Syntax

**Proposal:** A chaining syntax that mirrors the Opele (divination chain) for method sequences.

**Rejected because:**
- The existing `.` (dot) and `?.` (optional chain) operators already cover method chaining
- A new chaining operator adds syntax surface without semantic benefit
- Ifá metaphors are meaningful when they map to a computational concept; this one is decorative
- Adds parser complexity for no new expressiveness

---

## 6. Backtick Lifecycle Annotations

**Proposal:** Using backticks to annotate resource lifecycles: `` let `file` = open("x"); ``

**Rejected because:**
- Backticks in most languages denote raw identifiers or string interpolation — reusing them for lifecycle is confusing
- Resource lifecycle is better expressed through existing mechanisms:
  - `defer { close(file); }` (now implemented)
  - `ebo "scope" { ... }` epoch regions (now implemented)
  - Type-system RAII (future: OponView borrow checker)
- Annotating every resource with a backtick adds noise without conveying what the lifecycle *is*
