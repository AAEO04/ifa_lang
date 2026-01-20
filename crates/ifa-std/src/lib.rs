#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::type_complexity)]
#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::manual_repeat_n)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::manual_contains)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unnecessary_lazy_evaluations)]
#![allow(clippy::get_first)]

//! # Ifá-Std - The 16 Odù Domains
//!
//! Standard library implementing the 16 principal Odù as Rust modules.
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

pub mod traits;

// Core domains (always available)
pub mod ika; // 0100 - Strings

pub mod irosu; // 1100 - Console I/O
pub mod iwori; // 0110 - Time/Iteration
pub mod obara; // 1000 - Math Add/Mul
#[cfg(feature = "backend")]
pub mod odi; // 1001 - Files/DB
pub mod ofun;
pub mod ogbe; // 1111 - System/Lifecycle
pub mod ogunda; // 1110 - Arrays
pub mod okanran; // 0001 - Errors
pub mod oturupon; // 0010 - Math Sub/Div
pub mod owonrin; // 0011 - Random
pub mod oyeku; // 0000 - Exit/Sleep // 0101 - Permissions

// Optional domains (feature-gated)
// Optional domains (feature-gated)
#[cfg(feature = "backend")]
pub mod osa; // 0111 - Concurrency

#[cfg(feature = "game")]
pub mod ose; // 1010 - Graphics/UI

#[cfg(feature = "backend")]
pub mod otura; // 1011 - Networking

#[cfg(feature = "crypto")]
pub mod irete; // 1101 - Crypto




// Priority Stacks (Phase 4)
pub mod stacks;

// Infrastructure Layer (Hardware/OS)
pub mod infra;

// FFI - Foreign Function Interface
pub mod ffi;

// Opele - Divination chain and Odu patterns
pub mod opele;

// Re-exports
pub use traits::OduDomain;
