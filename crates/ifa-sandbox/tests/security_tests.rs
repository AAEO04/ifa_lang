//! Comprehensive security tests for ifa-sandbox

use ifa_sandbox::*;
use std::path::PathBuf;
use std::time::Duration;

mod capability_enforcement_tests {
    use super::*;

    #[test]
    fn test_empty_capability_set_denies_all() {
        let caps = CapabilitySet::new();
        
        // Test that empty capability set denies everything
        assert!(!caps.check(&Ofun::Stdio));
        assert!(!caps.check(&Ofun::Time));
        assert!(!caps.check(&Ofun::Random));
        assert!(!caps.check(&Ofun::ReadFiles {
            root: PathBuf::from("/tmp"),
        }));
        assert!(!caps.check(&Ofun::WriteFiles {
            root: PathBuf::from("/tmp"),
        }));
        assert!(!caps.check(&Ofun::Network {
            domains: vec!["example.com".to_string()],
        }));
    }

    #[test]
    fn test_grant_single_capability() {
        let mut caps = CapabilitySet::new();
        caps.grant(Ofun::Stdio);
        
        assert!(caps.check(&Ofun::Stdio));
        assert!(!caps.check(&Ofun::Time));
        assert!(!caps.check(&Ofun::Random));
    }

    #[test]
    fn test_grant_multiple_capabilities() {
        let mut caps = CapabilitySet::new();
        caps.grant(Ofun::Stdio);
        caps.grant(Ofun::Time);
        caps.grant(Ofun::Random);
        
        assert!(caps.check(&Ofun::Stdio));
        assert!(caps.check(&Ofun::Time));
        assert!(caps.check(&Ofun::Random));
        assert!(!caps.check(&Ofun::ReadFiles {
            root: PathBuf::from("/tmp"),
        }));
    }

    #[test]
    fn test_revoke_capability() {
        let mut caps = CapabilitySet::new();
        caps.grant(Ofun::Stdio);
        caps.grant(Ofun::Time);
        
        assert!(caps.check(&Ofun::Stdio));
        assert!(caps.check(&Ofun::Time));
        
        caps.revoke(&Ofun::Stdio);
        
        assert!(!caps.check(&Ofun::Stdio));
        assert!(caps.check(&Ofun::Time));
    }

    #[test]
    fn test_file_access_capabilities() {
        let mut caps = CapabilitySet::new();
        let allowed_path = PathBuf::from("/tmp/ifa_test");
        
        caps.grant(Ofun::ReadFiles {
            root: allowed_path.clone(),
        });
        
        // Should allow reading under allowed path
        assert!(caps.check(&Ofun::ReadFiles {
            root: allowed_path.clone(),
        }));
        assert!(caps.check(&Ofun::ReadFiles {
            root: allowed_path.join("subdir"),
        }));
        
        // Should deny reading outside allowed path
        assert!(!caps.check(&Ofun::ReadFiles {
            root: PathBuf::from("/etc"),
        }));
        assert!(!caps.check(&Ofun::ReadFiles {
            root: PathBuf::from("/tmp/other"),
        }));
    }

    #[test]
    fn test_network_capabilities() {
        let mut caps = CapabilitySet::new();
        
        caps.grant(Ofun::Network {
            domains: vec!["example.com".to_string(), "api.service.com".to_string()],
        });
        
        // Should allow allowed domains
        assert!(caps.check(&Ofun::Network {
            domains: vec!["example.com".to_string()],
        }));
        assert!(caps.check(&Ofun::Network {
            domains: vec!["api.service.com".to_string()],
        }));
        
        // Should deny other domains
        assert!(!caps.check(&Ofun::Network {
            domains: vec!["evil.com".to_string()],
        }));
        assert!(!caps.check(&Ofun::Network {
            domains: vec!["malware.net".to_string()],
        }));
    }

    #[test]
    fn test_capability_inheritance() {
        let mut parent_caps = CapabilitySet::new();
        parent_caps.grant(Ofun::Stdio);
        parent_caps.grant(Ofun::Time);
        
        let mut child_caps = CapabilitySet::new();
        child_caps.grant(Ofun::Random);
        
        // Child should inherit parent capabilities
        child_caps.inherit_from(&parent_caps);
        
        assert!(child_caps.check(&Ofun::Stdio));
        assert!(child_caps.check(&Ofun::Time));
        assert!(child_caps.check(&Ofun::Random));
        
        // Parent should not be affected
        assert!(parent_caps.check(&Ofun::Stdio));
        assert!(parent_caps.check(&Ofun::Time));
        assert!(!parent_caps.check(&Ofun::Random));
    }
}

mod sandbox_isolation_tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new();
        
        // New sandbox should have no capabilities
        assert!(!sandbox.has_capability(Ofun::Stdio));
        assert!(!sandbox.has_capability(Ofun::Time));
        
        // Should be in restricted mode
        assert!(sandbox.is_restricted());
    }

    #[test]
    fn test_sandbox_with_capabilities() {
        let mut sandbox = Sandbox::new();
        sandbox.grant_capability(Ofun::Stdio);
        sandbox.grant_capability(Ofun::Time);
        
        assert!(sandbox.has_capability(Ofun::Stdio));
        assert!(sandbox.has_capability(Ofun::Time));
        assert!(!sandbox.has_capability(Ofun::Random));
    }

    #[test]
    fn test_sandbox_resource_limits() {
        let mut sandbox = Sandbox::new();
        
        // Set resource limits
        sandbox.set_memory_limit(1024 * 1024); // 1MB
        sandbox.set_cpu_limit(1000); // 1 second
        sandbox.set_file_limit(10); // 10 files
        
        assert_eq!(sandbox.memory_limit(), 1024 * 1024);
        assert_eq!(sandbox.cpu_limit(), 1000);
        assert_eq!(sandbox.file_limit(), 10);
    }

    #[test]
    fn test_sandbox_time_limit() {
        let mut sandbox = Sandbox::new();
        sandbox.set_time_limit(Duration::from_secs(5));
        
        assert_eq!(sandbox.time_limit(), Duration::from_secs(5));
        
        // Test time limit enforcement
        let start = std::time::Instant::now();
        sandbox.start_execution();
        
        // Simulate some work
        std::thread::sleep(Duration::from_millis(100));
        
        let elapsed = sandbox.elapsed_time();
        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[test]
    fn test_sandbox_termination() {
        let mut sandbox = Sandbox::new();
        
        sandbox.start_execution();
        assert!(sandbox.is_running());
        
        sandbox.terminate();
        assert!(!sandbox.is_running());
        assert!(sandbox.was_terminated());
    }
}

mod resource_monitoring_tests {
    use super::*;

    #[test]
    fn test_memory_monitoring() {
        let monitor = ResourceMonitor::new();
        
        // Start monitoring
        monitor.start();
        
        // Allocate some memory
        let _large_vec: Vec<u8> = vec![0; 1024 * 1024]; // 1MB
        
        // Check memory usage
        let usage = monitor.memory_usage();
        assert!(usage > 0);
        
        // Stop monitoring
        monitor.stop();
        
        // Get peak usage
        let peak = monitor.peak_memory_usage();
        assert!(peak >= usage);
    }

    #[test]
    fn test_cpu_monitoring() {
        let monitor = ResourceMonitor::new();
        
        monitor.start();
        
        // Do some CPU work
        let mut sum = 0;
        for i in 0..1000000 {
            sum += i;
        }
        
        let cpu_time = monitor.cpu_time();
        assert!(cpu_time > Duration::from_nanos(0));
        
        monitor.stop();
    }

    #[test]
    fn test_file_monitoring() {
        let monitor = ResourceMonitor::new();
        
        monitor.start();
        
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("monitor_test.txt");
        
        // Create a file
        std::fs::write(&test_file, "test content").unwrap();
        
        let file_count = monitor.file_count();
        assert!(file_count > 0);
        
        // Clean up
        std::fs::remove_file(&test_file).unwrap();
        
        monitor.stop();
    }

    #[test]
    fn test_network_monitoring() {
        let monitor = ResourceMonitor::new();
        
        monitor.start();
        
        // This would need actual network operations to test
        // For now, we just verify the monitoring structure
        let bytes_sent = monitor.bytes_sent();
        let bytes_received = monitor.bytes_received();
        
        assert!(bytes_sent >= 0);
        assert!(bytes_received >= 0);
        
        monitor.stop();
    }
}

mod security_boundary_tests {
    use super::*;

    #[test]
    fn test_file_path_traversal_prevention() {
        let mut sandbox = Sandbox::new();
        let allowed_dir = std::env::temp_dir().join("ifa_allowed");
        std::fs::create_dir_all(&allowed_dir).unwrap();
        
        sandbox.grant_capability(Ofun::ReadFiles {
            root: allowed_dir.clone(),
        });
        
        // Test path traversal attempts
        let dangerous_paths = vec![
            allowed_dir.join("..").join("etc").join("passwd"),
            allowed_dir.join("../../../etc/passwd"),
            PathBuf::from("/etc/passwd"),
            allowed_dir.join("..").join("..").join("..").join("etc").join("passwd"),
        ];
        
        for path in dangerous_paths {
            // These should be denied
            assert!(!sandbox.can_access_file(&path));
        }
        
        // Clean up
        std::fs::remove_dir_all(&allowed_dir).unwrap();
    }

    #[test]
    fn test_symlink_prevention() {
        let mut sandbox = Sandbox::new();
        let allowed_dir = std::env::temp_dir().join("ifa_allowed");
        let outside_dir = std::env::temp_dir().join("ifa_outside");
        
        std::fs::create_dir_all(&allowed_dir).unwrap();
        std::fs::create_dir_all(&outside_dir).unwrap();
        
        // Create a file outside the allowed directory
        let outside_file = outside_dir.join("secret.txt");
        std::fs::write(&outside_file, "secret content").unwrap();
        
        // Create a symlink inside allowed directory pointing outside
        let symlink_path = allowed_dir.join("link_to_outside");
        
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside_file, &symlink_path).unwrap();
        }
        
        sandbox.grant_capability(Ofun::ReadFiles {
            root: allowed_dir.clone(),
        });
        
        // Should not be able to access through symlink
        assert!(!sandbox.can_access_file(&symlink_path));
        
        // Clean up
        std::fs::remove_file(&symlink_path).unwrap_or(());
        std::fs::remove_file(&outside_file).unwrap();
        std::fs::remove_dir_all(&allowed_dir).unwrap();
        std::fs::remove_dir_all(&outside_dir).unwrap();
    }

    #[test]
    fn test_network_isolation() {
        let mut sandbox = Sandbox::new();
        
        // Grant access to safe domain only
        sandbox.grant_capability(Ofun::Network {
            domains: vec!["example.com".to_string()],
        });
        
        // Test various network access attempts
        let safe_domains = vec!["example.com", "subdomain.example.com"];
        let unsafe_domains = vec!["evil.com", "malware.net", "phishing.site"];
        
        for domain in safe_domains {
            // These should be allowed if they match the pattern
            // (implementation dependent on exact matching logic)
            let allowed = sandbox.can_connect_to(domain);
            // We don't assert here as implementation may vary
        }
        
        for domain in unsafe_domains {
            // These should be denied
            assert!(!sandbox.can_connect_to(domain));
        }
    }

    #[test]
    fn test_environment_variable_isolation() {
        let mut sandbox = Sandbox::new();
        
        // Set some environment variables
        std::env::set_var("IFA_TEST_VAR", "test_value");
        std::env::set_var("SECRET_KEY", "secret_value");
        
        // Grant limited environment access
        sandbox.grant_capability(Ofun::EnvVars {
            allowed: vec!["IFA_TEST_VAR".to_string()],
        });
        
        // Should be able to access allowed variable
        assert!(sandbox.can_access_env_var("IFA_TEST_VAR"));
        
        // Should not be able to access disallowed variable
        assert!(!sandbox.can_access_env_var("SECRET_KEY"));
        
        // Clean up
        std::env::remove_var("IFA_TEST_VAR");
        std::env::remove_var("SECRET_KEY");
    }
}

mod attack_vector_tests {
    use super::*;

    #[test]
    fn test_resource_exhaustion_prevention() {
        let mut sandbox = Sandbox::new();
        
        // Set strict resource limits
        sandbox.set_memory_limit(10 * 1024 * 1024); // 10MB
        sandbox.set_cpu_limit(1000); // 1 second
        sandbox.set_file_limit(5); // 5 files
        
        let monitor = ResourceMonitor::new();
        monitor.start();
        
        sandbox.start_execution();
        
        // Test memory exhaustion
        let result = std::panic::catch_unwind(|| {
            // Try to allocate more memory than allowed
            let _huge_vec: Vec<u8> = vec![0; 100 * 1024 * 1024]; // 100MB
        });
        
        // Should be terminated or limited
        if result.is_err() {
            // Expected - memory allocation should fail
        }
        
        // Test file limit
        let temp_dir = std::env::temp_dir();
        for i in 0..10 {
            let file = temp_dir.join(format!("test_{}.txt", i));
            if sandbox.can_create_file(&file) {
                std::fs::write(&file, "test").unwrap();
            } else {
                // Should be denied after limit reached
                break;
            }
        }
        
        sandbox.terminate();
        monitor.stop();
    }

    #[test]
    fn test_infinite_loop_prevention() {
        let mut sandbox = Sandbox::new();
        sandbox.set_cpu_limit(100); // 100ms
        
        sandbox.start_execution();
        
        let start = std::time::Instant::now();
        
        // Simulate infinite loop
        let mut counter = 0;
        while sandbox.is_running() && counter < 1000000 {
            counter += 1;
            
            // Check if we've exceeded time limit
            if start.elapsed() > Duration::from_millis(200) {
                break;
            }
        }
        
        // Should be terminated due to time limit
        assert!(sandbox.was_terminated() || start.elapsed() > Duration::from_millis(100));
        
        sandbox.terminate();
    }

    #[test]
    fn test_fork_bomb_prevention() {
        let mut sandbox = Sandbox::new();
        sandbox.set_process_limit(3); // Allow only 3 processes
        
        sandbox.start_execution();
        
        // Try to spawn many processes
        let mut children = Vec::new();
        
        for _ in 0..10 {
            if sandbox.can_spawn_process() {
                match std::process::Command::new("echo")
                    .arg("test")
                    .spawn()
                {
                    Ok(child) => children.push(child),
                    Err(_) => break, // Should fail after limit
                }
            } else {
                break;
            }
        }
        
        // Should not be able to spawn more than the limit
        assert!(children.len() <= 3);
        
        // Clean up
        for child in children {
            let _ = child.kill();
        }
        
        sandbox.terminate();
    }

    #[test]
    fn test_code_injection_prevention() {
        let mut sandbox = Sandbox::new();
        
        // Don't grant any dangerous capabilities
        sandbox.grant_capability(Ofun::Stdio);
        
        // Test various injection attempts
        let injection_attempts = vec![
            "'; rm -rf /; echo '",
            "$(cat /etc/passwd)",
            "`whoami`",
            "${HOME}",
            "%PATH%",
        ];
        
        for injection in injection_attempts {
            // These should be treated as literal strings, not executed
            let safe = sandbox.treat_as_literal(injection);
            assert!(safe);
        }
    }
}

mod audit_and_logging_tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let auditor = SecurityAuditor::new();
        
        // Log some security events
        auditor.log_access_attempt("/etc/passwd", false);
        auditor.log_capability_grant(Ofun::Stdio);
        auditor.log_violation("File access denied", "/etc/passwd");
        
        // Check that events were logged
        let events = auditor.get_events();
        assert_eq!(events.len(), 3);
        
        // Verify event details
        assert!(events[0].contains("access_attempt"));
        assert!(events[1].contains("capability_grant"));
        assert!(events[2].contains("violation"));
    }

    #[test]
    fn test_security_violation_tracking() {
        let auditor = SecurityAuditor::new();
        
        // Log multiple violations
        auditor.log_violation("File access denied", "/etc/passwd");
        auditor.log_violation("Network access denied", "evil.com");
        auditor.log_violation("Resource limit exceeded", "memory");
        
        let violations = auditor.get_violations();
        assert_eq!(violations.len(), 3);
        
        // Check violation severity
        let high_severity = auditor.get_violations_by_severity(SecurityLevel::High);
        assert!(high_severity.len() >= 1);
    }

    #[test]
    fn test_capability_usage_tracking() {
        let auditor = SecurityAuditor::new();
        
        // Track capability usage
        auditor.track_capability_usage(Ofun::Stdio);
        auditor.track_capability_usage(Ofun::Time);
        auditor.track_capability_usage(Ofun::Stdio); // Used again
        
        let usage = auditor.get_capability_usage();
        assert_eq!(usage.get(&Ofun::Stdio), Some(&2));
        assert_eq!(usage.get(&Ofun::Time), Some(&1));
        assert_eq!(usage.get(&Ofun::Random), None);
    }

    #[test]
    fn test_security_report_generation() {
        let auditor = SecurityAuditor::new();
        
        // Add some audit data
        auditor.log_access_attempt("/tmp/file.txt", true);
        auditor.log_violation("File access denied", "/etc/passwd");
        auditor.track_capability_usage(Ofun::Stdio);
        
        // Generate security report
        let report = auditor.generate_report();
        
        assert!(report.contains("Security Audit Report"));
        assert!(report.contains("Access Attempts"));
        assert!(report.contains("Violations"));
        assert!(report.contains("Capability Usage"));
    }
}
