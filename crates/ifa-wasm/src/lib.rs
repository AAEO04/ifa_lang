//! # Ifá-Lang WASM Bindings
//!
//! WebAssembly bindings for the Ifá-Lang playground.
//! Bridges the browser to the core Rust runtime.

use wasm_bindgen::prelude::*;
use ifa_core::interpreter::Interpreter;
use ifa_core::parser::parse;

// =============================================================================
// INTERPRETER WASM EXPORTS
// =============================================================================

/// Run Ifá-Lang code and return the output
#[wasm_bindgen]
pub fn run_code(source: &str) -> String {
    // Initialize interpreter
    let mut interpreter = Interpreter::new();
    
    // Parse source code
    match parse(source) {
        Ok(program) => {
            // Execute program
            match interpreter.execute(&program) {
                Ok(_) => {
                    // Get text output
                    let mut output = interpreter.get_output().join("\n");
                    
                    // Check if canvas has content (non-blank)
                    let canvas_output = interpreter.get_canvas();
                    if !canvas_output.chars().all(|c| c == ' ' || c == '\n') {
                        if !output.is_empty() {
                            output.push_str("\n\n");
                        }
                        output.push_str("═══ Canvas Output ═══\n");
                        output.push_str(&canvas_output);
                    }
                    
                    output
                }
                Err(e) => {
                    format!("Runtime Error: {}", e)
                }
            }
        }
        Err(e) => {
            format!("Parse Error: {}", e)
        }
    }
}

/// Get version information
#[wasm_bindgen]
pub fn get_version() -> String {
    format!("Ifá-Lang v{} (WASM Core)", env!("CARGO_PKG_VERSION"))
}

/// Cast the Opele and return an Odu name
#[wasm_bindgen]
pub fn cast_opele() -> String {
    let seed = js_sys::Date::now() as u64;
    let random = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let odu_byte = ((random >> 32) as u8) % 16;
    
    let names = [
        "Ogbe", "Oyeku", "Iwori", "Odi",
        "Irosu", "Owonrin", "Obara", "Okanran",
        "Ogunda", "Osa", "Ika", "Oturupon",
        "Otura", "Irete", "Ose", "Ofun"
    ];
    
    names[odu_byte as usize].to_string()
}
