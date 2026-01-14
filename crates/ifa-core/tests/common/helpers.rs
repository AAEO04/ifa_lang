//! Test helper functions and utilities

use ifa_core::*;
use std::time::Instant;

/// Assert that two IfaValues are equal with a helpful error message
pub fn assert_ifa_value_eq(actual: &IfaValue, expected: &IfaValue) {
    assert_eq!(
        actual, expected,
        "Expected {:?} but got {:?}",
        expected, actual
    );
}

/// Assert that an IfaResult is an error with the expected type
pub fn assert_ifa_error<T>(result: IfaResult<T>, expected_error: &str) {
    match result {
        Ok(_) => panic!("Expected error containing '{}', but got Ok", expected_error),
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains(expected_error),
                "Expected error containing '{}', but got: '{}'",
                expected_error, error_msg
            );
        }
    }
}

/// Measure execution time of a function
pub fn measure_time<F, R>(f: F) -> (R, std::time::Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Assert that execution completes within time limit
pub fn assert_executes_within<F, R>(f: F, max_duration: std::time::Duration)
where
    F: FnOnce() -> R,
{
    let (_, duration) = measure_time(f);
    assert!(
        duration <= max_duration,
        "Execution took {:?}, which exceeds maximum allowed {:?}",
        duration, max_duration
    );
}

/// Create a temporary directory for tests
#[cfg(feature = "fs")]
pub fn create_temp_dir() -> std::path::PathBuf {
    use std::env;
    use std::fs;
    
    let temp_dir = env::temp_dir().join("ifa_test_").join(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    );
    
    fs::create_dir_all(&temp_dir).unwrap();
    temp_dir
}

/// Clean up temporary directory
#[cfg(feature = "fs")]
pub fn cleanup_temp_dir(path: &std::path::Path) {
    use std::fs;
    
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}

/// Helper to test that a function panics
pub fn assert_panic<F>(f: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(f).expect_err("Expected function to panic");
}

/// Helper to test that a function does NOT panic
pub fn assert_no_panic<F>(f: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(f).expect("Function should not panic");
}

/// Compare two floating point values with tolerance
pub fn assert_float_eq(a: f64, b: f64, tolerance: f64) {
    let diff = (a - b).abs();
    assert!(
        diff <= tolerance,
        "Float values {} and {} differ by {}, which exceeds tolerance {}",
        a, b, diff, tolerance
    );
}

/// Helper to create a test VM with a simple program loaded
pub fn create_vm_with_program(program: Program) -> IfaVM {
    let mut vm = create_test_vm();
    let bytecode = compile(program).unwrap();
    vm.execute(&bytecode).unwrap();
    vm
}

/// Execute a string of Ifá code and return the result
pub fn execute_code(code: &str) -> IfaResult<IfaValue> {
    let tokens = tokenize(code)?;
    let ast = parse(tokens)?;
    let bytecode = compile(ast)?;
    let mut vm = create_test_vm();
    vm.execute(&bytecode)
}

/// Execute code and assert expected result
pub fn assert_code_executes_to(code: &str, expected: IfaValue) {
    let result = execute_code(code).unwrap();
    assert_ifa_value_eq(&result, &expected);
}

/// Execute code and assert it errors
pub fn assert_code_errors(code: &str, expected_error: &str) {
    let result = execute_code(code);
    assert_ifa_error(result, expected_error);
}

/// Test helper for memory usage
#[cfg(target_os = "linux")]
pub fn get_memory_usage() -> usize {
    use std::fs;
    use std::process;
    
    let status = fs::read_to_string(format!("/proc/{}/status", process::id())).unwrap();
    
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return parts[1].parse::<usize>().unwrap() * 1024; // Convert KB to bytes
        }
    }
    
    0
}

/// Test helper for stack depth
pub fn get_stack_depth() -> usize {
    // This is a rough approximation
    let mut depth = 0;
    
    // Use recursion to measure stack depth
    fn recurse(count: usize) -> usize {
        if count > 10000 {
            return count;
        }
        
        let mut buffer = [0u8; 1024];
        // Use buffer to ensure stack usage
        let _ = buffer;
        
        recurse(count + 1)
    }
    
    // Catch stack overflow
    std::panic::catch_unwind(|| {
        depth = recurse(0);
    }).ok();
    
    depth
}

/// Macro for creating parameterized tests
#[macro_export]
macro_rules! parameterized_test {
    ($test_name:ident, $test_cases:expr, $test_body:block) => {
        #[test]
        fn $test_name() {
            for (i, case) in $test_cases.iter().enumerate() {
                let case = case.clone();
                let test_fn = || $test_body;
                
                std::panic::catch_unwind(|| {
                    println!("Running test case {}: {:?}", i, case);
                    test_fn();
                }).unwrap_or_else(|e| {
                    panic!("Test case {} failed: {:?}", i, e);
                });
            }
        }
    };
}

/// Macro for benchmarking tests
#[macro_export]
macro_rules! benchmark_test {
    ($test_name:ident, $iterations:expr, $code:block) => {
        #[test]
        fn $test_name() {
            use std::time::Instant;
            
            let start = Instant::now();
            
            for _ in 0..$iterations {
                $code
            }
            
            let duration = start.elapsed();
            let avg_time = duration / $iterations;
            
            println!("Benchmark {}: {} iterations in {:?} (avg: {:?})",
                     stringify!($test_name), $iterations, duration, avg_time);
            
            // Assert reasonable performance (adjust as needed)
            assert!(avg_time.as_millis() < 100, "Performance regression detected");
        }
    };
}
