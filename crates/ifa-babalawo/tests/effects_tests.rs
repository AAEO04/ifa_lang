use ifa_babalawo::{Diagnostic, Severity, check_program};
use ifa_parser::parse;

fn errors_only(diags: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diags
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect()
}

fn run_check(code: &str) -> Vec<Diagnostic> {
    let program = parse(code).unwrap();
    check_program(&program, "test.ifa").diagnostics
}

#[test]
fn test_pure_function_cannot_call_async() {
    // A function with no effects annotation cannot call Osa.ran (Async domain)
    let code = r#"
        ayanmo actor = Osa.ise(ese(msg) { pada msg; });
        ayanmo payload = [1, 2, 3];
        ese pure_fn() {
            Osa.ran(actor, yanda payload);
        }
    "#;
    let errors = errors_only(run_check(code));
    assert!(
        !errors.is_empty(),
        "Expected effect violation error for pure function calling async Osa.ran"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.error.message.contains("EFFECT_VIOLATION")
                || e.error.code == "EFFECT_VIOLATION"
                || e.error.message.contains("missing effect")),
        "Expected EFFECT_VIOLATION, got: {:?}",
        errors
    );
}

#[test]
fn test_async_function_can_call_async() {
    // A function declaring effects(Async) can call Osa.ran
    let code = r#"
        ayanmo actor = Osa.ise(ese(msg) { pada msg; });
        ese async_fn(payload) -> effects(Async) {
            Osa.ran(actor, yanda payload);
        }
    "#;
    let errors = errors_only(run_check(code));
    assert!(
        errors.is_empty(),
        "Expected no errors for async function calling async Osa.ran, got: {:?}",
        errors
    );
}

#[test]
fn test_network_function_can_call_network() {
    // A function declaring effects(Network) can call Otura.get
    let code = r#"
        ese fetch_data(url) -> effects(Network) {
            pada Otura.get(url);
        }
    "#;
    let errors = errors_only(run_check(code));
    assert!(
        errors.is_empty(),
        "Expected no errors for network function calling Otura.get, got: {:?}",
        errors
    );
}

#[test]
fn test_pure_function_cannot_call_network() {
    // A function with no effects annotation cannot call Otura.get (Network domain)
    let code = r#"
        ese fetch_data(url) {
            pada Otura.get(url);
        }
    "#;
    let errors = errors_only(run_check(code));
    assert!(
        !errors.is_empty(),
        "Expected effect violation error for pure function calling network Otura.get"
    );
    assert!(
        errors.iter().any(|e| e.error.code == "EFFECT_VIOLATION"),
        "Expected EFFECT_VIOLATION, got: {:?}",
        errors
    );
}
