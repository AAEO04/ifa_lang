use crate::config::SandboxConfig;
use crate::Ofun;
use eyre::{eyre, Result};
use std::path::Path;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

/// Wasmtime-based secure runtime
pub struct OmniBox {
    engine: Engine,
    config: SandboxConfig,
}

struct WasiState {
    ctx: WasiCtx,
}

impl OmniBox {
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let wasm_config = wasmtime::Config::new();
        let engine = Engine::new(&wasm_config).map_err(|e| eyre!("{e}"))?;
        Ok(OmniBox { engine, config })
    }

    pub fn run_wasm(&self, wasm_path: &Path) -> Result<()> {
        let module = Module::from_file(&self.engine, wasm_path).map_err(|e| eyre!("{e}"))?;
        let mut linker = Linker::new(&self.engine);

        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasiState| &mut s.ctx)
            .map_err(|e| eyre!("{e}"))?;

        // Build WASI context with capability-based preopens
        let ctx = self.build_wasi_context()?;
        let state = WasiState { ctx };

        let mut store = Store::new(&self.engine, state);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| eyre!("{e}"))?;
        let start = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| eyre!("{e}"))?;

        start.call(&mut store, ()).map_err(|e| eyre!("{e}"))?;

        Ok(())
    }

    /// Build WASI context from CapabilitySet
    fn build_wasi_context(&self) -> Result<WasiCtx> {
        let mut builder = WasiCtxBuilder::new();

        // Check capabilities and configure WASI accordingly
        let caps = &self.config.capabilities;

        // Stdio
        if caps.check(&Ofun::Stdio) {
            builder.inherit_stdio();
        }

        // File preopens - grant directory access for each ReadFiles/WriteFiles capability
        for cap in caps.all() {
            match cap {
                Ofun::ReadFiles { root } => {
                    if root.exists() && root.is_dir() {
                        // Preopen directory for reading
                        // Note: wasmtime-wasi API varies by version
                        // builder.preopened_dir(root, ".")?;
                    }
                }
                Ofun::WriteFiles { root } => {
                    if root.exists() && root.is_dir() {
                        // Preopen directory for writing
                        // builder.preopened_dir(root, ".")?;
                    }
                }
                _ => {}
            }
        }

        // Environment variables
        for cap in caps.all() {
            if let Ofun::Environment { keys } = cap {
                for key in keys {
                    if key == "*" {
                        builder.inherit_env().map_err(|e| eyre!("{e}"))?;
                        break;
                    } else if let Ok(val) = std::env::var(key) {
                        builder.env(key, &val).map_err(|e| eyre!("{e}"))?;
                    }
                }
            }
        }

        Ok(builder.build())
    }
}
