//! # Ifá-Embedded
//! 
//! Minimal no_std runtime for embedded Ifá-Lang applications.
//! 
//! ## Targets
//! - ESP32 (Xtensa/RISC-V)
//! - STM32 (ARM Cortex-M)
//! - RP2040 (Raspberry Pi Pico)

#![cfg_attr(not(test), no_std)]

// Re-export core types
pub use ifa_core::{IfaValue, Bytecode, OpCode};

/// Embedded VM configuration
pub struct EmbeddedConfig {
    /// Stack size in slots
    pub stack_size: usize,
    /// Memory (Opon) size in slots
    pub opon_size: usize,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        EmbeddedConfig {
            stack_size: 64,
            opon_size: 256,
        }
    }
}

impl EmbeddedConfig {
    /// Minimal config for very constrained devices
    pub fn minimal() -> Self {
        EmbeddedConfig {
            stack_size: 32,
            opon_size: 64,
        }
    }
}

/// Placeholder for embedded-specific functionality
/// Will be expanded with embassy/RTIC integration
pub mod hal {
    /// GPIO placeholder
    pub struct Gpio;
    
    /// LED control
    pub struct Led;
    
    /// Serial/UART
    pub struct Serial;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = EmbeddedConfig::default();
        assert_eq!(config.stack_size, 64);
        assert_eq!(config.opon_size, 256);
    }
}
