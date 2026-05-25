//! # Ifá-Lang WASM Bindings
//!
//! WebAssembly bindings for the Ifá-Lang playground.
//! Bridges the browser to the core Rust runtime.

use ifa_vm::Compiler;
use ifa_vm::IfaVM;
use ifa_types::bytecode::OponSize;
use wasm_bindgen::prelude::*;
use ifa_parser::parse;

// =============================================================================
// INTERPRETER WASM EXPORTS
// =============================================================================

/// Run Ifá-Lang code and return the output via Promise to avoid blocking the main thread synchronously
#[wasm_bindgen]
pub async fn run_code(source: String) -> String {
    // Yield to the browser event loop immediately before starting heavy synchronous work
    let promise = js_sys::Promise::resolve(&JsValue::undefined());
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

    // Parse source code
    match parse(&source) {
        Ok(program) => {
            let compiler = Compiler::new("wasm");
            match compiler.compile(&program) {
                Ok(bytecode) => {
                    let mut vm = IfaVM::new();
                    match vm.execute(&bytecode) {
                        Ok(value) => {
                            // Currently, standard output is printed to the console directly
                            // Returning the evaluated value or a generic success message
                            format!("Success: {}", value)
                        }
                        Err(e) => format!("Runtime Error: {}", e),
                    }
                }
                Err(e) => format!("Compile Error: {}", e),
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
        "Ogbe", "Oyeku", "Iwori", "Odi", "Irosu", "Owonrin", "Obara", "Okanran", "Ogunda", "Osa",
        "Ika", "Oturupon", "Otura", "Irete", "Ose", "Ofun",
    ];

    names[odu_byte as usize].to_string()
}
