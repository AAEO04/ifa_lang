use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum Profile {
    Standard,
    Fusion,  // Hybrid Development (Default)
    Minimal,
    Dev,     // Complete toolchain
    Custom,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub selected: bool,
}

impl Profile {
    pub fn components(&self) -> Vec<Component> {
        match self {
            Profile::Standard => vec![
                Component { name: "ifa-compiler".into(), description: "Ifá Compiler & Runtime".into(), required: true, selected: true },
                Component { name: "oja".into(), description: "package Manager".into(), required: true, selected: true },
                Component { name: "ifa-std".into(), description: "Standard Library".into(), required: false, selected: true },
            ],
            Profile::Fusion => vec![
                Component { name: "ifa-compiler".into(), description: "Ifá Compiler (Hybrid Support)".into(), required: true, selected: true },
                Component { name: "oja".into(), description: "Oja (Registry & Publish)".into(), required: true, selected: true },
                Component { name: "ifa-std".into(), description: "IFA-STD (Stacks: ML, Game, Fusion)".into(), required: true, selected: true },
                Component { name: "python-bridge".into(), description: "Python 3.11 Bridge (for ML)".into(), required: false, selected: true },
            ],
            Profile::Minimal => vec![
                Component { name: "ifa-compiler".into(), description: "Ifá Runtime".into(), required: true, selected: true },
                Component { name: "oja".into(), description: "package Manager".into(), required: true, selected: true },
            ],
            Profile::Dev => vec![
                 Component { name: "ifa-compiler".into(), description: "Compiler".into(), required: true, selected: true },
                 Component { name: "oja".into(), description: "Oja".into(), required: true, selected: true },
                 Component { name: "ifa-std".into(), description: "Std Lib".into(), required: true, selected: true },
                 Component { name: "ifa-lsp".into(), description: "LSP Server".into(), required: false, selected: true },
                 Component { name: "babalawo".into(), description: "Babalawo Linter".into(), required: false, selected: true },
            ],
            _ => vec![],
        }
    }
}
