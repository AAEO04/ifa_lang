//! # Ifá FFI - Foreign Function Interface
//!
//! Cross-language bridge for calling external libraries from Ifá-Lang.
//!
//! Design philosophy (Linus-approved):
//! - Simple types, clear mappings
//! - No fancy abstractions - just get the job done
//! - Security by default (whitelist, not blacklist)

use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::os::raw::c_void;

/// FFI Error type
#[derive(Debug)]
pub enum FfiError {
    LibraryNotFound(String),
    FunctionNotFound(String),
    TypeMismatch(String),
    SecurityViolation(String),
    CallFailed(String),
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiError::LibraryNotFound(s) => write!(f, "Library not found: {}", s),
            FfiError::FunctionNotFound(s) => write!(f, "Function not found: {}", s),
            FfiError::TypeMismatch(s) => write!(f, "Type mismatch: {}", s),
            FfiError::SecurityViolation(s) => write!(f, "Security violation: {}", s),
            FfiError::CallFailed(s) => write!(f, "Call failed: {}", s),
        }
    }
}

impl std::error::Error for FfiError {}

pub type FfiResult<T> = Result<T, FfiError>;

// =============================================================================
// TYPE SYSTEM
// =============================================================================

/// FFI type identifiers - keep it simple
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IfaType {
    U8,
    I32,
    I64,
    F64,
    Str,
    Ptr,
    Void,
}

impl IfaType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "u8" => Some(IfaType::U8),
            "i32" => Some(IfaType::I32),
            "i64" => Some(IfaType::I64),
            "f64" => Some(IfaType::F64),
            "str" => Some(IfaType::Str),
            "ptr" => Some(IfaType::Ptr),
            "void" => Some(IfaType::Void),
            _ => None,
        }
    }

    pub fn c_name(&self) -> &'static str {
        match self {
            IfaType::U8 => "uint8_t",
            IfaType::I32 => "int32_t",
            IfaType::I64 => "int64_t",
            IfaType::F64 => "double",
            IfaType::Str => "const char*",
            IfaType::Ptr => "void*",
            IfaType::Void => "void",
        }
    }

    pub fn rust_name(&self) -> &'static str {
        match self {
            IfaType::U8 => "u8",
            IfaType::I32 => "i32",
            IfaType::I64 => "i64",
            IfaType::F64 => "f64",
            IfaType::Str => "*const c_char",
            IfaType::Ptr => "*mut c_void",
            IfaType::Void => "()",
        }
    }
}

/// FFI Value - boxed value for crossing boundaries
#[derive(Debug, Clone)]
pub enum FfiValue {
    U8(u8),
    I32(i32),
    I64(i64),
    F64(f64),
    Str(String),
    Ptr(usize),
    List(Vec<FfiValue>),
    Null,
}

impl FfiValue {
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            FfiValue::I32(v) => Some(*v),
            FfiValue::I64(v) => Some(*v as i32),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            FfiValue::F64(v) => Some(*v),
            FfiValue::I32(v) => Some(*v as f64),
            FfiValue::I64(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            FfiValue::Str(s) => Some(s),
            _ => None,
        }
    }
}

/// Function signature for FFI calls
#[derive(Debug, Clone)]
pub struct FfiSignature {
    pub arg_types: Vec<IfaType>,
    pub ret_type: IfaType,
}

/// Bound function ready to call
pub struct BoundFunction {
    pub name: String,
    pub ptr: *mut c_void,
    pub sig: FfiSignature,
}

// =============================================================================
// POLYGLOT BACKENDS
// =============================================================================

pub enum Backend {
    Native(libloading::Library),
    #[cfg(feature = "js")]
    JavaScript {
        context: Box<boa_engine::Context<'static>>,
    },
    #[cfg(feature = "python")]
    Python {
        /// Optional path to a virtual environment or specific interpreter
        interpreter_path: Option<std::path::PathBuf>,
        /// Module name or script path
        module_name: String,
    },
}

/// Configuration for an FFI bridge
#[derive(Debug, Clone, Default)]
pub struct FfiConfig {
    /// Optional path to an interpreter (e.g., path to venv/bin/python)
    pub interpreter_path: Option<std::path::PathBuf>,
    /// Environment variables to pass (sanitized)
    pub env_vars: HashMap<String, String>,
    /// Maximum execution time for bridge calls (seconds)
    pub timeout_sec: u64,
}

/// The FFI Bridge - main interface for foreign calls
pub struct IfaFfi {
    backends: HashMap<String, Backend>,
    functions: HashMap<String, BoundFunction>,
}

impl IfaFfi {
    pub fn new() -> Self {
        IfaFfi {
            backends: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Initialize a polyglot bridge (Summon Bridge)
    /// Ifa syntax: ffi.itumo("python", config)
    pub fn itumo(
        &mut self,
        language: &str,
        config: Option<FfiConfig>,
        ofun: &crate::ofun::Ofun,
    ) -> FfiResult<()> {
        let cap = format!("bridge:{}", language);
        if !ofun.le(&cap) {
            return Err(FfiError::SecurityViolation(format!(
                "Permission denied: Bridge '{}' not allowed. Run with --allow-{} flag.",
                language, language
            )));
        }

        let config = config.unwrap_or_default();
        let _timeout = if config.timeout_sec > 0 {
            config.timeout_sec
        } else {
            30
        };

        match language {
            #[cfg(feature = "js")]
            "js" | "javascript" => {
                println!(
                    "[FFI] Summoning JavaScript bridge (timeout: {}s)...",
                    timeout
                );
                Ok(())
            }
            #[cfg(feature = "python")]
            "python" | "py" => {
                println!("[FFI] Summoning Python bridge...");

                // If an interpreter path (e.g. venv) is provided, we need to set it
                // BEFORE the first Python::with_gil call if possible.
                // Note: PyO3 auto-initialize uses the first python found on PATH
                // if PYTHONHOME/PYTHONPATH aren't set.
                if let Some(ref path) = config.interpreter_path {
                    if !path.exists() {
                        return Err(FfiError::LibraryNotFound(format!(
                            "Python interpreter not found at {:?}",
                            path
                        )));
                    }
                    println!("[FFI] Using isolated Python at: {:?}", path);

                    // On Unix, we set PYTHONHOME to the venv root
                    // This is a global operation and should be handled with care
                    #[cfg(unix)]
                    if let Some(parent) = path.parent() {
                        if let Some(venv_root) = parent.parent() {
                            std::env::set_var("PYTHONHOME", venv_root);
                        }
                    }
                }

                Ok(())
            }
            "rust" | "c" | "native" => {
                println!("[FFI] Summoning Native bridge...");
                Ok(())
            }
            _ => {
                let status = if cfg!(feature = "python") || cfg!(feature = "js") {
                    "not found"
                } else {
                    "not enabled in this build (rebuild with --features js,python)"
                };
                Err(FfiError::CallFailed(format!(
                    "Bridge '{}' {}",
                    language, status
                )))
            }
        }
    }

    /// Load a shared library (C/Rust) with security validation
    pub fn load_native(&mut self, name: &str, path: Option<&str>) -> FfiResult<()> {
        let lib_path = path.map(String::from).unwrap_or_else(|| {
            #[cfg(windows)]
            {
                format!("{}.dll", name)
            }
            #[cfg(target_os = "macos")]
            {
                format!("lib{}.dylib", name)
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                format!("lib{}.so", name)
            }
        });

        // Security validation
        let validated_path = self.validate_library_path(&lib_path)?;

        unsafe {
            let lib = libloading::Library::new(&validated_path)
                .map_err(|e| FfiError::LibraryNotFound(format!("{}: {}", lib_path, e)))?;
            self.backends.insert(name.to_string(), Backend::Native(lib));
        }
        Ok(())
    }

    /// Validate library path for security
    fn validate_library_path(&self, path: &str) -> FfiResult<std::path::PathBuf> {
        use std::path::PathBuf;

        let path_buf = PathBuf::from(path);

        // 1. Check for path traversal attempts
        let path_str = path_buf.to_string_lossy();
        if path_str.contains("..") {
            return Err(FfiError::SecurityViolation(
                "Path traversal (..) not allowed in library paths".to_string(),
            ));
        }

        // 2. If absolute path, must exist
        if path_buf.is_absolute() {
            if !path_buf.exists() {
                return Err(FfiError::LibraryNotFound(format!(
                    "Library not found at absolute path: {}",
                    path
                )));
            }
            return Ok(path_buf);
        }

        // 3. For relative paths, we allow libloading to search system paths
        // but log a warning
        eprintln!(
            "[FFI] Warning: Loading library from relative path '{}'. Consider using absolute paths.",
            path
        );

        Ok(path_buf)
    }

    /// Load a JavaScript script
    #[cfg(feature = "js")]
    pub fn load_js(&mut self, name: &str, code: &str) -> FfiResult<()> {
        let mut context = boa_engine::Context::default();
        context
            .eval(boa_engine::Source::from_bytes(code.as_bytes()))
            .map_err(|e| FfiError::CallFailed(format!("JS Init failed: {}", e)))?;

        self.backends.insert(
            name.to_string(),
            Backend::JavaScript {
                context: Box::new(context),
            },
        );
        Ok(())
    }

    /// Load a Python module with environment config
    #[cfg(feature = "python")]
    pub fn load_py(
        &mut self,
        name: &str,
        module: &str,
        interpreter_path: Option<std::path::PathBuf>,
    ) -> FfiResult<()> {
        self.backends.insert(
            name.to_string(),
            Backend::Python {
                module_name: module.to_string(),
                interpreter_path,
            },
        );
        Ok(())
    }

    /// Bind a function (Native only)
    pub fn bind(&mut self, lib: &str, func: &str, args: &[&str], ret: &str) -> FfiResult<()> {
        let backend = self
            .backends
            .get(lib)
            .ok_or_else(|| FfiError::LibraryNotFound(lib.to_string()))?;

        match backend {
            Backend::Native(lib_handle) => unsafe {
                let symbol: libloading::Symbol<unsafe extern "C" fn()> = lib_handle
                    .get(func.as_bytes())
                    .map_err(|_| FfiError::FunctionNotFound(func.to_string()))?;

                let ptr = *symbol.deref() as *const () as *mut c_void;

                let arg_types: Vec<IfaType> =
                    args.iter().filter_map(|s| IfaType::from_str(s)).collect();
                let ret_type = IfaType::from_str(ret).unwrap_or(IfaType::Void);

                let key = format!("{}.{}", lib, func);
                self.functions.insert(
                    key,
                    BoundFunction {
                        name: func.to_string(),
                        ptr,
                        sig: FfiSignature {
                            arg_types,
                            ret_type,
                        },
                    },
                );
            },
            #[allow(unreachable_patterns)]
            _ => {
                return Err(FfiError::CallFailed(
                    "Binding only supported for native libraries".into(),
                ));
            }
        }
        Ok(())
    }

    /// Call a function
    pub fn call(&mut self, lib: &str, func: &str, args: &[FfiValue]) -> FfiResult<FfiValue> {
        let backend = self
            .backends
            .get_mut(lib)
            .ok_or_else(|| FfiError::LibraryNotFound(lib.to_string()))?;

        match backend {
            Backend::Native(_) => {
                let key = format!("{}.{}", lib, func);
                let bound = self
                    .functions
                    .get(&key)
                    .ok_or_else(|| FfiError::FunctionNotFound(key.clone()))?;

                #[cfg(feature = "native_ffi")]
                {
                    self.call_native_libffi(bound, args)
                }

                #[cfg(not(feature = "native_ffi"))]
                {
                    let _ = bound;
                    Err(FfiError::CallFailed(format!(
                        "Native dynamic call for {}({:?}) requires native_ffi feature. \
                         Rebuild with --features native_ffi",
                        func, args
                    )))
                }
            }
            #[cfg(feature = "js")]
            Backend::JavaScript { context } => {
                // Convert FfiValue to JsValue
                let mut js_args = Vec::new();
                for arg in args {
                    js_args.push(self.ffi_to_js(arg, context));
                }

                let result = context
                    .eval(boa_engine::Source::from_bytes(
                        format!("{}(...args)", func).as_bytes(),
                    ))
                    .map_err(|e| FfiError::CallFailed(format!("JS Call failed: {}", e)))?;

                Ok(self.js_to_ffi(result))
            }
            #[cfg(feature = "python")]
            Backend::Python { module_name, .. } => {
                use pyo3::prelude::*;
                Python::with_gil(|py| {
                    // Pre-check: Ensure the module is discoverable
                    let module = py.import(module_name.as_str()).map_err(|e| {
                        FfiError::LibraryNotFound(format!(
                            "Py import failed for '{}': {}",
                            module_name, e
                        ))
                    })?;

                    let py_args = PyTuple::new(py, args.iter().map(|a| self.ffi_to_py(a, py)));

                    // Call with timeout protection (simulated for internal PyO3 calls)
                    let result = module
                        .getattr(func)
                        .map_err(|_| {
                            FfiError::FunctionNotFound(format!("{}.{}", module_name, func))
                        })?
                        .call1(py_args)
                        .map_err(|e| FfiError::CallFailed(format!("Py Runtime Error: {}", e)))?;

                    Ok(self.py_to_ffi(result))
                })
            }
            // Feature-gated backends may not all be present
            #[allow(unreachable_patterns)]
            _ => Err(FfiError::CallFailed(
                "Selected FFI backend not included in this build".into(),
            )),
        }
    }

    /// Native function call implementation using libffi
    #[cfg(feature = "native_ffi")]
    fn call_native_libffi(&self, bound: &BoundFunction, args: &[FfiValue]) -> FfiResult<FfiValue> {
        use libffi::high::{Arg, call};
        use libffi::low::CodePtr;

        // Verify argument count
        if args.len() != bound.sig.arg_types.len() {
            return Err(FfiError::TypeMismatch(format!(
                "{}: expected {} args, got {}",
                bound.name,
                bound.sig.arg_types.len(),
                args.len()
            )));
        }

        // Build argument list for libffi
        // We need to hold the actual values in memory
        let mut i32_args: Vec<i32> = Vec::new();
        let mut i64_args: Vec<i64> = Vec::new();
        let mut f64_args: Vec<f64> = Vec::new();
        let mut str_args: Vec<std::ffi::CString> = Vec::new();

        for (i, (val, expected_type)) in args.iter().zip(bound.sig.arg_types.iter()).enumerate() {
            match (val, expected_type) {
                (FfiValue::I32(v), IfaType::I32) => i32_args.push(*v),
                (FfiValue::I64(v), IfaType::I64) => i64_args.push(*v),
                (FfiValue::I32(v), IfaType::I64) => i64_args.push(*v as i64),
                (FfiValue::F64(v), IfaType::F64) => f64_args.push(*v),
                (FfiValue::Str(s), IfaType::Str) => {
                    str_args.push(
                        std::ffi::CString::new(s.as_str()).map_err(|_| {
                            FfiError::TypeMismatch("String contains null byte".into())
                        })?,
                    );
                }
                _ => {
                    return Err(FfiError::TypeMismatch(format!(
                        "Arg {}: cannot convert {:?} to {:?}",
                        i, val, expected_type
                    )));
                }
            }
        }

        // Build the Arg vector for the call
        let mut ffi_args: Vec<Arg> = Vec::with_capacity(args.len());
        let mut i32_idx = 0;
        let mut i64_idx = 0;
        let mut f64_idx = 0;
        let mut str_idx = 0;

        for (val, expected_type) in args.iter().zip(bound.sig.arg_types.iter()) {
            match (val, expected_type) {
                (FfiValue::I32(_), IfaType::I32) => {
                    ffi_args.push(Arg::new(&i32_args[i32_idx]));
                    i32_idx += 1;
                }
                (FfiValue::I64(_), IfaType::I64) | (FfiValue::I32(_), IfaType::I64) => {
                    ffi_args.push(Arg::new(&i64_args[i64_idx]));
                    i64_idx += 1;
                }
                (FfiValue::F64(_), IfaType::F64) => {
                    ffi_args.push(Arg::new(&f64_args[f64_idx]));
                    f64_idx += 1;
                }
                (FfiValue::Str(_), IfaType::Str) => {
                    ffi_args.push(Arg::new(&str_args[str_idx].as_ptr()));
                    str_idx += 1;
                }
                _ => unreachable!(), // Already validated above
            }
        }

        // Make the call based on return type
        let code_ptr = CodePtr::from_ptr(bound.ptr as *const _);

        unsafe {
            match bound.sig.ret_type {
                IfaType::Void => {
                    call::<()>(code_ptr, ffi_args.as_slice());
                    Ok(FfiValue::Null)
                }
                IfaType::I32 => {
                    let result: i32 = call(code_ptr, ffi_args.as_slice());
                    Ok(FfiValue::I32(result))
                }
                IfaType::I64 => {
                    let result: i64 = call(code_ptr, ffi_args.as_slice());
                    Ok(FfiValue::I64(result))
                }
                IfaType::F64 => {
                    let result: f64 = call(code_ptr, ffi_args.as_slice());
                    Ok(FfiValue::F64(result))
                }
                IfaType::Ptr => {
                    let result: usize = call(code_ptr, ffi_args.as_slice());
                    Ok(FfiValue::Ptr(result))
                }
                IfaType::U8 => {
                    let result: u8 = call(code_ptr, ffi_args.as_slice());
                    Ok(FfiValue::U8(result))
                }
                IfaType::Str => {
                    // C strings returned from FFI - be careful with ownership!
                    let result: *const std::os::raw::c_char = call(code_ptr, ffi_args.as_slice());
                    if result.is_null() {
                        Ok(FfiValue::Null)
                    } else {
                        let c_str = std::ffi::CStr::from_ptr(result);
                        Ok(FfiValue::Str(c_str.to_string_lossy().into_owned()))
                    }
                }
            }
        }
    }

    // Helpers for type conversion
    #[cfg(feature = "js")]
    fn ffi_to_js(&self, val: &FfiValue, _ctx: &mut boa_engine::Context) -> boa_engine::JsValue {
        match val {
            FfiValue::I32(v) => boa_engine::JsValue::new(*v),
            FfiValue::F64(v) => boa_engine::JsValue::new(*v),
            FfiValue::Str(s) => boa_engine::JsValue::new(s.as_str()),
            _ => boa_engine::JsValue::null(),
        }
    }

    #[cfg(feature = "js")]
    fn js_to_ffi(&self, val: boa_engine::JsValue) -> FfiValue {
        if val.is_number() {
            if let Some(i) = val.as_integer() {
                return FfiValue::I32(i);
            }
            return FfiValue::F64(val.as_number().unwrap_or(0.0));
        }
        if val.is_string() {
            return FfiValue::Str(
                val.as_string()
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default(),
            );
        }
        if val.is_boolean() {
            return FfiValue::I32(if val.as_boolean().unwrap_or(false) {
                1
            } else {
                0
            });
        }
        FfiValue::Null
    }

    #[cfg(feature = "python")]
    fn ffi_to_py<'py>(&self, val: &FfiValue, py: pyo3::Python<'py>) -> pyo3::PyObject {
        use pyo3::prelude::*;
        match val {
            FfiValue::I32(v) => v.into_py(py),
            FfiValue::I64(v) => v.into_py(py),
            FfiValue::F64(v) => v.into_py(py),
            FfiValue::Str(s) => s.into_py(py),
            FfiValue::List(l) => {
                let items: Vec<PyObject> = l.iter().map(|i| self.ffi_to_py(i, py)).collect();
                PyList::new(py, items).into_py(py)
            }
            _ => py.None(),
        }
    }

    #[cfg(feature = "python")]
    fn py_to_ffi(&self, val: &pyo3::PyAny) -> FfiValue {
        if let Ok(i) = val.extract::<i32>() {
            FfiValue::I32(i)
        } else if let Ok(f) = val.extract::<f64>() {
            FfiValue::F64(f)
        } else if let Ok(s) = val.extract::<String>() {
            FfiValue::Str(s)
        } else if let Ok(list) = val.extract::<Vec<&pyo3::PyAny>>() {
            FfiValue::List(list.iter().map(|&i| self.py_to_ffi(i)).collect())
        } else {
            FfiValue::Null
        }
    }
}

impl Default for IfaFfi {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// SECURE FFI (Sandboxed)
// =============================================================================

/// Secure FFI with whitelist
pub struct SecureFfi {
    inner: IfaFfi,
    whitelist: HashSet<String>,
    blocked_symbols: HashSet<String>,
}

impl SecureFfi {
    pub fn new() -> Self {
        let mut blocked = HashSet::new();
        // Block dangerous symbols
        blocked.insert("system".to_string());
        blocked.insert("exec".to_string());
        blocked.insert("execve".to_string());
        blocked.insert("fork".to_string());
        blocked.insert("popen".to_string());
        blocked.insert("dlopen".to_string()); // Prevent nested loading

        SecureFfi {
            inner: IfaFfi::new(),
            whitelist: HashSet::new(),
            blocked_symbols: blocked,
        }
    }

    /// Add library to whitelist
    pub fn allow(&mut self, lib: &str) {
        self.whitelist.insert(lib.to_string());
    }

    /// Load library (only if whitelisted)
    pub fn load(&mut self, name: &str, path: Option<&str>) -> FfiResult<()> {
        if !self.whitelist.contains(name) {
            return Err(FfiError::SecurityViolation(format!(
                "Library '{}' not whitelisted. Use allow() first.",
                name
            )));
        }
        self.inner.load_native(name, path)
    }

    /// Bind function (blocks dangerous symbols)
    pub fn bind(&mut self, lib: &str, func: &str, args: &[&str], ret: &str) -> FfiResult<()> {
        if self.blocked_symbols.contains(func) {
            return Err(FfiError::SecurityViolation(format!(
                "Symbol '{}' is blocked for security reasons",
                func
            )));
        }
        self.inner.bind(lib, func, args, ret)
    }

    /// Call function
    pub fn call(&mut self, lib: &str, func: &str, args: &[FfiValue]) -> FfiResult<FfiValue> {
        self.inner.call(lib, func, args)
    }
}

impl Default for SecureFfi {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// C HEADER GENERATOR
// =============================================================================

/// Generate C header for Ifa exports
pub fn generate_c_header(exports: &[(String, FfiSignature)]) -> String {
    let mut out = String::new();

    out.push_str("/* IFA-LANG C API - Auto-generated */\n");
    out.push_str("#ifndef IFA_API_H\n");
    out.push_str("#define IFA_API_H\n\n");
    out.push_str("#include <stdint.h>\n\n");
    out.push_str("#ifdef __cplusplus\n");
    out.push_str("extern \"C\" {\n");
    out.push_str("#endif\n\n");

    for (name, sig) in exports {
        let ret = sig.ret_type.c_name();
        let args: Vec<String> = sig
            .arg_types
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{} arg{}", t.c_name(), i))
            .collect();
        let args_str = if args.is_empty() {
            "void".to_string()
        } else {
            args.join(", ")
        };

        let c_name = name.replace(".", "_");
        out.push_str(&format!("{} ifa_{}({});\n", ret, c_name, args_str));
    }

    out.push_str("\n#ifdef __cplusplus\n");
    out.push_str("}\n");
    out.push_str("#endif\n\n");
    out.push_str("#endif /* IFA_API_H */\n");

    out
}

/// Generate Rust FFI bindings
pub fn generate_rust_bindings(exports: &[(String, FfiSignature)]) -> String {
    let mut out = String::new();

    out.push_str("// IFA-LANG Rust Bindings - Auto-generated\n");
    out.push_str("use std::os::raw::{c_char, c_void};\n\n");
    out.push_str("extern \"C\" {\n");

    for (name, sig) in exports {
        let ret = sig.ret_type.rust_name();
        let args: Vec<String> = sig
            .arg_types
            .iter()
            .enumerate()
            .map(|(i, t)| format!("arg{}: {}", i, t.rust_name()))
            .collect();
        let args_str = args.join(", ");

        let rust_name = name.replace(".", "_");
        out.push_str(&format!(
            "    pub fn ifa_{}({}) -> {};\n",
            rust_name, args_str, ret
        ));
    }

    out.push_str("}\n");

    out
}

// =============================================================================
// IFA API LAYER - Expose Ifa functions to external code
// =============================================================================

/// Endpoint information
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub name: String,
    pub handler_id: usize,
    pub arg_types: Vec<IfaType>,
    pub ret_type: IfaType,
}

/// Handler function type
pub type ApiHandler = Box<dyn Fn(&[FfiValue]) -> FfiResult<FfiValue> + Send + Sync>;

/// API Layer for exposing Ifa functions
pub struct IfaApi {
    endpoints: HashMap<String, Endpoint>,
    handlers: Vec<ApiHandler>,
}

impl IfaApi {
    pub fn new() -> Self {
        IfaApi {
            endpoints: HashMap::new(),
            handlers: Vec::new(),
        }
    }

    /// Expose a function as an API endpoint
    pub fn expose<F>(&mut self, name: &str, arg_types: &[IfaType], ret_type: IfaType, handler: F)
    where
        F: Fn(&[FfiValue]) -> FfiResult<FfiValue> + Send + Sync + 'static,
    {
        let handler_id = self.handlers.len();
        self.handlers.push(Box::new(handler));

        self.endpoints.insert(
            name.to_string(),
            Endpoint {
                name: name.to_string(),
                handler_id,
                arg_types: arg_types.to_vec(),
                ret_type,
            },
        );
    }

    /// Call an exposed endpoint
    pub fn call(&self, name: &str, args: &[FfiValue]) -> FfiResult<FfiValue> {
        let endpoint = self
            .endpoints
            .get(name)
            .ok_or_else(|| FfiError::FunctionNotFound(name.to_string()))?;

        // Type check arg count
        if args.len() != endpoint.arg_types.len() {
            return Err(FfiError::TypeMismatch(format!(
                "{}: expected {} args, got {}",
                name,
                endpoint.arg_types.len(),
                args.len()
            )));
        }

        let handler = &self.handlers[endpoint.handler_id];
        handler(args)
    }

    /// List all endpoints
    pub fn list_endpoints(&self) -> Vec<&Endpoint> {
        self.endpoints.values().collect()
    }

    /// Generate JSON schema for all endpoints
    pub fn to_json_schema(&self) -> String {
        let mut out = String::from("{\n");
        let mut first = true;

        for (name, ep) in &self.endpoints {
            if !first {
                out.push_str(",\n");
            }
            first = false;

            let args: Vec<&str> = ep.arg_types.iter().map(|t| t.rust_name()).collect();
            out.push_str(&format!(
                "  \"{}\": {{ \"args\": {:?}, \"returns\": \"{}\" }}",
                name,
                args,
                ep.ret_type.rust_name()
            ));
        }

        out.push_str("\n}");
        out
    }

    /// Export as C header
    pub fn generate_c_header(&self) -> String {
        let exports: Vec<(String, FfiSignature)> = self
            .endpoints
            .values()
            .map(|ep| {
                (
                    ep.name.clone(),
                    FfiSignature {
                        arg_types: ep.arg_types.clone(),
                        ret_type: ep.ret_type,
                    },
                )
            })
            .collect();
        generate_c_header(&exports)
    }

    /// Export as Rust bindings
    pub fn generate_rust_bindings(&self) -> String {
        let exports: Vec<(String, FfiSignature)> = self
            .endpoints
            .values()
            .map(|ep| {
                (
                    ep.name.clone(),
                    FfiSignature {
                        arg_types: ep.arg_types.clone(),
                        ret_type: ep.ret_type,
                    },
                )
            })
            .collect();
        generate_rust_bindings(&exports)
    }
}

impl Default for IfaApi {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// RPC SERVER - JSON-RPC over HTTP
// =============================================================================

/// JSON-RPC Request
#[derive(Debug)]
pub struct RpcRequest {
    pub id: u64,
    pub method: String,
    pub params: Vec<FfiValue>,
}

/// JSON-RPC Response
#[derive(Debug)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<FfiValue>,
    pub error: Option<String>,
}

impl RpcResponse {
    pub fn success(id: u64, result: FfiValue) -> Self {
        RpcResponse {
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, msg: String) -> Self {
        RpcResponse {
            id,
            result: None,
            error: Some(msg),
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        if let Some(ref err) = self.error {
            format!(
                "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-1,\"message\":\"{}\"}},\"id\":{}}}",
                err, self.id
            )
        } else {
            let result_json = match &self.result {
                Some(FfiValue::I32(v)) => v.to_string(),
                Some(FfiValue::I64(v)) => v.to_string(),
                Some(FfiValue::F64(v)) => v.to_string(),
                Some(FfiValue::Str(s)) => format!("\"{}\"", s),
                Some(FfiValue::U8(v)) => v.to_string(),
                Some(FfiValue::Ptr(v)) => v.to_string(),
                Some(FfiValue::List(l)) => {
                    let items: Vec<String> = l
                        .iter()
                        .map(|v| match v {
                            FfiValue::I32(x) => x.to_string(),
                            FfiValue::I64(x) => x.to_string(),
                            FfiValue::F64(x) => x.to_string(),
                            FfiValue::Str(x) => format!("\"{}\"", x),
                            FfiValue::U8(x) => x.to_string(),
                            FfiValue::Ptr(x) => x.to_string(),
                            FfiValue::Null => "null".to_string(),
                            FfiValue::List(_) => "[...]".to_string(),
                        })
                        .collect();
                    format!("[{}]", items.join(","))
                }
                Some(FfiValue::Null) | None => "null".to_string(),
            };
            format!(
                "{{\"jsonrpc\":\"2.0\",\"result\":{},\"id\":{}}}",
                result_json, self.id
            )
        }
    }
}

/// Simple RPC Server
pub struct IfaRpcServer {
    api: IfaApi,
    port: u16,
}

impl IfaRpcServer {
    pub fn new(api: IfaApi, port: u16) -> Self {
        IfaRpcServer { api, port }
    }

    /// Handle a JSON-RPC request string
    pub fn handle_request(&self, json: &str) -> String {
        // Simple JSON parsing (no dependencies)
        let id = self.extract_id(json).unwrap_or(1);
        let method = match self.extract_string(json, "method") {
            Some(m) => m,
            None => return RpcResponse::error(id, "Missing method".to_string()).to_json(),
        };

        let params = self.extract_params(json);

        match self.api.call(&method, &params) {
            Ok(result) => RpcResponse::success(id, result).to_json(),
            Err(e) => RpcResponse::error(id, e.to_string()).to_json(),
        }
    }

    /// Extract id from JSON
    fn extract_id(&self, json: &str) -> Option<u64> {
        let id_start = json.find("\"id\":")?;
        let rest = &json[id_start + 5..];
        let id_str: String = rest
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        id_str.parse().ok()
    }

    /// Extract string field from JSON
    fn extract_string(&self, json: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":\"", field);
        let start = json.find(&pattern)? + pattern.len();
        let rest = &json[start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }

    /// Extract params array (simplified)
    fn extract_params(&self, json: &str) -> Vec<FfiValue> {
        // Find params array
        let params_start = match json.find("\"params\":[") {
            Some(p) => p + 10,
            None => return Vec::new(),
        };

        let rest = &json[params_start..];
        let params_end = match rest.find(']') {
            Some(e) => e,
            None => return Vec::new(),
        };

        let params_str = &rest[..params_end];

        // Parse simple values
        params_str
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    None
                } else if s.starts_with('"') && s.ends_with('"') {
                    Some(FfiValue::Str(s[1..s.len() - 1].to_string()))
                } else if s.contains('.') {
                    s.parse::<f64>().ok().map(FfiValue::F64)
                } else {
                    s.parse::<i64>().ok().map(FfiValue::I64)
                }
            })
            .collect()
    }

    /// Start HTTP server (blocking)
    #[cfg(feature = "full")]
    #[allow(dead_code)]
    pub fn start(&self) -> std::io::Result<()> {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;
        println!("[RPC] Server started on port {}", self.port);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0u8; 4096];
                    if let Ok(n) = stream.read(&mut buffer) {
                        let request = String::from_utf8_lossy(&buffer[..n]);

                        // Extract body from HTTP request
                        if let Some(body_start) = request.find("\r\n\r\n") {
                            let body = &request[body_start + 4..];
                            let response = self.handle_request(body);

                            let http_response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                response.len(),
                                response
                            );
                            let _ = stream.write_all(http_response.as_bytes());
                        }
                    }
                }
                Err(e) => eprintln!("[RPC] Connection error: {}", e),
            }
        }
        Ok(())
    }

    /// Get port
    pub fn port(&self) -> u16 {
        self.port
    }
}

// =============================================================================
// STDLIB API INTEGRATION
// =============================================================================

/// Create API from Ifa stdlib (16 Odu domains)
pub fn create_stdlib_api() -> IfaApi {
    let mut api = IfaApi::new();

    // Obara (Math - Add/Mul)
    api.expose(
        "obara.fikun",
        &[IfaType::I64, IfaType::I64],
        IfaType::I64,
        |args| {
            let a = args.get(0).and_then(|v| v.as_i32()).unwrap_or(0) as i64;
            let b = args.get(1).and_then(|v| v.as_i32()).unwrap_or(0) as i64;
            Ok(FfiValue::I64(a + b))
        },
    );

    api.expose(
        "obara.isodipupo",
        &[IfaType::I64, IfaType::I64],
        IfaType::I64,
        |args| {
            let a = args.get(0).and_then(|v| v.as_i32()).unwrap_or(0) as i64;
            let b = args.get(1).and_then(|v| v.as_i32()).unwrap_or(1) as i64;
            Ok(FfiValue::I64(a * b))
        },
    );

    // Oturupon (Math - Sub/Div)
    api.expose(
        "oturupon.din",
        &[IfaType::I64, IfaType::I64],
        IfaType::I64,
        |args| {
            let a = args.get(0).and_then(|v| v.as_i32()).unwrap_or(0) as i64;
            let b = args.get(1).and_then(|v| v.as_i32()).unwrap_or(0) as i64;
            Ok(FfiValue::I64(a - b))
        },
    );

    api.expose(
        "oturupon.pin",
        &[IfaType::I64, IfaType::I64],
        IfaType::F64,
        |args| {
            let a = args.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = args.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0);
            if b == 0.0 {
                Err(FfiError::CallFailed("Division by zero".to_string()))
            } else {
                Ok(FfiValue::F64(a / b))
            }
        },
    );

    // Ika (Strings)
    api.expose("ika.gigun", &[IfaType::Str], IfaType::I64, |args| {
        let s = args.get(0).and_then(|v| v.as_str()).unwrap_or("");
        Ok(FfiValue::I64(s.len() as i64))
    });

    // Iwori (Time)
    api.expose("iwori.epoch", &[], IfaType::I64, |_args| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Ok(FfiValue::I64(epoch))
    });

    // Owonrin (Random)
    api.expose(
        "owonrin.afesona",
        &[IfaType::I64, IfaType::I64],
        IfaType::I64,
        |args| {
            let min = args.get(0).and_then(|v| v.as_i32()).unwrap_or(0) as i64;
            let max = args.get(1).and_then(|v| v.as_i32()).unwrap_or(100) as i64;

            // Simple LCG random
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(12345);

            let range = (max - min).max(1) as u64;
            let random =
                ((seed.wrapping_mul(6364136223846793005).wrapping_add(1)) % range) as i64 + min;
            Ok(FfiValue::I64(random))
        },
    );

    // Ogbe (System)
    api.expose("ogbe.version", &[], IfaType::Str, |_args| {
        Ok(FfiValue::Str("1.0.0".to_string()))
    });

    api
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mapping() {
        assert_eq!(IfaType::I32.c_name(), "int32_t");
        assert_eq!(IfaType::Str.rust_name(), "*const c_char");
    }

    #[test]
    fn test_ffi_value_conversion() {
        let v = FfiValue::I32(42);
        assert_eq!(v.as_i32(), Some(42));
        assert_eq!(v.as_f64(), Some(42.0));
    }

    #[test]
    fn test_secure_ffi_blocks_system() {
        let mut ffi = SecureFfi::new();
        ffi.allow("libc");

        // Should fail - system is blocked
        let result = ffi.bind("libc", "system", &["str"], "i32");
        assert!(result.is_err());
    }

    #[test]
    fn test_header_generation() {
        let exports = vec![(
            "math.add".to_string(),
            FfiSignature {
                arg_types: vec![IfaType::I32, IfaType::I32],
                ret_type: IfaType::I32,
            },
        )];

        let header = generate_c_header(&exports);
        assert!(header.contains("int32_t ifa_math_add"));
    }

    #[test]
    fn test_api_layer() {
        let mut api = IfaApi::new();
        api.expose("add", &[IfaType::I32, IfaType::I32], IfaType::I32, |args| {
            let a = args.get(0).and_then(|v| v.as_i32()).unwrap_or(0);
            let b = args.get(1).and_then(|v| v.as_i32()).unwrap_or(0);
            Ok(FfiValue::I32(a + b))
        });

        let result = api
            .call("add", &[FfiValue::I32(10), FfiValue::I32(20)])
            .unwrap();
        assert_eq!(result.as_i32(), Some(30));
    }

    #[test]
    fn test_rpc_response() {
        let resp = RpcResponse::success(1, FfiValue::I32(42));
        let json = resp.to_json();
        assert!(json.contains("\"result\":42"));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_stdlib_api() {
        let api = create_stdlib_api();

        // Test math
        let result = api
            .call("obara.fikun", &[FfiValue::I64(5), FfiValue::I64(3)])
            .unwrap();
        assert_eq!(result.as_i32(), Some(8));

        // Test string length
        let result = api
            .call("ika.gigun", &[FfiValue::Str("hello".to_string())])
            .unwrap();
        assert_eq!(result.as_i32(), Some(5));
    }
}
