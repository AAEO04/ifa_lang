use crate::Ofun;
use crate::config::SandboxConfig;
use eyre::{Result, eyre};
use std::path::Path;
use wasmtime::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

/// Wasmtime-based secure runtime
pub struct OmniBox {
    engine: Engine,
    config: SandboxConfig,
}

struct StoreState {
    wasi: WasiP1Ctx,
    limits: StoreLimits,
}

impl OmniBox {
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let mut wasm_config = Config::new();

        // Enable epoch interruption for timeout enforcement
        wasm_config.epoch_interruption(true);

        // Configure memory limits
        wasm_config.max_wasm_stack(config.limits.max_stack_depth * 1024);

        let engine = Engine::new(&wasm_config).map_err(|e| eyre!("{e}"))?;
        Ok(OmniBox { engine, config })
    }

    pub fn run_wasm(&self, wasm_path: &Path) -> Result<()> {
        let module = Module::from_file(&self.engine, wasm_path)
            .map_err(|e| eyre!("Failed to load module: {e}"))?;

        let mut linker = Linker::new(&self.engine);

        // Add WASI P1 to linker with closure to get WasiP1Ctx
        p1::add_to_linker_sync(&mut linker, |state: &mut StoreState| &mut state.wasi)
            .map_err(|e| eyre!("Failed to add WASI to linker: {e}"))?;

        // Build WASI context and store limits
        let wasi = self.build_wasi_context()?;
        let limits = self.build_store_limits();

        let mut store = Store::new(&self.engine, StoreState { wasi, limits });

        // Apply resource limits to store
        store.limiter(|state| &mut state.limits);

        // Set execution timeout via epoch deadline
        let timeout_epochs = self.config.limits.max_execution_time.as_secs();
        store.set_epoch_deadline(timeout_epochs);

        // Instantiate and run
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| eyre!("Failed to instantiate: {e}"))?;

        let start = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| eyre!("Failed to get _start function: {e}"))?;

        start
            .call(&mut store, ())
            .map_err(|e| eyre!("Execution failed: {e}"))?;

        Ok(())
    }

    /// Build WASI context from CapabilitySet
    fn build_wasi_context(&self) -> Result<WasiP1Ctx> {
        let mut builder = WasiCtxBuilder::new();

        let caps = &self.config.capabilities;

        // Configure stdio based on capabilities
        if caps.check(&Ofun::Stdio) {
            builder.inherit_stdio();
        }

        // Configure file system access
        for cap in caps.all() {
            match cap {
                Ofun::ReadFiles { root } => {
                    if root.exists() && root.is_dir() {
                        builder
                            .preopened_dir(root, "/", DirPerms::READ, FilePerms::READ)
                            .map_err(|e| {
                                eyre!("Failed to preopen read directory {}: {e}", root.display())
                            })?;
                    }
                }
                Ofun::WriteFiles { root } => {
                    if root.exists() && root.is_dir() {
                        builder
                            .preopened_dir(root, "/", DirPerms::all(), FilePerms::all())
                            .map_err(|e| {
                                eyre!("Failed to preopen write directory {}: {e}", root.display())
                            })?;
                    }
                }
                _ => {}
            }
        }

        // Configure environment variables
        for cap in caps.all() {
            if let Ofun::Environment { keys } = cap {
                for key in keys {
                    if key == "*" {
                        builder.inherit_env();
                        break;
                    } else if let Ok(val) = std::env::var(key) {
                        builder.env(key, &val);
                    }
                }
            }
        }

        // Configure arguments (empty by default)
        builder.args(&[] as &[&str]);

        // Build Preview 1 context
        Ok(builder.build_p1())
    }

    /// Build store limits from config
    fn build_store_limits(&self) -> StoreLimits {
        StoreLimitsBuilder::new()
            .memory_size(self.config.limits.max_memory_bytes)
            .table_elements(10000)
            .instances(1)
            .tables(10)
            .memories(1)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SandboxConfig, SecurityProfile};

    #[test]
    fn test_omnibox_creation() {
        let config = SandboxConfig::new(SecurityProfile::Untrusted);
        let omnibox = OmniBox::new(config);
        assert!(omnibox.is_ok());
    }

    #[test]
    fn test_wasi_context_with_stdio() {
        let config = SandboxConfig::new(SecurityProfile::Standard).with_capability(Ofun::Stdio);
        let omnibox = OmniBox::new(config).unwrap();
        let ctx = omnibox.build_wasi_context();
        assert!(ctx.is_ok());
    }

    #[test]
    fn test_wasi_context_with_env() {
        let config =
            SandboxConfig::new(SecurityProfile::Standard).with_capability(Ofun::Environment {
                keys: vec!["PATH".to_string()],
            });
        let omnibox = OmniBox::new(config).unwrap();
        let ctx = omnibox.build_wasi_context();
        assert!(ctx.is_ok());
    }

    #[test]
    fn test_store_limits() {
        let config = SandboxConfig::new(SecurityProfile::Untrusted);
        let omnibox = OmniBox::new(config).unwrap();
        let limits = omnibox.build_store_limits();
        drop(limits);
    }
}
