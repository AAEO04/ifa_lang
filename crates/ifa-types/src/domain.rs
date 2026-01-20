//! # OduDomain - The 16 Odù + Infrastructure + Stacks
//!
//! This module defines all callable domains in Ifá-Lang.

/// The Odù domains enumeration
/// 
/// Includes the 16 principal Odù, pseudo-domains (Coop, Opele),
/// infrastructure layer (Cpu, Gpu, Storage), and application stacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OduDomain {
    // =========================================================================
    // Core 16 Odù (Traditional)
    // =========================================================================
    /// Ogbè (1111) - System, CLI Args, Lifecycle
    Ogbe,
    /// Ọ̀yẹ̀kú (0000) - Exit, Sleep
    Oyeku,
    /// Ìwòrì (0110) - Time, DateTime
    Iwori,
    /// Òdí (1001) - Files, Database
    Odi,
    /// Ìrosù (1100) - Console I/O, Logging
    Irosu,
    /// Ọ̀wọ́nrín (0011) - Random
    Owonrin,
    /// Ọ̀bàrà (1000) - Math (Add/Mul)
    Obara,
    /// Ọ̀kànràn (0001) - Errors, Assertions
    Okanran,
    /// Ògúndá (1110) - Arrays, Collections
    Ogunda,
    /// Ọ̀sá (0111) - Concurrency
    Osa,
    /// Ìká (0100) - Strings
    Ika,
    /// Òtúúrúpọ̀n (0010) - Math (Sub/Div)
    Oturupon,
    /// Òtúrá (1011) - Networking
    Otura,
    /// Ìrẹtẹ̀ (1101) - Crypto, Compression
    Irete,
    /// Ọ̀ṣẹ́ (1010) - Graphics, UI
    Ose,
    /// Òfún (0101) - Permissions, Reflection
    Ofun,

    // =========================================================================
    // Pseudo-domains
    // =========================================================================
    /// Co-op / Àjọṣe - FFI Bridge
    Coop,
    /// Ọpẹlẹ - Divination/Compound Odù
    Opele,

    // =========================================================================
    // Infrastructure Layer (Hardware/OS)
    // =========================================================================
    /// CPU - Parallel computing (rayon)
    Cpu,
    /// GPU - GPU compute (wgpu)
    Gpu,
    /// Storage - Key-value store
    Storage,

    // =========================================================================
    // Application Stacks
    // =========================================================================
    /// Backend - HTTP server, ORM
    Backend,
    /// Frontend - HTML, CSS generation
    Frontend,
    /// Crypto - Hashing, encryption (extends Irete)
    Crypto,
    /// ML - Machine learning, tensors
    Ml,
    /// GameDev - Game engine, ECS
    GameDev,
    /// IoT - Embedded, GPIO
    Iot,
}

impl OduDomain {
    /// Get the binary pattern for traditional Odù
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
            _ => None, // Non-traditional domains
        }
    }

    /// Get the Yoruba name
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
            OduDomain::Coop => "Àjọṣe",
            OduDomain::Opele => "Ọpẹlẹ",
            OduDomain::Cpu => "Ẹrọ-ìṣirò",
            OduDomain::Gpu => "Ẹrọ-àwòrán",
            OduDomain::Storage => "Àkójọpọ̀",
            OduDomain::Backend => "Ẹ̀hìn-ọ̀nà",
            OduDomain::Frontend => "Ojú-ọ̀nà",
            OduDomain::Crypto => "Àṣírí",
            OduDomain::Ml => "Ẹ̀kọ́-ẹ̀rọ",
            OduDomain::GameDev => "Eré-ìdárayá",
            OduDomain::Iot => "Ẹ̀rọ-kékeré",
        }
    }

    /// Check if this is a traditional Odù (not infra/stack)
    pub fn is_traditional(&self) -> bool {
        self.binary().is_some()
    }

    /// Check if this is an infrastructure domain
    pub fn is_infrastructure(&self) -> bool {
        matches!(self, OduDomain::Cpu | OduDomain::Gpu | OduDomain::Storage)
    }

    /// Check if this is an application stack
    pub fn is_stack(&self) -> bool {
        matches!(
            self,
            OduDomain::Backend
                | OduDomain::Frontend
                | OduDomain::Crypto
                | OduDomain::Ml
                | OduDomain::GameDev
                | OduDomain::Iot
        )
    }
}

impl std::fmt::Display for OduDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.yoruba_name())
    }
}
