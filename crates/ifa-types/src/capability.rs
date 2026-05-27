use std::path::PathBuf;

/// Ọ̀fún Capability Definition
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Ofun {
    /// Read access to filesystem path
    ReadFiles { root: PathBuf },
    /// Write access to filesystem path
    WriteFiles { root: PathBuf },
    /// Network access to specific domains
    Network { domains: Vec<String> },
    /// Execute subprocesses
    Execute { programs: Vec<String> },
    /// Access environment variables
    Environment { keys: Vec<String> },
    /// High-resolution time
    Time,
    /// Random number generation
    Random,
    /// Standard I/O (stdin/stdout/stderr)
    Stdio,
    /// Cryptographic operations (hashing, encryption, etc.)
    Crypto,
    /// Polyglot bridge access (js, python, etc.)
    Bridge { language: String },
}

fn covers(granted: &Ofun, required: &Ofun) -> bool {
    match (granted, required) {
        (Ofun::ReadFiles { root: g }, Ofun::ReadFiles { root: r }) => {
            let r_canon = canonicalize_safe(r);
            let g_canon = canonicalize_safe(g);
            r_canon.starts_with(&g_canon)
        }
        (Ofun::WriteFiles { root: g }, Ofun::WriteFiles { root: r }) => {
            let r_canon = canonicalize_safe(r);
            let g_canon = canonicalize_safe(g);
            r_canon.starts_with(&g_canon)
        }
        (Ofun::Network { domains: g }, Ofun::Network { domains: r }) => {
            r.iter().all(|d| g.contains(d))
        }
        (Ofun::Environment { keys: g }, Ofun::Environment { keys: r }) => {
            r.iter().all(|k| g.contains(k))
        }
        (Ofun::Execute { programs: g }, Ofun::Execute { programs: r }) => {
            r.iter().all(|p| g.contains(p))
        }
        (Ofun::Time, Ofun::Time) => true,
        (Ofun::Random, Ofun::Random) => true,
        (Ofun::Stdio, Ofun::Stdio) => true,
        (Ofun::Crypto, Ofun::Crypto) => true,
        (Ofun::Bridge { language: g }, Ofun::Bridge { language: r }) => g == r || g == "*",
        _ => false,
    }
}

/// A set of granted capabilities
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    capabilities: Vec<Ofun>,
    violations: Vec<CapabilityViolation>,
    sacrificed: Vec<Ofun>,
}

#[derive(Debug, Clone)]
pub struct CapabilityViolation {
    pub capability: Ofun,
    pub call_site: String,
    pub timestamp: String,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, cap: Ofun) {
        // Block the grant if any sacrificed capability covers the target capability
        if self.sacrificed.iter().any(|sac| covers(sac, &cap)) {
            return;
        }
        if !self.capabilities.contains(&cap) {
            self.capabilities.push(cap);
        }
    }

    /// Check if an operation is allowed
    pub fn check(&self, required: &Ofun) -> bool {
        // If the required capability is covered by any sacrificed capability, it is denied
        if self.sacrificed.iter().any(|sac| covers(sac, required)) {
            return false;
        }
        self.capabilities
            .iter()
            .any(|granted| covers(granted, required))
    }

    /// Get all granted capabilities
    pub fn all(&self) -> &[Ofun] {
        &self.capabilities
    }

    /// Get all sacrificed capabilities
    pub fn sacrificed(&self) -> &[Ofun] {
        &self.sacrificed
    }

    /// Get recorded violations (for audit/debugging)
    pub fn violations(&self) -> &[CapabilityViolation] {
        &self.violations
    }

    /// Record a capability violation
    pub fn record_violation(&mut self, cap: Ofun, call_site: &str) {
        self.violations.push(CapabilityViolation {
            capability: cap,
            call_site: call_site.to_string(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
        });
    }

    /// Revoke a previously granted capability (Sacrifice)
    pub fn revoke(&mut self, cap: &Ofun) {
        self.capabilities.retain(|c| c != cap);
        if !self.sacrificed.contains(cap) {
            self.sacrificed.push(cap.clone());
        }
    }

    /// Revoke capabilities matching a predicate (Sacrifice)
    pub fn remove_matching<F>(&mut self, mut f: F)
    where
        F: FnMut(&Ofun) -> bool,
    {
        let mut removed = Vec::new();
        self.capabilities.retain(|c| {
            if f(c) {
                removed.push(c.clone());
                false
            } else {
                true
            }
        });
        for c in removed {
            if !self.sacrificed.contains(&c) {
                self.sacrificed.push(c);
            }
        }
    }

    /// Inherit capabilities from a parent set
    pub fn inherit_from(&mut self, parent: &CapabilitySet) {
        // Inherit the sacrificed list first to prevent illegal grants
        for cap in &parent.sacrificed {
            if !self.sacrificed.contains(cap) {
                self.sacrificed.push(cap.clone());
            }
        }
        // Remove any existing capabilities that are now covered by newly inherited sacrifices
        self.capabilities
            .retain(|c| !self.sacrificed.iter().any(|sac| covers(sac, c)));

        for cap in &parent.capabilities {
            if !self.capabilities.contains(cap)
                && !self.sacrificed.iter().any(|sac| covers(sac, cap))
            {
                self.capabilities.push(cap.clone());
            }
        }
    }

    /// Check if a specific capability is granted (exact match)
    pub fn has(&self, cap: &Ofun) -> bool {
        self.capabilities.contains(cap) && !self.sacrificed.contains(cap)
    }
}

fn normalize_path(path: &std::path::Path) -> PathBuf {
    let mut stack = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if stack.len() > 1 {
                    stack.pop();
                }
            }
            std::path::Component::Normal(c) => {
                stack.push(c);
            }
            c => {
                stack.push(c.as_os_str());
            }
        }
    }
    stack.iter().collect()
}

pub fn canonicalize_safe(path: &std::path::Path) -> PathBuf {
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(path),
            Err(_) => path.to_path_buf(),
        }
    };

    let mut ancestor = abs_path.clone();
    let mut components = Vec::new();

    while ancestor.symlink_metadata().is_err() && ancestor.parent().is_some() {
        if let Some(file_name) = ancestor.file_name() {
            components.push(file_name.to_owned());
        }
        ancestor.pop();
    }

    let mut resolved = if ancestor.symlink_metadata().is_ok() {
        match ancestor.canonicalize() {
            Ok(canon) => canon,
            Err(_) => ancestor,
        }
    } else {
        ancestor
    };

    for comp in components.into_iter().rev() {
        resolved.push(comp);
    }

    normalize_path(&resolved)
}
