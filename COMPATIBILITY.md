# Compatibility Guide

Ifá-Lang is designed for **Binary Compatibility** across supported runtimes where possible, and **Source Compatibility** everywhere.

## Bytecode Compatibility (.ifab)

The `.ifab` format (Version 1) is the standard for checking compatibility.

### Supported Runtimes

| Feature | ifa-core (Desktop/Server) | ifa-embedded (Microcontrollers) | ifa-wasm (Browser) |
|---------|---------------------------|---------------------------------|--------------------|
| **OpCodes** | Full Support | ~95% Support (No File/Net/OS) | ~90% (No File/OS direct) |
| **Types** | Dynamic (All) | 32-bit Integer/Float Optimized | Dynamic (All) |
| **Strings** | Full Unicode | UTF-8 Stored, Streamed Access | Full Unicode |
| **Lists** | `Vec<T>` | `heapless::Vec` (Fixed Capacity) | `Vec<T>` |
| **Objects** | `HashMap` | Bounded `BTreeMap` | `HashMap` |

### Breaking Changes
- **Embedded**: Code dealing with `File`, `Network`, or `Process` APIs will result in a runtime error (`ErrorCode::NotImplemented`) on embedded devices unless using specific HAL bridges.
- **Float Precision**: `ifa-embedded` may use software floating-point emulation depending on target hardware (e.g. Cortex-M0+), which may have slight precision variances compared to x86_64 hardware floats.

## Versioning Policy

We follow [Semantic Versioning 2.0.0](https://semver.org/).

- **Major (1.x -> 2.x)**: Breaking changes to `.ifab` binary format or core syntax.
- **Minor (1.2 -> 1.3)**: New OpCodes, new Standard Library modules.
- **Patch (1.2.1 -> 1.2.2)**: Bug fixes, performance improvements, no schema changes.
