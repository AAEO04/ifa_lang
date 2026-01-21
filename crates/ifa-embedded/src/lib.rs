//! # Ifá-Embedded
//!
//! Minimal no_std runtime for embedded Ifá-Lang applications.
//!
//! ## Features
//!
//! - **HAL Traits**: Generic GPIO, Serial, Timer traits for hardware abstraction
//! - **EmbeddedVm**: Minimal bytecode interpreter with fixed-size stack
//! - **Target Support**: ESP32, STM32, RP2040 (via feature flags)
//!
//! ## Usage
//!
//! ```ignore
//! use ifa_embedded::{EmbeddedVm, EmbeddedConfig};
//!
//! let mut vm = EmbeddedVm::new(EmbeddedConfig::default());
//! let result = vm.run(&bytecode);
//! ```

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::string::String;
use core::fmt;
use heapless::Vec as HVec;

// =============================================================================
// ERRORS
// =============================================================================

/// Embedded runtime errors
#[derive(Debug, Clone)]
pub enum EmbeddedError {
    /// Stack overflow
    StackOverflow,
    /// Stack underflow
    StackUnderflow,
    /// Unknown opcode encountered
    UnknownOpcode(u8),
    /// Division by zero
    DivisionByZero,
    /// Memory access out of bounds
    MemoryOutOfBounds,
    /// Invalid bytecode format
    InvalidBytecode,
    /// HAL error
    HalError(String),
}

impl fmt::Display for EmbeddedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackOverflow => write!(f, "Stack overflow"),
            Self::StackUnderflow => write!(f, "Stack underflow"),
            Self::UnknownOpcode(op) => write!(f, "Unknown opcode: 0x{:02X}", op),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::MemoryOutOfBounds => write!(f, "Memory access out of bounds"),
            Self::InvalidBytecode => write!(f, "Invalid bytecode format"),
            Self::HalError(msg) => write!(f, "HAL error: {}", msg),
        }
    }
}

pub type EmbeddedResult<T> = Result<T, EmbeddedError>;

// =============================================================================
// HAL TRAITS
// =============================================================================

/// Input pin trait (read-only GPIO)
pub trait InputPin {
    /// Read the pin state (true = high, false = low)
    fn is_high(&self) -> bool;

    /// Read the pin state (true = low, false = high)
    fn is_low(&self) -> bool {
        !self.is_high()
    }
}

/// Output pin trait (writable GPIO)
pub trait OutputPin {
    /// Set pin high
    fn set_high(&mut self);

    /// Set pin low
    fn set_low(&mut self);

    /// Toggle pin state
    fn toggle(&mut self);
}

/// Serial/UART trait
pub trait Serial {
    /// Write a single byte (blocking)
    fn write_byte(&mut self, byte: u8) -> EmbeddedResult<()>;

    /// Read a single byte if available (non-blocking)
    fn read_byte(&mut self) -> Option<u8>;

    /// Check if data is available to read
    fn available(&self) -> bool;

    /// Write a slice of bytes
    fn write_bytes(&mut self, data: &[u8]) -> EmbeddedResult<()> {
        for byte in data {
            self.write_byte(*byte)?;
        }
        Ok(())
    }
}

/// Delay/Timer trait
pub trait DelayUs {
    /// Delay for the given number of microseconds
    fn delay_us(&mut self, us: u32);

    /// Delay for the given number of milliseconds
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1000);
    }
}

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Embedded VM configuration
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedConfig {
    /// Maximum stack size in slots
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
    /// Minimal config for very constrained devices (< 2KB RAM)
    pub fn minimal() -> Self {
        EmbeddedConfig {
            stack_size: 16,
            opon_size: 32,
        }
    }

    /// Standard config for typical MCUs (8-32KB RAM)
    pub fn standard() -> Self {
        EmbeddedConfig {
            stack_size: 64,
            opon_size: 256,
        }
    }

    /// Extended config for capable devices (64KB+ RAM)
    pub fn extended() -> Self {
        EmbeddedConfig {
            stack_size: 256,
            opon_size: 1024,
        }
    }
}

// =============================================================================
// EMBEDDED VALUE (Simplified for no_std)
// =============================================================================

/// Simplified value type for embedded contexts
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EmbeddedValue {
    /// Null/None
    #[default]
    Null,
    /// Boolean
    Bool(bool),
    /// 32-bit integer (smaller footprint than i64)
    Int(i32),
    /// 32-bit float (smaller footprint than f64)
    Float(f32),
}

impl EmbeddedValue {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            EmbeddedValue::Null => false,
            EmbeddedValue::Bool(b) => *b,
            EmbeddedValue::Int(n) => *n != 0,
            EmbeddedValue::Float(f) => *f != 0.0,
        }
    }
}

// =============================================================================
// OPCODES (Subset for embedded)
// =============================================================================

/// Embedded opcodes - minimal subset for constrained devices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EmbeddedOpCode {
    /// Push null
    PushNull = 0x00,
    /// Push integer (followed by 4 bytes, little-endian)
    PushInt = 0x01,
    /// Push float (followed by 4 bytes, little-endian)
    PushFloat = 0x02,
    /// Push true
    PushTrue = 0x04,
    /// Push false
    PushFalse = 0x05,

    /// Pop and discard
    Pop = 0x10,
    /// Duplicate top
    Dup = 0x11,

    /// Add
    Add = 0x20,
    /// Subtract
    Sub = 0x21,
    /// Multiply
    Mul = 0x22,
    /// Divide
    Div = 0x23,

    /// Equal
    Eq = 0x30,
    /// Less than
    Lt = 0x32,
    /// Greater than
    Gt = 0x34,

    /// Logical NOT
    Not = 0x42,

    /// Load local variable (followed by 1-byte index)
    LoadLocal = 0x50,
    /// Store local variable (followed by 1-byte index)
    StoreLocal = 0x51,

    /// Jump (followed by 2-byte offset, little-endian)
    Jump = 0x60,
    /// Jump if false
    JumpIfFalse = 0x61,

    /// Halt execution
    Halt = 0xFF,
}

impl EmbeddedOpCode {
    /// Parse opcode from byte
    pub fn from_byte(byte: u8) -> EmbeddedResult<Self> {
        match byte {
            0x00 => Ok(EmbeddedOpCode::PushNull),
            0x01 => Ok(EmbeddedOpCode::PushInt),
            0x02 => Ok(EmbeddedOpCode::PushFloat),
            0x04 => Ok(EmbeddedOpCode::PushTrue),
            0x05 => Ok(EmbeddedOpCode::PushFalse),
            0x10 => Ok(EmbeddedOpCode::Pop),
            0x11 => Ok(EmbeddedOpCode::Dup),
            0x20 => Ok(EmbeddedOpCode::Add),
            0x21 => Ok(EmbeddedOpCode::Sub),
            0x22 => Ok(EmbeddedOpCode::Mul),
            0x23 => Ok(EmbeddedOpCode::Div),
            0x30 => Ok(EmbeddedOpCode::Eq),
            0x32 => Ok(EmbeddedOpCode::Lt),
            0x34 => Ok(EmbeddedOpCode::Gt),
            0x42 => Ok(EmbeddedOpCode::Not),
            0x50 => Ok(EmbeddedOpCode::LoadLocal),
            0x51 => Ok(EmbeddedOpCode::StoreLocal),
            0x60 => Ok(EmbeddedOpCode::Jump),
            0x61 => Ok(EmbeddedOpCode::JumpIfFalse),
            0xFF => Ok(EmbeddedOpCode::Halt),
            _ => Err(EmbeddedError::UnknownOpcode(byte)),
        }
    }
}

// =============================================================================
// EMBEDDED VM
// =============================================================================

/// Minimal embedded VM with fixed-size stack
///
/// Uses heapless collections for deterministic memory usage.
pub struct EmbeddedVm {
    /// Value stack (fixed size)
    stack: HVec<EmbeddedValue, 64>,
    /// Local variables (fixed size)
    locals: HVec<EmbeddedValue, 32>,
    /// Instruction pointer
    ip: usize,
    /// Running flag
    running: bool,
}

impl EmbeddedVm {
    /// Create a new embedded VM
    pub fn new(_config: EmbeddedConfig) -> Self {
        EmbeddedVm {
            stack: HVec::new(),
            locals: HVec::new(),
            ip: 0,
            running: false,
        }
    }

    /// Reset VM state
    pub fn reset(&mut self) {
        self.stack.clear();
        self.locals.clear();
        self.ip = 0;
        self.running = false;
    }

    /// Push value onto stack
    fn push(&mut self, value: EmbeddedValue) -> EmbeddedResult<()> {
        self.stack
            .push(value)
            .map_err(|_| EmbeddedError::StackOverflow)
    }

    /// Pop value from stack
    fn pop(&mut self) -> EmbeddedResult<EmbeddedValue> {
        self.stack.pop().ok_or(EmbeddedError::StackUnderflow)
    }

    /// Peek at top of stack
    fn peek(&self) -> EmbeddedResult<EmbeddedValue> {
        self.stack
            .last()
            .copied()
            .ok_or(EmbeddedError::StackUnderflow)
    }

    /// Read u8 from bytecode
    fn read_u8(&mut self, code: &[u8]) -> EmbeddedResult<u8> {
        if self.ip >= code.len() {
            return Err(EmbeddedError::InvalidBytecode);
        }
        let byte = code[self.ip];
        self.ip += 1;
        Ok(byte)
    }

    /// Read i32 from bytecode (little-endian)
    fn read_i32(&mut self, code: &[u8]) -> EmbeddedResult<i32> {
        if self.ip + 4 > code.len() {
            return Err(EmbeddedError::InvalidBytecode);
        }
        let bytes = [
            code[self.ip],
            code[self.ip + 1],
            code[self.ip + 2],
            code[self.ip + 3],
        ];
        self.ip += 4;
        Ok(i32::from_le_bytes(bytes))
    }

    /// Read f32 from bytecode (little-endian)
    fn read_f32(&mut self, code: &[u8]) -> EmbeddedResult<f32> {
        if self.ip + 4 > code.len() {
            return Err(EmbeddedError::InvalidBytecode);
        }
        let bytes = [
            code[self.ip],
            code[self.ip + 1],
            code[self.ip + 2],
            code[self.ip + 3],
        ];
        self.ip += 4;
        Ok(f32::from_le_bytes(bytes))
    }

    /// Read u16 from bytecode (little-endian) for jump offsets
    fn read_u16(&mut self, code: &[u8]) -> EmbeddedResult<u16> {
        if self.ip + 2 > code.len() {
            return Err(EmbeddedError::InvalidBytecode);
        }
        let bytes = [code[self.ip], code[self.ip + 1]];
        self.ip += 2;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Execute bytecode
    pub fn run(&mut self, code: &[u8]) -> EmbeddedResult<EmbeddedValue> {
        self.reset();
        self.running = true;

        while self.running && self.ip < code.len() {
            let opcode_byte = self.read_u8(code)?;
            let opcode = EmbeddedOpCode::from_byte(opcode_byte)?;

            match opcode {
                EmbeddedOpCode::PushNull => {
                    self.push(EmbeddedValue::Null)?;
                }
                EmbeddedOpCode::PushInt => {
                    let value = self.read_i32(code)?;
                    self.push(EmbeddedValue::Int(value))?;
                }
                EmbeddedOpCode::PushFloat => {
                    let value = self.read_f32(code)?;
                    self.push(EmbeddedValue::Float(value))?;
                }
                EmbeddedOpCode::PushTrue => {
                    self.push(EmbeddedValue::Bool(true))?;
                }
                EmbeddedOpCode::PushFalse => {
                    self.push(EmbeddedValue::Bool(false))?;
                }
                EmbeddedOpCode::Pop => {
                    self.pop()?;
                }
                EmbeddedOpCode::Dup => {
                    let value = self.peek()?;
                    self.push(value)?;
                }
                EmbeddedOpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (a, b) {
                        (EmbeddedValue::Int(x), EmbeddedValue::Int(y)) => {
                            EmbeddedValue::Int(x.wrapping_add(y))
                        }
                        (EmbeddedValue::Float(x), EmbeddedValue::Float(y)) => {
                            EmbeddedValue::Float(x + y)
                        }
                        (EmbeddedValue::Int(x), EmbeddedValue::Float(y)) => {
                            EmbeddedValue::Float(x as f32 + y)
                        }
                        (EmbeddedValue::Float(x), EmbeddedValue::Int(y)) => {
                            EmbeddedValue::Float(x + y as f32)
                        }
                        _ => EmbeddedValue::Null,
                    };
                    self.push(result)?;
                }
                EmbeddedOpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (a, b) {
                        (EmbeddedValue::Int(x), EmbeddedValue::Int(y)) => {
                            EmbeddedValue::Int(x.wrapping_sub(y))
                        }
                        (EmbeddedValue::Float(x), EmbeddedValue::Float(y)) => {
                            EmbeddedValue::Float(x - y)
                        }
                        _ => EmbeddedValue::Null,
                    };
                    self.push(result)?;
                }
                EmbeddedOpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (a, b) {
                        (EmbeddedValue::Int(x), EmbeddedValue::Int(y)) => {
                            EmbeddedValue::Int(x.wrapping_mul(y))
                        }
                        (EmbeddedValue::Float(x), EmbeddedValue::Float(y)) => {
                            EmbeddedValue::Float(x * y)
                        }
                        _ => EmbeddedValue::Null,
                    };
                    self.push(result)?;
                }
                EmbeddedOpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (a, b) {
                        (EmbeddedValue::Int(x), EmbeddedValue::Int(y)) => {
                            if y == 0 {
                                return Err(EmbeddedError::DivisionByZero);
                            }
                            EmbeddedValue::Int(x / y)
                        }
                        (EmbeddedValue::Float(x), EmbeddedValue::Float(y)) => {
                            if y == 0.0 {
                                return Err(EmbeddedError::DivisionByZero);
                            }
                            EmbeddedValue::Float(x / y)
                        }
                        _ => EmbeddedValue::Null,
                    };
                    self.push(result)?;
                }
                EmbeddedOpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(EmbeddedValue::Bool(a == b))?;
                }
                EmbeddedOpCode::Lt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (a, b) {
                        (EmbeddedValue::Int(x), EmbeddedValue::Int(y)) => x < y,
                        (EmbeddedValue::Float(x), EmbeddedValue::Float(y)) => x < y,
                        _ => false,
                    };
                    self.push(EmbeddedValue::Bool(result))?;
                }
                EmbeddedOpCode::Gt => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = match (a, b) {
                        (EmbeddedValue::Int(x), EmbeddedValue::Int(y)) => x > y,
                        (EmbeddedValue::Float(x), EmbeddedValue::Float(y)) => x > y,
                        _ => false,
                    };
                    self.push(EmbeddedValue::Bool(result))?;
                }
                EmbeddedOpCode::Not => {
                    let value = self.pop()?;
                    self.push(EmbeddedValue::Bool(!value.is_truthy()))?;
                }
                EmbeddedOpCode::LoadLocal => {
                    let index = self.read_u8(code)? as usize;
                    let value = self
                        .locals
                        .get(index)
                        .copied()
                        .unwrap_or(EmbeddedValue::Null);
                    self.push(value)?;
                }
                EmbeddedOpCode::StoreLocal => {
                    let index = self.read_u8(code)? as usize;
                    let value = self.pop()?;
                    // Extend locals if needed
                    while self.locals.len() <= index {
                        let _ = self.locals.push(EmbeddedValue::Null);
                    }
                    if index < self.locals.len() {
                        self.locals[index] = value;
                    }
                }
                EmbeddedOpCode::Jump => {
                    let offset = self.read_u16(code)? as usize;
                    self.ip = offset;
                }
                EmbeddedOpCode::JumpIfFalse => {
                    let offset = self.read_u16(code)? as usize;
                    let condition = self.pop()?;
                    if !condition.is_truthy() {
                        self.ip = offset;
                    }
                }
                EmbeddedOpCode::Halt => {
                    self.running = false;
                }
            }
        }

        // Return top of stack or Null
        Ok(self.stack.last().copied().unwrap_or(EmbeddedValue::Null))
    }
}

impl Default for EmbeddedVm {
    fn default() -> Self {
        Self::new(EmbeddedConfig::default())
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EmbeddedConfig::default();
        assert_eq!(config.stack_size, 64);
        assert_eq!(config.opon_size, 256);
    }

    #[test]
    fn test_minimal_config() {
        let config = EmbeddedConfig::minimal();
        assert_eq!(config.stack_size, 16);
        assert_eq!(config.opon_size, 32);
    }

    #[test]
    fn test_vm_push_int_halt() {
        let mut vm = EmbeddedVm::default();
        // PushInt(42), Halt
        let bytecode = [
            0x01, // PushInt
            42, 0, 0, 0,    // 42 as i32 little-endian
            0xFF, // Halt
        ];
        let result = vm.run(&bytecode).unwrap();
        assert_eq!(result, EmbeddedValue::Int(42));
    }

    #[test]
    fn test_vm_add() {
        let mut vm = EmbeddedVm::default();
        // PushInt(10), PushInt(32), Add, Halt
        let bytecode = [
            0x01, 10, 0, 0, 0, // PushInt(10)
            0x01, 32, 0, 0, 0,    // PushInt(32)
            0x20, // Add
            0xFF, // Halt
        ];
        let result = vm.run(&bytecode).unwrap();
        assert_eq!(result, EmbeddedValue::Int(42));
    }

    #[test]
    fn test_vm_div_by_zero() {
        let mut vm = EmbeddedVm::default();
        // PushInt(10), PushInt(0), Div
        let bytecode = [
            0x01, 10, 0, 0, 0, // PushInt(10)
            0x01, 0, 0, 0, 0,    // PushInt(0)
            0x23, // Div
        ];
        let result = vm.run(&bytecode);
        assert!(matches!(result, Err(EmbeddedError::DivisionByZero)));
    }

    #[test]
    fn test_vm_comparison() {
        let mut vm = EmbeddedVm::default();
        // PushInt(5), PushInt(10), Lt, Halt -> true
        let bytecode = [
            0x01, 5, 0, 0, 0, // PushInt(5)
            0x01, 10, 0, 0, 0,    // PushInt(10)
            0x32, // Lt
            0xFF, // Halt
        ];
        let result = vm.run(&bytecode).unwrap();
        assert_eq!(result, EmbeddedValue::Bool(true));
    }

    #[test]
    fn test_vm_local_variables() {
        let mut vm = EmbeddedVm::default();
        // PushInt(100), StoreLocal(0), PushInt(0), LoadLocal(0), Halt
        let bytecode = [
            0x01, 100, 0, 0, 0, // PushInt(100)
            0x51, 0, // StoreLocal(0)
            0x01, 0, 0, 0, 0, // PushInt(0)
            0x50, 0,    // LoadLocal(0)
            0xFF, // Halt
        ];
        let result = vm.run(&bytecode).unwrap();
        assert_eq!(result, EmbeddedValue::Int(100));
    }

    #[test]
    fn test_embedded_value_truthy() {
        assert!(!EmbeddedValue::Null.is_truthy());
        assert!(!EmbeddedValue::Bool(false).is_truthy());
        assert!(EmbeddedValue::Bool(true).is_truthy());
        assert!(!EmbeddedValue::Int(0).is_truthy());
        assert!(EmbeddedValue::Int(1).is_truthy());
        assert!(!EmbeddedValue::Float(0.0).is_truthy());
        assert!(EmbeddedValue::Float(3.14).is_truthy());
    }
}
