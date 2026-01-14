use ifa_core::oracle::oracle::verify_equivalence;

#[test]
fn test_irosu_io_equivalence() {
    // Note: We avoid 'ka' (input) as it blocks the oracle
    let source = r#"
    Irosu.fo("Hello, Ifá!");
    Irosu.fo(10, 20);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_obara_math_equivalence() {
    let source = r#"
    ayanmo a = Obara.add(10, 20);
    ayanmo b = Obara.mul(a, 2);
    Irosu.fo(a, b);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_oturupon_math_equivalence() {
    let source = r#"
    ayanmo a = Oturupon.sub(100, 40);
    ayanmo b = Oturupon.div(a, 2);
    Irosu.fo(a, b);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_ika_string_equivalence() {
    let source = r#"
    ayanmo s = Ika.concat("Ifa", "-", "Lang");
    ayanmo len = Ika.len(s);
    Irosu.fo(s, len);
    "#;
    verify_equivalence(source);
}

#[test]
fn test_ogunda_list_equivalence() {
    let source = r#"
    ayanmo list = Ogunda.create(1, 2, 3);
    Irosu.fo(Ogunda.len(list));
    "#;
    verify_equivalence(source);
}

#[test]
fn test_case_insensitivity_equivalence() {
    // Verifying that domain/method casing doesn't break the bridge
    let source = r#"
    Irosu.FO("Uppercase");
    OBARA.ADD(1, 1);
    "#;
    verify_equivalence(source);
}
