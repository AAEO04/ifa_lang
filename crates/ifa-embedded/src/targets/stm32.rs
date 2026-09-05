// Intentionally dormant until embedded CI is standing
//! STM32 Target Support
//!
//! MMIO base: 0x40000000
//! Flashing: openocd or st-flash

use crate::{DelayUs, InputPin, MmioBus, OutputPin, Serial};

pub const STM32_MMIO_BASE: u32 = 0x4000_0000;
pub const STM32_DEFAULT_BAUD: u32 = 115200;
pub const STM32_CLOCK_HZ: u32 = 168_000_000; // 168 MHz (typical F4 clock)

pub struct Stm32Target {
    pub mmio_base: u32,
    pub clock_hz: u32,
    pub baud_rate: u32,
    pub variant: Stm32Variant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stm32Variant {
    F1,
    F4,
    H7,
}

impl Default for Stm32Target {
    fn default() -> Self {
        Self {
            mmio_base: STM32_MMIO_BASE,
            clock_hz: STM32_CLOCK_HZ,
            baud_rate: STM32_DEFAULT_BAUD,
            variant: Stm32Variant::F4,
        }
    }
}

pub struct Stm32GpioPin {
    pin: u8,
    is_output: bool,
    high: bool,
}

impl Stm32GpioPin {
    pub fn new(pin: u8, is_output: bool) -> Self {
        Self {
            pin,
            is_output,
            high: false,
        }
    }
}

impl InputPin for Stm32GpioPin {
    fn is_high(&self) -> bool {
        self.high
    }
}

impl OutputPin for Stm32GpioPin {
    fn set_high(&mut self) {
        if self.is_output {
            self.high = true;
        }
    }

    fn set_low(&mut self) {
        if self.is_output {
            self.high = false;
        }
    }

    fn toggle(&mut self) {
        if self.is_output {
            self.high = !self.high;
        }
    }
}

pub struct Stm32Serial {
    baud: u32,
    tx_buffer: alloc::vec::Vec<u8>,
}

impl Stm32Serial {
    pub fn new(baud: u32) -> Self {
        Self {
            baud,
            tx_buffer: alloc::vec::Vec::new(),
        }
    }
}

impl Serial for Stm32Serial {
    fn write_byte(&mut self, byte: u8) -> Result<(), crate::EmbeddedError> {
        self.tx_buffer.push(byte);
        Ok(())
    }

    fn read_byte(&mut self) -> Option<u8> {
        None
    }

    fn available(&self) -> bool {
        false
    }
}

pub struct Stm32Delay;

impl DelayUs for Stm32Delay {
    fn delay_us(&mut self, _us: u32) {
        // Stub delay
    }
}

pub struct Stm32MmioBus {
    base: u32,
}

impl Default for Stm32MmioBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Stm32MmioBus {
    pub fn new() -> Self {
        Self {
            base: STM32_MMIO_BASE,
        }
    }
}

impl MmioBus for Stm32MmioBus {
    fn read(&mut self, _addr: u32) -> u32 {
        0
    }

    fn write(&mut self, _addr: u32, _val: u32) {}
}
