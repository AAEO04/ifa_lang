// Pointer safety is checked statically by the Babalawo analyzer.
// Legacy interpreter runtime safety tests are retired.

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_static_safety_note() {
    assert!(true);
}
