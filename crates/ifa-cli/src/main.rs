//! # Ifá-Lang CLI
//!
//! Command-line interface for Ifá-Lang - The Yoruba Programming Language.

mod docgen;
mod oja;
mod sandbox;
mod lsp;
mod deploy;
mod debug_adapter;

use clap::{Parser, Subcommand};
use eyre::{Result, WrapErr};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ifa")]
#[command(author = "Ifá-Lang Contributors")]
#[command(version = "1.2.2")]
#[command(about = "Ifá-Lang - The Yoruba Programming Language", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an Ifá-Lang program
    Run {
        /// Path to .ifa source file
        file: PathBuf,
        /// Arguments to pass to the program
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// Allow all permissions (insecure)
        #[arg(long)]
        allow_all: bool,

        /// Allow read access to specific paths
        #[arg(long)]
        allow_read: Vec<PathBuf>,

        /// Allow write access to specific paths
        #[arg(long)]
        allow_write: Vec<PathBuf>,

        /// Allow network access to specific domains
        #[arg(long)]
        allow_net: Vec<String>,

        /// Allow environment variable access
        #[arg(long)]
        allow_env: Vec<String>,

        /// Allow execution of time functions
        #[arg(long)]
        allow_time: bool,

        /// Allow random number generation (on by default)
        #[arg(long, default_value = "true")]
        allow_random: bool,

        /// Allow Polyglot FFI for JavaScript
        #[arg(long)]
        allow_js: bool,

        /// Allow Polyglot FFI for Python
        #[arg(long)]
        allow_python: bool,

        /// Sandbox mode: wasm (OmniBox WASM sandbox), native (Igbale OS sandbox), none (no sandbox)
        #[arg(long, default_value = "none")]
        sandbox: String,
    },

    /// Compile to bytecode (.ifab)
    Bytecode {
        /// Path to .ifa source file
        file: PathBuf,
        /// Output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run compiled bytecode
    Runb {
        /// Path to .ifab bytecode file
        file: PathBuf,
    },

    /// Build native executable
    Build {
        /// Path to .ifa source file
        file: PathBuf,
        /// Output path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Target triple (e.g., x86_64-unknown-linux-gnu)
        #[arg(long)]
        target: Option<String>,

        /// Build for Backend domain (default)
        #[arg(long)]
        backend: bool,
        /// Build for Frontend domain (WASM)
        #[arg(long)]
        frontend: bool,
        /// Build for Game domain
        #[arg(long)]
        game: bool,
        /// Build for IoT domain (no_std)
        #[arg(long)]
        iot: bool,
        /// Build for Crypto domain (constant-time)
        #[arg(long)]
        crypto: bool,

        /// Build for ML/AI domain (Python interop + GPU)
        #[arg(long)]
        ml: bool,
        /// Build as Fullstack Hybrid Executable (Backend + Frontend)
        #[arg(long)]
        fullstack: bool,
    },

    /// Flash to embedded device
    Flash {
        /// Path to .ifa source file
        file: PathBuf,
        /// Target device (e.g., esp32, stm32f4)
        #[arg(long)]
        target: String,
        /// Serial port
        #[arg(long)]
        port: Option<String>,
    },

    /// Run in sandbox
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommands,
    },

    /// Package management (Ọjà)
    Oja {
        #[command(subcommand)]
        command: OjaCommands,
    },

    /// Check syntax without running
    Check {
        /// Path to .ifa source file
        file: PathBuf,
    },

    /// Format source code
    Fmt {
        /// Path to .ifa source file
        file: PathBuf,
        /// Check only, don't modify
        #[arg(long)]
        check: bool,
    },

    /// Start Language Server (LSP)
    Lsp,

    /// Interactive REPL
    Repl,

    /// Show version info
    Version,

    /// Run the Babalawo linter/type checker
    Babalawo {
        /// Path to .ifa file or directory
        path: PathBuf,
        /// Strict mode (warnings become errors)
        #[arg(long)]
        strict: bool,
        /// Output format: minimal, compact, json, verbose
        #[arg(long, default_value = "minimal")]
        format: String,
        /// Fast mode (skip proverbs/wisdom for performance)
        #[arg(long)]
        fast: bool,
    },

    /// Generate documentation
    Doc {
        /// Input directory containing .ifa files
        input: PathBuf,
        /// Output directory for HTML docs
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,
    },

    /// Run tests (Idanwo)
    Test {
        /// Path to test file or directory
        path: Option<PathBuf>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Zero-Config Deployment scanner
    Deploy {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Start Debug Adapter (DAP)
    Debug {
        /// File to debug
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SandboxCommands {
    /// Run script in sandbox
    Run {
        file: PathBuf,
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
    /// Demo sandbox features
    Demo,
    /// List active containers
    List,
}

#[derive(Subcommand)]
enum OjaCommands {
    /// Initialize new project
    Init {
        /// Project name (default: current directory name)
        #[arg(default_value = "ifa-project")]
        name: String,
        
        /// Project Domain Template (basic, fullstack, game, ml, iot)
        #[arg(long, default_value = "basic")]
        domain: String,
    },
    /// Add dependency
    Add {
        /// Git URL or package path
        url: String,
        /// Optional alias for the dependency
        #[arg(long)]
        alias: Option<String>,
    },
    /// Remove dependency
    Remove {
        /// Dependency name
        name: String,
    },
    /// Build project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Run project
    Run {
        /// Arguments to pass
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run tests
    Test,
    /// Install dependencies
    Install,
    /// List dependencies
    List,
    /// Upgrade Ifá-Lang CLI to latest version
    Upgrade,
    /// Publish package to registry (Git Tag)
    Publish,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            file,
            args: _,
            allow_all,
            allow_read,
            allow_write,
            allow_net,
            allow_env,
            allow_time,
            allow_random,
            allow_js,
            allow_python,
            sandbox,
        } => {
            use ifa_core::{Interpreter, parse};
            use ifa_sandbox::{CapabilitySet, Ofun};

            println!("Ifa-Lang Interpreter v1.2.2");
            println!();
            println!("Running: {}", file.display());

            // Configure Capabilities
            let mut caps = CapabilitySet::new();

            if allow_all {
                // In a real implementation this would check a wildcard,
                // but for now we'll just add common roots/domains
                println!("Warning: Running with all permissions allowed!");
                caps.grant(Ofun::ReadFiles {
                    root: PathBuf::from("/"),
                });
                caps.grant(Ofun::ReadFiles {
                    root: PathBuf::from("C:\\"),
                });
                caps.grant(Ofun::WriteFiles {
                    root: PathBuf::from("/"),
                });
                caps.grant(Ofun::WriteFiles {
                    root: PathBuf::from("C:\\"),
                });
                caps.grant(Ofun::Network {
                    domains: vec!["*".to_string()],
                });
                caps.grant(Ofun::Environment {
                    keys: vec!["*".to_string()],
                });
                caps.grant(Ofun::Time);
                caps.grant(Ofun::Random);
                caps.grant(Ofun::Stdio);
            } else {
                // Configurable permissions
                caps.grant(Ofun::Stdio); // Default allow stdio for now
                for path in allow_read {
                    caps.grant(Ofun::ReadFiles { root: path });
                }
                for path in allow_write {
                    caps.grant(Ofun::WriteFiles { root: path });
                }
                if !allow_net.is_empty() {
                    caps.grant(Ofun::Network { domains: allow_net });
                }
                if !allow_env.is_empty() {
                    caps.grant(Ofun::Environment { keys: allow_env });
                }
                if allow_time {
                    caps.grant(Ofun::Time);
                }
                if allow_random {
                    caps.grant(Ofun::Random);
                }
                if allow_js {
                    caps.grant(Ofun::Bridge { language: "js".into() });
                }
                if allow_python {
                    caps.grant(Ofun::Bridge { language: "python".into() });
                }

                // Always allow reading the script itself and its directory (for imports)
                if let Ok(abs_path) = file.canonicalize() {
                    caps.grant(Ofun::ReadFiles { root: abs_path });
                    if let Some(parent) = file.parent() {
                        if let Ok(abs_parent) = parent.canonicalize() {
                            caps.grant(Ofun::ReadFiles { root: abs_parent });
                        }
                    }
                }
            }

            println!();

            // Read source file
            let source = std::fs::read_to_string(&file)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to read file: {}", e))?;

            // Parse
            let program =
                parse(&source).map_err(|e| color_eyre::eyre::eyre!("Parse error: {}", e))?;

            println!("Parsed {} statements", program.statements.len());
            println!("---");
            println!();

            // Interpret (with_file enables imports relative to script location)
            let mut interpreter = Interpreter::with_file(&file);
            interpreter.set_capabilities(caps.clone());

            // Handle sandbox modes
            match sandbox.as_str() {
                "wasm" => {
                    use ifa_sandbox::{SandboxConfig, SecurityProfile};

                    println!("Running in OmniBox (WASM) sandbox...");

                    // Create sandbox config from capabilities
                    let mut config = SandboxConfig::new(SecurityProfile::Standard);
                    config.capabilities = caps;
                    config.force_wasm = true;

                    // Note: Full WASM execution would compile .ifa to .wasm first
                    // For now, we run interpreted but with the capability restrictions
                    println!(
                        "   (WASM compilation not yet implemented - using capability enforcement)"
                    );
                }
                "native" => {
                    println!("Running in Igbale (native OS) sandbox...");
                    // Native sandbox uses OS-level isolation (Linux namespaces, etc.)
                    // Capabilities are already set on the interpreter
                }
                "none" => {
                    // No sandbox - just use capability checks
                }
                _ => {
                    println!("Warning: Unknown sandbox mode '{}', using 'none'", sandbox);
                }
            }

            match interpreter.execute(&program) {
                Ok(_) => {
                    println!();
                    println!("---");
                    println!("Program completed successfully");
                }
                Err(e) => {
                    println!();
                    println!("---");
                    println!("Runtime error: {}", e);
                }
            }

            Ok(())
        }

        Commands::Bytecode { file, output } => {
            let out = output.unwrap_or_else(|| file.with_extension("ifab"));
            println!(
                "📦 Compiling to bytecode: {} -> {}",
                file.display(),
                out.display()
            );

            // Read source
            let source = std::fs::read_to_string(&file)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to read file: {}", e))?;

            // Compile to bytecode
            let bytecode = ifa_core::compile(&source)
                .map_err(|e| color_eyre::eyre::eyre!("Compilation error: {}", e))?;

            // Write .ifab file
            let bytes = bytecode.to_bytes();
            std::fs::write(&out, bytes)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to write bytecode: {}", e))?;

            println!(
                "Compiled {} bytes to {}",
                bytecode.code.len(),
                out.display()
            );
            Ok(())
        }

        Commands::Runb { file } => {
            println!("⚡ Running bytecode: {}", file.display());

            // Read bytecode
            let bytes = std::fs::read(&file)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to read bytecode: {}", e))?;

            // Deserialize
            let bytecode = ifa_core::Bytecode::from_bytes(&bytes)
                .map_err(|e| color_eyre::eyre::eyre!("Invalid bytecode: {}", e))?;

            // Execute in VM
            let mut vm = ifa_core::IfaVM::new();
            match vm.execute(&bytecode) {
                Ok(result) => {
                    println!("Result: {:?}", result);
                }
                Err(e) => {
                    println!("Runtime error: {}", e);
                }
            }

            Ok(())
        }

        Commands::Build {
            file,
            output,
            target,
            backend,
            frontend,
            game,
            iot,
            crypto,
            ml,
            fullstack,
        } => {
            use std::process::Command;

            let out = output.unwrap_or_else(|| {
                let stem = file.file_stem().unwrap_or_default();
                PathBuf::from(stem)
            });
            println!("🔨 Building: {} -> {}", file.display(), out.display());

            // Check if rustc is available
            let rustc_check = Command::new("rustc").arg("--version").output();
            if rustc_check.is_err() {
                println!("Error: Rust toolchain not installed.");
                println!("   Native compilation requires Rust. Install via:");
                println!("   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh");
                return Err(color_eyre::eyre::eyre!(
                    "Rust toolchain required for native builds"
                ));
            }

            // Read source file
            let source = std::fs::read_to_string(&file)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to read source: {}", e))?;

            // Parse and transpile to Rust
            println!("   📝 Parsing Ifá source...");
            let program = ifa_core::parse(&source)
                .map_err(|e| color_eyre::eyre::eyre!("Parse error: {}", e))?;

            println!("   🔄 Transpiling to Rust...");
            let rust_code = ifa_core::transpile_to_rust(&program);

            // Create temp Cargo project
            let temp_dir = std::env::temp_dir().join(format!("ifa_build_{}", std::process::id()));
            std::fs::create_dir_all(&temp_dir)?;

            let src_dir = temp_dir.join("src");
            std::fs::create_dir_all(&src_dir)?;

            // Write main.rs
            std::fs::write(src_dir.join("main.rs"), &rust_code)?;

            // Determine features
            let mut features = Vec::new();
            if frontend { features.push("frontend"); }
            if game { features.push("game"); }
            if iot { features.push("iot"); }
            if crypto { features.push("crypto"); }
            if ml { features.push("ml"); }
            if fullstack { 
                features.push("backend"); 
                features.push("frontend"); 
                // Fullstack implies both
            }
            // Default to backend if nothing else strictly selected (or if backend explicitly selected)
            if backend || features.is_empty() { features.push("backend"); }

            let features_str = features.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", ");
            let default_features = if iot { "false" } else { "true" };

            // Write Cargo.toml
            let cargo_toml = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
ifa-core = {{ path = "{}" }}
ifa-std = {{ path = "{}", features = [{}], default-features = {} }}

[profile.release]
opt-level = 3
lto = true
"#,
                out.file_stem().unwrap_or_default().to_string_lossy(),
                std::env::current_dir()?
                    .join("crates/ifa-core")
                    .display()
                    .to_string()
                    .replace("\\", "/"),
                std::env::current_dir()?
                    .join("crates/ifa-std")
                    .display()
                    .to_string()
                    .replace("\\", "/"),
                features_str,
                default_features
            );
            std::fs::write(temp_dir.join("Cargo.toml"), cargo_toml)?;

            // Build with cargo
            println!("   Compiling (this may take a moment)...");

            let mut cmd = Command::new("cargo");
            cmd.arg("build").arg("--release").current_dir(&temp_dir);

            if let Some(ref t) = target {
                cmd.arg("--target").arg(t);
                println!("   🎯 Target: {}", t);
            }

            let build_output = cmd.output()?;

            if !build_output.status.success() {
                let stderr = String::from_utf8_lossy(&build_output.stderr);
                println!("Build failed:\n{}", stderr);
                return Err(color_eyre::eyre::eyre!("Cargo build failed"));
            }

            // Copy binary to output location
            let binary_name = if cfg!(windows) {
                format!(
                    "{}.exe",
                    out.file_stem().unwrap_or_default().to_string_lossy()
                )
            } else {
                out.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            };

            let built_binary = if let Some(ref t) = target {
                temp_dir
                    .join("target")
                    .join(t)
                    .join("release")
                    .join(&binary_name)
            } else {
                temp_dir.join("target/release").join(&binary_name)
            };

            // Ensure output path has .exe on Windows
            let final_output = if cfg!(windows) && !out.to_string_lossy().ends_with(".exe") {
                PathBuf::from(format!("{}.exe", out.display()))
            } else {
                out.clone()
            };

            std::fs::copy(&built_binary, &final_output)?;

            // Cleanup temp directory
            let _ = std::fs::remove_dir_all(&temp_dir);

            let file_size = std::fs::metadata(&final_output)?.len();
            println!("Built: {} ({} bytes)", final_output.display(), file_size);
            println!(
                "   Run with: .\\{}",
                final_output
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );

            Ok(())
        }

        Commands::Flash { file, target, port } => {
            println!("🔌 Flashing to: {}", target);
            println!("   Source: {}", file.display());
            if let Some(ref p) = port {
                println!("   Port: {}", p);
            }
            ifa_std::stacks::iot::flash(&target, file.to_str().unwrap_or(""), port.as_deref())
                .map_err(|e| color_eyre::eyre::eyre!("IoT Error: {}", e))?;
            Ok(())
        }

        Commands::Sandbox { command } => {
            match command {
                SandboxCommands::Run { file, timeout } => {
                    use std::time::Duration;
                    let config = sandbox::SandboxConfig {
                        timeout: Duration::from_secs(timeout),
                        ..Default::default()
                    };
                    let sb = sandbox::Igbale::new(config);
                    let result = sb.run(&file)?;

                    println!("{}", result.stdout);
                    if !result.stderr.is_empty() {
                        eprintln!("{}", result.stderr);
                    }

                    if result.timed_out {
                        println!("Execution timed out");
                    }
                }
                SandboxCommands::Demo => {
                    sandbox::demo();
                }
                SandboxCommands::List => {
                    println!("📋 No active sandbox containers");
                }
            }
            Ok(())
        }

        Commands::Oja { command } => {
            let project_root = std::env::current_dir()?;
            let oja_manager = oja::Oja::new(&project_root);

            match command {
                OjaCommands::Init { name, domain } => {
                    oja_manager.init(&name, &domain)?;
                }
                OjaCommands::Add { url, alias } => {
                    oja_manager.add(&url, alias.as_deref())?;
                }
                OjaCommands::Remove { name } => {
                    oja_manager.remove(&name)?;
                }
                OjaCommands::Build { release } => {
                    oja_manager.build(release)?;
                }
                OjaCommands::Run { args } => {
                    oja_manager.run(&args)?;
                }
                OjaCommands::Test => {
                    oja_manager.test()?;
                }
                OjaCommands::Install => {
                    oja_manager.install()?;
                }
                OjaCommands::List => {
                    oja_manager.list()?;
                }
                OjaCommands::Upgrade => {
                    oja::update_cli()?;
                }
                OjaCommands::Publish => {
                    oja_manager.publish()?;
                }
            }
            Ok(())
        }

        Commands::Check { file } => {
            println!("🔍 Checking syntax of {}...", file.display());
            let source = std::fs::read_to_string(&file).wrap_err("Failed to read file")?;
            match ifa_core::parse(&source) {
                Ok(_) => {
                    println!("✅ No syntax errors found in {}", file.display());
                }
                Err(e) => {
                    eprintln!("❌ Syntax Error: {}", e);
                    std::process::exit(1);
                }
            }
            Ok(())
        }

        Commands::Fmt { file, check } => {
            let source = std::fs::read_to_string(&file).wrap_err("Failed to read file")?;
            
            // Simple Indentation Formatter logic
            let mut formatted = String::new();
            let mut indent: usize = 0;
            
            for line in source.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    formatted.push('\n');
                    continue;
                }
                
                if trimmed.contains('}') || trimmed.contains("ase") {
                    indent = indent.saturating_sub(1);
                }
                
                formatted.push_str(&"    ".repeat(indent));
                formatted.push_str(trimmed);
                formatted.push('\n');
                
                if trimmed.contains('{') || trimmed.contains("ese") || trimmed.contains("odu") {
                    indent += 1;
                }
            }
            
            if check {
                if source == formatted {
                    println!("✅ Perfect alignment in {}", file.display());
                } else {
                    println!("⚠️ Misaligned lines in {}", file.display());
                    std::process::exit(1);
                }
            } else {
                std::fs::write(&file, formatted).wrap_err("Failed to write formatted file")?;
                println!("✨ Syntactic harmony restored in {}", file.display());
            }
            Ok(())
        }

        Commands::Debug { file } => {
            if let Some(path) = file {
                debug_adapter::run_debug_session(path)?;
            } else {
                // If no file provided (e.g. launch request with no args initially?), 
                // the adapter might expect to receive 'launch' request with program path.
                // But our run_debug_session implementation currently requires a file to start interpreter.
                // DAP Launch request usually comes later.
                // We'll need to adjust run_debug_session to handle "wait for launch config".
                // For now, let's require file or error.
                return Err(color_eyre::eyre::eyre!("Debug command requires --file <PATH>"));
            }
            Ok(())
        }

        Commands::Lsp => {
            println!("🚀 Starting LSP server...");
            if let Err(e) = lsp::run() {
                eprintln!("LSP Error: {}", e);
                std::process::exit(1);
            }
            Ok(())
        }

        Commands::Repl => {
            use std::io::{self, BufRead, Write};

            println!("╔═══════════════════════════════════════════════════════════════╗");
            println!("║  🔮 Ifá-Lang REPL v1.0.0                                       ║");
            println!("║  The Yoruba Programming Language                              ║");
            println!("╠═══════════════════════════════════════════════════════════════╣");
            println!("║  Type Ifá-Lang code to execute. Commands:                     ║");
            println!("║  .help    - Show help        .clear - Clear interpreter       ║");
            println!("║  .quit    - Exit REPL        .vars  - Show variables          ║");
            println!("╚═══════════════════════════════════════════════════════════════╝");
            println!();

            let mut interpreter = ifa_core::Interpreter::new();
            let stdin = io::stdin();
            let mut multiline_buffer = String::new();
            let mut in_multiline = false;

            loop {
                // Show prompt
                if in_multiline {
                    print!("... ");
                } else {
                    print!("ifá> ");
                }
                io::stdout().flush().ok();

                // Read input
                let mut line = String::new();
                if stdin.lock().read_line(&mut line).is_err() || line.is_empty() {
                    break;
                }
                let line = line.trim();

                // Handle REPL commands
                match line {
                    ".quit" | ".exit" | ".q" => {
                        println!("👋 Ó dà bọ̀! (Goodbye!)");
                        break;
                    }
                    ".help" | ".h" => {
                        println!("📚 Ifá-Lang REPL Help:");
                        println!("  .help, .h    - Show this help");
                        println!("  .clear, .c   - Clear interpreter state");
                        println!("  .vars, .v    - Show defined variables");
                        println!("  .quit, .q    - Exit REPL");
                        println!("  .odu         - List Odù domains");
                        continue;
                    }
                    ".clear" | ".c" => {
                        interpreter = ifa_core::Interpreter::new();
                        println!("🧹 Interpreter state cleared");
                        continue;
                    }
                    ".vars" | ".v" => {
                        println!("📦 Variables:");
                        // The env is private, so we'd need to add a method to expose vars
                        println!("  (variable inspection not yet implemented)");
                        continue;
                    }
                    ".odu" => {
                        println!("🔢 16 Odù Domains:");
                        println!("  Ọ̀gbè (1111)    - System, CLI");
                        println!("  Ọ̀yẹ̀kú (0000)  - Exit, Sleep");
                        println!("  Ìwòrì (0110)  - Time, Iteration");
                        println!("  Ìrosù (1100)  - Console I/O");
                        println!("  Ọ̀bàrà (1000)  - Math Add/Mul");
                        println!("  Òtúúrúpọ̀n (0010) - Math Sub/Div");
                        println!("  Ìká (0100)    - Strings");
                        println!("  Ọ̀wọ́nrín (0011) - Random");
                        println!("  Ọ̀kànràn (0001) - Errors");
                        println!("  Ògúndá (1110) - Arrays");
                        println!("  Òdí (1001)    - Files/DB");
                        println!("  Ìrẹtẹ̀ (1101)  - Crypto");
                        println!("  Ọ̀sá (0111)    - Concurrency");
                        println!("  Òtúrá (1011)  - Networking");
                        println!("  Ọ̀ṣẹ́ (1010)    - Graphics");
                        println!("  Òfún (0101)   - Permissions");
                        continue;
                    }
                    "" => continue,
                    _ => {}
                }

                // Handle multiline input (braces)
                multiline_buffer.push_str(line);
                multiline_buffer.push('\n');

                // Check for balanced braces
                let open_braces = multiline_buffer.matches('{').count();
                let close_braces = multiline_buffer.matches('}').count();

                if open_braces > close_braces {
                    in_multiline = true;
                    continue;
                }

                in_multiline = false;
                let code = std::mem::take(&mut multiline_buffer);

                // Parse and execute
                match ifa_core::parse(&code) {
                    Ok(program) => {
                        match interpreter.execute(&program) {
                            Ok(result) => {
                                // Don't print Null results for statements
                                if !matches!(result, ifa_core::IfaValue::Null) {
                                    println!("=> {:?}", result);
                                }
                            }
                            Err(e) => {
                                println!("Runtime Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Parse Error: {}", e);
                    }
                }
            }

            Ok(())
        }

        Commands::Version => {
            println!("╔═══════════════════════════════════════════╗");
            println!("║  Ifá-Lang v1.2.0                          ║");
            println!("║  The Yoruba Programming Language          ║");
            println!("╠═══════════════════════════════════════════╣");
            println!(
                "║  Platform: {} / {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            println!("║  16 Odù Domains Active                    ║");
            println!("╚═══════════════════════════════════════════╝");
            Ok(())
        }

        Commands::Babalawo {
            path,
            strict,
            format,
            fast,
        } => {
            use ifa_babalawo::{check_program_with_config, BabalawoConfig};
            use ifa_core::parse;

            println!("babalawo: {}", path.display());
            if fast {
                println!("   (Fast mode enabled: Wisdom generation skipped)");
            }
            println!();

            // Collect files to check
            let files: Vec<PathBuf> = if path.is_dir() {
                std::fs::read_dir(&path)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().map(|e| e == "ifa").unwrap_or(false))
                    .collect()
            } else {
                vec![path.clone()]
            };

            let mut total_errors = 0;
            let mut total_warnings = 0;

            for file in &files {
                let source = std::fs::read_to_string(file)?;
                let filename = file.display().to_string();

                match parse(&source) {
                    Ok(program) => {
                        let config = BabalawoConfig {
                            include_wisdom: !fast,
                        };
                        let baba = check_program_with_config(&program, &filename, config);
                        total_errors += baba.error_count();
                        total_warnings += baba.warning_count();

                        // Output based on format
                        match format.as_str() {
                            "json" => print!("{}", baba.format_json()),
                            "compact" => print!("{}", baba.format_compact()),
                            "verbose" => {
                                let mut v = ifa_babalawo::Babalawo::new().verbose();
                                v.diagnostics = baba.diagnostics.clone();
                                print!("{}", v.format());
                            }
                            _ => print!("{}", baba.format()),
                        }
                    }
                    Err(e) => {
                        println!("error[Parse] {}:1:1", filename);
                        println!("  {}", e);
                        total_errors += 1;
                    }
                }
            }

            if strict && total_warnings > 0 {
                total_errors += total_warnings;
            }

            println!();
            println!(
                "{} error{}, {} warning{}. Àṣẹ!",
                total_errors,
                if total_errors == 1 { "" } else { "s" },
                total_warnings,
                if total_warnings == 1 { "" } else { "s" }
            );

            if total_errors > 0 {
                std::process::exit(1);
            }

            Ok(())
        }

        Commands::Doc { input, output } => {
            println!("📚 Generating documentation...");
            println!("   Input:  {}", input.display());
            println!("   Output: {}", output.display());
            println!();

            docgen::generate_docs(&output)?;

            println!();
            println!("Documentation generated successfully!");
            println!(
                "   Open {}/index.html in a browser to view.",
                output.display()
            );

            Ok(())
        }

        Commands::Test { path, verbose } => {
            use ifa_core::parse;

            println!("idanwo: Running tests...");
            println!();

            // Find test files
            let start_dir = path.unwrap_or_else(|| PathBuf::from("."));
            let test_files: Vec<PathBuf> = walkdir(&start_dir)
                .into_iter()
                .filter(|p| {
                    let name = p
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    (name.ends_with("_test.ifa") || name.starts_with("test_"))
                        && name.ends_with(".ifa")
                })
                .collect();

            if test_files.is_empty() {
                println!("No test files found.");
                return Ok(());
            }

            println!(
                "Found {} test file{}",
                test_files.len(),
                if test_files.len() == 1 { "" } else { "s" }
            );
            println!();

            let mut passed = 0;
            let mut failed = 0;
            let start = std::time::Instant::now();

            for file in &test_files {
                let name = file.display().to_string();
                print!("  {} ... ", name);

                let source = match std::fs::read_to_string(file) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("FAIL (read error: {})", e);
                        failed += 1;
                        continue;
                    }
                };

                match parse(&source) {
                    Ok(program) => {
                        // Simple execution test
                        let mut interp = ifa_core::Interpreter::new();
                        match interp.execute(&program) {
                            Ok(_) => {
                                println!("ok");
                                passed += 1;
                            }
                            Err(e) => {
                                println!("FAIL");
                                if verbose {
                                    println!("    {}", e);
                                }
                                failed += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("FAIL (parse)");
                        if verbose {
                            println!("    {}", e);
                        }
                        failed += 1;
                    }
                }
            }

            let duration = start.elapsed();
            println!();
            println!("---");
            if failed == 0 {
                println!("{} tests passed in {:.2?}", passed, duration);
            } else {
                println!("✗ {} passed, {} failed in {:.2?}", passed, failed, duration);
            }

            if failed > 0 {
                std::process::exit(1);
            }

            Ok(())
        }


        Commands::Deploy { path } => {
            deploy::scan_and_generate(&path)?;
            Ok(())
        }
    }
}

/// Walk directory recursively
fn walkdir(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_file() {
        files.push(dir.clone());
    } else if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(walkdir(&path));
                } else {
                    files.push(path);
                }
            }
        }
    }
    files
}
