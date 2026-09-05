// Intentionally dormant until embedded CI is standing
//! # ESP32 Target Implementation
//!
//! MMIO base: 0x3FF00000
//! Flashing: esptool.py write_flash

use crate::{DelayUs, InputPin, MmioBus, OutputPin, Serial};

pub const ESP32_MMIO_BASE: u32 = 0x3FF0_0000;
pub const ESP32_DEFAULT_BAUD: u32 = 115200;
pub const ESP32_CLOCK_HZ: u32 = 240_000_000; // 240 MHz

pub struct Esp32Target {
    pub mmio_base: u32,
    pub clock_hz: u32,
    pub baud_rate: u32,
}

impl Default for Esp32Target {
    fn default() -> Self {
        Self {
            mmio_base: ESP32_MMIO_BASE,
            clock_hz: ESP32_CLOCK_HZ,
            baud_rate: ESP32_DEFAULT_BAUD,
        }
    }
}

pub struct Esp32GpioPin {
    pin: u8,
    is_output: bool,
    high: bool,
}

impl Esp32GpioPin {
    pub fn new(pin: u8, is_output: bool) -> Self {
        Self {
            pin,
            is_output,
            high: false,
        }
    }
}

impl InputPin for Esp32GpioPin {
    fn is_high(&self) -> bool {
        self.high
    }
}

impl OutputPin for Esp32GpioPin {
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

pub struct Esp32Serial {
    baud: u32,
    tx_buffer: alloc::vec::Vec<u8>,
}

impl Esp32Serial {
    pub fn new(baud: u32) -> Self {
        Self {
            baud,
            tx_buffer: alloc::vec::Vec::new(),
        }
    }
}

impl Serial for Esp32Serial {
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

pub struct Esp32Delay;

impl DelayUs for Esp32Delay {
    fn delay_us(&mut self, _us: u32) {
        // Stub delay
    }
}

pub struct Esp32MmioBus {
    base: u32,
}

impl Default for Esp32MmioBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Esp32MmioBus {
    pub fn new() -> Self {
        Self {
            base: ESP32_MMIO_BASE,
        }
    }
}

impl MmioBus for Esp32MmioBus {
    fn read(&mut self, _addr: u32) -> u32 {
        0
    }

    fn write(&mut self, _addr: u32, _val: u32) {}
}
