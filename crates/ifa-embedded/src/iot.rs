//! # IoT Stack (Tier 1)
//!
//! This module contains higher-level features for "Smart" embedded devices.
//! Enabled by `feature = "iot"` (implies `alloc`).
//!
//! Includes:
//! - HAL Traits (GPIO, I2C, SPI, UART)
//! - JSON Serialization (serde_json)
//! - Networking Stubs

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::fmt;

// We use the `log` crate which we added to Cargo.toml
// If log feature is enabled, these macros will output to the global logger.
// Otherwise they are no-ops.
#[allow(unused_macros)]
macro_rules! log_info {
    ($($arg:tt)*) => {
        #[cfg(feature = "dep:log")]
        log::info!($($arg)*);
    }
}

// =============================================================================
// ERRORS
// =============================================================================

/// Errors for IoT operations
#[derive(Debug, Clone)]
pub enum IotError {
    /// Pin not available or already in use
    PinError(String),
    /// Communication error
    IoError(String),
    /// Timeout expired
    Timeout,
    /// Hardware not initialized
    NotInitialized,
    /// Invalid parameter
    InvalidParameter(String),
    /// Serialization Error
    SerializationError(String),
    /// Network Error
    NetworkError(String),
}

impl fmt::Display for IotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PinError(msg) => write!(f, "Pin error: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::NotInitialized => write!(f, "Hardware not initialized"),
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

pub type IotResult<T> = Result<T, IotError>;

// =============================================================================
// GPIO & HARDWARE TRAITS (Ported from ifa-std)
// =============================================================================

/// Pin mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PinMode {
    Input,
    Output,
    InputPullUp,
    InputPullDown,
    OpenDrain,
}

/// Pin state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PinState {
    Low,
    High,
}

impl From<bool> for PinState {
    fn from(v: bool) -> Self {
        if v { PinState::High } else { PinState::Low }
    }
}

impl From<PinState> for bool {
    fn from(s: PinState) -> bool {
        s == PinState::High
    }
}

/// GPIO pin abstraction with proper error handling
#[derive(Debug)]
pub struct GpioPin {
    pin: u8,
    mode: Option<PinMode>,
    state: PinState,
}

impl GpioPin {
    /// Create a new GPIO pin reference
    pub fn new(pin: u8) -> Self {
        GpioPin {
            pin,
            mode: None,
            state: PinState::Low,
        }
    }

    /// Configure pin mode
    pub fn set_mode(&mut self, mode: PinMode) -> IotResult<()> {
        log_info!("[GPIO] Pin {} configured as {:?}", self.pin, mode);
        self.mode = Some(mode);
        Ok(())
    }

    /// Set output state
    pub fn set_state(&mut self, state: PinState) -> IotResult<()> {
        match self.mode {
            Some(PinMode::Output) | Some(PinMode::OpenDrain) => {
                log_info!("[GPIO] Pin {} = {:?}", self.pin, state);
                self.state = state;
                Ok(())
            }
            _ => Err(IotError::PinError(
                "Pin not configured as output".into(),
            )),
        }
    }

    /// Set high
    pub fn set_high(&mut self) -> IotResult<()> {
        self.set_state(PinState::High)
    }

    /// Set low
    pub fn set_low(&mut self) -> IotResult<()> {
        self.set_state(PinState::Low)
    }

    /// Toggle output
    pub fn toggle(&mut self) -> IotResult<()> {
        let new_state = if self.state == PinState::High {
            PinState::Low
        } else {
            PinState::High
        };
        self.set_state(new_state)
    }

    /// Read input state
    pub fn read(&self) -> IotResult<PinState> {
        match self.mode {
            Some(PinMode::Input) | Some(PinMode::InputPullUp) | Some(PinMode::InputPullDown) => {
                // Placeholder - would read actual hardware
                Ok(self.state)
            }
            _ => Err(IotError::PinError(
                "Pin not configured as input".into(),
            )),
        }
    }
}

// =============================================================================
// COMMUNICATION (Serial, I2C, SPI)
// =============================================================================

/// Serial/UART communication with error handling
/// Uses bounded buffer (heapless) since we are in embedded context
#[derive(Debug)]
pub struct EmbeddedSerial {
    baud: u32,
    initialized: bool,
    // Buffer for RX
    buffer: heapless::Deque<u8, 256>, 
}

impl EmbeddedSerial {
    pub fn new() -> Self {
        EmbeddedSerial {
            baud: 0,
            initialized: false,
            buffer: heapless::Deque::new(),
        }
    }

    /// Initialize UART at baud rate
    pub fn init(&mut self, baud: u32) -> IotResult<()> {
        if ![9600, 19200, 38400, 57600, 115200].contains(&baud) {
            return Err(IotError::InvalidParameter(format!(
                "Unsupported baud rate: {}",
                baud
            )));
        }
        log_info!("[UART] Initialized at {} baud", baud);
        self.baud = baud;
        self.initialized = true;
        Ok(())
    }

    /// Write bytes
    pub fn write(&mut self, data: &[u8]) -> IotResult<usize> {
        if !self.initialized {
            return Err(IotError::NotInitialized);
        }
        // In real HAL, this would block or DMA
        log_info!("[UART] TX: {:?}", data);
        Ok(data.len())
    }

    /// Write string
    pub fn print(&mut self, s: &str) -> IotResult<usize> {
        self.write(s.as_bytes())
    }

    /// Read available bytes
    pub fn read(&mut self, buffer: &mut [u8]) -> IotResult<usize> {
        if !self.initialized {
            return Err(IotError::NotInitialized);
        }
        let count = self.buffer.len().min(buffer.len());
        for i in 0..count {
            buffer[i] = self.buffer.pop_front().unwrap_or(0);
        }
        Ok(count)
    }
}

// =============================================================================
// JSON HELPERS (Requires alloc)
// =============================================================================

#[cfg(feature = "dep:serde_json")]
pub struct JsonHelper;

#[cfg(feature = "dep:serde_json")]
impl JsonHelper {
    pub fn parse(json: &str) -> IotResult<serde_json::Value> {
        serde_json::from_str(json).map_err(|e| IotError::SerializationError(e.to_string()))
    }
    
    pub fn stringify<T: serde::Serialize>(value: &T) -> IotResult<String> {
        serde_json::to_string(value).map_err(|e| IotError::SerializationError(e.to_string()))
    }
}
