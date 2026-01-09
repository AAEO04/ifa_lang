//! # Ọjà Package Manager
//!
//! Cargo wrapper for Ifá-Lang package management.
//!
//! Uses Git URLs for dependencies, similar to Go modules.

#![allow(dead_code)]

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Ọjà project manifest (ifa.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfaManifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, Dependency>,
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
    /// Simple version string
    Version(String),
    /// Git dependency
    Git {
        git: String,
        branch: Option<String>,
        tag: Option<String>,
    },
    /// Path dependency
    Path { path: String },
}

/// Ọjà package manager
pub struct Oja {
    project_root: PathBuf,
}

impl Oja {
    /// Create Ọjà for given project root
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Oja {
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    /// Initialize new Ifá project
    pub fn init(&self, project_name: &str) -> Result<()> {
        // Create ifa.toml
        let manifest = IfaManifest {
            package: PackageInfo {
                name: project_name.to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                authors: vec![],
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        };

        let manifest_path = self.project_root.join("ifa.toml");
        let toml = toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
        fs::write(&manifest_path, toml).wrap_err("Failed to write ifa.toml")?;

        // Create src directory with main.ifa
        let src_dir = self.project_root.join("src");
        fs::create_dir_all(&src_dir).wrap_err("Failed to create src directory")?;

        let main_content = r#"// Ifá-Lang main entry point
// Àṣà - Tradition, Culture

jẹ́ kí a sọ "Ẹ káàbọ̀ sí Ifá-Lang!" // Hello, welcome to Ifá-Lang!

// Using the 16 Odù domains:
// Ìrosù.fo("Hello")  -- Console I/O
// Ọ̀bàrà.fikun(5, 3) -- Math
// Ọ̀wọ́nrín.pese(1, 100) -- Random
"#;

        fs::write(src_dir.join("main.ifa"), main_content).wrap_err("Failed to write main.ifa")?;

        // Create .gitignore
        let gitignore_content = r#"# Build outputs
target/
*.ifab

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
"#;
        fs::write(self.project_root.join(".gitignore"), gitignore_content)
            .wrap_err("Failed to write .gitignore")?;

        println!("Created new Ifa project: {}", project_name);
        println!("   src/main.ifa");
        println!("   ifa.toml");
        println!();
        println!("Get started:");
        println!("   ifa run src/main.ifa");

        Ok(())
    }

    /// Load project manifest
    pub fn load_manifest(&self) -> Result<IfaManifest> {
        let manifest_path = self.project_root.join("ifa.toml");
        let content = fs::read_to_string(&manifest_path).wrap_err("Failed to read ifa.toml")?;
        let manifest: IfaManifest =
            toml::from_str(&content).wrap_err("Failed to parse ifa.toml")?;
        Ok(manifest)
    }

    /// Add a Git dependency
    pub fn add(&self, url: &str, alias: Option<&str>) -> Result<()> {
        let mut manifest = self.load_manifest()?;

        // Extract name from URL if no alias
        let name = alias.map(String::from).unwrap_or_else(|| {
            url.rsplit('/')
                .next()
                .unwrap_or("dep")
                .trim_end_matches(".git")
                .to_string()
        });

        // Determine if it's a Git URL or version
        let dep =
            if url.starts_with("http") || url.starts_with("git@") || url.contains("github.com") {
                Dependency::Git {
                    git: url.to_string(),
                    branch: None,
                    tag: None,
                }
            } else if url.starts_with("./") || url.starts_with("../") {
                Dependency::Path {
                    path: url.to_string(),
                }
            } else {
                Dependency::Version(url.to_string())
            };

        manifest.dependencies.insert(name.clone(), dep);

        // Write back manifest
        let toml = toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
        fs::write(self.project_root.join("ifa.toml"), toml).wrap_err("Failed to write ifa.toml")?;

        println!("Added dependency: {}", name);

        Ok(())
    }

    /// Remove a dependency
    pub fn remove(&self, name: &str) -> Result<()> {
        let mut manifest = self.load_manifest()?;

        if manifest.dependencies.remove(name).is_some() {
            let toml =
                toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
            fs::write(self.project_root.join("ifa.toml"), toml)
                .wrap_err("Failed to write ifa.toml")?;
            println!("Removed dependency: {}", name);
        } else {
            println!("Warning: Dependency not found: {}", name);
        }

        Ok(())
    }

    /// Build the project (wraps cargo build)
    pub fn build(&self, release: bool) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build").current_dir(&self.project_root);

        if release {
            cmd.arg("--release");
        }

        let status = cmd.status().wrap_err("Failed to run cargo build")?;

        if status.success() {
            println!("Build successful");
        } else {
            println!("Build failed");
        }

        Ok(())
    }

    /// Run the project
    pub fn run(&self, args: &[String]) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--")
            .args(args)
            .current_dir(&self.project_root);

        let status = cmd.status().wrap_err("Failed to run project")?;

        if !status.success() {
            println!("Run failed");
        }

        Ok(())
    }

    /// Run tests
    pub fn test(&self) -> Result<()> {
        let status = Command::new("cargo")
            .arg("test")
            .current_dir(&self.project_root)
            .status()
            .wrap_err("Failed to run cargo test")?;

        if status.success() {
            println!("All tests passed");
        } else {
            println!("Tests failed");
        }

        Ok(())
    }

    /// Install dependencies
    pub fn install(&self) -> Result<()> {
        // For Cargo-based projects, this just ensures deps are fetched
        let status = Command::new("cargo")
            .arg("fetch")
            .current_dir(&self.project_root)
            .status()
            .wrap_err("Failed to fetch dependencies")?;

        if status.success() {
            println!("Dependencies installed");
        } else {
            println!("Failed to install dependencies");
        }

        Ok(())
    }

    /// List dependencies
    pub fn list(&self) -> Result<()> {
        let manifest = self.load_manifest()?;

        println!("{} v{}", manifest.package.name, manifest.package.version);
        println!();

        if manifest.dependencies.is_empty() {
            println!("No dependencies");
        } else {
            println!("Dependencies:");
            for (name, dep) in &manifest.dependencies {
                match dep {
                    Dependency::Version(v) => println!("  {} = {}", name, v),
                    Dependency::Git { git, branch, tag } => {
                        let extra = branch
                            .as_ref()
                            .map(|b| format!(" (branch: {})", b))
                            .or_else(|| tag.as_ref().map(|t| format!(" (tag: {})", t)))
                            .unwrap_or_default();
                        println!("  {} = {}{}", name, git, extra);
                    }
                    Dependency::Path { path } => println!("  {} = path:{}", name, path),
                }
            }
        }

        Ok(())
    }
}

/// Generate Cargo.toml from ifa.toml
pub fn generate_cargo_toml(manifest: &IfaManifest, output: &Path) -> Result<()> {
    let mut cargo = String::new();

    cargo.push_str(&format!(
        r#"[package]
name = "{}"
version = "{}"
edition = "2021"
"#,
        manifest.package.name, manifest.package.version
    ));

    if !manifest.package.description.is_empty() {
        cargo.push_str(&format!(
            "description = \"{}\"\n",
            manifest.package.description
        ));
    }

    if !manifest.package.authors.is_empty() {
        let authors: Vec<String> = manifest
            .package
            .authors
            .iter()
            .map(|a| format!("\"{}\"", a))
            .collect();
        cargo.push_str(&format!("authors = [{}]\n", authors.join(", ")));
    }

    // Add ifa-std dependency
    cargo.push_str("\n[dependencies]\n");
    cargo.push_str("ifa-std = { path = \"../path/to/ifa-std\" }  # TODO: publish to crates.io\n");

    for (name, dep) in &manifest.dependencies {
        match dep {
            Dependency::Version(v) => {
                cargo.push_str(&format!("{} = \"{}\"\n", name, v));
            }
            Dependency::Git { git, branch, tag } => {
                let extra = branch
                    .as_ref()
                    .map(|b| format!(", branch = \"{}\"", b))
                    .or_else(|| tag.as_ref().map(|t| format!(", tag = \"{}\"", t)))
                    .unwrap_or_default();
                cargo.push_str(&format!("{} = {{ git = \"{}\"{}  }}\n", name, git, extra));
            }
            Dependency::Path { path } => {
                cargo.push_str(&format!("{} = {{ path = \"{}\" }}\n", name, path));
            }
        }
    }

    fs::write(output, cargo).wrap_err("Failed to write Cargo.toml")?;

    Ok(())
}

/// Update CLI to latest version via GitHub Releases
pub fn update_cli() -> Result<()> {
    println!("Checking for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("AAEO04")
        .repo_name("ifa-lang")
        .bin_name("ifa")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
        .wrap_err("Failed to build updater")?
        .update()
        .wrap_err("Failed to update binary")?;

    match status {
        self_update::Status::UpToDate(v) => {
            println!("Already up to date: v{}", v);
        }
        self_update::Status::Updated(v) => {
            println!("Upgrade successful! New version: v{}", v);
        }
    }

    Ok(())
}
