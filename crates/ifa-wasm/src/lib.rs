//! # Ifá-Lang WASM Bindings
//!
//! WebAssembly bindings for the Ifá-Lang playground.
//! Bridges the browser to the core Rust runtime.

use ifa_parser::parse;
use ifa_vm::Compiler;
use ifa_vm::IfaVM;
use wasm_bindgen::prelude::*;

// =============================================================================
// STRUCTURED RESULT
// =============================================================================

/// Structured execution result returned from `run_code`
#[wasm_bindgen]
pub struct RunResult {
    success: bool,
    output: String,
    error: Option<String>,
    events: String,
}

#[wasm_bindgen]
impl RunResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn events(&self) -> String {
        self.events.clone()
    }
}

// =============================================================================
// INTERPRETER WASM EXPORTS
// =============================================================================

/// Run Ifá-Lang code and return the structured RunResult.
/// This runs synchronously as the WASM execution is fast and event-loop yielding
/// belongs at a higher orchestration layer (e.g. Web Workers) rather than fake promises.
#[wasm_bindgen]
pub fn run_code(source: String) -> RunResult {
    // Parse source code
    match parse(&source) {
        Ok(program) => {
            let compiler = Compiler::new("wasm");
            match compiler.compile(&program) {
                Ok(bytecode) => {
                    let mut caps = ifa_types::capability::CapabilitySet::new();
                    caps.grant(ifa_types::capability::Ofun::Stdio);
                    caps.grant(ifa_types::capability::Ofun::Random);
                    caps.grant(ifa_types::capability::Ofun::Time);
                    caps.grant(ifa_types::capability::Ofun::Crypto);

                    let mut registry = ifa_std::vm_registry::StdRegistry::new();
                    registry.set_capabilities(caps);
                    let mut vm = IfaVM::new().with_registry(Box::new(registry));
                    match vm.execute(&bytecode) {
                        Ok(value) => {
                            let history = vm.opon.get_history();
                            let events = serde_json::to_string(&history)
                                .unwrap_or_else(|_| "[]".to_string());
                            let mut printed = String::new();
                            for event in &history {
                                if event.spirit == "Ìrosù"
                                    && (event.action == "fọ̀ (spoke)"
                                        || event.action == "fọ̀ (spoke_raw)"
                                        || event.action == "kígbe (screamed)")
                                {
                                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                                        &event.value,
                                    ));
                                    printed.push_str(&event.value);
                                    if event.action == "fọ̀ (spoke)"
                                        || event.action == "kígbe (screamed)"
                                    {
                                        printed.push('\n');
                                    }
                                }
                            }
                            let output = if !printed.is_empty() {
                                printed
                            } else {
                                value.to_string()
                            };
                            RunResult {
                                success: true,
                                output,
                                error: None,
                                events,
                            }
                        }
                        Err(e) => RunResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Runtime Error: {}", e)),
                            events: "[]".to_string(),
                        },
                    }
                }
                Err(e) => RunResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Compile Error: {}", e)),
                    events: "[]".to_string(),
                },
            }
        }
        Err(e) => RunResult {
            success: false,
            output: String::new(),
            error: Some(format!("Parse Error: {}", e)),
            events: "[]".to_string(),
        },
    }
}

/// Format Ifá-Lang source code
#[wasm_bindgen]
pub fn format_code(source: String) -> String {
    let config = ifa_fmt::FormatterConfig::default();
    ifa_fmt::format(&source, config)
}

/// Get version information
#[wasm_bindgen]
pub fn get_version() -> String {
    format!("Ifá-Lang v{} (WASM Core)", env!("CARGO_PKG_VERSION"))
}

/// Cast the Opele and return an Odu name using JS-native Math.random()
#[wasm_bindgen]
pub fn cast_opele() -> String {
    let random_val = js_sys::Math::random();
    let byte = (random_val * 256.0).floor() as u8;
    let odu = ifa_std::opele::Odu::from_byte(byte);
    odu.name()
}
