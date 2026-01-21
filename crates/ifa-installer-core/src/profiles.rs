use serde::{Deserialize, Serialize};

/// Component that can be installed by the Ifá-Lang installer.
#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub selected: bool,
}

/// Returns all components that the ifa binary provides.
///
/// The `ifa` binary is a complete toolchain that includes:
/// - Runtime/interpreter
/// - Oja package manager (`ifa oja` subcommand)
/// - Babalawo error checker (compiled in)
/// - WASM sandbox (compiled in)
/// - All 16 Odù domains and stacks
pub fn all_components() -> Vec<Component> {
    vec![Component {
        name: "ifa".into(),
        description: "Ifá-Lang CLI — Runtime, Oja package manager, Babalawo, Sandbox".into(),
        required: true,
        selected: true,
    }]
}

// Keep Profile struct for backwards compatibility with existing config files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Profile;

impl Profile {
    /// Returns all components — Profile is now a no-op, everything is bundled.
    pub fn components(&self) -> Vec<Component> {
        all_components()
    }
}
