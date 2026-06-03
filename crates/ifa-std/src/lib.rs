
//! # Ifá-Std - The 16 Odù Domains
//!
//! Standard library implementing the 16 principal Odù as Rust modules.
//!
//! ### ⚠️ SECURITY ADVISORY (FFI Bridge)
//! The `ffi` module is under a hardening mandate. Refer to `patch.md` for details
//! on BUG-018 through BUG-021. Use `ffi.itumo` only for sanctified bridges.
//!
//! ## Domain Overview
//!
//! | Binary | Odù | Purpose |
//! |--------|-----|---------|
//! | 1111 | Ọ̀gbè | System, CLI Args, Lifecycle |
//! | 0000 | Ọ̀yẹ̀kú | Exit, Sleep |
//! | 0110 | Ìwòrì | Time, Iteration |
//! | 1001 | Òdí | Files, Database |
//! | 1100 | Ìrosù | Console I/O |
//! | 0011 | Ọ̀wọ́nrín | Random |
//! | 1000 | Ọ̀bàrà | Math (Add/Mul) |
//! | 0001 | Ọ̀kànràn | Errors, Assertions |
//! | 1110 | Ògúndá | Arrays, Processes |
//! | 0111 | Ọ̀sá | Concurrency |
//! | 0100 | Ìká | Strings |
//! | 0010 | Òtúúrúpọ̀n | Math (Sub/Div) |
//! | 1011 | Òtúrá | Networking |
//! | 1101 | Ìrẹtẹ̀ | Crypto, Compression |
//! | 1010 | Ọ̀ṣẹ́ | Graphics, UI |
//! | 0101 | Òfún | Permissions, Reflection |

pub mod esu;
pub mod sandbox_shim;
pub mod traits;

pub mod vm_registry;

// Core Odù Domains
pub mod odu;

// Hardware layer
pub mod hardware;

// Priority Stacks (Phase 4)

// Infrastructure Layer (Hardware/OS)
// Removed infra module

// FFI - Foreign Function Interface
#[cfg(feature = "native_ffi")]
pub mod ffi;

// Opele - Divination chain and Odu patterns
pub mod opele;

// Re-exports
pub use esu::Esu;
pub use traits::OduDomain;
