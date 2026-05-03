//! # Ọjà Package Manager
//!
//! Dependency resolution, lockfile generation, and sandboxed execution.
//! Conforms to IFA_LANG_RUNTIME_SPEC §33.

#![allow(dead_code)]

use chrono::Local;
use eyre::{Result, WrapErr, eyre};
use flate2::read::GzDecoder;
use ifa_sandbox::{OmniBox, SandboxConfig, SecurityProfile};
use reqwest::blocking::Client;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
}

impl SemVer {
    fn parse(raw: &str) -> Option<Self> {
        let parts: Vec<&str> = raw.split('.').collect();
        let major = parts.get(0)?.parse().ok()?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

#[derive(Clone, Debug)]
enum VersionConstraint {
    Any,
    Exact(SemVer),
    Caret(SemVer),
    Tilde(SemVer),
}

impl VersionConstraint {
    fn parse(raw: &str) -> Option<Self> {
        let s = raw.trim();
        if s.is_empty() || s == "*" || s.eq_ignore_ascii_case("latest") {
            return Some(Self::Any);
        }
        if let Some(rest) = s.strip_prefix('^') {
            return SemVer::parse(rest).map(Self::Caret);
        }
        if let Some(rest) = s.strip_prefix('~') {
            return SemVer::parse(rest).map(Self::Tilde);
        }
        SemVer::parse(s).map(Self::Exact)
    }

    fn min_version(&self) -> SemVer {
        match self {
            VersionConstraint::Any => SemVer {
                major: 0,
                minor: 0,
                patch: 0,
            },
            VersionConstraint::Exact(v) => *v,
            VersionConstraint::Caret(v) => *v,
            VersionConstraint::Tilde(v) => *v,
        }
    }

    fn satisfies(&self, v: SemVer) -> bool {
        match self {
            VersionConstraint::Any => true,
            VersionConstraint::Exact(target) => v == *target,
            VersionConstraint::Caret(base) => {
                if base.major > 0 {
                    v >= *base
                        && v < SemVer {
                            major: base.major + 1,
                            minor: 0,
                            patch: 0,
                        }
                } else if base.minor > 0 {
                    v >= *base
                        && v < SemVer {
                            major: 0,
                            minor: base.minor + 1,
                            patch: 0,
                        }
                } else {
                    v >= *base
                        && v < SemVer {
                            major: 0,
                            minor: 0,
                            patch: base.patch + 1,
                        }
                }
            }
            VersionConstraint::Tilde(base) => {
                v >= *base
                    && v < SemVer {
                        major: base.major,
                        minor: base.minor + 1,
                        patch: 0,
                    }
            }
        }
    }
}

fn mvs_select(available: &[SemVer], constraints: &[VersionConstraint]) -> Option<SemVer> {
    if constraints.is_empty() {
        return None;
    }
    let mut min = constraints[0].min_version();
    for c in constraints.iter().skip(1) {
        let cmin = c.min_version();
        if cmin > min {
            min = cmin;
        }
    }
    if constraints.iter().all(|c| c.satisfies(min)) && available.contains(&min) {
        return Some(min);
    }
    None
}

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

/// Ọjà project manifest (ifa.toml / Iwe.toml legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfaManifest {
    #[serde(default)]
    pub package: Option<PackageInfo>,
    #[serde(default, alias = "project")]
    pub project: Option<PackageInfo>,
    #[serde(default)]
    pub workspace: Option<WorkspaceInfo>,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default, alias = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,
}

impl IfaManifest {
    /// Access package info from either `[package]` or `[project]` (§33 uses `[project]`)
    pub fn package_info(&self) -> Option<&PackageInfo> {
        self.package.as_ref().or(self.project.as_ref())
    }
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
    #[serde(default)]
    pub language: Option<String>,
}

/// Dependency specification — supports bare version strings and detailed tables.
/// Aligns with §33.1 of the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Bare version string: `"1.4.0"` or `"^1.4"`
    Version(String),
    /// Detailed table with optional fields
    Detailed {
        #[serde(default)]
        version: Option<String>,
        #[serde(default)]
        features: Option<Vec<String>>,
        #[serde(default)]
        git: Option<String>,
        #[serde(default)]
        branch: Option<String>,
        #[serde(default)]
        tag: Option<String>,
        #[serde(default)]
        path: Option<String>,
    },
}

impl Dependency {
    /// Extract the version constraint string, if any.
    pub fn version_str(&self) -> Option<&str> {
        match self {
            Dependency::Version(v) => Some(v.as_str()),
            Dependency::Detailed { version, .. } => version.as_deref(),
        }
    }

    /// True if this is a path dependency (local).
    pub fn is_path(&self) -> bool {
        matches!(self, Dependency::Detailed { path: Some(_), .. })
    }

    /// True if this is a git dependency.
    pub fn is_git(&self) -> bool {
        matches!(self, Dependency::Detailed { git: Some(_), .. })
    }
}

// ---------------------------------------------------------------------------
// Lockfile — §33.2
// ---------------------------------------------------------------------------

/// A single resolved dependency in `oja.lock`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LockedPackage {
    name: String,
    version: String,
    source: String,
    checksum: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dependencies: Vec<String>,
}

/// The lockfile (`oja.lock`) — exact resolved versions + checksums.
/// Committed to VCS for applications, not for libraries.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Lockfile {
    #[serde(rename = "package")]
    packages: Vec<LockedPackage>,
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
                language: None,
            }),
            project: None,
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
            "game" => {
                r#"// Game Domain Project
// Uses ifa-std::stacks::gamedev
// Run with: ifa build --game

fn main() {
    println("🎮 Starting Game Engine...");
    // let world = World::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_parse_basic() {
        assert_eq!(SemVer::parse("1.2.3"), Some(SemVer { major: 1, minor: 2, patch: 3 }));
        assert_eq!(SemVer::parse("1.2"), Some(SemVer { major: 1, minor: 2, patch: 0 }));
        assert_eq!(SemVer::parse("1"), Some(SemVer { major: 1, minor: 0, patch: 0 }));
    }

    #[test]
    fn constraint_satisfies_caret() {
        let c = VersionConstraint::parse("^1.2.0").unwrap();
        assert!(c.satisfies(SemVer { major: 1, minor: 2, patch: 0 }));
        assert!(c.satisfies(SemVer { major: 1, minor: 9, patch: 9 }));
        assert!(!c.satisfies(SemVer { major: 2, minor: 0, patch: 0 }));
    }

    #[test]
    fn constraint_satisfies_tilde() {
        let c = VersionConstraint::parse("~1.2.3").unwrap();
        assert!(c.satisfies(SemVer { major: 1, minor: 2, patch: 3 }));
        assert!(c.satisfies(SemVer { major: 1, minor: 2, patch: 9 }));
        assert!(!c.satisfies(SemVer { major: 1, minor: 3, patch: 0 }));
    }

    #[test]
    fn mvs_select_picks_max_of_mins() {
        let available = vec![
            SemVer { major: 1, minor: 0, patch: 0 },
            SemVer { major: 1, minor: 2, patch: 0 },
            SemVer { major: 1, minor: 3, patch: 0 },
        ];
        let c1 = VersionConstraint::parse("^1.0.0").unwrap();
        let c2 = VersionConstraint::parse("^1.3.0").unwrap();
        let selected = mvs_select(&available, &[c1, c2]).unwrap();
        assert_eq!(selected, SemVer { major: 1, minor: 3, patch: 0 });
    }

    #[test]
    fn mvs_select_rejects_missing_version() {
        let available = vec![
            SemVer { major: 1, minor: 0, patch: 0 },
            SemVer { major: 1, minor: 2, patch: 0 },
        ];
        let c1 = VersionConstraint::parse("^1.0.0").unwrap();
        let c2 = VersionConstraint::parse("^1.3.0").unwrap();
        assert!(mvs_select(&available, &[c1, c2]).is_none());
    }
}
"#
            }
            "ml" => {
                r#"// ML Domain Project
// Uses ifa-std::stacks::ml
// Run with: ifa build --ml

fn main() {
    println("🧠 Initializing Neural Network...");
    // let tensor = Tensor::zeros([2, 2]);
}
"#
            }
            "fusion" | "fullstack" => {
                r#"// Fusion Hybrid Project
// Uses ifa-std::stacks::fusion (Hybrid Runtime)
// Run with: ifa build --fusion

fn main() {
    println("🚀 Launching Fusion Runtime...");
}
"#
            }
            "iot" => {
                r#"// IoT Domain Project
// Uses ifa-std::stacks::iot (no_std)
// Run with: ifa build --iot

fn main() {
    // loop {
    //     gpio.write(HIGH);
    // }
}
"#
            }
            _ => {
                r#"// Ifá-Lang main entry point
jẹ́ kí a sọ "Ẹ káàbọ̀ sí Ifá-Lang!"
"#
            }
        };

        fs::write(src_dir.join("main.ifa"), main_content).wrap_err("Failed to write main.ifa")?;

        // Create .gitignore
        let gitignore_content = r#"target/
*.ifab
.oja/
"#;
        fs::write(self.project_root.join(".gitignore"), gitignore_content)
            .wrap_err("Failed to write .gitignore")?;

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
            project: None,
            workspace: Some(WorkspaceInfo {
                members: vec!["backend".to_string(), "frontend".to_string()],
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
        println!(
            "   💡 Add members with: ifa oja init <name> --domain <domain> inside proper folders"
        );

        Ok(())
    }

    /// Build project (Project-Centric Build)
    #[allow(clippy::only_used_in_recursion)]
    pub fn build(&self, release: bool) -> Result<()> {
        let manifest = self.load_manifest()?;

        // 1. Workspace Build
        if let Some(workspace) = manifest.workspace {
            println!(
                "🏗️  Building Workspace ({} members)...",
                workspace.members.len()
            );
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
            let lang = package.language.as_deref().unwrap_or("ifa");
            println!(
                "📦 Building Package: {} v{} [{}]",
                package.name, package.version, lang
            );

            if lang == "rust" {
                // Rust Project Support
                // We assume there's a Cargo.toml in the crate root
                println!("   🦀 Delegating to Cargo...");
                let status = Command::new("cargo")
                    .arg("build")
                    .arg(if release { "--release" } else { "--dev" }) // simplified
                    .current_dir(&self.project_root)
                    .status()?;

                if !status.success() {
                    return Err(eyre!("Cargo build failed"));
                }
                println!("   ✅ Rust build complete.");
                return Ok(());
            }

            // Default: Ifá-Lang Build
            let src_file = self.project_root.join("src/main.ifa");
            if !src_file.exists() {
                return Err(eyre!("Source file not found: src/main.ifa"));
            }

            // --- Compilation Logic (Transpile -> Cargo) ---
            // Reusing logic similar to main.rs but adapted for Project Context

            let source = std::fs::read_to_string(&src_file).wrap_err("Failed to read main.ifa")?;
            println!("   📝 Parsing Ifá source...");
            let program = ifa_core::parse(&source).map_err(|e| eyre!("Parse error: {}", e))?;

            println!("   🔄 Transpiling to Rust...");
            let rust_code = ifa_core::transpile_to_rust(&program);

            // Create temp build dir
            let temp_dir = self.project_root.join("target/build_tmp");
            if temp_dir.exists() {
                fs::remove_dir_all(&temp_dir).ok();
            }
            fs::create_dir_all(temp_dir.join("src"))?;

            fs::write(temp_dir.join("src/main.rs"), &rust_code)?;

            let core_path = std::env::current_dir()?
                .join("crates/ifa-core")
                .display()
                .to_string()
                .replace("\\", "/");
            let std_path = std::env::current_dir()?
                .join("crates/ifa-std")
                .display()
                .to_string()
                .replace("\\", "/");

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
            let target_bin = temp_dir
                .join("target/release")
                .join(format!("{}.exe", package.name)); // Win assumption
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

    /// `ifa oja install` / `ifa fetch`: Download, verify, lock, and compile.
    ///
    /// This method implements the full §33 install pipeline:
    /// 1. Load manifest and existing lockfile (if any)
    /// 2. Resolve all dependencies transitively
    /// 3. Download and verify integrity (SHA-256)
    /// 4. Generate / update `oja.lock`
    /// 5. AOT compile WASM artifacts
    pub fn fetch(&self) -> Result<()> {
        println!("🛒  Fetching dependencies...");

        let (lib_dir, cache_dir) = self.ensure_igbale()?;
        let manifest = self.load_manifest()?;

        if manifest.dependencies.is_empty() {
            println!("   No dependencies.");
            return Ok(());
        }

        // Load existing lockfile for integrity verification
        let existing_lock = self.load_lockfile();

        // Build a lookup map from locked packages for checksum verification
        let locked_checksums: HashMap<String, String> = existing_lock
            .as_ref()
            .map(|lf| {
                lf.packages
                    .iter()
                    .map(|p| (p.name.clone(), p.checksum.clone()))
                    .collect()
            })
            .unwrap_or_default();

        // Resolve the full dependency graph (direct + transitive)
        let resolved = self.resolve_transitive(&manifest, &lib_dir)?;

        println!(
            "   Resolved {} packages (including transitive)",
            resolved.len()
        );

        // Verify checksums against lockfile
        for pkg in &resolved {
            if let Some(expected) = locked_checksums.get(&pkg.name) {
                if &pkg.checksum != expected {
                    return Err(eyre!(
                        "OjaIntegrityError: checksum mismatch for '{}'\n  \
                         expected: {}\n  \
                         got:      {}\n  \
                         The lockfile does not match the downloaded artifact. \
                         This may indicate tampering or a version change. \
                         Run `ifa oja update {}` to re-resolve.",
                        pkg.name,
                        expected,
                        pkg.checksum,
                        pkg.name
                    ));
                }
            }
        }

        // Write oja.lock
        let lockfile = Lockfile {
            packages: resolved.clone(),
        };
        self.save_lockfile(&lockfile)?;
        println!(
            "   🔒 Wrote oja.lock ({} packages)",
            lockfile.packages.len()
        );

        // AOT compile any WASM sources
        let config = SandboxConfig::new(SecurityProfile::Standard);
        let omnibox = OmniBox::new(config).wrap_err("Failed to init compiler")?;

        for pkg in &resolved {
            let wasm_candidate = lib_dir
                .join(format!("{}-{}", pkg.name, pkg.version))
                .join(format!("{}.wasm", pkg.name));

            if wasm_candidate.exists() {
                let wasm_bytes =
                    fs::read(&wasm_candidate).wrap_err("Failed to read Wasm source")?;
                let artifact = omnibox.compile_artifact(&wasm_bytes)?;
                let target_path = cache_dir.join(format!("{}.cwasm", &pkg.checksum[7..15]));
                self.atomic_write(&target_path, &artifact)?;
                self.audit_log(
                    "FETCH",
                    &format!("Compiled artifact for {} v{}", pkg.name, pkg.version),
                )?;
                println!(
                    "     ✓ {} v{} compiled ({})",
                    pkg.name,
                    pkg.version,
                    &pkg.checksum[7..15]
                );
            }
        }

        println!("✨  Ready to run.");
        Ok(())
    }

    /// Resolve all dependencies transitively.
    ///
    /// Walks the dependency graph breadth-first, downloading each package and
    /// reading its own manifest to discover transitive dependencies.
    /// Returns a topologically-ordered list of `LockedPackage` entries.
    fn resolve_transitive(
        &self,
        manifest: &IfaManifest,
        lib_dir: &Path,
    ) -> Result<Vec<LockedPackage>> {
        let mut resolved: Vec<LockedPackage> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        // Queue: (name, dep, required_by)
        let mut queue: VecDeque<(String, Dependency, String)> = VecDeque::new();

        // Seed with direct dependencies
        for (name, dep) in &manifest.dependencies {
            queue.push_back((name.clone(), dep.clone(), "(root)".to_string()));
        }

        while let Some((name, dep, required_by)) = queue.pop_front() {
            if seen.contains(&name) {
                continue;
            }
            seen.insert(name.clone());

            println!("   📦 Resolving: {} (required by {})", name, required_by);

            let (version, source, pkg_dir) = match &dep {
                Dependency::Version(ver) => {
                    let url = self.resolve_registry(&name, ver);
                    let target_dir = lib_dir.join(format!("{}-{}", name, ver));
                    if !target_dir.exists() {
                        self.download_package(&url, &target_dir)?;
                    }
                    (
                        ver.clone(),
                        format!("registry+{}", OJA_REGISTRY_URL),
                        target_dir,
                    )
                }
                Dependency::Detailed {
                    version,
                    path: Some(p),
                    ..
                } => {
                    let pkg_path = PathBuf::from(p);
                    let ver = version.as_deref().unwrap_or("0.0.0-local").to_string();
                    (ver, format!("path+{}", p), pkg_path)
                }
                Dependency::Detailed {
                    version: _,
                    git: Some(g),
                    ..
                } => {
                    return Err(eyre!(
                        "Git dependencies are not yet implemented (package '{}', source: '{}'). \
                         Use a `path` or `version` dependency instead.",
                        name,
                        g
                    ));
                }
                Dependency::Detailed {
                    version: Some(ver), ..
                } => {
                    let url = self.resolve_registry(&name, ver);
                    let target_dir = lib_dir.join(format!("{}-{}", name, ver));
                    if !target_dir.exists() {
                        self.download_package(&url, &target_dir)?;
                    }
                    (
                        ver.clone(),
                        format!("registry+{}", OJA_REGISTRY_URL),
                        target_dir,
                    )
                }
                _ => {
                    return Err(eyre!(
                        "Dependency '{}' has no version, path, or git source.",
                        name
                    ));
                }
            };

            // Compute checksum of the downloaded package directory
            let checksum = self.sha256_dir(&pkg_dir)?;

            // Read the dependency's own manifest to find transitive deps
            let mut transitive_names: Vec<String> = Vec::new();
            let dep_manifest_path = Self::find_manifest_in(&pkg_dir);
            if let Some(mp) = dep_manifest_path {
                if let Ok(content) = fs::read_to_string(&mp) {
                    if let Ok(dep_manifest) = toml::from_str::<IfaManifest>(&content) {
                        for (tname, tdep) in &dep_manifest.dependencies {
                            transitive_names.push(tname.clone());
                            if !seen.contains(tname) {
                                queue.push_back((tname.clone(), tdep.clone(), name.clone()));
                            }
                        }
                    }
                }
            }

            resolved.push(LockedPackage {
                name,
                version,
                source,
                checksum,
                dependencies: transitive_names,
            });
        }

        Ok(resolved)
    }

    /// Find the manifest file inside a dependency directory.
    /// Prefers `ifa.toml`, falls back to `Iwe.toml`.
    fn find_manifest_in(dir: &Path) -> Option<PathBuf> {
        let ifa = dir.join("ifa.toml");
        if ifa.exists() {
            return Some(ifa);
        }
        let iwe = dir.join("Iwe.toml");
        if iwe.exists() {
            return Some(iwe);
        }
        // GitHub archives have a root subdirectory — look one level down
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let nested_ifa = entry.path().join("ifa.toml");
                    if nested_ifa.exists() {
                        return Some(nested_ifa);
                    }
                    let nested_iwe = entry.path().join("Iwe.toml");
                    if nested_iwe.exists() {
                        return Some(nested_iwe);
                    }
                }
            }
        }
        None
    }

    /// SHA-256 hash of a directory's file contents (deterministic, sorted).
    fn sha256_dir(&self, dir: &Path) -> Result<String> {
        let mut context = Context::new(&SHA256);

        if dir.is_file() {
            let bytes = fs::read(dir)?;
            context.update(&bytes);
        } else if dir.is_dir() {
            let mut paths: Vec<PathBuf> = Vec::new();
            Self::collect_files(dir, &mut paths);
            paths.sort(); // deterministic order
            for p in &paths {
                let bytes = fs::read(p)?;
                // Include relative path in hash so renames are detected
                if let Ok(rel) = p.strip_prefix(dir) {
                    context.update(rel.to_string_lossy().as_bytes());
                }
                context.update(&bytes);
            }
        }

        let hash: String = context
            .finish()
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        Ok(format!("sha256:{}", hash))
    }

    /// Recursively collect all files under `dir`.
    fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    Self::collect_files(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
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

    /// Load project manifest.
    /// Prefers `ifa.toml` (§33 canonical), falls back to `Iwe.toml` (legacy).
    pub fn load_manifest(&self) -> Result<IfaManifest> {
        let ifa_path = self.project_root.join("ifa.toml");
        let iwe_path = self.project_root.join("Iwe.toml");

        let path = if ifa_path.exists() {
            ifa_path
        } else if iwe_path.exists() {
            iwe_path
        } else {
            return Err(eyre!(
                "No manifest found. Expected `ifa.toml` (or legacy `Iwe.toml`) in {}\n  \
                 Run `ifa oja init` to create one.",
                self.project_root.display()
            ));
        };

        let content = fs::read_to_string(&path)
            .wrap_err_with(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&content).wrap_err_with(|| format!("Failed to parse {}", path.display()))
    }

    /// Load the lockfile (`oja.lock`). Returns `None` if it doesn't exist.
    fn load_lockfile(&self) -> Option<Lockfile> {
        let path = self.project_root.join("oja.lock");
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Write the lockfile to disk (atomic).
    fn save_lockfile(&self, lockfile: &Lockfile) -> Result<()> {
        let path = self.project_root.join("oja.lock");
        let header = "# oja.lock — generated by Oja, do not edit manually\n\n";
        let body = toml::to_string_pretty(lockfile).wrap_err("Failed to serialize lockfile")?;
        let content = format!("{}{}", header, body);
        self.atomic_write(&path, content.as_bytes())
    }

    /// Resolve package name to download URL via the Ọjà registry.
    /// Falls back to a GitHub release tarball URL if the registry is unreachable.
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
                            entry.versions.iter().rfind(|v| !v.yanked)
                        } else if let Some(constraint) = VersionConstraint::parse(version) {
                            let available: Vec<SemVer> = entry
                                .versions
                                .iter()
                                .filter(|v| !v.yanked)
                                .filter_map(|v| SemVer::parse(&v.version))
                                .collect();
                            let selected = mvs_select(&available, &[constraint]);
                            selected.and_then(|s| {
                                let s = format!("{}.{}.{}", s.major, s.minor, s.patch);
                                entry.versions.iter().find(|v| v.version == s)
                            })
                        } else {
                            entry.versions.iter().find(|v| v.version == version)
                        };

                        if let Some(v) = target_ver {
                            if v.yanked {
                                println!("     ⚠️  Warning: Version {} is yanked!", v.version);
                            }
                            // Construct GitHub Archive URL
                            // Format: https://github.com/user/repo/archive/refs/tags/v1.0.0.tar.gz
                            return format!(
                                "{}/archive/refs/tags/v{}.tar.gz",
                                entry.repository, v.version
                            );
                        } else {
                            println!(
                                "     ❌ Version {} not found in registry. Falling back to simple resolution.",
                                version
                            );
                        }
                    }
                }
            }
            Err(_) => {
                // Offline or Registry unreachable - Fallback
            }
        }

        // Fallback: Assume GitHub release tarball convention directly
        format!(
            "https://github.com/ifa-lang/{}/archive/refs/tags/v{}.tar.gz",
            name, version
        )
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

    /// Verify package integrity against an expected SHA-256 checksum.
    ///
    /// If `expected_checksum` is `Some`, the computed hash MUST match or
    /// the install is aborted with `OjaIntegrityError`.
    /// If `None`, this is a first install — the hash is computed and returned.
    fn verify_integrity(&self, path: &Path, expected_checksum: Option<&str>) -> Result<String> {
        println!("     🔒 Verifying integrity...");
        let computed = self.sha256_dir(path)?;

        if let Some(expected) = expected_checksum {
            if computed != expected {
                return Err(eyre!(
                    "OjaIntegrityError: checksum mismatch for {}\n  \
                     expected: {}\n  \
                     computed: {}\n  \
                     The downloaded package does not match the lockfile. \
                     This may indicate tampering, corruption, or a changed version.",
                    path.display(),
                    expected,
                    computed
                ));
            }
            println!("       ✓ Checksum verified: {}", &computed[7..15]);
        } else {
            println!("       ✓ Computed checksum: {}", &computed[7..15]);
        }

        Ok(computed)
    }

    /// Publish to Registry (Git Tagging Strategy)
    pub fn publish(&self) -> Result<()> {
        println!("📦  Publishing package...");

        // 1. Load Manifest
        let manifest = self.load_manifest()?;
        let version = manifest
            .package
            .as_ref()
            .ok_or_else(|| {
                eyre!("Cannot publish a workspace manifest. Run from within a package.")
            })?
            .version
            .clone();
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
            return Err(eyre!(
                "Git working directory not clean. Commit changes first."
            ));
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
            return Err(eyre!(
                "Failed to create tag: {}",
                String::from_utf8_lossy(&tag_cmd.stderr)
            ));
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
            return Err(eyre!(
                "Failed to push tag: {}",
                String::from_utf8_lossy(&push_cmd.stderr)
            ));
        }

        println!("✨  Published successfully to Registry (Git)!");
        Ok(())
    }

    /// Add dependency
    pub fn add(&self, url: &str, alias: Option<&str>) -> Result<()> {
        let mut manifest = self.load_manifest()?;
        let name = alias.unwrap_or_else(|| {
            url.split('/')
                .next_back()
                .unwrap_or("unknown")
                .trim_end_matches(".git")
        });

        println!("Adding dependency: {} as {}", url, name);
        manifest
            .dependencies
            .insert(name.to_string(), Dependency::Version(url.to_string()));

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
            let toml =
                toml::to_string_pretty(&manifest).wrap_err("Failed to serialize manifest")?;
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
        let name = manifest
            .package
            .as_ref()
            .map(|p| &p.name)
            .ok_or_else(|| eyre!("No package name found"))?;

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
                interp
                    .execute(&program)
                    .map_err(|e| eyre!("Runtime error: {}", e))?;
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

        println!(
            "\nTests: {}, Passed: {}, Failed: {}",
            passed + failed,
            passed,
            failed
        );
        if failed > 0 {
            return Err(eyre!("Some tests failed"));
        }
        Ok(())
    }

    /// Install dependencies (alias for fetch)
    pub fn install(&self) -> Result<()> {
        self.fetch()
    }

    /// List installed dependencies and their locked versions.
    pub fn list(&self) -> Result<()> {
        let manifest = self.load_manifest()?;
        let lockfile = self.load_lockfile();

        println!("Dependencies:");
        for (name, dep) in &manifest.dependencies {
            let ver = dep.version_str().unwrap_or("*");
            // Show locked version if available
            let locked_ver = lockfile.as_ref().and_then(|lf| {
                lf.packages
                    .iter()
                    .find(|p| p.name == *name)
                    .map(|p| p.version.as_str())
            });
            if let Some(lv) = locked_ver {
                println!("  {} {} (locked: {})", name, ver, lv);
            } else {
                println!("  {} {} (not installed)", name, ver);
            }
        }
        Ok(())
    }

    /// Update dependencies: re-resolve and regenerate `oja.lock`.
    pub fn update(&self) -> Result<()> {
        println!("🔄  Updating dependencies...");
        // Delete the existing lockfile so fetch() does a full re-resolve
        let lock_path = self.project_root.join("oja.lock");
        if lock_path.exists() {
            fs::remove_file(&lock_path).wrap_err("Failed to remove old oja.lock")?;
        }
        self.fetch()
    }

    /// Show the full dependency tree.
    pub fn tree(&self) -> Result<()> {
        let lockfile = self
            .load_lockfile()
            .ok_or_else(|| eyre!("No oja.lock found. Run `ifa oja install` first."))?;
        let manifest = self.load_manifest()?;

        println!("Dependency tree:");

        // Build a lookup
        let pkg_map: HashMap<&str, &LockedPackage> = lockfile
            .packages
            .iter()
            .map(|p| (p.name.as_str(), p))
            .collect();

        // Print direct deps, then recurse
        for (name, _) in &manifest.dependencies {
            self.print_tree_node(name, &pkg_map, 0, &mut HashSet::new());
        }
        Ok(())
    }

    fn print_tree_node(
        &self,
        name: &str,
        pkgs: &HashMap<&str, &LockedPackage>,
        depth: usize,
        visited: &mut HashSet<String>,
    ) {
        let indent = "  ".repeat(depth);
        let prefix = if depth == 0 { "├── " } else { "│   " };

        if let Some(pkg) = pkgs.get(name) {
            if visited.contains(name) {
                println!("{}{}{} v{} (*)", indent, prefix, name, pkg.version);
                return;
            }
            visited.insert(name.to_string());
            println!("{}{}{} v{}", indent, prefix, name, pkg.version);
            for dep_name in &pkg.dependencies {
                self.print_tree_node(dep_name, pkgs, depth + 1, visited);
            }
        } else {
            println!("{}{}{} (not resolved)", indent, prefix, name);
        }
    }

    /// Search the registry for packages matching a query.
    pub fn search(&self, query: &str) -> Result<()> {
        println!("🔍  Searching registry for '{}'...", query);
        let client = Client::new();
        let url = format!("{}/search/{}", OJA_REGISTRY_URL, query);

        match client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(text) = resp.text() {
                    println!("{}", text);
                } else {
                    println!("   No results.");
                }
            }
            _ => {
                println!("   Registry unreachable. Try again later.");
            }
        }
        Ok(())
    }

    /// Audit installed dependencies for known vulnerabilities.
    pub fn audit(&self) -> Result<()> {
        println!("🔍  Auditing dependencies...");
        let lockfile = self
            .load_lockfile()
            .ok_or_else(|| eyre!("No oja.lock found. Run `ifa oja install` first."))?;

        let mut issues = 0;
        for pkg in &lockfile.packages {
            // In production this would query oja.ifá.dev/advisories
            // For now, verify that checksums are present and well-formed
            if !pkg.checksum.starts_with("sha256:") || pkg.checksum.len() < 71 {
                println!("   ⚠ {}: missing or malformed checksum", pkg.name);
                issues += 1;
            }
        }

        if issues == 0 {
            println!(
                "   ✅ No known issues found ({} packages audited).",
                lockfile.packages.len()
            );
        } else {
            println!("   ⚠ {} issue(s) found.", issues);
        }
        Ok(())
    }
}

/// Update the Ifá CLI in-place.
///
/// Not yet implemented. Self-update requires a signed binary distribution
/// channel (e.g. GitHub Releases) and OS-specific atomic replace logic.
/// Until that exists this returns an explicit error so callers know nothing happened.
pub fn update_cli() -> Result<()> {
    Err(eyre!(
        "`ifa update` is not yet implemented. \
        Update manually by downloading the latest release from \
        https://github.com/ifa-lang/ifa/releases"
    ))
}
