//! # Domain Operation Traits
//!
//! Traits that define the interface for Odù domain operations.
//! These traits can be implemented by ifa-std and called from ifa-core.

use crate::error::IfaResult;
use crate::value::IfaValue;

// =============================================================================
// INFRASTRUCTURE TRAITS
// =============================================================================

/// CPU operations - parallel computing
pub trait CpuOps {
    /// Get number of available CPU threads
    fn num_threads() -> usize;

    /// Parallel sum of numeric values
    fn par_sum(data: &[IfaValue]) -> IfaResult<IfaValue>;

    /// Parallel map operation
    fn par_map<F>(data: &[IfaValue], f: F) -> IfaResult<Vec<IfaValue>>
    where
        F: Fn(&IfaValue) -> IfaValue + Sync + Send;

    /// Configure thread pool
    fn configure(threads: usize) -> IfaResult<()>;
}

/// GPU operations - compute shaders
pub trait GpuOps {
    /// Check if GPU is available
    fn available() -> bool;

    /// Get GPU info
    fn info() -> String;

    /// Matrix multiplication on GPU
    fn matmul(a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> IfaResult<Vec<f32>>;
}

/// Storage operations - key-value store
pub trait StorageOps {
    /// Get value by key
    fn get(key: &str) -> IfaResult<IfaValue>;

    /// Set key-value pair
    fn set(key: &str, value: IfaValue) -> IfaResult<()>;

    /// Delete key
    fn delete(key: &str) -> IfaResult<bool>;

    /// Check if key exists
    fn exists(key: &str) -> bool;
}

// =============================================================================
// APPLICATION STACK TRAITS
// =============================================================================

/// Backend operations - HTTP server, ORM
pub trait BackendOps {
    /// Start HTTP server
    fn serve(port: u16) -> IfaResult<()>;

    /// Add route
    fn route(method: &str, path: &str, handler: IfaValue) -> IfaResult<()>;

    /// Make HTTP request
    fn request(method: &str, url: &str, body: Option<&str>) -> IfaResult<IfaValue>;
}

/// Frontend operations - HTML/CSS generation
pub trait FrontendOps {
    /// Escape HTML content
    fn escape_html(content: &str) -> String;

    /// Create HTML element
    fn element(tag: &str, content: &str, attrs: Option<&[(String, String)]>) -> String;

    /// Generate CSS
    fn css(selector: &str, properties: &[(String, String)]) -> String;
}

/// Crypto operations - hashing, encryption
pub trait CryptoOps {
    /// SHA-256 hash
    fn sha256(input: &str) -> String;

    /// SHA-512 hash
    fn sha512(input: &str) -> String;

    /// Generate random bytes
    fn random_bytes(count: usize) -> Vec<u8>;

    /// Argon2 password hash
    fn argon2_hash(password: &str) -> IfaResult<String>;

    /// Verify Argon2 hash
    fn argon2_verify(password: &str, hash: &str) -> bool;
}

/// ML operations - machine learning
pub trait MlOps {
    /// Create tensor from data
    fn tensor(data: Vec<IfaValue>, shape: &[usize]) -> IfaResult<IfaValue>;

    /// Matrix multiplication
    fn matmul(a: &IfaValue, b: &IfaValue) -> IfaResult<IfaValue>;

    /// ReLU activation
    fn relu(tensor: &IfaValue) -> IfaResult<IfaValue>;

    /// Softmax
    fn softmax(tensor: &IfaValue) -> IfaResult<IfaValue>;

    /// Dot product
    fn dot(a: &IfaValue, b: &IfaValue) -> IfaResult<f64>;
}

/// GameDev operations - game engine
pub trait GameDevOps {
    /// Create 2D vector
    fn vec2(x: f64, y: f64) -> IfaValue;

    /// Create 3D vector
    fn vec3(x: f64, y: f64, z: f64) -> IfaValue;

    /// Calculate distance between points
    fn distance(a: &IfaValue, b: &IfaValue) -> IfaResult<f64>;

    /// Normalize vector
    fn normalize(v: &IfaValue) -> IfaResult<IfaValue>;
}

/// IoT operations - embedded/GPIO
pub trait IotOps {
    /// Set pin mode
    fn pin_mode(pin: u8, mode: &str) -> IfaResult<()>;

    /// Digital write
    fn digital_write(pin: u8, value: bool) -> IfaResult<()>;

    /// Digital read
    fn digital_read(pin: u8) -> IfaResult<bool>;

    /// Analog read
    fn analog_read(pin: u8) -> IfaResult<u16>;

    /// PWM write
    fn pwm_write(pin: u8, duty: u8) -> IfaResult<()>;
}
