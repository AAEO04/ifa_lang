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
    println!("TRACKING BALANCED DIAGNOSTICS: {:#?}", baba.diagnostics);
    assert!(!baba.has_errors(), "Should pass when balanced");
}

#[test]
fn test_reference_syntax_parsing() {
    let src = r#"
    ayanmo x: Int = 10;
    ayanmo ref_x: &Int = &x;
    ayanmo y: Int = 20;
    ayanmo mut_ref_y: &mut Int = &mut y;
    "#;
    let baba = check(src);
    assert!(
        !baba.has_errors(),
        "Reference syntax parsing and typing should pass"
    );
}

#[test]
fn test_borrow_checker_double_mutable() {
    let src = r#"
    ayanmo x = 42;
    ayanmo r1 = &mut x;
    ayanmo r2 = &mut x;
    "#;
    let baba = check(src);
    assert!(baba.has_errors(), "Should flag double mutable borrow");
    let errors = &baba.diagnostics;
    assert!(errors.iter().any(|d| d.error.code == "BORROW_ERROR"));

    let err_msg = errors
        .iter()
        .find(|d| d.error.code == "BORROW_ERROR")
        .unwrap()
        .error
        .message
        .clone();
    println!("ERR MSG IS: {}", err_msg);
    assert!(
        err_msg.contains("State transition history:"),
        "Should contain state transition history"
    );
    assert!(
        err_msg.contains("Borrowed (Mutable)"),
        "Should log mutable borrow event"
    );
}

#[test]
fn test_borrow_checker_read_while_mutably_borrowed() {
    let src = r#"
    ayanmo x = 42;
    ayanmo r = &mut x;
    ayanmo y = x;
    "#;
    let baba = check(src);
    assert!(baba.has_errors(), "Should flag read during mutable borrow");
    let errors = &baba.diagnostics;
    assert!(errors.iter().any(|d| d.error.code == "BORROW_ERROR"));
}

#[test]
fn test_borrow_checker_mutation_while_borrowed() {
    let src = r#"
    ayanmo x = 42;
    ayanmo r = &x;
    x = 100;
    "#;
    let baba = check(src);
    assert!(baba.has_errors(), "Should flag mutation during borrow");
    let errors = &baba.diagnostics;
    assert!(errors.iter().any(|d| d.error.code == "BORROW_ERROR"));
}

#[test]
fn test_borrow_checker_move_while_borrowed() {
    let src = r#"
    ayanmo x = 42;
    ayanmo r = &x;
    ayanmo y = yanda x;
    "#;
    let baba = check(src);
    assert!(baba.has_errors(), "Should flag move during borrow");
    let errors = &baba.diagnostics;
    assert!(errors.iter().any(|d| d.error.code == "MOVE_WHILE_BORROWED"));
}

#[test]
fn test_iso_expr_parsing_and_checking() {
    let src = r#"
    ayanmo x = iso 42;
    "#;
    let baba = check(src);
    assert!(
        !baba.has_errors(),
        "Isolated graph parsing should pass checks"
    );
}
