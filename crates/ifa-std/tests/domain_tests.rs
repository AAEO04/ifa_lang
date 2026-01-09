//! Integration tests for ifa-std Odù domains
//!
//! Tests each of the 16 Odù domains.
//!
//! NOTE: Many tests are temporarily disabled pending API updates for CapabilitySet

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use ifa_core::IfaValue;
use ifa_std::*;

// =============================================================================
// ÌROSÙ (1100) - Console I/O
// =============================================================================

mod irosu_tests {
    use super::*;
    use ifa_std::irosu::Irosu;
    use ifa_std::OduDomain;

    #[test]
    fn test_domain_info() {
        let irosu = Irosu;
        assert_eq!(irosu.name(), "Ìrosù");
        assert_eq!(irosu.binary(), "1100");
        assert!(!irosu.help().is_empty());
    }

    // Note: Console I/O tests are limited to non-interactive functions
}

// =============================================================================
// Ọ̀BÀRÀ (1000) - Math Add/Mul
// =============================================================================

mod obara_tests {
    use super::*;
    use ifa_std::obara::Obara;

    #[test]
    fn test_basic_arithmetic() {
        let obara = Obara;

        assert_eq!(obara.fikun(5.0, 3.0), 8.0);
        assert_eq!(obara.isodipupo(4.0, 3.0), 12.0);
        assert_eq!(obara.agbara(2.0, 3.0), 8.0);
        assert_eq!(obara.gbongbo(16.0), 4.0);
    }

    #[test]
    fn test_rounding() {
        let obara = Obara;

        assert_eq!(obara.ile(3.7), 3.0);
        assert_eq!(obara.orule(3.2), 4.0);
        assert_eq!(obara.yika(3.14159, 2), 3.14);
    }

    #[test]
    fn test_trigonometry() {
        let obara = Obara;

        let pi = obara.pi();
        assert!((obara.sin(pi / 2.0) - 1.0).abs() < 0.0001);
        assert!((obara.cos(0.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_statistics() {
        let obara = Obara;
        let items = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(obara.apapo(&items), 15.0);
        assert_eq!(obara.aropin(&items), 3.0);
        assert_eq!(obara.nla_julo(&items), 5.0);
        assert_eq!(obara.kere_julo(&items), 1.0);
    }

    #[test]
    fn test_constants() {
        let obara = Obara;

        assert!((obara.pi() - std::f64::consts::PI).abs() < 0.0001);
        assert!((obara.e() - std::f64::consts::E).abs() < 0.0001);
    }
}

// =============================================================================
// ÒTÚÚRÚPỌ̀N (0010) - Math Sub/Div
// =============================================================================

mod oturupon_tests {
    use super::*;
    use ifa_std::oturupon::{Oturupon, RoundingMode};

    #[test]
    fn test_checked_subtraction() {
        let oturupon = Oturupon;

        assert_eq!(oturupon.din(10, 3).unwrap(), 7);
        assert_eq!(oturupon.din(5, 10).unwrap(), -5);
    }

    #[test]
    fn test_checked_division() {
        let oturupon = Oturupon;

        let result = oturupon.pin(10, 4).unwrap();
        assert_eq!(result, 2.5);
    }

    #[test]
    fn test_division_by_zero() {
        let oturupon = Oturupon;

        assert!(oturupon.pin(10, 0).is_err());
        assert!(oturupon.pin_odidi(10, 0).is_err());
        assert!(oturupon.ku(10, 0).is_err());
    }

    #[test]
    fn test_modulo() {
        let oturupon = Oturupon;

        assert_eq!(oturupon.ku(17, 5).unwrap(), 2);
        assert_eq!(oturupon.ku(-7, 5).unwrap(), -2);
        assert_eq!(oturupon.ku_euclidean(-7, 5).unwrap(), 3); // Always positive
    }

    #[test]
    fn test_rounding_modes() {
        let oturupon = Oturupon;

        assert_eq!(oturupon.pin_f(7.0, 2.0, RoundingMode::Floor).unwrap(), 3.0);
        assert_eq!(
            oturupon.pin_f(7.0, 2.0, RoundingMode::Ceiling).unwrap(),
            4.0
        );
        assert_eq!(
            oturupon.pin_f(7.0, 2.0, RoundingMode::Truncate).unwrap(),
            3.0
        );
    }
}

// =============================================================================
// ÌKÁ (0100) - Strings
// =============================================================================

mod ika_tests {
    use super::*;
    use ifa_std::ika::Ika;

    #[test]
    fn test_string_operations() {
        let ika = Ika;

        assert_eq!(ika.so(&["Hello", " ", "World"]), "Hello World");
        assert_eq!(ika.gigun("Ifá"), 3);
        assert_eq!(ika.nla("hello"), "HELLO");
        assert_eq!(ika.kekere("HELLO"), "hello");
    }

    #[test]
    fn test_find_and_contains() {
        let ika = Ika;

        assert_eq!(ika.wa("Hello World", "World"), Some(6));
        assert!(ika.ni("Hello World", "World"));
        assert!(!ika.ni("Hello World", "Mars"));
    }

    #[test]
    fn test_split_join() {
        let ika = Ika;

        let parts = ika.pin("a,b,c", ",");
        assert_eq!(parts, vec!["a", "b", "c"]);

        let joined = ika.dapo(&["a", "b", "c"], "-");
        assert_eq!(joined, "a-b-c");
    }

    #[test]
    fn test_replace() {
        let ika = Ika;

        assert_eq!(ika.yi_pada("hello world", "world", "Ifá"), "hello Ifá");
    }

    #[test]
    fn test_trim_reverse() {
        let ika = Ika;

        assert_eq!(ika.ge("  hello  "), "hello");
        assert_eq!(ika.pada("abc"), "cba");
    }

    #[test]
    fn test_regex() {
        let ika = Ika;

        assert!(ika.ba_mu(r"\d+", "abc123").unwrap());
        assert!(!ika.ba_mu(r"\d+", "abc").unwrap());

        let matched = ika.wa_akoko(r"\d+", "abc123xyz").unwrap();
        assert_eq!(matched, Some("123".to_string()));

        let all = ika.wa_gbogbo(r"\d+", "1a2b3").unwrap();
        assert_eq!(all, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_regex_replace() {
        let ika = Ika;

        let result = ika.ropo(r"\d+", "a1b2c3", "X").unwrap();
        assert_eq!(result, "aXbXcX");
    }
}

// =============================================================================
// Ọ̀WỌNRÍN (0011) - Random
// =============================================================================

mod owonrin_tests {
    use super::*;
    use ifa_std::owonrin::Owonrin;

    #[test]
    fn test_reproducible_seed() {
        let mut rng1 = Owonrin::from_seed(42);
        let mut rng2 = Owonrin::from_seed(42);

        assert_eq!(rng1.pese(0, 100), rng2.pese(0, 100));
        assert_eq!(rng1.pese_odidi(), rng2.pese_odidi());
    }

    #[test]
    fn test_range() {
        let mut owonrin = Owonrin::new();

        for _ in 0..100 {
            let n = owonrin.pese(10, 20);
            assert!(n >= 10 && n <= 20);
        }
    }

    #[test]
    fn test_probability() {
        let mut owonrin = Owonrin::from_seed(123);

        // With probability 1.0, should always return true
        for _ in 0..10 {
            assert!(owonrin.boya(1.0));
        }

        // With probability 0.0, should always return false
        for _ in 0..10 {
            assert!(!owonrin.boya(0.0));
        }
    }

    #[test]
    fn test_shuffle() {
        let mut owonrin = Owonrin::from_seed(42);
        let mut items = vec![1, 2, 3, 4, 5];
        let original = items.clone();

        owonrin.dapo(&mut items);

        // Should still contain same elements
        items.sort();
        assert_eq!(items, original);
    }

    #[test]
    fn test_uuid_format() {
        let mut owonrin = Owonrin::new();
        let uuid = owonrin.uuid();

        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn test_hex() {
        let mut owonrin = Owonrin::new();
        let hex = owonrin.hex(16);

        assert_eq!(hex.len(), 32); // 16 bytes * 2 hex chars
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

// =============================================================================
// Ọ̀YẸ̀KÚ (0000) - Exit/Sleep
// =============================================================================

mod oyeku_tests {
    use super::*;
    use ifa_std::oyeku::{Ebo, Oyeku};
    use std::cell::Cell;
    use std::time::Instant;

    #[test]
    fn test_sleep() {
        let oyeku = Oyeku;
        let start = Instant::now();
        oyeku.sun_ms(50);
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() >= 45);
    }

    #[test]
    fn test_ebo_cleanup() {
        let cleaned = Cell::new(false);

        {
            let _guard = Ebo::new(|| cleaned.set(true));
        }

        assert!(cleaned.get());
    }

    #[test]
    fn test_ebo_dismiss() {
        let cleaned = Cell::new(false);

        {
            let guard = Ebo::new(|| cleaned.set(true));
            guard.dismiss();
        }

        assert!(!cleaned.get());
    }
}

// =============================================================================
// ÌWÒRÌ (0110) - Time/Iteration
// =============================================================================

mod iwori_tests {
    use super::*;
    use ifa_std::iwori::{range, range_step, repeat, Iwori};

    #[test]
    fn test_current_time() {
        let iwori = Iwori;
        let now = iwori.akoko();

        assert!(now > 0);
        assert!(iwori.odun() >= 2024);
    }

    #[test]
    fn test_format() {
        let iwori = Iwori;
        let formatted = iwori.ojo("%Y");

        assert!(formatted.parse::<i32>().unwrap() >= 2024);
    }

    #[test]
    fn test_leap_year() {
        let iwori = Iwori;

        assert!(iwori.odun_abule(2024));
        assert!(!iwori.odun_abule(2023));
        assert!(iwori.odun_abule(2000));
        assert!(!iwori.odun_abule(1900));
    }

    #[test]
    fn test_range() {
        let items: Vec<i64> = range(1, 5).collect();
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_range_step() {
        let items: Vec<i64> = range_step(0, 10, 2).collect();
        assert_eq!(items, vec![0, 2, 4, 6, 8, 10]);

        // Negative step
        let items: Vec<i64> = range_step(10, 0, -2).collect();
        assert_eq!(items, vec![10, 8, 6, 4, 2, 0]);
    }

    #[test]
    fn test_repeat() {
        let items: Vec<i32> = repeat(42, 5).collect();
        assert_eq!(items, vec![42, 42, 42, 42, 42]);
    }
}

// =============================================================================
// Ọ̀GBÈ (1111) - System/Lifecycle
// =============================================================================

mod ogbe_tests {
    use super::*;
    use ifa_std::ogbe::Ogbe;

    #[test]
    fn test_env_vars() {
        let ogbe = Ogbe;

        // These env vars should exist on all platforms
        assert!(!ogbe.eto().is_empty()); // OS
        assert!(!ogbe.apẹrẹ().is_empty()); // Arch
    }

    #[test]
    fn test_args() {
        let ogbe = Ogbe;

        // Should have at least the program name
        assert!(ogbe.iye_ohun() >= 1);
    }

    #[test]
    fn test_env_with_default() {
        let ogbe = Ogbe;

        let value = ogbe.ayika_tabi("NONEXISTENT_VAR_12345", "default");
        assert_eq!(value, "default");
    }
}

// =============================================================================
// ÒDÍ (1001) - Files/Database
// =============================================================================

mod odi_tests {
    use super::*;
    use ifa_std::odi::Odi;
    use std::path::PathBuf;

    #[test]
    fn test_file_operations() {
        let odi = Odi::default();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("ifa_test_file.txt");
        let path_str = file_path.to_str().unwrap();

        // Write
        odi.ko(path_str, "Hello, Ifá!").unwrap();
        assert!(odi.wa(path_str));

        // Read
        let content = odi.ka(path_str).unwrap();
        assert_eq!(content, "Hello, Ifá!");

        // Append
        odi.fi(path_str, "\nMore text").unwrap();
        let content = odi.ka(path_str).unwrap();
        assert!(content.contains("More text"));

        // Size
        let size = odi.iwon(path_str).unwrap();
        assert!(size > 0);

        // Cleanup
        odi.pa_faili(path_str).unwrap();
        assert!(!odi.wa(path_str));
    }

    #[test]
    fn test_read_lines() {
        let odi = Odi::default();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("ifa_test_lines.txt");
        let path_str = file_path.to_str().unwrap();

        odi.ko(path_str, "line1\nline2\nline3").unwrap();

        let lines = odi.ka_ila(path_str).unwrap();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);

        odi.pa_faili(path_str).unwrap();
    }

    #[test]
    fn test_sandbox() {
        let temp_dir = std::env::temp_dir();
        let odi = Odi::sandboxed(temp_dir.clone());

        // Should succeed - inside sandbox
        let file_path = temp_dir.join("sandbox_test.txt");
        odi.ko(file_path.to_str().unwrap(), "test").unwrap();
        odi.pa_faili(file_path.to_str().unwrap()).unwrap();
    }

    #[test]
    fn test_sqlite() {
        let odi = Odi::default();

        let conn = odi.so_db_iranti().unwrap();

        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)", [])
            .unwrap();

        conn.execute("INSERT INTO test (name) VALUES (?)", ["Ifá"])
            .unwrap();

        let name: String = conn
            .query_row("SELECT name FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        assert_eq!(name, "Ifá");
    }
}

// =============================================================================
// ÌRẸTẸ̀ (1101) - Crypto/Compression
// =============================================================================

mod irete_tests {
    use super::*;
    use ifa_std::irete::Irete;

    #[test]
    fn test_sha256() {
        let irete = Irete;

        let hash = irete.sha256_hex(b"hello");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Known hash
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hmac() {
        let irete = Irete;
        let key = b"secret";
        let data = b"message";

        let sig = irete.hmac_sha256(key, data);

        assert!(irete.hmac_verify(key, data, &sig));
        assert!(!irete.hmac_verify(key, b"wrong", &sig));
    }

    #[test]
    fn test_base64() {
        let irete = Irete;
        let original = b"Hello, Ifa!";

        let encoded = irete.base64_encode(original);
        let decoded = irete.base64_decode(&encoded).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_hex() {
        let irete = Irete;
        let original = vec![0xDE, 0xAD, 0xBE, 0xEF];

        let hex = irete.hex_encode(&original);
        assert_eq!(hex, "deadbeef");

        let decoded = irete.hex_decode(&hex).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_compression() {
        let irete = Irete;
        let data = b"Hello Hello Hello Hello Hello".repeat(100);

        let compressed = irete.funpo(&data, 3).unwrap();
        let decompressed = irete.tu(&compressed).unwrap();

        assert!(compressed.len() < data.len());
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_random_bytes() {
        let irete = Irete;

        let bytes1 = irete.random_bytes(32).unwrap();
        let bytes2 = irete.random_bytes(32).unwrap();

        assert_eq!(bytes1.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }
}

// =============================================================================
// Ọ̀KÀNRÀN (0001) - Errors/Assertions
// =============================================================================

mod okanran_tests {
    use super::*;
    use ifa_core::error::IfaError;
    use ifa_std::okanran::Okanran;

    #[test]
    fn test_assertion_pass() {
        let okanran = Okanran;
        assert!(okanran.beeni(true, "should pass").is_ok());
    }

    #[test]
    fn test_assertion_fail() {
        let okanran = Okanran;
        assert!(okanran.beeni(false, "should fail").is_err());
    }

    #[test]
    fn test_gbiyanju_with_error() {
        let okanran = Okanran;

        let result = okanran.gbiyanju(|| Err(IfaError::Custom("test error".into())), 42);

        assert_eq!(result, 42);
    }

    #[test]
    fn test_gbiyanju_with_success() {
        let okanran = Okanran;

        let result = okanran.gbiyanju(|| Ok(100), 42);
        assert_eq!(result, 100);
    }
}

// =============================================================================
// ÒGÚNDÁ (1110) - Arrays/Processes
// =============================================================================

mod ogunda_tests {
    use super::*;
    use ifa_std::ogunda::Ogunda;

    #[test]
    fn test_list_operations() {
        let ogunda = Ogunda;
        let mut list = vec![1, 2, 3];

        ogunda.fi(&mut list, 4);
        assert_eq!(list, vec![1, 2, 3, 4]);

        assert_eq!(ogunda.mu(&mut list), Some(4));
        assert_eq!(ogunda.iwon(&list), 3);
        assert!(!ogunda.sofo(&list));
    }

    #[test]
    fn test_sort() {
        let ogunda = Ogunda;
        let list = vec![3, 1, 4, 1, 5, 9, 2, 6];

        let sorted = ogunda.to(&list);
        assert_eq!(sorted, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_reverse() {
        let ogunda = Ogunda;
        let list = vec![1, 2, 3];

        let reversed = ogunda.pada(&list);
        assert_eq!(reversed, vec![3, 2, 1]);
    }

    #[test]
    fn test_filter() {
        let ogunda = Ogunda;
        let list = vec![1, 2, 3, 4, 5, 6];

        let evens = ogunda.yan(&list, |x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4, 6]);
    }

    #[test]
    fn test_map() {
        let ogunda = Ogunda;
        let list = vec![1, 2, 3];

        let doubled = ogunda.yi_pada(list, |x| x * 2);
        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn test_concat() {
        let ogunda = Ogunda;
        let a = vec![1, 2];
        let b = vec![3, 4];

        let merged = ogunda.dapo(&a, &b);
        assert_eq!(merged, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_predicates() {
        let ogunda = Ogunda;
        let list = vec![2, 4, 6, 8];

        assert!(ogunda.gbogbo(&list, |x| x % 2 == 0)); // All even
        assert!(ogunda.eyikeyi(&list, |x| *x > 5)); // Any > 5
    }
}

// =============================================================================
// ÒFÚN (0101) - Permissions/Reflection
// =============================================================================

mod ofun_tests {
    use super::*;
    use ifa_std::ofun::{Capabilities, Ofun};

    #[test]
    fn test_capabilities() {
        let ofun = Ofun::default();

        assert!(ofun.le("read"));
        assert!(ofun.le("write"));
        assert!(ofun.le("network"));
    }

    #[test]
    fn test_drop_capability() {
        let mut ofun = Ofun::default();

        assert!(ofun.le("write"));
        ofun.ju("write");
        assert!(!ofun.le("write"));
    }

    #[test]
    fn test_sandboxed() {
        let ofun = Ofun::with_capabilities(Capabilities::none());

        assert!(!ofun.le("read"));
        assert!(!ofun.le("write"));
        assert!(!ofun.le("network"));
    }

    #[test]
    fn test_read_only() {
        let ofun = Ofun::with_capabilities(Capabilities::read_only());

        assert!(ofun.le("read"));
        assert!(!ofun.le("write"));
        assert!(!ofun.le("network"));
    }

    #[test]
    fn test_reflection() {
        let ofun = Ofun::default();

        assert_eq!(ofun.iru(&IfaValue::Int(42)), "Int");
        assert_eq!(ofun.iru(&IfaValue::Str("test".into())), "Str");

        assert!(ofun.je(&IfaValue::Int(42), "int"));
        assert!(ofun.je(&IfaValue::Int(42), "Int"));
        assert!(!ofun.je(&IfaValue::Int(42), "Str"));
    }
}
