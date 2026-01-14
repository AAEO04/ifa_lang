pub mod platform {
    #[cfg(target_os = "windows")]
    pub mod windows;
    
    #[cfg(unix)]
    pub mod unix;
}

pub mod check;
pub mod profiles;
pub mod config;
pub mod net;
pub mod install;
pub mod args;
pub mod extraction;
pub mod uninstall;
