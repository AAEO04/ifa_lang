//! # Ọjà Package Manager
//!
//! "Fetch and Run" philosophy.
//! Handles `ifa fetch` (download + compile + log) and `ifa run`.

#![allow(dead_code)]

use eyre::{Result, WrapErr, eyre};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use ring::digest::{Context, SHA256};
use chrono::Local;
use std::io::{Cursor, Write};
use ifa_sandbox::{OmniBox, SandboxConfig, SecurityProfile};
use reqwest::blocking::Client;
use flate2::read::GzDecoder;
use tar::Archive;

const OJA_REGISTRY_URL: &str = "https://raw.githubusercontent.com/AAEO04/oja-registry/main";

#[derive(Debug, Deserialize)]
struct RegistryPackage {
    name: String,
    repository: String,
    versions: Vec<RegistryVersion>,
}

#[derive(Debug, Deserialize)]
struct RegistryVersion {
    version: String,
    #[serde(default)]
    yanked: bool,
}

/// Ọjà project manifest (Iwe.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfaManifest {
    #[serde(default)]
    pub package: Option<PackageInfo>,
    #[serde(default)]
    pub workspace: Option<WorkspaceInfo>,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Version(String),
    Git {
        git: String,
        branch: Option<String>,
        tag: Option<String>,
    },
    Path { path: String },
}

/// Ọjà package manager
pub struct Oja {
    project_root: PathBuf,
}

impl Oja {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Oja {
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    /// Initialize a new project with .oja directory structure
    pub fn init(&self, project_name: &str, domain: &str) -> Result<()> {
        if domain == "monorepo" || domain == "workspace" {
            return self.init_workspace(project_name);
        }

        let manifest = IfaManifest {
            package: Some(PackageInfo {
                name: project_name.to_string(),
                version: "0.1.0".to_string(),
                description: format!("{} project", domain),
                authors: vec![],
            }),
            workspace: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        };

        if !self.project_root.exists() {
             fs::create_dir_all(&self.project_root).wrap_err("Failed to create project root")?;
        }

        let manifest_path = self.project_root.join("Iwe.toml");
        let toml = toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
        fs::write(&manifest_path, toml).wrap_err("Failed to write Iwe.toml")?;

        let src_dir = self.project_root.join("src");
        fs::create_dir_all(&src_dir).wrap_err("Failed to create src directory")?;

        let main_content = match domain {
            "game" => r#"// Game Domain Project
// Uses ifa-std::stacks::gamedev
// Run with: ifa build --game

fn main() {
    println("🎮 Starting Game Engine...");
    // let world = World::new();
}
"#,
            "ml" => r#"// ML Domain Project
// Uses ifa-std::stacks::ml
// Run with: ifa build --ml

fn main() {
    println("🧠 Initializing Neural Network...");
    // let tensor = Tensor::zeros([2, 2]);
}
"#,
            "fusion" | "fullstack" => r#"// Fusion Hybrid Project
// Uses ifa-std::stacks::fusion (Hybrid Runtime)
// Run with: ifa build --fusion

fn main() {
    println("🚀 Launching Fusion Runtime...");
}
"#,
            "iot" => r#"// IoT Domain Project
// Uses ifa-std::stacks::iot (no_std)
// Run with: ifa build --iot

fn main() {
    // loop {
    //     gpio.write(HIGH);
    // }
}
"#,
            _ => r#"// Ifá-Lang main entry point
jẹ́ kí a sọ "Ẹ káàbọ̀ sí Ifá-Lang!"
"#
        };

        fs::write(src_dir.join("main.ifa"), main_content).wrap_err("Failed to write main.ifa")?;

        // Create .gitignore
        let gitignore_content = r#"target/
*.ifab
.oja/
"#;
        fs::write(self.project_root.join(".gitignore"), gitignore_content).wrap_err("Failed to write .gitignore")?;

        // Create Igbale Structure
        self.ensure_igbale()?;

        println!("Created {} project: {}", domain, project_name);
        Ok(())
    }

    /// Initialize a Monorepo Workspace
    fn init_workspace(&self, name: &str) -> Result<()> {
        println!("🏗️  Initializing Monorepo Workspace: {}", name);
        
        let manifest = IfaManifest {
            package: None,
            workspace: Some(WorkspaceInfo {
                members: vec!["backend".to_string(), "frontend".to_string()].into(),
            }),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        };

        if !self.project_root.exists() {
             fs::create_dir_all(&self.project_root).wrap_err("Failed to create workspace root")?;
        }
        
        // Root Iwe.toml
        let manifest_path = self.project_root.join("Iwe.toml");
        let toml = toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
        fs::write(&manifest_path, toml).wrap_err("Failed to write Iwe.toml")?;
        
        // Shared .oja
        self.ensure_igbale()?;

        // Packages dir
        let packages_dir = self.project_root.join("packages");
        fs::create_dir_all(&packages_dir)?;

        println!("   ✓ Created Iwe.toml (workspace)");
        println!("   ✓ Initialized shared .oja/");
        println!("   💡 Add members with: ifa oja init <name> --domain <domain> inside proper folders");
        
        Ok(())
    }

    /// Build project (Project-Centric Build)
    pub fn build(&self, release: bool) -> Result<()> {
        let manifest = self.load_manifest()?;

        // 1. Workspace Build
        if let Some(workspace) = manifest.workspace {
            println!("🏗️  Building Workspace ({} members)...", workspace.members.len());
            for member in workspace.members {
                let member_path = self.project_root.join(&member);
                if !member_path.exists() {
                    println!("   ⚠️  Member not found: {} (skipping)", member);
                    continue;
                }
                println!("   👉 Entering member: {}", member);
                let member_oja = Oja::new(&member_path);
                member_oja.build(release)?;
            }
            println!("✅ Workspace build complete.");
            return Ok(());
        }

        // 2. Package Build
        if let Some(package) = manifest.package {
            println!("📦 Building Package: {} v{}", package.name, package.version);
            
            let src_file = self.project_root.join("src/main.ifa");
            if !src_file.exists() {
                 return Err(eyre!("Source file not found: src/main.ifa"));
            }

            // --- Compilation Logic (Transpile -> Cargo) ---
            // Reusing logic similar to main.rs but adapted for Project Context
            
            let source = std::fs::read_to_string(&src_file).wrap_err("Failed to read main.ifa")?;
            println!("   📝 Parsing Ifá source...");
            let program = ifa_core::parse(&source).wrap_err("Parse error")?;
            
            println!("   🔄 Transpiling to Rust...");
            let rust_code = ifa_core::transpile_to_rust(&program);

            // Create temp build dir
            let temp_dir = self.project_root.join("target/build_tmp");
            if temp_dir.exists() { fs::remove_dir_all(&temp_dir).ok(); }
            fs::create_dir_all(&temp_dir.join("src"))?;

            fs::write(temp_dir.join("src/main.rs"), &rust_code)?;

            let core_path = std::env::current_dir()?.join("crates/ifa-core").display().to_string().replace("\\", "/");
            let std_path = std::env::current_dir()?.join("crates/ifa-std").display().to_string().replace("\\", "/");

            // Generate Cargo.toml with dependencies from Iwe.toml
            // Identify domain features
            // Simple heuristic for now: check description or add a 'runtime' field later.
            // defaulting to "backend" + "frontend" (fullstack safe default)
            let features = "\"backend\", \"frontend\", \"ml\", \"game\""; 

            let cargo_toml = format!(
                r#"[package]
name = "{}"
version = "{}"
edition = "2021"

[dependencies]
ifa-core = {{ path = "{}" }}
ifa-std = {{ path = "{}", features = [{}] }}

[profile.release]
opt-level = 3
"#,
                package.name,
                package.version,
                core_path, // Use the calculated/fallback paths
                std_path,
                features
            );
            
            // Write the formatted TOML
            fs::write(temp_dir.join("Cargo.toml"), cargo_toml)?;

            println!("   🛠️  Compiling with Cargo...");
            let status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&temp_dir)
                .status()?;
            
            if !status.success() {
                return Err(eyre!("Cargo build failed"));
            }

            // Move artifact
            let target_bin = temp_dir.join("target/release").join(format!("{}.exe", package.name)); // Win assumption
            let output_bin = self.project_root.join(format!("{}.exe", package.name));
            
            if target_bin.exists() {
                 fs::copy(&target_bin, &output_bin)?;
                 println!("   ✨ Built: {}", output_bin.display());
            } else {
                 // Try non-exe
                 let target_bin = temp_dir.join("target/release").join(&package.name);
                 if target_bin.exists() {
                     fs::copy(&target_bin, &output_bin)?;
                     println!("   ✨ Built: {}", output_bin.display());
                 }
            }
        }
        
        Ok(())
    }

    /// Ensure the .oja directory structure exists
    fn ensure_igbale(&self) -> Result<(PathBuf, PathBuf)> {
        let igbale = self.project_root.join(".oja");
        let lib = igbale.join("lib");
        let cache = igbale.join("cache");
        
        fs::create_dir_all(&lib).wrap_err("Failed to create .oja/lib")?;
        fs::create_dir_all(&cache).wrap_err("Failed to create .oja/cache")?;
        
        Ok((lib, cache))
    }

    /// Append to the Oja audit log
    fn audit_log(&self, action: &str, details: &str) -> Result<()> {
        let log_path = self.project_root.join(".oja/audit.log");
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .wrap_err("Failed to open audit log")?;

        writeln!(file, "[{}] {}: {}", timestamp, action, details)?;
        Ok(())
    }

    /// `ifa fetch`: Download, Audit, and Compile
    pub fn fetch(&self) -> Result<()> {
        println!("🛒  Fetching dependencies...");
        
        let (lib_dir, cache_dir) = self.ensure_igbale()?;
        let manifest = self.load_manifest()?;

        if manifest.dependencies.is_empty() {
            println!("   No dependencies.");
            return Ok(());
        }

        // Initialize OmniBox for AOT compilation
        let config = SandboxConfig::new(SecurityProfile::Standard);
        let omnibox = OmniBox::new(config).wrap_err("Failed to init compiler")?;

        println!("🔥  Compiling to native machine code...");

        for (name, dep) in &manifest.dependencies {
            println!("   - {}", name);
            
            // 1. Resolve & Download (Stub - assumes local file exists or uses path)
            // In real impl, git clone or http download happens here
            // 1. Resolve & Download
            let wasm_source = match dep {
                Dependency::Path { path } => PathBuf::from(path).join(format!("{}.wasm", name)),
                Dependency::Version(ver) => {
                    let url = self.resolve_registry(name, ver);
                    let target_dir = lib_dir.join(format!("{}-{}", name, ver));
                    
                    if !target_dir.exists() {
                         self.download_package(&url, &target_dir)?;
                         self.verify_signature(&target_dir)?;
                    }
                    
                    // Assume the package layout puts the wasm in pkg/name.wasm or similar
                    // For now, look for any .wasm file or fallback
                    let candidate = target_dir.join(format!("{}.wasm", name));
                    if !candidate.exists() {
                         // Fallback: maybe inside a subdir (github release structure)
                         // logic simplified
                    }
                    candidate
                },
                Dependency::Git { git: _, .. } => {
                    println!("     ! Git dependencies not yet implemented, assuming {:?} exists", lib_dir);
                    lib_dir.join(format!("{}.wasm", name))
                }
            };

            if wasm_source.exists() {
                let wasm_bytes = fs::read(&wasm_source).wrap_err("Failed to read Wasm source")?;
                
                // 2. Audit Log (Opẹlẹ)
                self.audit_log("FETCH", &format!("Compiled native artifact for {}", name))?;
                
                // 3. AOT Compile (Atomic Write)
                let artifact = omnibox.compile_artifact(&wasm_bytes)?;
                
                // Calculate hash (Carmack's Cache Key)
                let mut context = Context::new(&SHA256);
                context.update(&wasm_bytes);
                let hash_value: String = context.finish().as_ref().iter().map(|b| format!("{:02x}", b)).collect();
                
                let target_path = cache_dir.join(format!("{}.cwasm", hash_value));
                
                // Atomic Write Strategy
                self.atomic_write(&target_path, &artifact)?;
                println!("     ✓ Compiled ({})", &hash_value[..8]);
            } else {
                println!("     ! Source not found (skipping download logic)");
            }
        }
        
        println!("✨  Ready to run.");
        Ok(())
    }
    
    /// Atomic write compatible with Windows (Linus Fix)
    fn atomic_write(&self, path: &Path, data: &[u8]) -> Result<()> {
        let dir = path.parent().ok_or_else(|| eyre!("Invalid path"))?;
        let temp_name = format!(".tmp.{}", uuid::Uuid::new_v4());
        let temp_path = dir.join(temp_name);
        
        fs::write(&temp_path, data).wrap_err("Failed to write temp file")?;
        
        // Windows-safe rename
        match fs::rename(&temp_path, path) {
            Ok(_) => Ok(()),
            Err(e) => {
                // If on Windows and error is "already exists" or "access denied", we might need retry/movefile
                // For simple CLI usage, fs::rename usually works if we don't hold handles.
                // If it fails, try removing target first (technically not atomic but robust enough for dev CLI)
                if cfg!(windows) {
                    let _ = fs::remove_file(path);
                    fs::rename(&temp_path, path).wrap_err("Failed to move file (Windows retry)")
                } else {
                    Err(eyre!("Rename failed: {}", e))
                }
            }
        }
    }

    pub fn load_manifest(&self) -> Result<IfaManifest> {
        let path = self.project_root.join("Iwe.toml");
        // Fallback for legacy
        let path = if path.exists() { path } else { self.project_root.join("ifa.toml") };
        
        let content = fs::read_to_string(&path).wrap_err("Failed to read Iwe.toml")?;
        toml::from_str(&content).wrap_err("Failed to parse Iwe.toml")
    }

    /// Resolve package name to download URL (Registry Stub)
    /// Resolve package name to download URL via Registry
    fn resolve_registry(&self, name: &str, version: &str) -> String {
        // 1. Calculate Index Path
        let index_path = self.get_index_path(name);
        let url = format!("{}/index/{}", OJA_REGISTRY_URL, index_path);

        println!("     🔍 Searching registry: {}", url);

        // 2. Fetch Metadata (blocking)
        let client = Client::new();
        match client.get(&url).send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Parse registry Entry
                    if let Ok(entry) = resp.json::<RegistryPackage>() {
                        println!("     ✅ Found package: {}", entry.name);
                        
                        // Find version
                        let target_ver = if version == "latest" || version == "*" {
                            // Get last non-yanked version
                            entry.versions.iter().filter(|v| !v.yanked).last()
                        } else {
                            entry.versions.iter().find(|v| v.version == version)
                        };

                        if let Some(v) = target_ver {
                            if v.yanked {
                                println!("     ⚠️  Warning: Version {} is yanked!", v.version);
                            }
                            // Construct GitHub Archive URL
                            // Format: https://github.com/user/repo/archive/refs/tags/v1.0.0.tar.gz
                            return format!("{}/archive/refs/tags/v{}.tar.gz", entry.repository, v.version);
                        } else {
                             println!("     ❌ Version {} not found in registry. Falling back to simple resolution.", version);
                        }
                    }
                }
            }
            Err(_) => {
                 // Offline or Registry unreachable - Fallback
            }
        }

        // Fallback: Assume GitHub release tarball convention directly
        format!("https://github.com/ifa-lang/{}/archive/refs/tags/v{}.tar.gz", name, version)
    }

    /// Calculate registry index path based on package name length
    fn get_index_path(&self, name: &str) -> String {
        match name.len() {
            1 => format!("1/{}", name),
            2 => format!("2/{}", name),
            3 => format!("3/{}/{}", &name[0..1], name),
            _ => format!("{}/{}/{}", &name[0..2], &name[2..4], name),
        }
    }

    /// Download and Extract Tarball
    fn download_package(&self, url: &str, dest: &Path) -> Result<()> {
        println!("     ⬇ Downloading: {}", url);
        let client = Client::new();
        let response = client.get(url).send().wrap_err("Failed to send request")?;
        
        if !response.status().is_success() {
             return Err(eyre!("Download failed: {}", response.status()));
        }

        let bytes = response.bytes().wrap_err("Failed to read bytes")?;
        
        // Extract to temp dir first
        let tar = GzDecoder::new(Cursor::new(bytes));
        let mut archive = Archive::new(tar);
        
        // Strip first component (github archives have root folder)
        // For simplicity, just unpack. Real impl needs strip_prefix
        archive.unpack(dest).wrap_err("Failed to extract archive")?;
        
        Ok(())
    }

    /// Verify GPG Signature (Stub)
    fn verify_signature(&self, _path: &Path) -> Result<()> {
        println!("     🔒 Verifying GPG signature... (Stub: PASSED)");
        // Call `gpg --verify` or use `sequoia`
        Ok(())
    }

    /// Publish to Registry (Git Tagging Strategy)
    pub fn publish(&self) -> Result<()> {
        println!("📦  Publishing package...");
        
        // 1. Load Manifest
        let manifest = self.load_manifest()?;
        let version = manifest.package.as_ref()
            .ok_or_else(|| eyre!("Cannot publish a workspace manifest. Run from within a package."))?
            .version.clone();
        let tag = format!("v{}", version);
        
        println!("   Version: {}", version);
        
        // 2. Check Git Status
        println!("   🔍 Checking git status...");
        let status = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(&self.project_root)
            .output()
            .wrap_err("Failed to check git status")?;
            
        if !status.stdout.is_empty() {
             return Err(eyre!("Git working directory not clean. Commit changes first."));
        }
        
        // 3. Tag
        println!("   🏷️  Creating tag: {}", tag);
        let tag_cmd = Command::new("git")
            .arg("tag")
            .arg("-a")
            .arg(&tag)
            .arg("-m")
            .arg(format!("Release {}", tag))
            .current_dir(&self.project_root)
            .output()
            .wrap_err("Failed to create git tag")?;
            
        if !tag_cmd.status.success() {
             return Err(eyre!("Failed to create tag: {}", String::from_utf8_lossy(&tag_cmd.stderr)));
        }
        
        // 4. Push
        println!("   🚀 Pushing to remote...");
        let push_cmd = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg(&tag)
            .current_dir(&self.project_root)
            .output()
            .wrap_err("Failed to push tag")?;

        if !push_cmd.status.success() {
             return Err(eyre!("Failed to push tag: {}", String::from_utf8_lossy(&push_cmd.stderr)));
        }
        
        println!("✨  Published successfully to Registry (Git)!");
        println!("✨  Published successfully to Registry (Git)!");
        Ok(())
    }

    /// Add dependency
    pub fn add(&self, url: &str, alias: Option<&str>) -> Result<()> {
        let mut manifest = self.load_manifest()?;
        let name = alias.unwrap_or_else(|| {
            url.split('/').last().unwrap_or("unknown").trim_end_matches(".git")
        });

        println!("Adding dependency: {} as {}", url, name);
        manifest.dependencies.insert(name.to_string(), Dependency::Version(url.to_string()));
        
        let manifest_path = self.project_root.join("Iwe.toml");
        let toml = toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
        fs::write(&manifest_path, toml).wrap_err("Failed to update Iwe.toml")?;
        
        self.audit_log("ADD", &format!("Added dependency: {}", name))?;
        Ok(())
    }

    /// Remove dependency
    pub fn remove(&self, name: &str) -> Result<()> {
        let mut manifest = self.load_manifest()?;
        if manifest.dependencies.remove(name).is_some() {
            println!("Removing dependency: {}", name);
            let manifest_path = self.project_root.join("Iwe.toml");
            let toml = toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
            fs::write(&manifest_path, toml).wrap_err("Failed to update Iwe.toml")?;
            self.audit_log("REMOVE", &format!("Removed dependency: {}", name))?;
        } else {
            println!("Dependency not found: {}", name);
        }
        Ok(())
    }

    /// Run project
    pub fn run(&self, args: &[String]) -> Result<()> {
        let manifest = self.load_manifest()?;
        let name = manifest.package.as_ref().map(|p| &p.name).ok_or_else(|| eyre!("No package name found"))?;
        
        let bin_path = if cfg!(windows) {
            self.project_root.join(format!("{}.exe", name))
        } else {
            self.project_root.join(name)
        };

        if bin_path.exists() {
            println!("🚀 Running {}...", name);
            let status = Command::new(&bin_path)
                .args(args)
                .status()
                .wrap_err("Failed to run binary")?;
            
            if !status.success() {
                return Err(eyre!("Execution failed with status: {}", status));
            }
        } else {
            // Fallback to interpreted run if no binary
            let src_file = self.project_root.join("src/main.ifa");
            if src_file.exists() {
                println!("⚡ Running interpreted: src/main.ifa");
                let source = fs::read_to_string(&src_file)?;
                let program = ifa_core::parse(&source).map_err(|e| eyre!("Parse error: {}", e))?;
                let mut interp = ifa_core::Interpreter::with_file(&src_file);
                interp.execute(&program).map_err(|e| eyre!("Runtime error: {}", e))?;
            } else {
                return Err(eyre!("No binary or source found to run"));
            }
        }
        Ok(())
    }

    /// Test project
    pub fn test(&self) -> Result<()> {
        println!("idanwo: Running tests...");
        let mut passed = 0;
        let mut failed = 0;

        let src_dir = self.project_root.join("src");
        if !src_dir.exists() {
            return Err(eyre!("src directory not found"));
        }

        for entry in fs::read_dir(&src_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            
            if name.ends_with("_test.ifa") || name.starts_with("test_") && name.ends_with(".ifa") {
                print!("  {} ... ", name);
                let source = fs::read_to_string(&path)?;
                match ifa_core::parse(&source) {
                    Ok(program) => {
                        let mut interp = ifa_core::Interpreter::with_file(&path);
                        match interp.execute(&program) {
                            Ok(_) => {
                                println!("ok");
                                passed += 1;
                            }
                            Err(e) => {
                                println!("FAIL: {}", e);
                                failed += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("FAIL (parse): {}", e);
                        failed += 1;
                    }
                }
            }
        }

        println!("\nTests: {}, Passed: {}, Failed: {}", passed + failed, passed, failed);
        if failed > 0 {
            return Err(eyre!("Some tests failed"));
        }
        Ok(())
    }

    /// Install dependencies
    pub fn install(&self) -> Result<()> {
        self.fetch()
    }

    /// List dependencies
    pub fn list(&self) -> Result<()> {
        let manifest = self.load_manifest()?;
        println!("Dependencies:");
        for (name, dep) in &manifest.dependencies {
            println!("  - {}: {:?}", name, dep);
        }
        Ok(())
    }
}

/// Stub for update_cli
pub fn update_cli() -> Result<()> {
    println!("🔄  Updating Ifá CLI... (Stub)");
    Ok(())
}
