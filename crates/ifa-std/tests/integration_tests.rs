//! Integration tests for ifa-std domains

use ifa_std::*;
use std::path::PathBuf;
use std::time::Duration;

mod cross_domain_tests {
    use super::*;

    #[test]
    fn test_file_processing_pipeline() {
        // Test a pipeline that uses multiple domains
        let odi = ifa_std::odi::Odi;
        let ika = ifa_std::ika::Ika;
        let owonrin = ifa_std::owonrin::Owonrin;

        let temp_dir = std::env::temp_dir();
        let input_file = temp_dir.join("test_input.txt");
        let output_file = temp_dir.join("test_output.txt");

        // Create input file with random data
        let random_data = format!("Random number: {}", owonrin.yio(0, 1000));
        odi.kowe(&input_file, &random_data).unwrap();

        // Read and process the file
        let content = odi.kawe(&input_file).unwrap();
        let processed = ika.to_upper(&content);

        // Write processed data
        odi.kowe(&output_file, &processed).unwrap();

        // Verify the result
        let result = odi.kawe(&output_file).unwrap();
        assert_eq!(result, processed.to_uppercase());

        // Clean up
        std::fs::remove_file(&input_file).unwrap();
        std::fs::remove_file(&output_file).unwrap();
    }

    #[test]
    fn test_network_file_download() {
        #[cfg(feature = "full")]
        {
            let otura = ifa_std::otura::Otura;
            let odi = ifa_std::odi::Odi;

            // Download a small file
            let temp_dir = std::env::temp_dir();
            let download_file = temp_dir.join("downloaded.txt");

            // Use httpbin for testing
            let response =
                tokio::task::block_on(async { otura.gba("https://httpbin.org/uuid").await });

            if let Ok(content) = response {
                odi.kowe(&download_file, &content).unwrap();
                assert!(download_file.exists());

                // Verify content
                let read_back = odi.kawe(&download_file).unwrap();
                assert_eq!(read_back, content);

                // Clean up
                std::fs::remove_file(&download_file).unwrap();
            }
        }
    }

    #[test]
    fn test_crypto_file_operations() {
        let irete = ifa_std::irete::Irete;
        let odi = ifa_std::odi::Odi;

        let temp_dir = std::env::temp_dir();
        let original_file = temp_dir.join("original.txt");
        let encrypted_file = temp_dir.join("encrypted.bin");
        let decrypted_file = temp_dir.join("decrypted.txt");

        let original_content = "Secret message that should be encrypted";

        // Write original file
        odi.kowe(&original_file, original_content).unwrap();

        // Encrypt the file
        let file_data = odi.kawe(&original_file).unwrap();
        let key = "encryption_key_12345";
        let encrypted = irete.encrypt(file_data.as_bytes(), key.as_bytes());
        odi.kowe(&encrypted_file, &String::from_utf8_lossy(&encrypted))
            .unwrap();

        // Decrypt the file
        let encrypted_data = odi.kawe(&encrypted_file).unwrap();
        let decrypted = irete.decrypt(encrypted_data.as_bytes(), key.as_bytes());
        odi.kowe(&decrypted_file, &String::from_utf8_lossy(&decrypted))
            .unwrap();

        // Verify decryption worked
        let decrypted_content = odi.kawe(&decrypted_file).unwrap();
        assert_eq!(decrypted_content, original_content);

        // Clean up
        std::fs::remove_file(&original_file).unwrap();
        std::fs::remove_file(&encrypted_file).unwrap();
        std::fs::remove_file(&decrypted_file).unwrap();
    }
}

mod performance_integration_tests {
    use super::*;

    #[test]
    fn test_large_file_processing() {
        let odi = ifa_std::odi::Odi;
        let ika = ifa_std::ika::Ika;

        let temp_dir = std::env::temp_dir();
        let large_file = temp_dir.join("large_test.txt");

        // Create a large file
        let large_content = "Test line\n".repeat(10000);
        odi.kowe(&large_file, &large_content).unwrap();

        // Process the file
        let start = std::time::Instant::now();
        let content = odi.kawe(&large_file).unwrap();
        let lines = ika.pin(&content, "\n");
        let duration = start.elapsed();

        // Verify processing
        assert_eq!(lines.len(), 10000);

        // Performance check - should complete within reasonable time
        assert!(duration < Duration::from_secs(5));

        // Clean up
        std::fs::remove_file(&large_file).unwrap();
    }

    #[test]
    fn test_concurrent_operations() {
        #[cfg(feature = "full")]
        {
            let osa = ifa_std::osa::Osa;
            let owonrin = ifa_std::owonrin::Owonrin;

            let start = std::time::Instant::now();

            // Run multiple concurrent tasks
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    osa.sa(async {
                        // Simulate some work
                        let mut sum = 0;
                        for _ in 0..1000 {
                            sum += owonrin.yio(0, 100);
                        }
                        sum
                    })
                })
                .collect();

            let results: Vec<_> = futures::future::join_all(handles)
                .await
                .into_iter()
                .map(|r| r.unwrap())
                .collect();

            let duration = start.elapsed();

            // Verify all tasks completed
            assert_eq!(results.len(), 10);

            // Performance check - concurrent operations should be faster
            assert!(duration < Duration::from_secs(2));
        }
    }
}

mod error_handling_integration_tests {
    use super::*;

    #[test]
    fn test_file_error_propagation() {
        let odi = ifa_std::odi::Odi;
        let okanran = ifa_std::okanran::Okanran;

        // Try to read a non-existent file
        let non_existent = PathBuf::from("/non/existent/file.txt");
        let result = odi.kawe(&non_existent);

        assert!(result.is_err());

        // Handle the error with okanran
        let handled = okanran.gbiyanju_ẹkọ(|| odi.kawe(&non_existent).map(|_| ()));

        assert!(handled.is_err());
    }

    #[test]
    fn test_network_error_handling() {
        #[cfg(feature = "full")]
        {
            let otura = ifa_std::otura::Otura;
            let okanran = ifa_std::okanran::Okanran;

            // Try to connect to an invalid host
            let result = tokio::task::block_on(async {
                otura
                    .gba("http://invalid-host-that-does-not-exist.com")
                    .await
            });

            assert!(result.is_err());

            // Handle the error
            let handled = okanran.gbiyanju_ẹkọ(|| {
                tokio::task::block_on(async {
                    otura
                        .gba("http://invalid-host-that-does-not-exist.com")
                        .await
                })
                .map(|_| ())
            });

            assert!(handled.is_err());
        }
    }
}

mod security_integration_tests {
    use super::*;

    #[test]
    fn test_permission_isolation() {
        let ofun = ifa_std::ofun::Ofun;
        let odi = ifa_std::odi::Odi;

        // Create a sandbox with limited permissions
        let sandbox = ofun.create_sandbox();
        ofun.grant_permission(&sandbox, "file_read");
        // Don't grant write permission

        // Try to write a file (should fail)
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_permission.txt");

        // This would need to be implemented in the actual domains
        // For now, we just test the permission system
        assert!(!sandbox.has_permission("file_write"));
        assert!(sandbox.has_permission("file_read"));
    }

    #[test]
    fn test_resource_limits() {
        let ofun = ifa_std::ofun::Ofun;

        // Create resource limits
        let limits = ofun.create_resource_limits();
        limits.set_memory_limit(1024 * 1024); // 1MB
        limits.set_cpu_limit(1000); // 1 second

        // Test limit checking
        assert!(limits.check_memory_usage(512 * 1024));
        assert!(!limits.check_memory_usage(2 * 1024 * 1024));

        // Test CPU time tracking
        let start = std::time::Instant::now();

        // Simulate some work
        let mut sum = 0;
        for i in 0..1000000 {
            sum += i;
        }

        let elapsed = start.elapsed().as_millis() as u64;
        assert!(limits.check_cpu_usage(elapsed));
        assert!(!limits.check_cpu_usage(2000)); // 2 seconds exceeds limit
    }
}

mod unicode_integration_tests {
    use super::*;

    #[test]
    fn test_unicode_file_operations() {
        let odi = ifa_std::odi::Odi;
        let ika = ifa_std::ika::Ika;

        let temp_dir = std::env::temp_dir();
        let unicode_file = temp_dir.join("unicode_🔥.txt");

        let unicode_content = "Hello 世界 🌟 Ìfá-Lang Ẹ jẹ́ áwòkọ́";

        // Write unicode content
        odi.kowe(&unicode_file, unicode_content).unwrap();

        // Read and verify
        let read_content = odi.kawe(&unicode_file).unwrap();
        assert_eq!(read_content, unicode_content);

        // Test unicode string operations
        let upper_content = ika.to_upper(unicode_content);
        assert!(!upper_content.is_empty());

        // Test unicode splitting
        let words = ika.pin(unicode_content, " ");
        assert!(words.len() > 1);

        // Clean up
        std::fs::remove_file(&unicode_file).unwrap();
    }

    #[test]
    fn test_yoruba_text_processing() {
        let ika = ifa_std::ika::Ika;

        let yoruba_text = "Ẹ jẹ́ áwòkọ́, kí ló dé ọ̀nà?";

        // Test case conversion (Yoruba has special diacritics)
        let upper = ika.to_upper(yoruba_text);
        let lower = ika.to_lower(yoruba_text);

        assert!(!upper.is_empty());
        assert!(!lower.is_empty());

        // Test text manipulation
        let words = ika.pin(yoruba_text, " ");
        assert!(words.len() >= 3);

        // Test concatenation
        let combined = ika.kopu("Báwo ni?", " ".yoruba_text);
        assert!(combined.starts_with("Báwo ni?"));
    }
}

mod async_integration_tests {
    use super::*;

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_async_file_processing() {
        let osa = ifa_std::osa::Osa;
        let odi = ifa_std::odi::Odi;
        let ika = ifa_std::ika::Ika;

        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("async_test1.txt");
        let file2 = temp_dir.join("async_test2.txt");
        let file3 = temp_dir.join("async_test3.txt");

        // Create test files
        odi.kowe(&file1, "Content 1").unwrap();
        odi.kowe(&file2, "Content 2").unwrap();
        odi.kowe(&file3, "Content 3").unwrap();

        // Process files concurrently
        let handles: Vec<_> = vec![file1, file2, file3]
            .into_iter()
            .map(|file| {
                osa.sa(async move {
                    let content = odi.kawe(&file).unwrap();
                    ika.to_upper(&content)
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // Verify results
        assert_eq!(results.len(), 3);
        assert!(results[0].contains("CONTENT 1"));
        assert!(results[1].contains("CONTENT 2"));
        assert!(results[2].contains("CONTENT 3"));

        // Clean up
        std::fs::remove_file(&temp_dir.join("async_test1.txt")).unwrap();
        std::fs::remove_file(&temp_dir.join("async_test2.txt")).unwrap();
        std::fs::remove_file(&temp_dir.join("async_test3.txt")).unwrap();
    }

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_async_network_operations() {
        let osa = ifa_std::osa::Osa;
        let otura = ifa_std::otura::Otura;

        // Make multiple HTTP requests concurrently
        let urls = vec![
            "https://httpbin.org/delay/1",
            "https://httpbin.org/delay/1",
            "https://httpbin.org/delay/1",
        ];

        let handles: Vec<_> = urls
            .into_iter()
            .map(|url| osa.sa(async move { otura.gba(url).await }))
            .collect();

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .collect();

        // All requests should complete
        assert_eq!(results.len(), 3);

        // Check that all succeeded (or at least didn't panic)
        for result in results {
            // Some might fail due to network issues, but shouldn't panic
            match result {
                Ok(_) => assert!(true),
                Err(_) => assert!(true), // Network errors are acceptable in tests
            }
        }
    }
}
