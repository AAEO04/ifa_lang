#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Target {
    /// Full desktop/server — 88 opcodes, full stdlib, OS features
    #[default]
    Native,
    /// Browser WASM — same language, but constrained sandbox
    Wasm,
    /// Tier 0 (bare-metal MCU) — no_std, no alloc, fixed stack, HAL traits
    EmbeddedTier0,
    /// Tier 1 (RTOS/IoT) — alloc + strings/JSON/storage/GPIO abstractions
    EmbeddedTier1,
}

impl Target {
    pub fn allows_strings(&self) -> bool {
        !matches!(self, Target::EmbeddedTier0)
    }
    pub fn allows_closures(&self) -> bool {
        matches!(self, Target::Native | Target::Wasm)
    }
    pub fn allows_collections(&self) -> bool {
        !self.is_embedded()
    }
    pub fn allows_imports(&self) -> bool {
        !self.is_embedded()
    }
    pub fn allows_odu_domains(&self) -> bool {
        matches!(self, Target::Native | Target::Wasm)
    }
    pub fn allows_exceptions(&self) -> bool {
        matches!(self, Target::Native | Target::Wasm)
    }
    pub fn allows_async(&self) -> bool {
        matches!(self, Target::Native | Target::Wasm)
    }
    pub fn is_embedded(&self) -> bool {
        matches!(self, Target::EmbeddedTier0 | Target::EmbeddedTier1)
    }
    pub fn max_local_index(&self) -> u16 {
        if self.is_embedded() { 255 } else { u16::MAX }
    }
    pub fn max_int_bits(&self) -> u8 {
        if self.is_embedded() { 32 } else { 64 }
    }
}
