// Intentionally dormant until embedded CI is standing
//! RP2040 Target Support
//!
//! MMIO base: 0x40000000
//! Flashing: picotool or uf2

use crate::{DelayUs, InputPin, MmioBus, OutputPin, Serial};

pub const RP2040_MMIO_BASE: u32 = 0x4000_0000;
pub const RP2040_DEFAULT_BAUD: u32 = 115200;
pub const RP2040_CLOCK_HZ: u32 = 133_000_000; // 133 MHz

pub struct Rp2040Target {
    pub mmio_base: u32,
    pub clock_hz: u32,
    pub baud_rate: u32,
    pub dual_core: bool,
}

impl Default for Rp2040Target {
    fn default() -> Self {
        Self {
            mmio_base: RP2040_MMIO_BASE,
            clock_hz: RP2040_CLOCK_HZ,
            baud_rate: RP2040_DEFAULT_BAUD,
            dual_core: true,
        }
    }
}

pub struct Rp2040GpioPin {
    pin: u8,
    is_output: bool,
    high: bool,
}

impl Rp2040GpioPin {
    pub fn new(pin: u8, is_output: bool) -> Self {
        Self {
            pin,
            is_output,
            high: false,
        }
    }
}

impl InputPin for Rp2040GpioPin {
    fn is_high(&self) -> bool {
        self.high
    }
}

impl OutputPin for Rp2040GpioPin {
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

pub struct Rp2040Serial {
    baud: u32,
    tx_buffer: alloc::vec::Vec<u8>,
}

impl Rp2040Serial {
    pub fn new(baud: u32) -> Self {
        Self {
            baud,
            tx_buffer: alloc::vec::Vec::new(),
        }
    }
}

impl Serial for Rp2040Serial {
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

pub struct Rp2040Delay;

impl DelayUs for Rp2040Delay {
    fn delay_us(&mut self, _us: u32) {
        // Stub delay
    }
}

pub struct Rp2040MmioBus {
    base: u32,
}

impl Default for Rp2040MmioBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Rp2040MmioBus {
    pub fn new() -> Self {
        Self {
            base: RP2040_MMIO_BASE,
        }
    }
}

impl MmioBus for Rp2040MmioBus {
    fn read(&mut self, _addr: u32) -> u32 {
        0
    }

    fn write(&mut self, _addr: u32, _val: u32) {}
}
