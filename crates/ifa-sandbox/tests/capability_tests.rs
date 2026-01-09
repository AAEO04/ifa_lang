//! Integration tests for Capability enforcement
//!
//! Tests that CapabilitySet correctly grants and denies permissions.

use ifa_sandbox::{CapabilitySet, Ofun};
use std::path::PathBuf;

#[test]
fn test_empty_capability_set_denies_all() {
    let caps = CapabilitySet::new();

    assert!(!caps.check(&Ofun::Stdio));
    assert!(!caps.check(&Ofun::Time));
    assert!(!caps.check(&Ofun::Random));
    assert!(!caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/")
    }));
}

#[test]
fn test_grant_stdio() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::Stdio);

    assert!(caps.check(&Ofun::Stdio));
    assert!(!caps.check(&Ofun::Time));
}

#[test]
fn test_grant_time_and_random() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::Time);
    caps.grant(Ofun::Random);

    assert!(caps.check(&Ofun::Time));
    assert!(caps.check(&Ofun::Random));
    assert!(!caps.check(&Ofun::Stdio));
}

#[test]
fn test_read_files_with_path_prefix() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::ReadFiles {
        root: PathBuf::from("/home/user"),
    });

    // Should allow reading under granted path
    assert!(caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/home/user")
    }));
    assert!(caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/home/user/docs")
    }));
    assert!(caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/home/user/docs/file.txt")
    }));

    // Should deny reading outside granted path
    assert!(!caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/")
    }));
    assert!(!caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/etc")
    }));
    assert!(!caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/home/other")
    }));
}

#[test]
fn test_write_files_with_path_prefix() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::WriteFiles {
        root: PathBuf::from("/tmp"),
    });

    assert!(caps.check(&Ofun::WriteFiles {
        root: PathBuf::from("/tmp")
    }));
    assert!(caps.check(&Ofun::WriteFiles {
        root: PathBuf::from("/tmp/myapp")
    }));
    assert!(!caps.check(&Ofun::WriteFiles {
        root: PathBuf::from("/home")
    }));
}

#[test]
fn test_network_domains() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::Network {
        domains: vec!["example.com".to_string(), "api.github.com".to_string()],
    });

    assert!(caps.check(&Ofun::Network {
        domains: vec!["example.com".to_string()]
    }));
    assert!(caps.check(&Ofun::Network {
        domains: vec!["api.github.com".to_string()]
    }));

    // Requesting domain not in allowed list
    assert!(!caps.check(&Ofun::Network {
        domains: vec!["malicious.com".to_string()]
    }));
}

#[test]
fn test_environment_keys() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::Environment {
        keys: vec!["HOME".to_string(), "PATH".to_string()],
    });

    assert!(caps.check(&Ofun::Environment {
        keys: vec!["HOME".to_string()]
    }));
    assert!(caps.check(&Ofun::Environment {
        keys: vec!["PATH".to_string()]
    }));

    // Requesting key not in allowed list
    assert!(!caps.check(&Ofun::Environment {
        keys: vec!["SECRET_KEY".to_string()]
    }));
}

#[test]
fn test_execute_programs() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::Execute {
        programs: vec!["/usr/bin/ls".to_string()],
    });

    assert!(caps.check(&Ofun::Execute {
        programs: vec!["/usr/bin/ls".to_string()]
    }));
    assert!(!caps.check(&Ofun::Execute {
        programs: vec!["/usr/bin/rm".to_string()]
    }));
}

#[test]
fn test_all_method() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::Stdio);
    caps.grant(Ofun::Time);
    caps.grant(Ofun::Random);

    let all = caps.all();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_multiple_read_paths() {
    let mut caps = CapabilitySet::new();
    caps.grant(Ofun::ReadFiles {
        root: PathBuf::from("/home/user"),
    });
    caps.grant(Ofun::ReadFiles {
        root: PathBuf::from("/tmp"),
    });

    assert!(caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/home/user/file.txt")
    }));
    assert!(caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/tmp/cache")
    }));
    assert!(!caps.check(&Ofun::ReadFiles {
        root: PathBuf::from("/etc/passwd")
    }));
}
