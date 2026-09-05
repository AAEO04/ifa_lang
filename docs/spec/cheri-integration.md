# CHERI Integration Analysis for Ifa-Lang

**Status:** `REFERENCE`  
**Scope:** hardware-target hardening analysis  
**Primary code:** `ifa-infra`, `ifa-types::gc`, `ifa-vm`, `ifa-transpiler`

CHERI is not an Ifa-Lang memory-management strategy by itself. It is a hardware capability architecture that can harden Ifa-Lang runtimes and transpiled binaries when they are compiled for a CHERI-capable target.

This document describes where CHERI fits, where it does not fit, and how it could compose with Ifa-Lang's Opon, `IfaGc`, `yanda`, and future `iso` memory-safety work.

## CHERI In One Sentence

CHERI turns pointers into hardware capabilities. A capability carries address authority, bounds, permissions, and a validity tag. Loads and stores through capabilities are checked by hardware.

CHERI is strongest for spatial safety: out-of-bounds and unauthorized pointer access can trap in hardware. Temporal safety requires an additional revocation strategy such as CHERIvoke, Cornucopia, CHERIoT-style revocation, or another allocator/runtime discipline.

## Current ifa-infra Position

`ifa-infra` is mostly a safe Rust infrastructure layer. It exposes CPU, GPU, storage, kernel, shader, compute, and runtime abstractions. The crate does not currently expose raw MMIO, raw pointer, or FFI APIs as its public memory model.

| Module | Current access model | CHERI relevance |
|--------|----------------------|-----------------|
| `compute.rs` | trait dispatch | no direct interaction |
| `cpu.rs` | Rayon-backed safe Rust compute | CHERI hardens ordinary heap pointers underneath |
| `gpu.rs` | `wgpu` handles | GPU memory is outside CHERI's CPU capability checks |
| `storage.rs` | Tokio FS plus serialization | file I/O mostly unaffected |
| `kernel.rs` | system inspection | possible CHERI detection point |
| `shaders.rs` | WGSL source strings | no direct pointer interaction |
| `runtime.rs` | Tokio runtime helpers | no direct pointer interaction |

The main caveat is `gpu.rs`, which uses `unsafe impl Send` and `unsafe impl Sync` for `GpuContext`. CHERI cannot validate those concurrency invariants; it only checks capability use where CPU memory is dereferenced.

## Where CHERI Actually Matters

### 1. `IfaGc` Internal Pointers

The most relevant current code is `IfaGc<T>` in `crates/ifa-types/src/gc.rs`. It stores `NonNull<CycleNode<T>>` and manually controls allocation, tracing, dropping, and deallocation.

On a CHERI-aware Rust target, those internal pointers would be represented as capabilities. That can harden:

- bounds on `CycleNode<T>` access;
- header and payload dereferences;
- stale pointer use when paired with revocation;
- accidental pointer arithmetic or invalid casts in unsafe code paths.

CHERI does not remove the need for `IfaGc`. It does not decide when a cycle is unreachable, and it does not reclaim memory. It only makes invalid capability use fail.

### 2. Opon Backing Storage

`Opon` is backed by `Vec<IfaValue>`. On CHERI, the `Vec` data pointer becomes a bounded capability under a CHERI-aware Rust compiler.

This gives defense-in-depth for the backing allocation. It does not replace Opon's own slot checks, epoch truncation, or future generational handle checks.

The best composition is:

| Opon feature | Software role | CHERI role |
|--------------|---------------|------------|
| slot bounds | reject invalid slot indexes | harden backing memory access |
| ebo epochs | scoped truncation | no direct lifetime proof |
| generational handles | detect stale slot handles | trap invalid dereference if stale capability is revoked |
| `#opon` size | configure memory budget | no direct policy role |

### 3. Actor Transfer and `yanda`

Current actor messages are serialized before crossing actor boundaries. CHERI does not make serialization faster or more memory-safe in a meaningful way.

CHERI becomes interesting if Ifa-Lang later implements zero-copy transfer for `iso` object graphs:

```ifa
ayanmo packet = iso {
    "samples": [1, 2, 3],
    "tag": "frame"
};

Osa.ran(worker, yanda packet);
```

In that model:

- Babalawo proves the graph is isolated and uniquely owned.
- `yanda` consumes the sender's authority.
- The runtime derives a bounded capability for the receiver.
- CHERI enforces bounds, permissions, and monotonic capability derivation in hardware.

This is the strongest Ifa-Lang/CHERI design point: source-level ownership plus hardware-enforced authority.

### 4. Transpiled Rust Backend

`ifa-transpiler` generates Rust. If that Rust is compiled for a CHERI target, ordinary Rust pointers in the generated binary can become CHERI capabilities.

This gives the transpiled backend hardware hardening without changing Ifa source syntax. It does not automatically make every unsafe construct semantically safe. The compiler and runtime still need to preserve:

- aliasing rules;
- initialization rules;
- lifetime rules;
- destructor correctness;
- concurrency invariants;
- resource ownership.

## What CHERI Does Not Solve

CHERI should not be documented as a replacement for Ifa-Lang's memory model.

| Problem | Does CHERI solve it alone? | Reason |
|---------|----------------------------|--------|
| Buffer overflow | mostly yes | bounded capabilities can trap out-of-bounds access |
| Pointer permission violation | mostly yes | capabilities carry permissions |
| Use-after-free | only with revocation | baseline capabilities can remain valid after free/reuse |
| Reference cycles | no | reachability and reclamation are runtime/compiler problems |
| Data races | no | CHERI is not a concurrency type system |
| Double drop/destructor bugs | no | Rust/VM logic must still be correct |
| Actor ownership transfer | no | Babalawo/runtime must enforce ownership |
| Resource capability policy | no | Ifa capability sets and registries still define authority |

## Proposed Integration Layers

### Layer 1: Target Detection

Add a small detection API in `ifa-infra::kernel`.

Expected shape:

```rust
pub enum CapabilityArchitecture {
    None,
    Cheri,
    Cheriot,
    Morello,
    CheriRiscV,
}

pub fn capability_architecture() -> CapabilityArchitecture;
pub fn has_cheri() -> bool;
```

This should be diagnostic, not a security boundary. Runtime detection is useful for reporting and tests, while compile-time `cfg` remains the authoritative build mechanism.

### Layer 2: Runtime Audit

Audit code that stores, compares, hashes, or casts pointers:

- `ifa-types/src/gc.rs`: `NonNull`, `Box::from_raw`, pointer equality, type-erased trace callbacks.
- `ifa-vm/src/vm_ikin.rs`: pointer-to-`usize` identity maps for string storage.
- `ifa-types/src/value_union.rs`: pointer hashing for heap-backed variants.
- `ifa-vm/src/actor.rs`: serialization boundaries and type-erased actor handles.

On CHERI, pointer-to-integer casts may lose capability metadata. Avoid treating integerized pointers as reusable authority.

### Layer 3: Optional CHERI Module

If Ifa-Lang exposes CHERI operations, they should be feature-gated and capability-gated. They belong in a low-level infrastructure module, not in ordinary safe user code.

Expected file layout:

```text
crates/ifa-infra/src/cheri.rs
crates/ifa-infra/src/lib.rs
crates/ifa-infra/Cargo.toml
```

Possible Rust API:

```rust
pub struct Capability {
    // Opaque. Do not expose integer pointer authority.
}

pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub store_capability: bool,
    pub store_local: bool,
}

impl Capability {
    pub fn bounds(&self) -> (usize, usize);
    pub fn permissions(&self) -> Permissions;
    pub fn derive(&self, offset: usize, len: usize, perms: Permissions) -> Result<Self, CheriError>;
    pub fn seal(&self, key: &SealingKey) -> Result<Self, CheriError>;
    pub fn is_sealed(&self) -> bool;
}
```

Allocation, unsealing, and permission derivation should require explicit capability grants from Ifa's existing capability system.

### Layer 4: Ifa-Level Domain

If exposed to Ifa scripts, CHERI should appear only through an unsafe or capability-protected domain.

Candidate placements:

| Domain | Fit |
|--------|-----|
| `Ofun` | creation/root authority and capability policy |
| `Ogunda` | memory separation, cutting, derivation |
| infrastructure domain | explicit hardware/system integration |

The safest design is to keep CHERI operations unavailable unless the program has an explicit capability grant and is running on a CHERI target.

## Relationship To Ifa-Lang Memory Safety

CHERI composes with the proposed memory-safety roadmap as follows:

| Ifa mechanism | Ifa responsibility | CHERI responsibility |
|---------------|--------------------|----------------------|
| `IfaGc` | lifetime management and cycle collection | capability-checked internal pointer access |
| Opon epochs | scoped slot lifetime | backing allocation hardening |
| generational handles | stale handle detection | trap invalid stale capability use with revocation |
| `yanda` | ownership transfer intent and source-level consumption | hardware authority transfer if zero-copy is implemented |
| `iso` | static proof of unique object graph | bounded, permissioned graph access |
| actors | isolation policy | capability-enforced access if shared memory transfer exists |
| capability sets | language-level authority | hardware-level pointer authority |

The guiding rule is:

> Ifa-Lang proves who should have authority. CHERI enforces what that authority can touch.

## Recommended Roadmap

1. Document CHERI as a target hardening layer, not as an implemented backend.
2. Add `kernel::has_cheri()` / `capability_architecture()` only after a real target cfg strategy exists.
3. Audit pointer-to-integer assumptions in `ifa-types` and `ifa-vm`.
4. Implement generational Opon handles before any script-level CHERI API.
5. Implement `iso` before zero-copy CHERI actor transfer (runtime-hardened `yanda` with MoveLocal opcode is already done).
6. Consider a feature-gated `ifa-infra::cheri` module only when there is a supported CHERI Rust toolchain in CI.

## Non-Goals

- Do not make CHERI mandatory for Ifa-Lang memory safety.
- Do not expose raw capability allocation to ordinary safe Ifa code.
- Do not remove `IfaGc` just because CHERI exists.
- Do not claim temporal safety without an explicit revocation mechanism.
- Do not treat pointer-sized integers as portable capability handles.

## Summary

CHERI is best understood as a hardened backend for Ifa-Lang's memory-safety design. It can strengthen the VM, GC, Opon, and future `iso`/`yanda` transfer model, but it does not replace Babalawo, Opon epochs, generational handles, actor isolation, or `IfaGc`.

The strongest long-term design is:

```text
Babalawo proves unique ownership
        +
yanda consumes sender authority
        +
iso limits the object graph
        +
CHERI bounds and permissions enforce access in hardware
```

That combination would give Ifa-Lang a distinctive memory-safety model: source-visible authority, runtime ownership checks, and hardware-enforced pointer bounds.
