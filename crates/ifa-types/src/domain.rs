use serde::{Deserialize, Serialize};

/// The canonical Odù domain taxonomy.
///
/// Three tiers:
///   1. The 16 principal Odù — each has a 4-bit binary Ifá pattern and a
///      stable VM dispatch ID (0–15). These are the only constructs
///      addressable via `Odu.method()` syntax in user code.
///   2. Infrastructure — hardware/OS abstractions managed by the runtime
///      (Cpu, Gpu, Storage, Sys). Stable dispatch IDs 18–20, 29.
///   3. Reserved pseudo-domains — architecturally unresolved; no dispatch
///      ID assigned until the design is concrete.
///
/// Application stacks (crypto libraries, ML frameworks, game engines,
/// backend/frontend HTTP frameworks, audio/video) are **external packages**.
/// They use the library import system (`iba`), not VM domain dispatch.
/// They do not belong in this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OduDomain {
    // =========================================================================
    // Tier 1 — The 16 Canonical Odù (Stable IDs 0–15)
    // =========================================================================
    /// Ogbè (1111) — System, CLI args, lifecycle
    Ogbe,
    /// Ọ̀yẹ̀kú (0000) — Exit, sleep, termination
    Oyeku,
    /// Ìwòrì (0110) — Time, DateTime
    Iwori,
    /// Òdí (1001) — Files, database I/O
    Odi,
    /// Ìrosù (1100) — Console I/O, logging, audio output
    Irosu,
    /// Ọ̀wọ́nrín (0011) — Random number generation
    Owonrin,
    /// Ọ̀bàrà (1000) — Math (addition, multiplication)
    Obara,
    /// Ọ̀kànràn (0001) — Errors, assertions
    Okanran,
    /// Ògúndá (1110) — Arrays, collections
    Ogunda,
    /// Ọ̀sá (0111) — Concurrency, actors
    Osa,
    /// Ìká (0100) — Strings
    Ika,
    /// Òtúúrúpọ̀n (0010) — Math (subtraction, division)
    Oturupon,
    /// Òtúrá (1011) — Networking, HTTP
    Otura,
    /// Ìrẹtẹ̀ (1101) — Cryptography, hashing, compression
    Irete,
    /// Ọ̀ṣẹ́ (1010) — Graphics, UI
    Ose,
    /// Òfún (0101) — Permissions, reflection
    Ofun,

    // =========================================================================
    // Tier 2 — Infrastructure (Stable IDs 18–20, 29)
    // =========================================================================
    /// CPU — Parallel computing via rayon (ID 18)
    Cpu,
    /// GPU — GPU compute via wgpu (ID 19)
    Gpu,
    /// Storage — Key-value persistence (ID 20)
    Storage,
    /// Sys — Kernel/OS interface, extends Ogbe (ID 29)
    Sys,

    // =========================================================================
    // Tier 3 — Reserved (no dispatch ID assigned yet)
    // =========================================================================
    /// Àjọṣe (Coop) — FFI bridge mechanism.
    ///
    /// Not a dispatchable domain. Architecturally belongs at the grammar/
    /// compiler layer as a calling-convention qualifier (like `extern "C"`),
    /// not as a VM dispatch arm. Reserved for future grammar design.
    Coop,
    /// Ọpẹlẹ — Compound Odù / combination divination.
    ///
    /// Conceptually maps to the 240 derived Odù beyond the 16 principal.
    /// No concrete method surface yet. Reserved until design is settled.
    Opele,
}

impl OduDomain {
    /// The 4-bit Ifá binary pattern for the 16 canonical Odù.
    /// Returns `None` for infrastructure and reserved pseudo-domains.
    pub fn binary(&self) -> Option<u8> {
        match self {
            OduDomain::Ogbe => Some(0b1111),
            OduDomain::Oyeku => Some(0b0000),
            OduDomain::Iwori => Some(0b0110),
            OduDomain::Odi => Some(0b1001),
            OduDomain::Irosu => Some(0b1100),
            OduDomain::Owonrin => Some(0b0011),
            OduDomain::Obara => Some(0b1000),
            OduDomain::Okanran => Some(0b0001),
            OduDomain::Ogunda => Some(0b1110),
            OduDomain::Osa => Some(0b0111),
            OduDomain::Ika => Some(0b0100),
            OduDomain::Oturupon => Some(0b0010),
            OduDomain::Otura => Some(0b1011),
            OduDomain::Irete => Some(0b1101),
            OduDomain::Ose => Some(0b1010),
            OduDomain::Ofun => Some(0b0101),
            _ => None,
        }
    }

    /// The stable VM dispatch integer ID for this domain.
    ///
    /// The 16 canonical Odù use IDs 0–15 matching their enum position.
    /// Infrastructure occupies IDs 18–20 and 29.
    /// Reserved domains return `None` — they have no dispatch path.
    pub fn dispatch_id(&self) -> Option<u8> {
        match self {
            OduDomain::Ogbe => Some(0),
            OduDomain::Oyeku => Some(1),
            OduDomain::Iwori => Some(2),
            OduDomain::Odi => Some(3),
            OduDomain::Irosu => Some(4),
            OduDomain::Owonrin => Some(5),
            OduDomain::Obara => Some(6),
            OduDomain::Okanran => Some(7),
            OduDomain::Ogunda => Some(8),
            OduDomain::Osa => Some(9),
            OduDomain::Ika => Some(10),
            OduDomain::Oturupon => Some(11),
            OduDomain::Otura => Some(12),
            OduDomain::Irete => Some(13),
            OduDomain::Ose => Some(14),
            OduDomain::Ofun => Some(15),
            // Infrastructure
            OduDomain::Cpu => Some(18),
            OduDomain::Gpu => Some(19),
            OduDomain::Storage => Some(20),
            OduDomain::Sys => Some(29),
            // Reserved — no dispatch ID
            OduDomain::Coop | OduDomain::Opele => None,
        }
    }

    /// The Yorùbá name.
    pub fn yoruba_name(&self) -> &'static str {
        match self {
            OduDomain::Ogbe => "Ọ̀gbè",
            OduDomain::Oyeku => "Ọ̀yẹ̀kú",
            OduDomain::Iwori => "Ìwòrì",
            OduDomain::Odi => "Òdí",
            OduDomain::Irosu => "Ìrosù",
            OduDomain::Owonrin => "Ọ̀wọ́nrín",
            OduDomain::Obara => "Ọ̀bàrà",
            OduDomain::Okanran => "Ọ̀kànràn",
            OduDomain::Ogunda => "Ògúndá",
            OduDomain::Osa => "Ọ̀sá",
            OduDomain::Ika => "Ìká",
            OduDomain::Oturupon => "Òtúúrúpọ̀n",
            OduDomain::Otura => "Òtúrá",
            OduDomain::Irete => "Ìrẹtẹ̀",
            OduDomain::Ose => "Ọ̀ṣẹ́",
            OduDomain::Ofun => "Òfún",
            OduDomain::Cpu => "Ẹrọ-ìṣirò",
            OduDomain::Gpu => "Ẹrọ-àwòrán",
            OduDomain::Storage => "Àkójọpọ̀",
            OduDomain::Sys => "Ètò",
            OduDomain::Coop => "Àjọṣe",
            OduDomain::Opele => "Ọpẹlẹ",
        }
    }

    /// True for the 16 canonical principal Odù.
    pub fn is_traditional(&self) -> bool {
        self.binary().is_some()
    }

    /// True for hardware/OS infrastructure domains.
    pub fn is_infrastructure(&self) -> bool {
        matches!(
            self,
            OduDomain::Cpu | OduDomain::Gpu | OduDomain::Storage | OduDomain::Sys
        )
    }

    /// True for reserved pseudo-domains with no current dispatch path.
    pub fn is_reserved(&self) -> bool {
        matches!(self, OduDomain::Coop | OduDomain::Opele)
    }

    /// True for domains that perform I/O, concurrency, or other stateful side effects
    /// that are unsafe in parallel execution contexts.
    pub fn has_side_effects(&self) -> bool {
        matches!(
            self,
            OduDomain::Oyeku
                | OduDomain::Odi
                | OduDomain::Irosu
                | OduDomain::Osa
                | OduDomain::Otura
                | OduDomain::Ose
                | OduDomain::Storage
                | OduDomain::Sys
                | OduDomain::Coop
        )
    }
}

impl std::fmt::Display for OduDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.yoruba_name())
    }
}
