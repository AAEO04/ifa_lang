# Ifá Standard Library (ifa-std)

The **Ifá Standard Library** is the collection of core domains (Odù) that provide high-level functionality to Ifá-Lang programs. It acts as the "Service Layer" of the language, bridging the gap between user code and the raw power of the underlying hardware.

## Architectural Note: The `ifa-infra` Transition
As part of the **Hardening Phase**, the standard library is currently transitioning. High-performance, low-level Rust primitives (LSM-Trees, Task Graphs, WGPU wrappers) are being moved to the **`ifa-infra`** crate. `ifa-std` will remain as the idiomatic Ifá-Lang interface to these services.

## Core Domains (Odù)

The standard library is organized into specialized domains, each identified by its canonical Odù number:

- **Òdí (Domain 7) - The Seal**: Handles persistence, filesystem access, and SQLite integration.
- **Ọ̀ṣẹ́ (Domain 15) - The Painter**: Handles TUI rendering, declarative UI widgets, and visual output.
- **GPU (Domain 19) - The Accelerator**: High-performance compute shaders and GPGPU tasks via WGPU.
- **CPU (Domain 20) - The Multiplier**: Massive parallelism, work-stealing task graphs, and multi-core orchestration.

## Hardware Philosophy

### 1. Unified Memory Awareness (UMA)
`ifa-std` is designed to be hardware-aware. On systems with Unified Memory (like Apple Silicon or modern APUs), the library enables **Zero-Copy** data sharing between the CPU and GPU, eliminating redundant memory transfers.

### 2. Capability-Based Security
Access to the standard library is gated by **ResourceTokens**. A module cannot access the filesystem (`Òdí`) or the network unless it has been explicitly granted that capability in its **#opon** (Calabash) configuration.

### 3. Graceful Degradation
The library is built with a "Fallback First" mentality. If a GPU is not available, the `GPU` domain automatically degrades to high-speed SIMD operations on the CPU to ensure the program still runs, albeit slower.

## Usage

Standard library domains are typically invoked using the `iba` (import) or `opon` (permission) directives:

```ifa
opon Òdí   # Grant filesystem access
iba Òdí    # Import the domain handlers

ayanmo file = Òdí.si("data.txt")
file.ko("Hello, Ifá!")
file.pa()
```

## Internal Structure

- `src/infra/`: High-level wrappers for storage, gpu, and parallel processing.
- `src/lib.rs`: The main entry point and domain registry.
- `src/ose.rs`: Declarative UI rendering logic.
- `src/odi.rs`: File and persistence interface.
