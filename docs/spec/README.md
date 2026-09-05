# Ifá-Lang Specification Index

**Last updated:** 2026-05-29

This directory contains the formal specifications and design documents for the Ifá-Lang runtime, memory model, and concurrency architecture.

## Status Legend

| Status | Meaning |
|--------|---------|
| `IMPLEMENTED` | Spec describes code that exists and passes tests today |
| `APPROVED` | Spec is approved for implementation; work may be in progress |
| `DRAFT` | Proposal under review; not yet approved for implementation |
| `REFERENCE` | Analysis or audit document; not a buildable spec |

---

## Specifications (Buildable)

| # | Document | Status | Summary |
|---|----------|--------|---------|
| 1 | [opon-ebo-actor-taboo-spec.md](opon-ebo-actor-taboo-spec.md) | `IMPLEMENTED` | Formal operational semantics for Opon memory, Ebo RAII, Actor isolation, and Taboo enforcement. The definitive runtime spec. |
| 2 | [move_semantics.md](move_semantics.md) | `IMPLEMENTED` | The `yanda`/`move` keyword for zero-copy ownership transfer at actor boundaries. Enforced by `Babalawo` `MoveTracker`. |
| 3 | [effect_system.md](effect_system.md) | `IMPLEMENTED` | The `effects(...)` annotation system for side-effect boundaries. Enforced by `Babalawo` `EffectChecker`. |
| 4 | [iwori_arc_rc_duality.md](iwori_arc_rc_duality.md) | `DRAFT` | Compile-time `Arc`/`Rc` toggle via `feature = "parallel"` for zero-cost single-threaded mode. |
| 5 | [ofun_sandboxed_macros.md](ofun_sandboxed_macros.md) | `DRAFT` | Capability-gated metaprogramming via `Ofun` domain. Compile-time AST manipulation sandboxed by `CapabilitySet`. |
| 6 | [osa_mn_fiber_scheduler.md](osa_mn_fiber_scheduler.md) | `DRAFT` | M:N stackful fiber scheduler to replace 1:1 OS-thread-per-actor model. |
| 7 | [opon_slab_allocator.md](opon_slab_allocator.md) | `DRAFT` | Phase 3 slab allocator replacing `Arc` with `u32` indices for true zero-copy memory transfer. |

## Migration Roadmap

The memory and concurrency architecture evolves through four phases. Each phase is independently shippable. Later phases supersede earlier mechanisms.

```
Phase 1 (IMPLEMENTED) — freeze/thaw deep copy
    Current actor_send uses value.clone() (Arc pointer sharing).
    Babalawo MoveTracker enforces use-after-move at compile time.
    Source: crates/ifa-vm/src/actor.rs:275

Phase 1.5 (IMPLEMENTED) — yanda logical move with MoveLocal
    Babalawo statically guarantees the sender loses access after yanda.
    The MoveLocal opcode (0x1F) replaces the source slot with Null
    at runtime for identifier moves. Arc pointer clone for heap types.
    Source: crates/ifa-babalawo/src/movement.rs, crates/ifa-bytecode/src/lib.rs,
            crates/ifa-compiler/src/lib.rs, crates/ifa-vm/src/vm.rs

Phase 2 (DRAFT) — Ìwòrì Arc/Rc toggle
    Compile-time feature flag switches IfaValue heap wrappers
    from Arc to Rc for single-threaded deployments.
    Eliminates atomic overhead entirely.
    Prerequisite: Osa domain must be disabled when parallel=false.
    Spec: iwori_arc_rc_duality.md

Phase 3 (DRAFT) — Opon Slab Allocator
    Tear out Arc/Rc entirely. IfaValue becomes a u32 index
    into a per-actor slab. Cross-actor transfer becomes a
    global pool index handoff. Requires per-actor GC.
    Supersedes Phase 2 (no Arc or Rc to toggle).
    Spec: opon_slab_allocator.md
```

> [!IMPORTANT]
> Phase 2 and Phase 3 are **mutually exclusive long-term targets**. Phase 2 is a pragmatic
> short-term win. Phase 3 is the architectural endgame. If Phase 3 is pursued, Phase 2 becomes
> unnecessary. The team must decide which path to commit to before implementation begins.

## Reference Documents (Non-Buildable)

| Document | Summary |
|----------|---------|
| [unique-memory-safety.md](unique-memory-safety.md) | Source-verified summary of Ifa-Lang's layered memory-safety techniques and roadmap |
| [cheri-integration.md](cheri-integration.md) | Analysis of CHERI as a hardware hardening target for Ifa-Lang's VM, Opon, and future `iso`/`yanda` transfer model |
| [crates-engineering-review.md](crates-engineering-review.md) | Engineering review of the `crates/` tree through systems, compiler, VM, algorithm, and invariants lenses |
| [memory-concurrency-analysis.md](memory-concurrency-analysis.md) | Audit of current memory and concurrency architecture |
| [engineering-analysis.md](engineering-analysis.md) | Engineering evaluation of the codebase |
| [formal-foundations.md](formal-foundations.md) | Formal foundations and type theory analysis |
| [mathematical-foundations.md](mathematical-foundations.md) | Mathematical properties of the Odù domain system |
| [pl-design-analysis.md](pl-design-analysis.md) | Programming language design review (part 1) |
| [pl-design-analysis-2.md](pl-design-analysis-2.md) | Programming language design review (part 2) |
| [pl-design-analysis-3.md](pl-design-analysis-3.md) | Programming language design review (part 3) |
