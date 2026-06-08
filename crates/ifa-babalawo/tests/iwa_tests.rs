use ifa_babalawo::*;
use ifa_parser::parse;

fn check(src: &str) -> Babalawo {
    let program = parse(src).expect("Failed to parse source");
    check_program(&program, "test.ifa")
}

#[test]
fn test_iwa_shape_mismatch() {
    let src = r#"
    iwa Logger {
        open();
    }
    
    ayanmo log: Iwa<Logger> = {
        "wrong": ese() {}
    };
    "#;
    let baba = check(src);
    assert!(baba.has_errors(), "Should catch missing method");
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.code == "IWA_SHAPE_MISMATCH")
    );
}

#[test]
fn test_iwa_pele_lifecycle_tracking() {
    let src = r#"
    iwa Logger {
        #[iwa_pele_pair(open, close)]
        open();
        close();
    }
    
    ayanmo log: Iwa<Logger> = {
        "open": ese() { Odi.ko("Opened"); },
        "close": ese() { Odi.ko("Closed"); }
    };
    
    log.open();
    "#;
    let baba = check(src);
    println!("{:#?}", baba.diagnostics);
    assert!(baba.has_errors(), "Should catch unbalanced iwa_pele_pair");
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.message.contains("was never closed"))
    );
}

#[test]
fn test_iwa_pele_lifecycle_tracking_balanced() {
    let src = r#"
    iwa Logger {
        #[iwa_pele_pair(open, close)]
        open();
        close();
    }
    
    ayanmo log: Iwa<Logger> = {
        "open": ese() { Odi.ko("Opened"); },
        "close": ese() { Odi.ko("Closed"); }
    };
    
    log.open();
    log.close();
    "#;
    let baba = check(src);
    assert!(!baba.has_errors(), "Should pass when balanced");
}
