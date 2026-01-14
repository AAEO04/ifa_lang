use crate::Ofun;
use crate::config::SandboxConfig;
use eyre::{Result, eyre};
use std::path::Path;
use wasmtime::{
    Config, Engine, Linker, Module, PoolingAllocationConfig, Store, StoreLimits,
    StoreLimitsBuilder, InstanceAllocationStrategy,
};
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};

/// Wasmtime-based secure runtime with AOT, Pooling, and Ewo support
pub struct OmniBox {
    engine: Engine,
    config: SandboxConfig,
}

struct StoreState {
    wasi: WasiP1Ctx,
    limits: StoreLimits,
    config: SandboxConfig,
}

impl OmniBox {
    /// Initialize a new OmniBox with high-performance settings
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let mut wasm_config = Config::new();

        // 1. Enable Epoch Interruption (for Instruction Counting / Timeout)
        wasm_config.epoch_interruption(true);

        // 2. Configure Memory Limits
        // Treat max_wasm_stack as KB
        wasm_config.max_wasm_stack(config.limits.max_stack_depth * 1024);

        // 3. Enable Pooling Allocator (The "Linus Optimization")
        // This pre-allocates virtual memory to avoid `mmap` syscalls on every instantiation.
        let mut pooling_config = PoolingAllocationConfig::default();
        
        // Customize pooling based on our known limits
        // 128 concurrent instances should be plenty for a CLI tool
        pooling_config.total_core_instances(128); 
        
        // Ensure memory pages fit within our limits (default is often 6MB or 4GB depending on platform)
        // Here we align with typical Ifá embedded philosophy
        pooling_config.max_memory_size(10 * 1024 * 1024); // 10MB per instance max linear memory
        // Decrease table elements to save memory
        pooling_config.table_elements(10000);

        wasm_config.allocation_strategy(InstanceAllocationStrategy::Pooling(pooling_config));

        let engine = Engine::new(&wasm_config).map_err(|e| eyre!("Failed to init Wasmtime engine: {e}"))?;
        
        Ok(OmniBox { engine, config })
    }

    /// Compile Wasm source to AOT artifact (Serialized Machine Code)
    /// This is the "Install Time" operation.
    pub fn compile_artifact(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| eyre!("Compilation failed: {e}"))?;
        
        // Serialize to native machine code (.cwasm)
        module.serialize().map_err(|e| eyre!("Serialization failed: {e}"))
    }

    /// Load a pre-compiled artifact directly into memory
    /// This is the "Run Time" fast-path (Startup < 2ms).
    /// 
    /// # Security
    /// Artifacts MUST come from trusted sources (e.g., locally compiled).
    /// The artifact header is validated before deserialization.
    pub fn deserialize_artifact(&self, artifact_bytes: &[u8]) -> Result<Module> {
        // Validate artifact has minimum size (header)
        if artifact_bytes.len() < 8 {
            return Err(eyre!("Invalid artifact: too small (corrupted?)"));
        }
        
        // Validate wasmtime magic header (ELF or platform-specific)
        // Wasmtime artifacts start with platform ELF/Mach-O/PE headers
        #[cfg(target_os = "linux")]
        {
            if artifact_bytes.len() >= 4 && &artifact_bytes[0..4] != b"\x7fELF" {
                return Err(eyre!("Invalid artifact: not a valid ELF (Linux). Was this compiled on a different OS?"));
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if artifact_bytes.len() >= 2 && &artifact_bytes[0..2] != b"MZ" {
                return Err(eyre!("Invalid artifact: not a valid PE (Windows). Was this compiled on a different OS?"));
            }
        }
        
        // SAFETY: We validate the header above. The artifact should come from
        // our own compile_artifact() function with the same engine version.
        // For additional security, consider adding a hash/version check via Oja.
        unsafe {
            Module::deserialize(&self.engine, artifact_bytes)
                .map_err(|e| eyre!("Deserialization failed (engine mismatch?): {e}"))
        }
    }

    /// Run a Wasm module (source or pre-compiled)
    pub fn run_module(&self, module: &Module) -> Result<()> {
        let mut linker = Linker::new(&self.engine);

        // 2. Add WASI P1
        p1::add_to_linker_sync(&mut linker, |state: &mut StoreState| &mut state.wasi)
            .map_err(|e| eyre!("Failed to add WASI to linker: {e}"))?;

        // 3. Add Ewo Host Functions
        self.link_ewo_capabilities(&mut linker)?;

        // Build WASI context and store limits
        let wasi = self.build_wasi_context()?;
        let limits = self.build_store_limits();

        let mut store = Store::new(&self.engine, StoreState { wasi, limits, config: self.config.clone() });

        // Apply resource limits to store
        store.limiter(|state| &mut state.limits);

        // Set execution timeout via epoch deadline
        let timeout_epochs = self.config.limits.max_execution_time.as_secs();
        store.set_epoch_deadline(timeout_epochs);

        // Instantiate and run (Pooling Allocator makes this fast)
        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| eyre!("Failed to instantiate: {e}"))?;

        let start = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| eyre!("Failed to get _start function: {e}"))?;

        start
            .call(&mut store, ())
            .map_err(|e| eyre!("Execution failed: {e}"))?;

        Ok(())
    }

    /// Convenience: Load from file and run (Standard JIT path - Slow)
    pub fn run_wasm_file(&self, wasm_path: &Path) -> Result<()> {
        let module = Module::from_file(&self.engine, wasm_path)
            .map_err(|e| eyre!("Failed to load module: {e}"))?;
        self.run_module(&module)
    }

    /// Build WASI context from CapabilitySet (Ofun)
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

    /// Link Ewo (Capability) host functions to the WebAssembly module
    fn link_ewo_capabilities(&self, linker: &mut Linker<StoreState>) -> Result<()> {
        // ewo_can_read(path_ptr, path_len) -> i32
        linker.func_wrap("ewo", "can_read", |state: wasmtime::Caller<'_, StoreState>| {
            // Placeholder: For now just return if they have any read access
            if state.data().config.capabilities.all().iter().any(|c| matches!(c, Ofun::ReadFiles { .. })) {
                1
            } else {
                0
            }
        }).map_err(|e| eyre!("Failed to link ewo.can_read: {e}"))?;

        // ewo_can_write(path_ptr, path_len) -> i32
        linker.func_wrap("ewo", "can_write", |state: wasmtime::Caller<'_, StoreState>| {
            if state.data().config.capabilities.all().iter().any(|c| matches!(c, Ofun::WriteFiles { .. })) {
                1
            } else {
                0
            }
        }).map_err(|e| eyre!("Failed to link ewo.can_write: {e}"))?;
        
        // ewo_is_secure() -> i32
        linker.func_wrap("ewo", "is_secure", |state: wasmtime::Caller<'_, StoreState>| {
            if state.data().config.capabilities.all().is_empty() {
                0 // Wide open (unsafe)
            } else {
                1 // Some restrictions in place
            }
        }).map_err(|e| eyre!("Failed to link ewo.is_secure: {e}"))?;

        // ewo_can_network() -> i32 (Òtúrá)
        linker.func_wrap("ewo", "can_network", |state: wasmtime::Caller<'_, StoreState>| {
            if state.data().config.capabilities.check(&Ofun::Network { domains: vec![] }) {
                1
            } else {
                0
            }
        }).map_err(|e| eyre!("Failed to link ewo.can_network: {e}"))?;

        Ok(())
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
    fn test_omnibox_creation_with_pooling() {
        let config = SandboxConfig::new(SecurityProfile::Untrusted);
        let omnibox = OmniBox::new(config);
        assert!(omnibox.is_ok(), "Pooling allocator should initialize correctly");
    }

    #[test]
    fn test_aot_compilation_cycle() {
        let config = SandboxConfig::new(SecurityProfile::Untrusted);
        let omnibox = OmniBox::new(config).unwrap();
        
        // Minimal valid WASM (empty module)
        let wasm_bytes = wat::parse_str("(module)").unwrap();
        
        // 1. Compile
        let artifact = omnibox.compile_artifact(&wasm_bytes).expect("Compilation failed");
        assert!(!artifact.is_empty(), "Artifact should contain bytes");
        
        // 2. Deserialize
        let module = omnibox.deserialize_artifact(&artifact).expect("Deserialization failed");
        
        // 3. Verify it works
        // (Just ensure module is valid, running requires _start which this empty module lacks)
        assert!(module.exports().count() == 0);
    }
}
