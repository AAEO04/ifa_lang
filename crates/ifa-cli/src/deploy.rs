//! # Deployment Manager
//!
//! "Zero-Config Deployment": Scans code to detect required capabilities.
//! Generates secure manifests (Iwe.toml / AppArmor profiles).

use eyre::{Result, WrapErr};
use ifa_babalawo::infer_capabilities;
use ifa_core::parse;
use ifa_sandbox::{CapabilitySet, Ofun};
use std::path::{Path, PathBuf};

/// Scan directory and generate capability manifest
pub fn scan_and_generate(path: &Path) -> Result<()> {
    println!("🔍 Scanning project at: {}", path.display());

    let mut total_caps = CapabilitySet::new();
    // Default grants
    total_caps.grant(Ofun::Stdio);

    // Find sources
    let sources = traverse_dir(path)?;
    if sources.is_empty() {
        println!("⚠️ No .ifa source files found.");
        return Ok(());
    }

    println!("   Found {} source files.", sources.len());

    for src_path in sources {
        debug_assert!(src_path.exists());
        let content = std::fs::read_to_string(&src_path)
            .wrap_err_with(|| format!("Failed to read {}", src_path.display()))?;

        match parse(&content) {
            Ok(program) => {
                let caps = infer_capabilities(&program);
                // Manual merge since .merge() isn't on CapabilitySet
                for cap in caps.all() {
                    total_caps.grant(cap.clone());
                }
            }
            Err(e) => {
                println!("   ⚠️ Parse error in {}: {}", src_path.display(), e);
            }
        }
    }

    println!();
    println!("🛡️  Inferred Capabilities (Odu Ofun):");
    print_capabilities(&total_caps);

    println!();
    println!("📜 Generated Manifest Snippet (Iwe.toml):");
    println!("```toml");
    println!("[package.capabilities]");

    // Check for network - iterate manually
    let has_network_star = total_caps
        .all()
        .iter()
        .any(|c| matches!(c, Ofun::Network { domains } if domains.contains(&"*".to_string())));

    if has_network_star {
        println!("network = true # Detected 'Otura'");
    }

    for grant in total_caps.all() {
        match grant {
            Ofun::ReadFiles { root } => println!("read = [{:?}]", root),
            Ofun::WriteFiles { root } => println!("write = [{:?}]", root),
            Ofun::Network { domains } => println!("network = {:?}", domains),
            Ofun::Environment { keys } => println!("env = {:?}", keys),
            Ofun::Time => println!("time = true"),
            Ofun::Random => println!("random = true"),
            Ofun::Stdio => println!("stdio = true"),
            _ => {}
        }
    }
    println!("```");

    Ok(())
}

fn traverse_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_file() {
        if dir.extension().is_some_and(|e| e == "ifa") {
            files.push(dir.to_path_buf());
        }
    } else if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(traverse_dir(&path)?);
            } else if path.extension().is_some_and(|e| e == "ifa") {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn print_capabilities(caps: &CapabilitySet) {
    for grant in caps.all() {
        match grant {
            Ofun::ReadFiles { root } => println!("   - Read: {:?}", root),
            Ofun::WriteFiles { root } => println!("   - Write: {:?}", root),
            Ofun::Network { .. } => println!("   - Network: Full Access"), // simplified
            Ofun::Time => println!("   - Access to Time"),
            Ofun::Environment { .. } => println!("   - Access to Env"),
            Ofun::Execute { .. } => println!("   - Spawn Processes"),
            Ofun::Stdio => {} // default
            _ => println!("   - {:?}", grant),
        }
    }
}
