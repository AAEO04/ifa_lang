use ifa_babalawo::check_program;
use ifa_parser::parse;

fn check(src: &str) -> ifa_babalawo::Babalawo {
    let ast = parse(src).expect("Failed to parse");
    check_program(&ast, "test.ifa")
}

#[test]
fn test_iso_alias_hazard() {
    let src = r#"
        ese main() {
            ayanmo data = iso "secret";
            ayanmo ptr = &data;
        }
    "#;
    let baba = check(src);
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.code == "ISO_ALIAS_HAZARD"),
        "Expected ISO_ALIAS_HAZARD but got {:?}",
        baba.diagnostics
    );
}

#[test]
fn test_iso_alias_hazard_mut() {
    let src = r#"
        ese main() {
            ayanmo data = iso "secret";
            ayanmo ptr = &mut data;
        }
    "#;
    let baba = check(src);
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.code == "ISO_ALIAS_HAZARD"),
        "Expected ISO_ALIAS_HAZARD but got {:?}",
        baba.diagnostics
    );
}

#[test]
fn test_iso_explicit_move_success() {
    let src = r#"
        ese main() {
            ayanmo data = iso "secret";
            Osa.ise(yanda data);
        }
    "#;
    let baba = check(src);
    assert!(
        !baba
            .diagnostics
            .iter()
            .any(|d| d.error.code == "ISO_ALIAS_HAZARD"),
        "Unexpected ISO_ALIAS_HAZARD"
    );
    assert!(
        !baba
            .diagnostics
            .iter()
            .any(|d| d.error.code == "USE_AFTER_MOVE"),
        "Unexpected USE_AFTER_MOVE"
    );
}

#[test]
fn test_iso_use_after_move() {
    let src = r#"
        ese main() {
            ayanmo data = iso "secret";
            ayanmo transferred = yanda data;
            data;
        }
    "#;
    let baba = check(src);
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.code == "USE_AFTER_MOVE"),
        "Expected USE_AFTER_MOVE but got {:?}",
        baba.diagnostics
    );
}
