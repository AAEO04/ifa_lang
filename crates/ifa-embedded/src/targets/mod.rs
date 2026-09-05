//! Target-specific configurations and HAL implementations.

#[cfg(feature = "esp32")]
pub mod esp32;

#[cfg(feature = "stm32")]
pub mod stm32;

#[cfg(feature = "rp2040")]
pub mod rp2040;
