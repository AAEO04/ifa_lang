//! # IoT/Embedded Stack
//!
//! Extensions for embedded systems and IoT devices.
//!
//! Features:
//! - HAL-style GPIO traits
//! - Proper error handling
//! - Timer and delay utilities
//! - Serial/UART, I2C, SPI abstractions
//!
//! Targets: ESP32, STM32, RP2040
//!
//! Uses: embassy-executor, probe-rs (when targeting actual hardware)
#![cfg_attr(not(feature = "backend"), no_std)]

#[cfg(feature = "backend")]
extern crate std;

#[cfg(not(feature = "backend"))]
extern crate alloc;
#[cfg(not(feature = "backend"))]
use alloc::collections::VecDeque;
#[cfg(not(feature = "backend"))]
use alloc::format;
#[cfg(not(feature = "backend"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "backend"))]
use alloc::vec;
#[cfg(not(feature = "backend"))]
use alloc::vec::Vec;

#[cfg(feature = "backend")]
macro_rules! log {
    ($($arg:tt)*) => { println!($($arg)*) }
}

#[cfg(not(feature = "backend"))]
macro_rules! log {
    ($($arg:tt)*) => {{}};
}

// std::time moved to gated block below

/// Errors for embedded operations
#[derive(Debug, Clone)]
pub enum EmbeddedError {
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
}

impl core::fmt::Display for EmbeddedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PinError(msg) => write!(f, "Pin error: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::NotInitialized => write!(f, "Hardware not initialized"),
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

#[cfg(feature = "backend")]
impl std::error::Error for EmbeddedError {}

pub type EmbeddedResult<T> = Result<T, EmbeddedError>;

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
    pub fn set_mode(&mut self, mode: PinMode) -> EmbeddedResult<()> {
        log!("[GPIO] Pin {} configured as {:?}", self.pin, mode);
        self.mode = Some(mode);
        Ok(())
    }

    /// Set output state
    pub fn set_state(&mut self, state: PinState) -> EmbeddedResult<()> {
        match self.mode {
            Some(PinMode::Output) | Some(PinMode::OpenDrain) => {
                log!("[GPIO] Pin {} = {:?}", self.pin, state);
                self.state = state;
                Ok(())
            }
            _ => Err(EmbeddedError::PinError(
                "Pin not configured as output".into(),
            )),
        }
    }

    /// Set high
    pub fn set_high(&mut self) -> EmbeddedResult<()> {
        self.set_state(PinState::High)
    }

    /// Set low
    pub fn set_low(&mut self) -> EmbeddedResult<()> {
        self.set_state(PinState::Low)
    }

    /// Toggle output
    pub fn toggle(&mut self) -> EmbeddedResult<()> {
        let new_state = if self.state == PinState::High {
            PinState::Low
        } else {
            PinState::High
        };
        self.set_state(new_state)
    }

    /// Read input state
    pub fn read(&self) -> EmbeddedResult<PinState> {
        match self.mode {
            Some(PinMode::Input) | Some(PinMode::InputPullUp) | Some(PinMode::InputPullDown) => {
                // Placeholder - would read actual hardware
                Ok(self.state)
            }
            _ => Err(EmbeddedError::PinError(
                "Pin not configured as input".into(),
            )),
        }
    }

    /// Check if pin is high
    pub fn is_high(&self) -> EmbeddedResult<bool> {
        Ok(self.read()? == PinState::High)
    }

    /// Check if pin is low
    pub fn is_low(&self) -> EmbeddedResult<bool> {
        Ok(self.read()? == PinState::Low)
    }
}

/// Embedded GPIO abstraction (legacy API)
pub struct EmbeddedGpio;

impl EmbeddedGpio {
    /// Set pin mode (input/output)
    pub fn mode(&self, pin: u8, output: bool) -> EmbeddedResult<()> {
        let mode = if output {
            PinMode::Output
        } else {
            PinMode::Input
        };
        log!("[GPIO] Pin {} set to {:?}", pin, mode);
        Ok(())
    }

    /// Write digital value
    pub fn write(&self, pin: u8, high: bool) -> EmbeddedResult<()> {
        log!("[GPIO] Pin {} = {}", pin, if high { "HIGH" } else { "LOW" });
        Ok(())
    }

    /// Read digital value
    pub fn read(&self, _pin: u8) -> EmbeddedResult<bool> {
        // Placeholder
        Ok(false)
    }

    /// PWM output (duty cycle 0-255)
    pub fn pwm(&self, pin: u8, duty: u8) -> EmbeddedResult<()> {
        log!("[GPIO] Pin {} PWM duty = {}", pin, duty);
        Ok(())
    }

    /// Analog read (ADC)
    pub fn analog_read(&self, _pin: u8) -> EmbeddedResult<u16> {
        // Placeholder
        Ok(0)
    }
}

#[cfg(not(feature = "backend"))]
use core::time::Duration;
#[cfg(feature = "backend")]
use std::time::{Duration, Instant};

// ... (Error types remain, but impl std::error::Error needs gating)

// ...

/// Hardware Timer/Delay with non-blocking support
#[derive(Debug)]
pub struct EmbeddedTimer {
    #[cfg(feature = "backend")]
    deadline: Option<Instant>,
    #[cfg(not(feature = "backend"))]
    _dummy: (),
}

impl EmbeddedTimer {
    pub fn new() -> Self {
        EmbeddedTimer {
            #[cfg(feature = "backend")]
            deadline: None,
            #[cfg(not(feature = "backend"))]
            _dummy: (),
        }
    }

    /// Blocking delay in microseconds
    pub fn delay_us(&self, us: u32) {
        #[cfg(feature = "backend")]
        std::thread::sleep(Duration::from_micros(us as u64));
        #[cfg(not(feature = "backend"))]
        { /* No-op in no_std simulation */ }
    }

    /// Blocking delay in milliseconds
    pub fn delay_ms(&self, ms: u32) {
        #[cfg(feature = "backend")]
        std::thread::sleep(Duration::from_millis(ms as u64));
        #[cfg(not(feature = "backend"))]
        { /* No-op */ }
    }

    /// Non-blocking: start a timer
    pub fn start(&mut self, _duration: Duration) {
        #[cfg(feature = "backend")]
        {
            self.deadline = Some(Instant::now() + _duration);
        }
    }

    /// Non-blocking: check if timer expired
    pub fn is_expired(&self) -> bool {
        #[cfg(feature = "backend")]
        {
            self.deadline.map(|d| Instant::now() >= d).unwrap_or(false)
        }
        #[cfg(not(feature = "backend"))]
        {
            true
        }
    }

    /// Non-blocking: wait for timer (polling)
    pub fn wait(&mut self) -> EmbeddedResult<()> {
        #[cfg(feature = "backend")]
        match self.deadline {
            Some(deadline) => {
                while Instant::now() < deadline {
                    std::thread::sleep(Duration::from_micros(100));
                }
                self.deadline = None;
                Ok(())
            }
            None => Err(EmbeddedError::NotInitialized),
        }
        #[cfg(not(feature = "backend"))]
        Ok(())
    }

    /// Measure execution time
    #[cfg(feature = "backend")]
    pub fn measure<F: FnOnce() -> T, T>(f: F) -> (T, Duration) {
        let start = Instant::now();
        let result = f();
        (result, start.elapsed())
    }

    #[cfg(not(feature = "backend"))]
    pub fn measure<F: FnOnce() -> T, T>(f: F) -> (T, Duration) {
        (f(), Duration::from_secs(0))
    }
}

impl Default for EmbeddedTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Serial/UART communication with error handling
/// Uses bounded buffer to prevent memory exhaustion in embedded contexts.
#[derive(Debug)]
pub struct EmbeddedSerial {
    baud: u32,
    initialized: bool,
    #[cfg(feature = "iot")]
    buffer: heapless::Deque<u8, 256>, // Bounded ring buffer (256 bytes)
    #[cfg(not(feature = "iot"))]
    buffer: VecDeque<u8>,
}

impl EmbeddedSerial {
    pub fn new() -> Self {
        EmbeddedSerial {
            baud: 0,
            initialized: false,
            #[cfg(feature = "iot")]
            buffer: heapless::Deque::new(),
            #[cfg(not(feature = "iot"))]
            buffer: VecDeque::new(),
        }
    }

    /// Initialize UART at baud rate
    pub fn init(&mut self, baud: u32) -> EmbeddedResult<()> {
        if ![9600, 19200, 38400, 57600, 115200].contains(&baud) {
            return Err(EmbeddedError::InvalidParameter(format!(
                "Unsupported baud rate: {}",
                baud
            )));
        }
        log!("[UART] Initialized at {} baud", baud);
        self.baud = baud;
        self.initialized = true;
        Ok(())
    }

    /// Write bytes
    pub fn write(&mut self, data: &[u8]) -> EmbeddedResult<usize> {
        if !self.initialized {
            return Err(EmbeddedError::NotInitialized);
        }
        log!("[UART] TX: {:?}", data);
        Ok(data.len())
    }

    /// Write string
    pub fn print(&mut self, s: &str) -> EmbeddedResult<usize> {
        self.write(s.as_bytes())
    }

    /// Read available bytes (O(n) copy, O(1) per-byte drain via ring buffer)
    pub fn read(&mut self, buffer: &mut [u8]) -> EmbeddedResult<usize> {
        if !self.initialized {
            return Err(EmbeddedError::NotInitialized);
        }
        let count = self.buffer.len().min(buffer.len());
        for i in 0..count {
            // pop_front is O(1) for VecDeque and heapless::Deque
            buffer[i] = self.buffer.pop_front().unwrap_or(0);
        }
        Ok(count)
    }

    /// Check if data available
    pub fn available(&self) -> usize {
        self.buffer.len()
    }

    /// Buffer capacity
    #[cfg(feature = "iot")]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity() // heapless::Deque returns N (256)
    }

    #[cfg(not(feature = "iot"))]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity() // VecDeque's actual allocated capacity
    }
}

impl Default for EmbeddedSerial {
    fn default() -> Self {
        Self::new()
    }
}

/// I2C communication with error handling
#[derive(Debug)]
pub struct EmbeddedI2C {
    sda: u8,
    scl: u8,
    initialized: bool,
}

impl EmbeddedI2C {
    pub fn new() -> Self {
        EmbeddedI2C {
            sda: 0,
            scl: 0,
            initialized: false,
        }
    }

    /// Initialize I2C
    pub fn init(&mut self, sda: u8, scl: u8) -> EmbeddedResult<()> {
        log!("[I2C] SDA={}, SCL={}", sda, scl);
        self.sda = sda;
        self.scl = scl;
        self.initialized = true;
        Ok(())
    }

    /// Write to device
    pub fn write(&self, addr: u8, data: &[u8]) -> EmbeddedResult<()> {
        if !self.initialized {
            return Err(EmbeddedError::NotInitialized);
        }
        log!("[I2C] Write to 0x{:02X}: {:?}", addr, data);
        Ok(())
    }

    /// Read from device
    pub fn read(&self, addr: u8, buffer: &mut [u8]) -> EmbeddedResult<usize> {
        if !self.initialized {
            return Err(EmbeddedError::NotInitialized);
        }
        log!("[I2C] Read from 0x{:02X}, {} bytes", addr, buffer.len());
        // Placeholder - fill with zeros
        buffer.fill(0);
        Ok(buffer.len())
    }

    /// Write then read (common pattern)
    pub fn write_read(&self, addr: u8, write: &[u8], read: &mut [u8]) -> EmbeddedResult<usize> {
        self.write(addr, write)?;
        self.read(addr, read)
    }
}

impl Default for EmbeddedI2C {
    fn default() -> Self {
        Self::new()
    }
}

/// SPI communication with error handling
#[derive(Debug)]
pub struct EmbeddedSPI {
    mosi: u8,
    miso: u8,
    sck: u8,
    initialized: bool,
}

impl EmbeddedSPI {
    pub fn new() -> Self {
        EmbeddedSPI {
            mosi: 0,
            miso: 0,
            sck: 0,
            initialized: false,
        }
    }

    /// Initialize SPI
    pub fn init(&mut self, mosi: u8, miso: u8, sck: u8) -> EmbeddedResult<()> {
        log!("[SPI] MOSI={}, MISO={}, SCK={}", mosi, miso, sck);
        self.mosi = mosi;
        self.miso = miso;
        self.sck = sck;
        self.initialized = true;
        Ok(())
    }

    /// Transfer data (full duplex)
    pub fn transfer(&self, data: &mut [u8]) -> EmbeddedResult<()> {
        if !self.initialized {
            return Err(EmbeddedError::NotInitialized);
        }
        log!("[SPI] Transfer: {:?}", data);
        Ok(())
    }

    /// Write only
    pub fn write(&self, data: &[u8]) -> EmbeddedResult<()> {
        if !self.initialized {
            return Err(EmbeddedError::NotInitialized);
        }
        log!("[SPI] Write: {:?}", data);
        Ok(())
    }
}

impl Default for EmbeddedSPI {
    fn default() -> Self {
        Self::new()
    }
}

/// Flash to embedded device via probe-rs
///
/// Requires `probe-rs` CLI to be installed (`cargo install probe-rs-tools`).
/// Supported targets: esp32, esp32c3, stm32f4, rp2040, etc.
#[cfg(feature = "backend")]
pub fn flash(target: &str, binary_path: &str, port: Option<&str>) -> EmbeddedResult<()> {
    use std::process::Command;

    log!(
        "Flashing to {} via {}",
        target,
        port.unwrap_or("auto-detect")
    );
    log!("   Binary: {}", binary_path);

    let mut cmd = Command::new("probe-rs");
    cmd.arg("download")
        .arg("--chip")
        .arg(target)
        .arg(binary_path);

    if let Some(p) = port {
        cmd.arg("--probe").arg(p);
    }

    let output = cmd
        .output()
        .map_err(|e| EmbeddedError::IoError(format!("Failed to run probe-rs: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(EmbeddedError::IoError(format!(
            "probe-rs failed: {}",
            stderr
        )));
    }

    log!("Flash complete!");
    Ok(())
}

/// Flash stub for no_std (cannot invoke subprocesses)
#[cfg(not(feature = "backend"))]
pub fn flash(_target: &str, _binary_path: &str, _port: Option<&str>) -> EmbeddedResult<()> {
    Err(EmbeddedError::IoError(
        "Flashing requires std (use backend feature)".into(),
    ))
}

/// Sensor reading helper
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub value: f64,
    pub unit: String,
    pub timestamp_ms: u64,
}

impl SensorReading {
    pub fn new(value: f64, unit: &str) -> Self {
        #[cfg(feature = "backend")]
        let timestamp_ms = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default() // Avoid panic on pre-epoch clocks
                .as_millis() as u64
        };
        #[cfg(not(feature = "backend"))]
        let timestamp_ms = 0; // No system time in no_std

        SensorReading {
            value,
            unit: unit.to_string(),
            timestamp_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_pin() {
        let mut pin = GpioPin::new(13);
        assert!(pin.set_mode(PinMode::Output).is_ok());
        assert!(pin.set_high().is_ok());
        assert!(pin.toggle().is_ok());
    }

    #[test]
    fn test_gpio_error() {
        let pin = GpioPin::new(13);
        // Not configured, should error
        assert!(pin.read().is_err());
    }

    #[test]
    fn test_timer() {
        let timer = EmbeddedTimer::new();
        let (_, duration) = EmbeddedTimer::measure(|| {
            timer.delay_ms(10);
        });
        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_serial_not_initialized() {
        let mut serial = EmbeddedSerial::new();
        assert!(serial.write(b"test").is_err());
    }

    #[test]
    fn test_serial_init() {
        let mut serial = EmbeddedSerial::new();
        assert!(serial.init(115200).is_ok());
        assert!(serial.write(b"test").is_ok());
    }
}
