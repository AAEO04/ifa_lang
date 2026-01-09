//! # Ifá FFI - Foreign Function Interface
//!
//! Cross-language bridge for calling external libraries from Ifá-Lang.
//!
//! Design philosophy (Linus-approved):
//! - Simple types, clear mappings
//! - No fancy abstractions - just get the job done
//! - Security by default (whitelist, not blacklist)

use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

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

// =============================================================================
// LIBRARY HANDLE (Platform-specific)
// =============================================================================

/// Platform-agnostic library handle
#[cfg(windows)]
pub type LibHandle = *mut c_void;

#[cfg(not(windows))]
pub type LibHandle = *mut c_void;

/// Load a shared library
#[cfg(windows)]
pub fn load_library(path: &str) -> FfiResult<LibHandle> {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    // LoadLibraryW
    unsafe extern "system" {
        fn LoadLibraryW(lpFileName: *const u16) -> *mut c_void;
    }

    let handle = unsafe { LoadLibraryW(wide.as_ptr()) };
    if handle.is_null() {
        Err(FfiError::LibraryNotFound(path.to_string()))
    } else {
        Ok(handle)
    }
}

#[cfg(unix)]
pub fn load_library(path: &str) -> FfiResult<LibHandle> {
    unsafe extern "C" {
        fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    }

    const RTLD_NOW: c_int = 2;

    let c_path = CString::new(path).map_err(|_| FfiError::LibraryNotFound(path.to_string()))?;
    let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_NOW) };

    if handle.is_null() {
        Err(FfiError::LibraryNotFound(path.to_string()))
    } else {
        Ok(handle)
    }
}

/// Get function pointer from library
///
/// # Safety
/// The handle must be a valid library handle returned by load_library.
#[cfg(windows)]
pub unsafe fn get_proc(handle: LibHandle, name: &str) -> FfiResult<*mut c_void> {
    unsafe extern "system" {
        fn GetProcAddress(hModule: *mut c_void, lpProcName: *const c_char) -> *mut c_void;
    }

    let c_name = CString::new(name).map_err(|_| FfiError::FunctionNotFound(name.to_string()))?;
    let proc = unsafe { GetProcAddress(handle, c_name.as_ptr()) };

    if proc.is_null() {
        Err(FfiError::FunctionNotFound(name.to_string()))
    } else {
        Ok(proc)
    }
}

/// Get function pointer from library
///
/// # Safety
/// The handle must be a valid library handle returned by load_library.
#[cfg(unix)]
pub unsafe fn get_proc(handle: LibHandle, name: &str) -> FfiResult<*mut c_void> {
    unsafe extern "C" {
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    let c_name = CString::new(name).map_err(|_| FfiError::FunctionNotFound(name.to_string()))?;
    let proc = unsafe { dlsym(handle, c_name.as_ptr()) };

    if proc.is_null() {
        Err(FfiError::FunctionNotFound(name.to_string()))
    } else {
        Ok(proc)
    }
}

// =============================================================================
// FFI BRIDGE
// =============================================================================

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

/// The FFI Bridge - main interface for foreign calls
pub struct IfaFfi {
    libraries: HashMap<String, LibHandle>,
    functions: HashMap<String, BoundFunction>,
}

impl IfaFfi {
    pub fn new() -> Self {
        IfaFfi {
            libraries: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Load a shared library
    /// Ifa syntax: ffi.load("mylib")
    pub fn load(&mut self, name: &str, path: Option<&str>) -> FfiResult<()> {
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

        let handle = load_library(&lib_path)?;
        self.libraries.insert(name.to_string(), handle);
        println!("[FFI] Loaded: {}", name);
        Ok(())
    }

    /// Bind a function with signature
    /// Ifa syntax: ffi.bind("lib", "func", ["i32", "i32"], "i32")
    pub fn bind(&mut self, lib: &str, func: &str, args: &[&str], ret: &str) -> FfiResult<()> {
        let handle = self
            .libraries
            .get(lib)
            .ok_or_else(|| FfiError::LibraryNotFound(lib.to_string()))?;

        // Safety: handle was obtained from load_library and is valid
        let ptr = unsafe { get_proc(*handle, func)? };

        let arg_types: Vec<IfaType> = args.iter().filter_map(|s| IfaType::from_str(s)).collect();

        let ret_type = IfaType::from_str(ret).unwrap_or(IfaType::Void);

        let key = format!("{}.{}", lib, func);
        self.functions.insert(
            key.clone(),
            BoundFunction {
                name: func.to_string(),
                ptr,
                sig: FfiSignature {
                    arg_types,
                    ret_type,
                },
            },
        );

        Ok(())
    }

    /// Call a bound function
    /// Ifa syntax: ffi.call("lib", "func", arg1, arg2)
    pub fn call(&self, lib: &str, func: &str, args: &[FfiValue]) -> FfiResult<FfiValue> {
        let key = format!("{}.{}", lib, func);
        let bound = self
            .functions
            .get(&key)
            .ok_or_else(|| FfiError::FunctionNotFound(key))?;

        // Type check
        if args.len() != bound.sig.arg_types.len() {
            return Err(FfiError::TypeMismatch(format!(
                "Expected {} args, got {}",
                bound.sig.arg_types.len(),
                args.len()
            )));
        }

        // For now, return placeholder - real implementation needs libffi
        println!("[FFI] Call: {}({:?})", bound.name, args);
        Ok(FfiValue::Null)
    }

    /// Get list of loaded libraries
    pub fn loaded_libraries(&self) -> Vec<&str> {
        self.libraries.keys().map(|s| s.as_str()).collect()
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
        self.inner.load(name, path)
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
    pub fn call(&self, lib: &str, func: &str, args: &[FfiValue]) -> FfiResult<FfiValue> {
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
