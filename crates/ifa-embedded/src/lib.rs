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

// Embedded optimizations with Ikin & Iroke
// Embedded optimizations with Ikin & Iroke
pub mod embedded_ikin;
pub mod embedded_iroke;

// --- IoT Extensions (Tier 1) ---
#[cfg(feature = "iot")]
pub mod iot;

#[cfg(feature = "storage")]
pub mod storage;

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

/// VM Exit Status
#[derive(Debug, Clone, Copy, PartialEq)] // Added PartialEq for tests
pub enum VmExit {
    /// Program halted normally
    Halted(EmbeddedValue),
    /// Execution yielded (can be resumed)
    /// Parameter is duration hint in microseconds
    Yield(u32),
}

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

/// MMIO Bus trait for hardware register access
/// MMIO Bus trait for hardware register access
pub trait MmioBus {
    /// Read 32-bit value from address
    fn read(&mut self, addr: u32) -> u32;
    /// Read 16-bit value
    fn read_u16(&mut self, addr: u32) -> u16 {
        self.read(addr) as u16
    }
    /// Read 8-bit value
    fn read_u8(&mut self, addr: u32) -> u8 {
        self.read(addr) as u8
    }

    /// Write 32-bit value to address
    fn write(&mut self, addr: u32, val: u32);
    /// Write 16-bit value
    fn write_u16(&mut self, addr: u32, val: u16) {
        self.write(addr, val as u32);
    }
    /// Write 8-bit value
    fn write_u8(&mut self, addr: u32, val: u8) {
        self.write(addr, val as u32);
    }
}

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Embedded VM configuration
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedConfig {
    /// Base address for MMIO (addresses >= this are routed to bus)
    pub mmio_base: u32,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        EmbeddedConfig {
            mmio_base: 0x4000_0000,
        }
    }
}

impl EmbeddedConfig {
    /// Minimal config for very constrained devices (< 2KB RAM)
    pub fn minimal() -> Self {
        EmbeddedConfig {
            mmio_base: 0, // No MMIO on minimal
        }
    }

    /// Standard config for typical MCUs (8-32KB RAM)
    pub fn standard() -> Self {
        EmbeddedConfig {
            mmio_base: 0x4000_0000,
        }
    }

    /// Extended config for capable devices (64KB+ RAM)
    pub fn extended() -> Self {
        EmbeddedConfig {
            mmio_base: 0x4000_0000,
        }
    }
}

// =============================================================================
// EMBEDDED VALUE (Simplified for no_std)
// =============================================================================

// Adaptive Integer/Float types for 32/64 bit support
#[cfg(any(target_pointer_width = "64", feature = "force_64bit"))]
pub type IfaInt = i64;
#[cfg(any(target_pointer_width = "64", feature = "force_64bit"))]
pub type IfaFloat = f64;

#[cfg(not(any(target_pointer_width = "64", feature = "force_64bit")))]
pub type IfaInt = i32;
#[cfg(not(any(target_pointer_width = "64", feature = "force_64bit")))]
pub type IfaFloat = f32;

/// Simplified value type for embedded contexts
#[derive(Debug, Clone, PartialEq, Default)]
// Removed Copy if Alloc is used? No, must check.
// If we add String (which is not Copy), EmbeddedValue cannot be Copy.
// This is a breaking change for stack-only usage if we force it.
// SOLUTION: Derive Copy only if alloc feature is NOT enabled OR manually manage it?
// Rust requirement: Enum can only be Copy if all variants are Copy.
// So if 'alloc' is on, EmbeddedValue is NOT Copy.
// This changes semantics of passing by value vs reference.
// FOR NOW: We keep it Copy-compatible if alloc is off.
#[cfg_attr(not(feature = "alloc"), derive(Copy))]
pub enum EmbeddedValue {
    /// Null/None
    #[default]
    Null,
    /// Boolean
    Bool(bool),
    /// Integer (32 or 64 bit)
    Int(IfaInt),
    /// Float (32 or 64 bit)
    Float(IfaFloat),
    /// Pointer to Opon memory address (index)
    Ptr(u32),

    // --- Tier 1 (Alloc) Variants ---
    #[cfg(feature = "alloc")]
    String(String),
    #[cfg(feature = "alloc")]
    Blob(alloc::vec::Vec<u8>),
}

impl EmbeddedValue {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            EmbeddedValue::Null => false,
            EmbeddedValue::Bool(b) => *b,
            EmbeddedValue::Int(n) => *n != 0 as IfaInt,
            // D8: NaN is falsy per spec §5
            EmbeddedValue::Float(f) => *f != 0.0 && !f.is_nan(),
            // D7: Null pointer (0x0) is falsy per spec §5
            EmbeddedValue::Ptr(p) => *p != 0,
            #[cfg(feature = "alloc")]
            EmbeddedValue::String(s) => !s.is_empty(),
            #[cfg(feature = "alloc")]
            EmbeddedValue::Blob(b) => !b.is_empty(),
        }
    }
}

// Need to handle Copy semantics in VM if not Copy
// Manual Copy implementation removed to avoid conflict with derive(Copy)
// impl Copy for EmbeddedValue where String: Copy, alloc::vec::Vec<u8>: Copy {}
// If alloc is enabled, EmbeddedValue is Clone-only.
// VM operations that did `self.stack.push(val)` where val is Copy will work fine (Move).
// But `peek().copied()` will fail.
// We need to update VM to use `cloned()` instead of `copied()`.

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

    // ===================================
    // POINTER OPS (Address 0xA0 base)
    // ===================================
    /// Push Ref/Address (followed by 4 byte address/index)
    /// In full Ifa, &x gets ref. Here we might just push raw pointers.
    Ref = 0xA0,
    /// Dereference (Read): Pop addr -> Push value
    Deref = 0xA1,
    /// Store Dereference (Write): Pop addr, Pop val -> Write val to address (32-bit)
    StoreDeref = 0xA2,

    // Sized pointer ops
    /// Store 8-bit value to address
    Store8 = 0xA3,
    /// Store 16-bit value to address
    Store16 = 0xA4,
    /// Read 8-bit value from address
    Load8 = 0xA5,
    /// Read 16-bit value from address
    Load16 = 0xA6,

    /// Yield execution (pause without resetting)
    /// Followed by 4 byte duration hint (u32 microseconds), or 0 for indefinite
    Yield = 0xF0,
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
            0xA0 => Ok(EmbeddedOpCode::Ref),
            0xA1 => Ok(EmbeddedOpCode::Deref),
            0xA2 => Ok(EmbeddedOpCode::StoreDeref),
            0xA3 => Ok(EmbeddedOpCode::Store8),
            0xA4 => Ok(EmbeddedOpCode::Store16),
            0xA5 => Ok(EmbeddedOpCode::Load8),
            0xA6 => Ok(EmbeddedOpCode::Load16),
            0xF0 => Ok(EmbeddedOpCode::Yield),
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
/// Minimal embedded VM with fixed-size stack
///
/// Uses heapless collections for deterministic memory usage.
/// Minimal embedded VM with fixed-size stack
///
/// Uses heapless collections for deterministic memory usage.
pub struct EmbeddedVm<'a, const OPON_SIZE: usize, const STACK_SIZE: usize> {
    /// Value stack (fixed size)
    stack: HVec<EmbeddedValue, STACK_SIZE>,
    /// Local variables (fixed size)
    locals: HVec<EmbeddedValue, 32>,
    /// Opon (Memory) for pointer operations (fixed size)
    opon: HVec<EmbeddedValue, OPON_SIZE>,
    /// Instruction pointer
    ip: usize,
    /// Running flag
    running: bool,
    /// Check for MMIO
    config: EmbeddedConfig,
    /// Optional MMIO bus
    mmio: Option<&'a mut dyn MmioBus>,
}

impl<'a, const OPON_SIZE: usize, const STACK_SIZE: usize> EmbeddedVm<'a, OPON_SIZE, STACK_SIZE> {
    /// Create a new embedded VM
    pub fn new(config: EmbeddedConfig) -> Self {
        EmbeddedVm {
            stack: HVec::new(),
            locals: HVec::new(),
            opon: HVec::new(),
            ip: 0,
            running: false,
            config,
            mmio: None,
        }
    }

    /// Attach MMIO bus
    pub fn attach_mmio(&mut self, bus: &'a mut dyn MmioBus) {
        self.mmio = Some(bus);
    }

    /// Reset VM state
    pub fn reset(&mut self) {
        self.stack.clear();
        self.locals.clear();
        self.opon.clear();
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
            .cloned()
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

    /// Start execution from beginning
    /// Resets VM state.
    pub fn start(&mut self, code: &[u8]) -> EmbeddedResult<VmExit> {
        self.reset();
        self.resume_with_hook(code, || false)
    }

    /// Resume execution (legacy)
    pub fn resume(&mut self, code: &[u8]) -> EmbeddedResult<VmExit> {
        self.resume_with_hook(code, || false)
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

    /// Read u32 (for addresses) from bytecode (little-endian)
    fn read_u32(&mut self, code: &[u8]) -> EmbeddedResult<u32> {
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
        Ok(u32::from_le_bytes(bytes))
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

    /// Execute bytecode (Legacy API, returns Result<EmbeddedValue>)
    /// If program yields, this will return Ok(Null) which might be confusing.
    /// Used for existing tests.
    #[deprecated(note = "Use start() which returns VmExit")]
    pub fn run(&mut self, code: &[u8]) -> EmbeddedResult<EmbeddedValue> {
        match self.start(code)? {
            VmExit::Halted(val) => Ok(val),
            VmExit::Yield(_) => Ok(EmbeddedValue::Null), // Lossy conversion for compat
        }
    }

    /// Resume execution with a polling hook
    /// The hook returns `true` if execution should yield immediately.
    pub fn resume_with_hook<F>(&mut self, code: &[u8], mut hook: F) -> EmbeddedResult<VmExit>
    where
        F: FnMut() -> bool,
    {
        self.running = true;

        while self.running && self.ip < code.len() {
            // Check hook (Iroke Polling)
            if hook() {
                return Ok(VmExit::Yield(0));
            }

            let opcode_byte = self.read_u8(code)?;
            let opcode = EmbeddedOpCode::from_byte(opcode_byte)?;

            match opcode {
                EmbeddedOpCode::PushNull => {
                    self.push(EmbeddedValue::Null)?;
                }
                EmbeddedOpCode::PushInt => {
                    let value = self.read_i32(code)?;
                    self.push(EmbeddedValue::Int(value as IfaInt))?;
                }
                EmbeddedOpCode::PushFloat => {
                    let value = self.read_f32(code)?;
                    self.push(EmbeddedValue::Float(value as IfaFloat))?;
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
                            EmbeddedValue::Float(x as IfaFloat + y)
                        }
                        (EmbeddedValue::Float(x), EmbeddedValue::Int(y)) => {
                            EmbeddedValue::Float(x + y as IfaFloat)
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
                        .cloned()
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
                EmbeddedOpCode::Ref => {
                    // Push pointer to address
                    let addr = self.read_u32(code)?;
                    self.push(EmbeddedValue::Ptr(addr))?;
                }
                EmbeddedOpCode::Deref => {
                    let addr_val = self.pop()?;
                    if let EmbeddedValue::Ptr(addr) = addr_val {
                        // Check if MMIO
                        if addr >= self.config.mmio_base {
                            if let Some(bus) = &mut self.mmio {
                                let val = bus.read(addr);
                                self.push(EmbeddedValue::Int(val as IfaInt))?;
                            } else {
                                return Err(EmbeddedError::HalError(
                                    "MMIO bus not attached".into(),
                                ));
                            }
                        } else {
                            let val = self
                                .opon
                                .get(addr as usize)
                                .cloned()
                                .unwrap_or(EmbeddedValue::Null);
                            self.push(val)?;
                        }
                    } else {
                        return Err(EmbeddedError::HalError("Deref requires Ptr".into()));
                    }
                }
                EmbeddedOpCode::StoreDeref => {
                    let addr_val = self.pop()?;
                    let val = self.pop()?;

                    #[cfg(test)]
                    {
                        use std::println;
                        println!("StoreDeref Debug:");
                        println!("  Stack Top (Addr): {:?}", addr_val);
                        println!("  Stack Next (Val): {:?}", val);
                        println!("  Remaining Stack: {:?}", self.stack);
                    }

                    if let EmbeddedValue::Ptr(addr) = addr_val {
                        // Check if MMIO
                        if addr >= self.config.mmio_base {
                            if let Some(bus) = &mut self.mmio {
                                // Convert value to u32
                                let bus_val = match val {
                                    EmbeddedValue::Int(i) => i as u32,
                                    EmbeddedValue::Float(f) => f as u32, // Bitcast or cast? Casting for now
                                    EmbeddedValue::Bool(b) => {
                                        if b {
                                            1
                                        } else {
                                            0
                                        }
                                    }
                                    _ => 0,
                                };
                                bus.write(addr, bus_val);
                            } else {
                                return Err(EmbeddedError::HalError(
                                    "MMIO bus not attached".into(),
                                ));
                            }
                        } else {
                            let idx = addr as usize;
                            // Auto-grow opon if needed/possible (within OPON_SIZE limit)
                            if idx >= self.opon.len() {
                                // Check if within capacity
                                if idx < self.opon.capacity() {
                                    while self.opon.len() <= idx {
                                        let _ = self.opon.push(EmbeddedValue::Null);
                                    }
                                } else {
                                    return Err(EmbeddedError::MemoryOutOfBounds);
                                }
                            }
                            self.opon[idx] = val;
                        }
                    } else {
                        #[cfg(test)]
                        {
                            use std::println;
                            println!("ERROR: StoreDeref expected Ptr, got {:?}", addr_val);
                        }
                        return Err(EmbeddedError::HalError("StoreDeref requires Ptr".into()));
                    }
                }
                EmbeddedOpCode::Store8 | EmbeddedOpCode::Store16 => {
                    let addr_val = self.pop()?;
                    let val = self.pop()?;
                    if let EmbeddedValue::Ptr(addr) = addr_val {
                        // Check if MMIO
                        if addr >= self.config.mmio_base {
                            if let Some(bus) = &mut self.mmio {
                                let int_val = match val {
                                    EmbeddedValue::Int(i) => i as u32,
                                    EmbeddedValue::Float(f) => f as u32,
                                    EmbeddedValue::Bool(b) => {
                                        if b {
                                            1
                                        } else {
                                            0
                                        }
                                    }
                                    _ => 0,
                                };

                                if opcode == EmbeddedOpCode::Store8 {
                                    bus.write_u8(addr, int_val as u8);
                                } else {
                                    bus.write_u16(addr, int_val as u16);
                                }
                            } else {
                                return Err(EmbeddedError::HalError(
                                    "MMIO bus not attached".into(),
                                ));
                            }
                        } else {
                            // Writing sub-word to RAM?
                            // Opon is an array of 32-bit Values. We can't easily write 8 bits to it without bitmasking.
                            // But usually Store8/16 is ONLY for MMIO registers.
                            // If user tries to do it on RAM, we can either error or just do a full write (ignoring size).
                            // Let's do full write but truncated value to simulate behavior.
                            let idx = addr as usize;
                            if idx >= self.opon.len() {
                                if idx < self.opon.capacity() {
                                    while self.opon.len() <= idx {
                                        let _ = self.opon.push(EmbeddedValue::Null);
                                    }
                                } else {
                                    return Err(EmbeddedError::MemoryOutOfBounds);
                                }
                            }

                            let truncated = match val {
                                EmbeddedValue::Int(i) => {
                                    if opcode == EmbeddedOpCode::Store8 {
                                        EmbeddedValue::Int((i & 0xFF) as IfaInt)
                                    } else {
                                        EmbeddedValue::Int((i & 0xFFFF) as IfaInt)
                                    }
                                }
                                v => v,
                            };
                            self.opon[idx] = truncated;
                        }
                    } else {
                        return Err(EmbeddedError::HalError("Store8/16 requires Ptr".into()));
                    }
                }
                EmbeddedOpCode::Load8 | EmbeddedOpCode::Load16 => {
                    let addr_val = self.pop()?;
                    if let EmbeddedValue::Ptr(addr) = addr_val {
                        if addr >= self.config.mmio_base {
                            if let Some(bus) = &mut self.mmio {
                                let val = if opcode == EmbeddedOpCode::Load8 {
                                    bus.read_u8(addr) as u32
                                } else {
                                    bus.read_u16(addr) as u32
                                };
                                self.push(EmbeddedValue::Int(val as IfaInt))?;
                            } else {
                                return Err(EmbeddedError::HalError(
                                    "MMIO bus not attached".into(),
                                ));
                            }
                        } else {
                            // Read from RAM - just return full value
                            let val = self
                                .opon
                                .get(addr as usize)
                                .cloned()
                                .unwrap_or(EmbeddedValue::Null);
                            self.push(val)?;
                        }
                    } else {
                        return Err(EmbeddedError::HalError("Deref8/16 requires Ptr".into()));
                    }
                }
                EmbeddedOpCode::Yield => {
                    let duration = self.read_u32(code)?;
                    self.running = false; // Stop loop, but don't reset
                    return Ok(VmExit::Yield(duration));
                }
            }
        }

        // Return top of stack or Null
        Ok(VmExit::Halted(
            self.stack.last().cloned().unwrap_or(EmbeddedValue::Null),
        ))
    }
}

impl<'a, const OPON_SIZE: usize, const STACK_SIZE: usize> Default
    for EmbeddedVm<'a, OPON_SIZE, STACK_SIZE>
{
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
        // assert_eq!(config.stack_size, 64); // Removed
        assert_eq!(config.mmio_base, 0x4000_0000);
    }

    #[test]
    fn test_minimal_config() {
        let config = EmbeddedConfig::minimal();
        // assert_eq!(config.stack_size, 16); // Removed
        assert_eq!(config.mmio_base, 0);
    }

    #[test]
    fn test_vm_push_int_halt() {
        let mut vm = EmbeddedVm::<256, 64>::default();
        // PushInt(42), Halt
        let bytecode = [
            0x01, // PushInt
            42, 0, 0, 0,    // 42 as i32 little-endian
            0xFF, // Halt
        ];
        let result = vm.start(&bytecode).unwrap();
        assert_eq!(result, VmExit::Halted(EmbeddedValue::Int(42)));
    }

    #[test]
    fn test_vm_add() {
        let mut vm = EmbeddedVm::<256, 64>::default();
        // PushInt(10), PushInt(32), Add, Halt
        let bytecode = [
            0x01, 10, 0, 0, 0, // PushInt(10)
            0x01, 32, 0, 0, 0,    // PushInt(32)
            0x20, // Add
            0xFF, // Halt
        ];
        let result = vm.start(&bytecode).unwrap();
        assert_eq!(result, VmExit::Halted(EmbeddedValue::Int(42)));
    }

    #[test]
    fn test_vm_div_by_zero() {
        let mut vm = EmbeddedVm::<256, 64>::default();
        // PushInt(10), PushInt(0), Div
        let bytecode = [
            0x01, 10, 0, 0, 0, // PushInt(10)
            0x01, 0, 0, 0, 0,    // PushInt(0)
            0x23, // Div
        ];
        let result = vm.start(&bytecode);
        assert!(matches!(result, Err(EmbeddedError::DivisionByZero)));
    }

    #[test]
    fn test_vm_comparison() {
        let mut vm = EmbeddedVm::<256, 64>::default();
        // PushInt(5), PushInt(10), Lt, Halt -> true
        let bytecode = [
            0x01, 5, 0, 0, 0, // PushInt(5)
            0x01, 10, 0, 0, 0,    // PushInt(10)
            0x32, // Lt
            0xFF, // Halt
        ];
        let result = vm.start(&bytecode).unwrap();
        assert_eq!(result, VmExit::Halted(EmbeddedValue::Bool(true)));
    }

    #[test]
    fn test_vm_local_variables() {
        let mut vm = EmbeddedVm::<256, 64>::default();
        // PushInt(100), StoreLocal(0), PushInt(0), LoadLocal(0), Halt
        let bytecode = [
            0x01, 100, 0, 0, 0, // PushInt(100)
            0x51, 0, // StoreLocal(0)
            0x01, 0, 0, 0, 0, // PushInt(0)
            0x50, 0,    // LoadLocal(0)
            0xFF, // Halt
        ];
        let result = vm.start(&bytecode).unwrap();
        assert_eq!(result, VmExit::Halted(EmbeddedValue::Int(100)));
    }

    #[test]
    fn test_embedded_value_truthy() {
        assert!(!EmbeddedValue::Null.is_truthy());
        assert!(!EmbeddedValue::Bool(false).is_truthy());
        assert!(EmbeddedValue::Bool(true).is_truthy());
        assert!(!EmbeddedValue::Int(0).is_truthy());
        assert!(EmbeddedValue::Int(1).is_truthy());
        assert!(!EmbeddedValue::Float(0.0).is_truthy());
        assert!(EmbeddedValue::Float(1.5).is_truthy());
    }
}
