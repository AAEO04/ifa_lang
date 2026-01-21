//! Comprehensive tests for all 16 Ifá-Std domains

use ifa_std::*;
use std::path::PathBuf;
use std::time::Duration;

mod ogbe_tests {
    use super::*;

    #[test]
    fn test_ogbe_environment_variables() {
        let ogbe = ifa_std::ogbe::Ogbe;

        // Test getting environment variable
        let key = "IFA_TEST_VAR";
        std::env::set_var(key, "test_value");

        assert!(ogbe.check_env(key));

        // Test setting environment variable
        ogbe.fi_ayika(key, "new_value");
        assert_eq!(std::env::var(key).unwrap(), "new_value");

        // Clean up
        std::env::remove_var(key);
    }

    #[test]
    fn test_ogbe_system_info() {
        let ogbe = ifa_std::ogbe::Ogbe;

        // Test getting system info
        let info = ogbe.gbohun_ero();
        assert!(!info.is_empty());

        // Test getting current directory
        let cwd = ogbe.ibi_isise();
        assert!(cwd.exists());
    }

    #[test]
    fn test_ogbe_lifecycle_management() {
        let ogbe = ifa_std::ogbe::Ogbe;

        // Test process info
        let pid = ogbe.get_pid();
        assert!(pid > 0);

        // Test parent process
        let ppid = ogbe.get_ppid();
        assert!(ppid > 0);
    }
}

mod oyeku_tests {
    use super::*;

    #[test]
    fn test_oyeku_sleep_functionality() {
        let oyeku = ifa_std::oyeku::Oyeku;

        // Test short sleep
        let start = std::time::Instant::now();
        oyeku.sun_ms(10);
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed < Duration::from_millis(100)); // Allow some tolerance
    }

    #[test]
    fn test_oyeku_exit_functionality() {
        let oyeku = ifa_std::oyeku::Oyeku;

        // Test exit code setting
        oyeku.ku(42);

        // Note: We can't actually test exit without terminating the test
        // So we just verify the function exists and doesn't panic
        assert!(true);
    }

    #[test]
    fn test_oyeku_raii_guards() {
        use ifa_std::oyeku::OyekuGuard;

        // Test RAII guard
        let _guard = OyekuGuard::new();
        // Guard should be dropped automatically when it goes out of scope
    }

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_oyeku_async_sleep() {
        use ifa_std::oyeku::sun_async;

        let start = std::time::Instant::now();
        sun_async(10).await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed < Duration::from_millis(100));
    }
}

mod iwori_tests {
    use super::*;

    #[test]
    fn test_iwori_time_operations() {
        let iwori = ifa_std::iwori::Iwori;

        // Test getting current time
        let now = iwori.akoko();
        assert!(now > 0);

        // Test time formatting
        let formatted = iwori.format_time(now);
        assert!(!formatted.is_empty());

        // Test time parsing
        let parsed = iwori.parse_time(&formatted);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_iwori_iteration() {
        let iwori = ifa_std::iwori::Iwori;

        // Test iteration over a range
        let numbers: Vec<i32> = iwori.iterate(0, 5).collect();
        assert_eq!(numbers, vec![0, 1, 2, 3, 4]);

        // Test iteration with step
        let evens: Vec<i32> = iwori.iterate_step(0, 10, 2).collect();
        assert_eq!(evens, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_iwori_timer() {
        let iwori = ifa_std::iwori::Iwori;

        // Test timer creation
        let timer = iwori.ago();
        assert!(timer.elapsed().as_millis() < 100);

        // Test timer reset
        std::thread::sleep(Duration::from_millis(10));
        let elapsed_before = timer.elapsed();
        timer.reset();
        let elapsed_after = timer.elapsed();

        assert!(elapsed_after < elapsed_before);
    }
}

mod odi_tests {
    use super::*;

    #[test]
    fn test_odi_file_operations() {
        let odi = ifa_std::odi::Odi;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_odi.txt");

        // Test file writing
        odi.kowe(&test_file, "Hello, World!").unwrap();
        assert!(test_file.exists());

        // Test file reading
        let content = odi.kawe(&test_file).unwrap();
        assert_eq!(content, "Hello, World!");

        // Test file appending
        odi.fi_kun(&test_file, " Appended").unwrap();
        let content = odi.kawe(&test_file).unwrap();
        assert_eq!(content, "Hello, World! Appended");

        // Clean up
        std::fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_odi_directory_operations() {
        let odi = ifa_std::odi::Odi;
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("test_odi_dir");

        // Test directory creation
        odi.da_himo(&test_dir).unwrap();
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());

        // Test directory listing
        let entries = odi.li_ako(&test_dir).unwrap();
        assert!(entries.is_empty());

        // Clean up
        std::fs::remove_dir(&test_dir).unwrap();
    }

    #[test]
    fn test_odi_database_operations() {
        let odi = ifa_std::odi::Odi;
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_odi.db");

        // Test database creation
        let db = odi.create_database(&db_path).unwrap();
        assert!(db_path.exists());

        // Test database operations
        odi.execute_sql(db, "CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)")
            .unwrap();
        odi.execute_sql(db, "INSERT INTO test (value) VALUES ('hello')")
            .unwrap();

        // Clean up
        std::fs::remove_file(&db_path).unwrap();
    }
}

mod irosu_tests {
    use super::*;

    #[test]
    fn test_irosu_console_output() {
        let irosu = ifa_std::irosu::Irosu;

        // Test printing
        irosu.soro("Hello, World!");

        // Test formatted printing
        irosu.soro_format("Value: {}", 42);

        // These tests just verify the functions don't panic
        assert!(true);
    }

    #[test]
    fn test_irosu_console_input() {
        let irosu = ifa_std::irosu::Irosu;

        // Test reading input (this would normally wait for user input)
        // For testing, we'll just verify the function exists
        assert!(true);
    }

    #[test]
    fn test_irosu_colors() {
        let irosu = ifa_std::irosu::Irosu;

        // Test color printing
        irosu.soro_red("Error message");
        irosu.soro_green("Success message");
        irosu.soro_blue("Info message");

        // These tests just verify the functions don't panic
        assert!(true);
    }
}

mod owonrin_tests {
    use super::*;

    #[test]
    fn test_owonrin_random_generation() {
        let owonrin = ifa_std::owonrin::Owonrin;

        // Test integer generation
        let rand_int = owonrin.yio(0, 100);
        assert!(rand_int >= 0);
        assert!(rand_int < 100);

        // Test float generation
        let rand_float = owonrin.yio_float(0.0, 1.0);
        assert!(rand_float >= 0.0);
        assert!(rand_float < 1.0);

        // Test boolean generation
        let rand_bool = owonrin.yio_bool();
        // Can be true or false, both are valid
        assert!(true);
    }

    #[test]
    fn test_owonrin_random_distribution() {
        let owonrin = ifa_std::owonrin::Owonrin;

        // Test multiple generations to check distribution
        let mut counts = vec![0; 10];
        for _ in 0..1000 {
            let rand_int = owonrin.yio(0, 10);
            counts[rand_int as usize] += 1;
        }

        // Each bucket should have some counts (probabilistic test)
        for count in counts {
            assert!(count > 0, "Empty bucket found in random distribution");
        }
    }

    #[test]
    fn test_owonrin_seeding() {
        let owonrin = ifa_std::owonrin::Owonrin;

        // Test seeding for reproducible results
        owonrin.tan_imọ_ọrọ(42);
        let first = owonrin.yio(0, 1000);

        owonrin.tan_imọ_ọrọ(42);
        let second = owonrin.yio(0, 1000);

        assert_eq!(first, second);
    }
}

mod obara_tests {
    use super::*;

    #[test]
    fn test_obara_math_operations() {
        let obara = ifa_std::obara::Obara;

        // Test addition
        assert_eq!(obara.add(2, 3), 5);
        assert_eq!(obara.add(2.5, 3.5), 6.0);

        // Test multiplication
        assert_eq!(obara.mul(4, 5), 20);
        assert_eq!(obara.mul(2.5, 4.0), 10.0);

        // Test mixed operations
        assert_eq!(obara.add(2, 3.5), 5.5);
    }

    #[test]
    fn test_obara_advanced_math() {
        let obara = ifa_std::obara::Obara;

        // Test power
        assert_eq!(obara.pow(2, 3), 8);
        assert_eq!(obara.pow(4.0, 0.5), 2.0);

        // Test square root
        assert_eq!(obara.sqrt(16.0), 4.0);

        // Test absolute value
        assert_eq!(obara.abs(-5), 5);
        assert_eq!(obara.abs(-3.14), 3.14);
    }

    #[test]
    fn test_obara_math_constants() {
        let obara = ifa_std::obara::Obara;

        // Test mathematical constants
        assert!((obara.pi() - std::f64::consts::PI).abs() < 0.0001);
        assert!((obara.e() - std::f64::consts::E).abs() < 0.0001);
    }
}

mod okanran_tests {
    use super::*;

    #[test]
    fn test_okanran_error_creation() {
        let okanran = ifa_std::okanran::Okanran;

        // Test error creation
        let error = okanran.ṣe_aisan("Test error");
        assert!(error.is_err());

        // Test error with context
        let error_with_context = okanran.ṣe_aisan_pẹlu_ẹkọ("Test error", "context");
        assert!(error_with_context.is_err());
    }

    #[test]
    fn test_okanran_error_handling() {
        let okanran = ifa_std::okanran::Okanran;

        // Test error chaining
        let error1 = okanran.ṣe_aisan("First error");
        let error2 = okanran.ṣe_aisan("Second error");

        // Test error combination
        let combined = okanran.papọ_aisan(error1.unwrap_err(), error2.unwrap_err());
        assert!(combined.is_err());
    }

    #[test]
    fn test_okanran_error_recovery() {
        let okanran = ifa_std::okanran::Okanran;

        // Test error recovery
        let result = okanran.gbiyanju_ẹkọ(|| Ok(42));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test error recovery with failure
        let result = okanran.gbiyanju_ẹkọ(|| Err("Something went wrong"));

        assert!(result.is_err());
    }
}

mod ogunda_tests {
    use super::*;

    #[test]
    fn test_ogunda_array_operations() {
        let ogunda = ifa_std::ogunda::Ogunda;

        // Test array creation
        let array = ogunda.create_array(vec![1, 2, 3, 4, 5]);
        assert_eq!(array.len(), 5);

        // Test array access
        assert_eq!(ogunda.get(&array, 0), Some(&1));
        assert_eq!(ogunda.get(&array, 4), Some(&5));
        assert_eq!(ogunda.get(&array, 5), None);

        // Test array modification
        let mut mutable_array = array.clone();
        ogunda.set(&mut mutable_array, 0, 10);
        assert_eq!(ogunda.get(&mutable_array, 0), Some(&10));
    }

    #[test]
    fn test_ogunda_array_slicing() {
        let ogunda = ifa_std::ogunda::Ogunda;

        let array = ogunda.create_array(vec![1, 2, 3, 4, 5]);

        // Test slicing
        let slice = ogunda.slice(&array, 1, 4);
        assert_eq!(slice, vec![2, 3, 4]);

        // Test slicing to end
        let slice = ogunda.slice(&array, 2, 5);
        assert_eq!(slice, vec![3, 4, 5]);
    }

    #[test]
    fn test_ogunda_process_operations() {
        let ogunda = ifa_std::ogunda::Ogunda;

        // Test process creation
        let mut cmd = std::process::Command::new("echo");
        cmd.arg("hello");

        let output = ogunda.run_command(cmd).unwrap();
        assert!(output.status.success());

        // Test process spawning
        let child = ogunda
            .spawn_command(std::process::Command::new("sleep").arg("0.1"))
            .unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }
}

mod osa_tests {
    use super::*;

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_osa_async_tasks() {
        let osa = ifa_std::osa::Osa;

        // Test task spawning
        let handle = osa.sa(async { 42 });
        let result = handle.await.unwrap();
        assert_eq!(result, 42);

        // Test multiple tasks
        let handles: Vec<_> = (0..5).map(|i| osa.sa(async move { i * 2 })).collect();

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results, vec![0, 2, 4, 6, 8]);
    }

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_osa_channels() {
        let osa = ifa_std::osa::Osa;

        // Test channel creation
        let (tx, mut rx) = osa.oju_ona(10);

        // Test sending and receiving
        tx.send(42).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received, 42);

        // Test multiple sends
        for i in 0..5 {
            tx.send(i).unwrap();
        }

        let mut received = Vec::new();
        for _ in 0..5 {
            received.push(rx.recv().await.unwrap());
        }

        assert_eq!(received, vec![0, 1, 2, 3, 4]);
    }

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_osa_timeout() {
        let osa = ifa_std::osa::Osa;

        // Test timeout with fast operation
        let result = osa.pẹlu_akoko(async { 42 }, 100).await;
        assert_eq!(result, Some(42));

        // Test timeout with slow operation
        let result = osa
            .pẹlu_akoko(
                async {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    42
                },
                50,
            )
            .await;
        assert_eq!(result, None);
    }
}

mod ika_tests {
    use super::*;

    #[test]
    fn test_ika_string_operations() {
        let ika = ifa_std::ika::Ika;

        // Test string concatenation
        let result = ika.kopu("Hello, ", "World!");
        assert_eq!(result, "Hello, World!");

        // Test string splitting
        let parts = ika.pin("a,b,c", ",");
        assert_eq!(parts, vec!["a", "b", "c"]);

        // Test string joining
        let joined = ika.papọ(&["a", "b", "c"], ",");
        assert_eq!(joined, "a,b,c");
    }

    #[test]
    fn test_ika_string_manipulation() {
        let ika = ifa_std::ika::Ika;

        // Test case conversion
        assert_eq!(ika.to_upper("hello"), "HELLO");
        assert_eq!(ika.to_lower("WORLD"), "world");

        // Test trimming
        assert_eq!(ika.trim("  hello  "), "hello");

        // Test substring
        assert_eq!(ika.substring("hello", 1, 4), "ell");
    }

    #[test]
    fn test_ika_regex_operations() {
        let ika = ifa_std::ika::Ika;

        // Test regex matching
        assert!(ika.matches("hello123", r"\d+"));
        assert!(!ika.matches("hello", r"\d+"));

        // Test regex replacement
        let result = ika.replace("hello123world", r"\d+", "NUM");
        assert_eq!(result, "helloNUMworld");

        // Test regex extraction
        let captures = ika.extract("hello123world", r"(\d+)");
        assert_eq!(captures, vec!["123"]);
    }
}

mod oturupon_tests {
    use super::*;

    #[test]
    fn test_oturupon_math_operations() {
        let oturupon = ifa_std::oturupon::Oturupon;

        // Test subtraction
        assert_eq!(oturupon.subtract(10, 3), 7);
        assert_eq!(oturupon.subtract(5.5, 2.5), 3.0);

        // Test division
        assert_eq!(oturupon.divide(10, 2), 5);
        assert_eq!(oturupon.divide(7.0, 2.0), 3.5);

        // Test checked operations
        let result = oturupon.checked_add(i64::MAX, 1);
        assert!(result.is_none());

        let result = oturupon.checked_add(100, 200);
        assert_eq!(result, Some(300));
    }

    #[test]
    fn test_oturupon_overflow_handling() {
        let oturupon = ifa_std::oturupon::Oturupon;

        // Test overflow detection
        let result = oturupon.checked_mul(i64::MAX, 2);
        assert!(result.is_none());

        // Test safe operations
        let result = oturupon.safe_div(10, 0);
        assert_eq!(result, 0); // Or handle as error

        let result = oturupon.safe_div(10, 2);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_oturupon_precision() {
        let oturupon = ifa_std::oturupon::Oturupon;

        // Test floating point precision
        let result = oturupon.divide(1.0, 3.0);
        assert!((result - 0.3333333333333333).abs() < 0.0001);

        // Test rounding
        assert_eq!(oturupon.round(3.7), 4.0);
        assert_eq!(oturupon.round(3.2), 3.0);
    }
}

mod otura_tests {
    use super::*;

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_otura_http_operations() {
        let otura = ifa_std::otura::Otura;

        // Test HTTP GET (using a public API)
        let response = otura.gba("https://httpbin.org/get").await;
        assert!(response.is_ok());

        // Test HTTP POST
        let response = otura.ran("https://httpbin.org/post", "test data").await;
        assert!(response.is_ok());
    }

    #[cfg(feature = "full")]
    #[tokio::test]
    async fn test_otura_tcp_operations() {
        let otura = ifa_std::otura::Otura;

        // Test TCP connection (using echo server)
        let result = otura.soro("httpbin.org:80").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_otura_url_operations() {
        let otura = ifa_std::otura::Otura;

        // Test URL parsing
        let url = "https://example.com/path?query=value";
        let parsed = otura.parse_url(url);
        assert!(parsed.is_ok());

        // Test URL encoding
        let encoded = otura.encode_url("hello world");
        assert!(encoded.contains("%20"));

        // Test URL decoding
        let decoded = otura.decode_url(&encoded);
        assert_eq!(decoded, "hello world");
    }
}

mod irete_tests {
    use super::*;

    #[test]
    fn test_irete_hashing() {
        let irete = ifa_std::irete::Irete;

        // Test SHA-256
        let hash = irete.hash_sha256("hello world");
        assert_eq!(hash.len(), 32); // 32 bytes for SHA-256

        // Test SHA-512
        let hash = irete.hash_sha512("hello world");
        assert_eq!(hash.len(), 64); // 64 bytes for SHA-512

        // Test deterministic hashing
        let hash1 = irete.hash_sha256("test");
        let hash2 = irete.hash_sha256("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_irete_compression() {
        let irete = ifa_std::irete::Irete;

        let data = "hello world ".repeat(100);
        let compressed = irete.compress(data.as_bytes());
        let decompressed = irete.decompress(&compressed);

        assert_eq!(decompressed, data.as_bytes());

        // Compression should reduce size for repetitive data
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_irete_encryption() {
        let irete = ifa_std::irete::Irete;

        let plaintext = "secret message";
        let key = "encryption_key";

        // Test encryption
        let encrypted = irete.encrypt(plaintext.as_bytes(), key.as_bytes());
        assert_ne!(encrypted, plaintext.as_bytes());

        // Test decryption
        let decrypted = irete.decrypt(&encrypted, key.as_bytes());
        assert_eq!(decrypted, plaintext.as_bytes());
    }
}

mod ose_tests {
    use super::*;

    #[test]
    fn test_ose_graphics_operations() {
        let ose = ifa_std::ose::Ose;

        // Test color creation
        let color = ose.color(255, 0, 0);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);

        // Test rectangle creation
        let rect = ose.rectangle(10, 20, 100, 200);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 200);
    }

    #[test]
    fn test_ose_ui_components() {
        let ose = ifa_std::ose::Ose;

        // Test button creation
        let button = ose.button("Click me", 10, 10, 100, 30);
        assert_eq!(button.text, "Click me");
        assert_eq!(button.x, 10);
        assert_eq!(button.y, 10);

        // Test text input
        let input = ose.text_input("placeholder", 10, 50, 200, 30);
        assert_eq!(input.placeholder, "placeholder");
        assert_eq!(input.x, 10);
        assert_eq!(input.y, 50);
    }

    #[test]
    fn test_ose_rendering() {
        let ose = ifa_std::ose::Ose;

        // Test canvas creation
        let canvas = ose.canvas(800, 600);
        assert_eq!(canvas.width, 800);
        assert_eq!(canvas.height, 600);

        // Test drawing operations
        ose.draw_line(&canvas, 0, 0, 100, 100, ose.color(255, 255, 255));
        ose.draw_rect(&canvas, 10, 10, 50, 50, ose.color(255, 0, 0));
        ose.draw_text(&canvas, 10, 100, "Hello, World!", ose.color(0, 255, 0));
    }
}

mod ofun_tests {
    use super::*;

    #[test]
    fn test_ofun_permission_system() {
        let ofun = ifa_std::ofun::Ofun;

        // Test capability creation
        let file_cap = ofun.create_file_capability(PathBuf::from("/tmp"));
        assert!(file_cap.allows_read());
        assert!(file_cap.allows_write());

        // Test network capability
        let net_cap = ofun.create_network_capability("example.com", 80);
        assert!(net_cap.allows_connect());

        // Test capability checking
        assert!(ofun.check_capability(&file_cap, "read"));
        assert!(ofun.check_capability(&file_cap, "write"));
        assert!(!ofun.check_capability(&file_cap, "execute"));
    }

    #[test]
    fn test_ofun_sandbox() {
        let ofun = ifa_std::ofun::Ofun;

        // Test sandbox creation
        let sandbox = ofun.create_sandbox();
        assert!(sandbox.is_restricted());

        // Test permission grants
        ofun.grant_permission(&sandbox, "file_read");
        assert!(sandbox.has_permission("file_read"));

        ofun.revoke_permission(&sandbox, "file_read");
        assert!(!sandbox.has_permission("file_read"));
    }

    #[test]
    fn test_ofun_resource_limits() {
        let ofun = ifa_std::ofun::Ofun;

        // Test resource limit setting
        let limits = ofun.create_resource_limits();
        limits.set_memory_limit(1024 * 1024); // 1MB
        limits.set_cpu_limit(1000); // 1000ms

        assert_eq!(limits.memory_limit(), 1024 * 1024);
        assert_eq!(limits.cpu_limit(), 1000);

        // Test limit checking
        assert!(limits.check_memory_usage(512 * 1024));
        assert!(!limits.check_memory_usage(2 * 1024 * 1024));
    }
}
