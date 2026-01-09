//! # Ọ̀wọ́nrín Domain (0011)
//!
//! The Shuffler - Random Number Generation
//!
//! Uses cryptographic RNG via ChaCha20 for security-sensitive applications.

use crate::impl_odu_domain;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha20Rng;

use ifa_sandbox::{CapabilitySet, Ofun};

/// Ọ̀wọ́nrín - The Shuffler (Random)
pub struct Owonrin {
    rng: ChaCha20Rng,
    capabilities: CapabilitySet,
}

impl_odu_domain!(Owonrin, "Ọ̀wọ́nrín", "0011", "The Shuffler - Random");

impl Default for Owonrin {
    fn default() -> Self {
        Self::new(CapabilitySet::default())
    }
}

impl Owonrin {
    /// Create with entropy-seeded CSPRNG
    pub fn new(capabilities: CapabilitySet) -> Self {
        Owonrin {
            rng: ChaCha20Rng::from_entropy(),
            capabilities,
        }
    }

    /// Create with specific seed (for reproducibility)
    pub fn from_seed(seed: u64, capabilities: CapabilitySet) -> Self {
        Owonrin {
            rng: ChaCha20Rng::seed_from_u64(seed),
            capabilities,
        }
    }

    fn check(&self) -> bool {
        self.capabilities.check(&Ofun::Random)
    }

    /// Random integer in range (pèsè)
    pub fn pese(&mut self, min: i64, max: i64) -> i64 {
        if !self.check() {
            return 0;
        }
        self.rng.gen_range(min..=max)
    }

    /// Random float 0.0 to 1.0 (pèsè_odidi)
    pub fn pese_odidi(&mut self) -> f64 {
        if !self.check() {
            return 0.0;
        }
        self.rng.r#gen()
    }

    /// Random float in range
    pub fn pese_larin(&mut self, min: f64, max: f64) -> f64 {
        if !self.check() {
            return 0.0;
        }
        self.rng.gen_range(min..=max)
    }

    /// Random boolean with probability (bóyá)
    pub fn boya(&mut self, probability: f64) -> bool {
        if !self.check() {
            return false;
        }
        self.rng.gen_bool(probability.clamp(0.0, 1.0))
    }

    /// Random choice from slice (yàn)
    pub fn yan<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if !self.check() {
            return None;
        }
        items.choose(&mut self.rng)
    }

    /// Shuffle slice in place (dàpọ̀)
    pub fn dapo<T>(&mut self, items: &mut [T]) {
        if !self.check() {
            return;
        }
        items.shuffle(&mut self.rng);
    }

    /// Generate random bytes (àwọn bytes)
    pub fn awon_bytes(&mut self, count: usize) -> Vec<u8> {
        if !self.check() {
            return vec![0; count];
        }
        (0..count).map(|_| self.rng.r#gen()).collect()
    }

    /// Generate random hex string
    pub fn hex(&mut self, bytes: usize) -> String {
        self.awon_bytes(bytes)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// Generate UUID v4
    pub fn uuid(&mut self) -> String {
        let bytes = self.awon_bytes(16);
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            (bytes[6] & 0x0f) | 0x40,
            bytes[7], // Version 4
            (bytes[8] & 0x3f) | 0x80,
            bytes[9], // Variant 1
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15]
        )
    }

    /// Weighted random choice
    pub fn yan_iwuwo<'a, T>(&mut self, items: &'a [(T, f64)]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }

        let total: f64 = items.iter().map(|(_, w)| w).sum();
        let mut threshold = self.rng.gen_range(0.0..total);

        for (item, weight) in items {
            threshold -= weight;
            if threshold <= 0.0 {
                return Some(item);
            }
        }

        items.last().map(|(item, _)| item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reproducible_seed() {
        let caps = CapabilitySet::default();
        let mut rng1 = Owonrin::from_seed(42, caps.clone());
        let mut rng2 = Owonrin::from_seed(42, caps);

        assert_eq!(rng1.pese(0, 100), rng2.pese(0, 100));
        assert_eq!(rng1.pese_odidi(), rng2.pese_odidi());
    }

    #[test]
    fn test_uuid_format() {
        let mut owonrin = Owonrin::default();
        let uuid = owonrin.uuid();
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains('-'));
    }

    #[test]
    fn test_range() {
        // Grant Random capability for this test
        let mut caps = CapabilitySet::default();
        caps.grant(Ofun::Random);
        let mut owonrin = Owonrin::new(caps);
        for _ in 0..100 {
            let n = owonrin.pese(10, 20);
            assert!(n >= 10 && n <= 20);
        }
    }
}
