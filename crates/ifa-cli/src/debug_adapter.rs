//! DAP Adapter (Stub)
//! Currently being migrated to the new Bytecode VM.

pub fn run_debug_session(_file: std::path::PathBuf) -> color_eyre::Result<()> {
    eprintln!("Error: DAP debugging is currently being migrated from the AST Interpreter to the Bytecode VM.");
    eprintln!("Please use 'ifa run' for standard execution.");
    std::process::exit(1);
}
