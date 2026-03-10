//! # Unified System Logic (The Facade)
//!
//! Handles all system-level Odu calls:
//! - Sys.cpu.* -> infra::cpu
//! - Sys.gpu.* -> infra::gpu
//! - Sys.kernel.* -> infra::kernel
//! - Sys.storage.* -> infra::storage

use ifa_core::error::{IfaError, IfaResult};
use ifa_core::interpreter::environment::Environment;
use ifa_core::interpreter::handlers::OduHandler;
use ifa_core::lexer::OduDomain;
use ifa_core::value::IfaValue;

use crate::infra;

#[allow(unused_imports)]
use std::sync::{Arc, RwLock};

/// Unified System Handler
/// Holds state for GPU context, Storage connection, and Async Runtime.
pub struct SysHandler {
    #[cfg(feature = "gpu")]
    gpu: Arc<RwLock<Option<infra::gpu::GpuContext>>>,

    #[cfg(feature = "persistence")]
    store: Arc<tokio::sync::Mutex<Option<infra::storage::OduStore>>>,

    /// Runtime for executing async infrastructure calls
    #[allow(dead_code)]
    rt: Arc<infra::runtime::SysRuntime>,
}

impl SysHandler {
    pub fn new() -> Self {
        // Initialize SysRuntime (abstraction over tokio/stub)
        let rt = infra::runtime::SysRuntime::new().expect("Failed to create SysHandler runtime");

        Self {
            #[cfg(feature = "gpu")]
            gpu: Arc::new(RwLock::new(None)),

            #[cfg(feature = "persistence")]
            store: Arc::new(tokio::sync::Mutex::new(None)),

            rt: Arc::new(rt),
        }
    }

    /// Helper: Convert IfaValue::List(List(Float)) to params for matmul
    /// Returns (flat_vec, rows, cols)
    #[cfg(feature = "gpu")]
    fn parse_matrix(&self, val: &IfaValue) -> IfaResult<(Vec<f32>, u32, u32)> {
        match val {
            IfaValue::List(rows) => {
                let m = rows.len() as u32;
                if m == 0 {
                    return Err(IfaError::Runtime("Empty matrix".into()));
                }

                let mut flat = Vec::new();
                let mut n = 0;

                for (i, row) in rows.iter().enumerate() {
                    match row {
                        IfaValue::List(cols) => {
                            if i == 0 {
                                n = cols.len() as u32;
                            } else if cols.len() as u32 != n {
                                return Err(IfaError::Runtime(
                                    "Jagged matrix not supported".into(),
                                ));
                            }

                            for item in &**cols {
                                // Deref Arc<Vec>
                                if let IfaValue::Float(f) = item {
                                    flat.push(*f as f32);
                                } else if let IfaValue::Int(v) = item {
                                    flat.push(*v as f32);
                                } else {
                                    return Err(IfaError::Runtime(
                                        "Matrix must contain numbers".into(),
                                    ));
                                }
                            }
                        }
                        _ => return Err(IfaError::Runtime("Matrix must be list of lists".into())),
                    }
                }
                Ok((flat, m, n))
            }
            _ => Err(IfaError::Runtime("Expected matrix (List of Lists)".into())),
        }
    }
    // -------------------------------------------------------------------------
    // Domain Handlers (Refactored)
    // -------------------------------------------------------------------------

    fn handle_kernel(&self, method: &str, _args: &[IfaValue]) -> IfaResult<IfaValue> {
        match method {
            "core_count" => {
                let count = infra::kernel::num_cores();
                Ok(IfaValue::Int(count as i64))
            }
            "uptime" => {
                #[cfg(feature = "sysinfo")]
                {
                    Ok(IfaValue::Int(infra::kernel::uptime() as i64))
                }
                #[cfg(not(feature = "sysinfo"))]
                {
                    Ok(IfaValue::Int(0))
                }
            }
            "memory_stats" => {
                #[cfg(feature = "sysinfo")]
                {
                    let stats = infra::kernel::memory_stats();
                    let mut map = std::collections::HashMap::new();
                    map.insert("total".into(), IfaValue::Int(stats.total as i64));
                    map.insert("free".into(), IfaValue::Int(stats.available as i64));
                    map.insert("used".into(), IfaValue::Int(stats.used as i64));
                    Ok(IfaValue::Map(Arc::new(map)))
                }
                #[cfg(not(feature = "sysinfo"))]
                Err(IfaError::Runtime("Sysinfo disabled".into()))
            }
            "os_info" => Ok(IfaValue::Str("Ifa OS v1.0".into())), // Placeholder or impl
            _ => Err(IfaError::Runtime(format!(
                "Unknown kernel method: {}",
                method
            ))),
        }
    }

    fn handle_cpu(&self, method: &str, args: &[IfaValue]) -> IfaResult<IfaValue> {
        match method {
            "submit_task" => {
                #[cfg(feature = "parallel")]
                {
                    if let Some(IfaValue::Int(id)) = args.first() {
                        let id = *id;
                        rayon::spawn(move || {
                            println!("[Sys::Cpu] Executing task {}", id);
                        });
                        Ok(IfaValue::Bool(true))
                    } else {
                        Err(IfaError::Runtime("submit_task(id) required".into()))
                    }
                }
                #[cfg(not(feature = "parallel"))]
                Err(IfaError::Runtime("Parallel execution disabled".into()))
            }
            _ => Err(IfaError::Runtime(format!("Unknown cpu method: {}", method))),
        }
    }

    #[cfg(feature = "gpu")]
    fn handle_gpu(&self, method: &str, args: &[IfaValue]) -> IfaResult<IfaValue> {
        match method {
            "gpu_init" => {
                let mut gpu_guard = self.gpu.write().map_err(|_| IfaError::Runtime("GPU lock poisoned".into()))?;
                if gpu_guard.is_some() {
                    return Ok(IfaValue::Bool(true));
                }

                // Use SysRuntime abstraction
                let ctx = self
                    .rt
                    .block_on(infra::gpu::GpuContext::new())?
                    .map_err(IfaError::Runtime)?;
                *gpu_guard = Some(ctx);
                Ok(IfaValue::Bool(true))
            }
            "gpu_matmul" => {
                if args.len() < 2 {
                    return Err(IfaError::Runtime("gpu_matmul(A, B) required".into()));
                }
                let gpu_guard = self.gpu.read().map_err(|_| IfaError::Runtime("GPU lock poisoned".into()))?;
                let ctx = gpu_guard
                    .as_ref()
                    .ok_or(IfaError::Runtime("GPU not initialized".into()))?;

                let (a_flat, m, k1) = self.parse_matrix(&args[0])?;
                let (b_flat, k2, n) = self.parse_matrix(&args[1])?;

                if k1 != k2 {
                    return Err(IfaError::Runtime(format!(
                        "Matrix dimension mismatch: {} != {}",
                        k1, k2
                    )));
                }

                use wgpu::util::DeviceExt;
                let buf_a = ctx
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Matrix A"),
                        contents: bytemuck::cast_slice(&a_flat),
                        usage: wgpu::BufferUsages::STORAGE,
                    });
                let buf_b = ctx
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Matrix B"),
                        contents: bytemuck::cast_slice(&b_flat),
                        usage: wgpu::BufferUsages::STORAGE,
                    });

                let buf_c = ctx.matmul(&buf_a, &buf_b, m, n, k1);

                let data_vec = ctx.read_buffer(&buf_c).map_err(|e| IfaError::Runtime(e))?;
                let result: &[f32] = bytemuck::cast_slice(&data_vec);

                let mut matrix = Vec::new();
                for r in 0..m {
                    let mut row = Vec::new();
                    for c in 0..n {
                        let idx = (r * n + c) as usize;
                        row.push(IfaValue::Float(result[idx] as f64));
                    }
                    matrix.push(IfaValue::List(Arc::new(row)));
                }
                Ok(IfaValue::List(Arc::new(matrix)))
            }
            "gpu_sync" => {
                let gpu_guard = self.gpu.read().map_err(|_| IfaError::Runtime("GPU lock poisoned".into()))?;
                if let Some(ctx) = gpu_guard.as_ref() {
                    ctx.sync();
                }
                Ok(IfaValue::Bool(true))
            }
            "gpu_load_shader" => {
                if args.len() < 2 {
                    return Err(IfaError::Runtime(
                        "gpu_load_shader(name, source) required".into(),
                    ));
                }
                let name = match &args[0] {
                    IfaValue::Str(s) => s,
                    _ => return Err(IfaError::Runtime("name must be string".into())),
                };
                let source = match &args[1] {
                    IfaValue::Str(s) => s,
                    _ => return Err(IfaError::Runtime("source must be string".into())),
                };

                let gpu_guard = self.gpu.write().map_err(|_| IfaError::Runtime("GPU lock poisoned".into()))?;
                let ctx = gpu_guard
                    .as_ref()
                    .ok_or(IfaError::Runtime("GPU not initialized".into()))?;
                ctx.get_or_create_pipeline(name, source, "main");
                Ok(IfaValue::Bool(true))
            }
            "gpu_dispatch" => {
                if args.len() < 4 {
                    return Err(IfaError::Runtime(
                        "gpu_dispatch(name, x, y, z) required".into(),
                    ));
                }
                let name = match &args[0] {
                    IfaValue::Str(s) => s,
                    _ => return Err(IfaError::Runtime("name must be string".into())),
                };
                let x = match &args[1] {
                    IfaValue::Int(i) => *i as u32,
                    _ => return Err(IfaError::Runtime("x must be int".into())),
                };
                let y = match &args[2] {
                    IfaValue::Int(i) => *i as u32,
                    _ => return Err(IfaError::Runtime("y must be int".into())),
                };
                let z = match &args[3] {
                    IfaValue::Int(i) => *i as u32,
                    _ => return Err(IfaError::Runtime("z must be int".into())),
                };

                let gpu_guard = self.gpu.read().map_err(|_| IfaError::Runtime("GPU lock poisoned".into()))?;
                let ctx = gpu_guard
                    .as_ref()
                    .ok_or(IfaError::Runtime("GPU not initialized".into()))?;
                ctx.dispatch_pipeline(name, x, y, z)
                    .map_err(IfaError::Runtime)?;
                Ok(IfaValue::Bool(true))
            }
            _ => Err(IfaError::Runtime(format!("Unknown gpu method: {}", method))),
        }
    }

    #[cfg(feature = "persistence")]
    fn handle_storage(&self, method: &str, args: &[IfaValue]) -> IfaResult<IfaValue> {
        match method {
            "store_open" => {
                if let Some(IfaValue::Str(path_str)) = args.first() {
                    let path = std::path::PathBuf::from(path_str.as_ref());
                    // Using SysRuntime
                    let s = self
                        .rt
                        .block_on(infra::storage::OduStore::open(path))
                        .map_err(|e| {
                            IfaError::Runtime(format!("Store open failed (runtime): {}", e))
                        })?
                        .map_err(|e| IfaError::Custom(format!("Store error: {}", e)))?;
                    // rt.block_on returns IfaResult<OduResult<Store>>?
                    // OduStore::open returns Result<OduStore, StorageError>.
                    // So block_on returns IfaResult<Result<OduStore, StorageError>>.
                    // We map inner error.

                    let mut store_guard = self.rt.block_on(self.store.lock())?;
                    *store_guard = Some(s);
                    Ok(IfaValue::Bool(true))
                } else {
                    Err(IfaError::Runtime("store_open(path) required".into()))
                }
            }
            "store_set" => {
                if args.len() < 2 {
                    return Err(IfaError::Runtime("store_set(key, val) required".into()));
                }
                let key = match &args[0] {
                    IfaValue::Str(s) => s.clone(),
                    _ => return Err(IfaError::Runtime("Key must be string".into())),
                };

                let mut store_guard = self.rt.block_on(self.store.lock())?;
                let store = store_guard
                    .as_mut()
                    .ok_or(IfaError::Runtime("Store not open".into()))?;

                self.rt
                    .block_on(store.set(&key, &args[1]))?
                    .map(|_| IfaValue::Bool(true))
                    .map_err(|e| IfaError::Runtime(format!("Store set failed: {}", e)))
            }
            "store_get" => {
                let key = match args.first() {
                    Some(IfaValue::Str(s)) => s.clone(),
                    _ => return Err(IfaError::Runtime("store_get(key) required".into())),
                };
                let store_guard = self.rt.block_on(self.store.lock())?;
                let store = store_guard
                    .as_ref()
                    .ok_or(IfaError::Runtime("Store not open".into()))?;

                let res = self.rt.block_on(store.get(&key))?;
                match res {
                    Ok(val) => Ok(val),
                    Err(infra::storage::StorageError::KeyNotFound) => Ok(IfaValue::Null),
                    Err(e) => Err(IfaError::Runtime(format!("Store get failed: {}", e))),
                }
            }
            "store_delete" => {
                let key = match args.first() {
                    Some(IfaValue::Str(s)) => s.clone(),
                    _ => return Err(IfaError::Runtime("store_delete(key) required".into())),
                };
                let mut store_guard = self.rt.block_on(self.store.lock())?;
                let store = store_guard
                    .as_mut()
                    .ok_or(IfaError::Runtime("Store not open".into()))?;

                self.rt
                    .block_on(store.delete(&key))?
                    .map(IfaValue::Bool)
                    .map_err(|e| IfaError::Runtime(format!("Store delete failed: {}", e)))
            }
            "store_compact" => {
                let mut store_guard = self.rt.block_on(self.store.lock())?;
                let store = store_guard
                    .as_mut()
                    .ok_or(IfaError::Runtime("Store not open".into()))?;

                self.rt
                    .block_on(store.compact())?
                    .map(|_| IfaValue::Bool(true))
                    .map_err(|e| IfaError::Runtime(format!("Compact failed: {}", e)))
            }
            _ => Err(IfaError::Runtime(format!(
                "Unknown storage method: {}",
                method
            ))),
        }
    }
}

impl Default for SysHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl OduHandler for SysHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Sys
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            // Kernel
            "core_count",
            "memory_stats",
            "uptime",
            "os_info",
            "submit_task",
            // GPU
            "gpu_init",
            "gpu_matmul",
            "gpu_sync",
            "gpu_load_shader",
            "gpu_dispatch",
            // Storage
            "store_open",
            "store_get",
            "store_set",
            "store_delete",
            "store_compact",
        ]
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        if method.starts_with("gpu_") {
            #[cfg(feature = "gpu")]
            {
                return self.handle_gpu(method, &args);
            }
            #[cfg(not(feature = "gpu"))]
            {
                Err(IfaError::Runtime("GPU disabled".into()))
            }
        } else if method.starts_with("store_") {
            #[cfg(feature = "persistence")]
            {
                return self.handle_storage(method, &args);
            }
            #[cfg(not(feature = "persistence"))]
            {
                Ok(IfaValue::Bool(true))
            }
        } else if method == "submit_task" {
            self.handle_cpu(method, &args)
        } else {
            self.handle_kernel(method, &args)
        }
    }
}
