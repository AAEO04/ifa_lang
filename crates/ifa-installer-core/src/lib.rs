pub mod check;
pub mod profiles;
pub mod config;
pub mod net;
pub mod install;
pub mod args;
pub mod extraction;
pub mod uninstall;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(unix)]
pub mod unix;

// Re-export platform-specific items
#[cfg(target_os = "windows")]
pub use windows as platform;

#[cfg(unix)]
pub use unix as platform;
