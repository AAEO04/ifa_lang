use ifa_core::oracle::oracle::verify_equivalence;

#[test]
fn test_mixed_arithmetic_equivalence() {
    let source = r#"
    ayanmo int = 10;
    ayanmo float = 2.5;
    ayanmo result = int + float;
    Irosu.fo(result);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_string_repetition_equivalence() {
    let source = r#"
    ayanmo s = "Na";
    ayanmo n = 3;
    ayanmo result = s * n;
    Irosu.fo(result);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_list_concat_equivalence() {
    let source = r#"
    ayanmo a = [1, 2];
    ayanmo b = [3, 4];
    ayanmo result = a + b;
    Irosu.fo(result);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_division_by_zero_equivalence() {
    let source = r#"
    ayanmo a = 10;
    ayanmo b = 0;
    ayanmo result = a / b;
    Irosu.fo(result);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_truthiness_edge_cases_equivalence() {
    let source = r#"
    ayanmo empty_str = "";
    if empty_str {
        Irosu.fo("Wrong");
    } else {
        Irosu.fo("Empty string is false");
    }
    
    ayanmo empty_list = [];
    if empty_list {
        Irosu.fo("Wrong");
    } else {
        Irosu.fo("Empty list is false");
    }
    
    ayanmo null_val = àìsí;
    if !null_val {
        Irosu.fo("Null is false");
    }
    "#;
    verify_equivalence(source);
}

#[test]
fn test_overflow_promotion_equivalence() {
    // Note: We use a large literal to trigger potential overflow/promotion
    let source = r#"
    ayanmo max_int = 9223372036854775807;
    ayanmo result = max_int + 1;
    Irosu.fo(result);
    "#;
    verify_equivalence(source);
}
