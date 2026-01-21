pub mod args;
pub mod check;
pub mod config;
pub mod extraction;
pub mod install;
pub mod net;
pub mod profiles;
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
