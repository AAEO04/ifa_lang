# Ifá-Lang: Mission & Principles

## 1. The Why (Mission)
**To democratize systems programming for the Yoruba-speaking world (and beyond) by embedding cultural wisdom into the fabric of computation.**

Existing languages force a Western, anglophone conceptual model. Ifá-Lang proves that computation is universal. It is not just "Rust with Yoruba keywords"; it is a reimagining of how we interact with the machine, viewing memory (`Opon`) as a sacred space and execution (`Iṣẹ́`) as a ritual of transformation.

**Target Audience**:
- **Systems Educators**: Teaching low-level concepts (pointers, memory) through culturally relevant metaphors.
- **Embedded Engineers**: Creating robust, `no_std` firmware for IoT devices in West Africa.
- **Cultural Futurists**: Proving that technology can respect and elevate tradition.

## 2. Core Design Principles

### A. Safety as Sacred (`Àìléwu`)
*Principle*: "You do not touch the Oracle without preparation."
*Implementation*: Memory safety is default. Unsafe operations (`*ptr`) require explicit `àìléwu` (safety/security) blocks. We do not use Garbage Collection; we use discipline and ownership.

### B. Simplicity over Features (`Ìrọ̀rùn`)
*Principle*: "Truth is simple."
*Implementation*: 
- No complex inheritance. 
- No hidden control flow (exceptions).
- Explicit error handling (`Result`).
- A small, orthogonal instruction set (`OpCode`).

### C. Universality (`Gbogbo Ènìyàn`)
*Principle*: "The code flows everywhere."
*Implementation*:
- **Binary Compatibility**: A `.ifab` compiled on a server MUST run on a microcontroller.
- **No STD Dependency**: The core language is `no_std` (embedded) first.
- **Cross-Linguistic**: Keywords map 1:1 between Yoruba and English.

## 3. Architecture Mapped to Guide

| Guide Rule | Ifá Implementation |
|:---|:---|
| **Compilation** | **Bytecode VM**: Portable, verifiable stability. AOT optional via transpilation. |
| **Memory** | **Hybrid Ownership**: Strict ownership in Rust host, Stack-based VM for guest. |
| **Concurrency** | **Data Parallelism (`Ọ̀sá`)**: Rayon-backed parallel iterators. No GIL. |
| **Errors** | **Result-based**: No panics in production. Unified `ErrorCode`. |

## 4. The 10-Year Vision
- **Year 1-2**: Stability & Embedded Roots (Current).
- **Year 3-5**: Ecosystem Growth (Package Manager `Ọjà`, Standard Lib).
- **Year 5-10**: Self-Hosting & Native Compilation.

> "Proceed only when the solution feels boring." — *The User*
