use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ifa_bytecode::Bytecode;
use ifa_types::{IfaError, IfaResult};

#[cfg(feature = "compiler")]
use ifa_compiler::Compiler;
#[cfg(feature = "compiler")]
use ifa_parser::parse;
#[cfg(feature = "compiler")]
use ifa_types::ast::Program;

use crate::module_resolver::ImportGuard;

#[derive(Debug, Clone)]
pub struct CachedModule {
    pub hash: u64,
    pub bytecode: Bytecode,
    pub exports: Vec<String>,
}

#[derive(Default)]
pub struct ModuleLoader {
    pub cache: HashMap<PathBuf, CachedModule>,
    pub import_guard: ImportGuard,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self, path: &Path) {
        self.cache.remove(path);
    }

    #[cfg(feature = "compiler")]
    pub fn load_from_source(
        &mut self,
        path: &Path,
        source: &str,
    ) -> IfaResult<CachedModule> {
        let module_key = path.to_string_lossy().to_string();
        self.import_guard.enter(&module_key)?;

        let result = (|| -> IfaResult<CachedModule> {
            let hash = crate::vm::IfaVM::hash_source(source);
            if let Some(cached) = self.cache.get(path) {
                if cached.hash == hash {
                    return Ok(cached.clone());
                }
            }

            let program = parse(source).map_err(|e| {
                IfaError::Runtime(format!("Parse error in module '{}': {}", module_key, e))
            })?;
            
            let mut exports = Vec::new();
            for stmt in &program.statements {
                if let ifa_types::ast::Statement::Export { name, .. } = stmt {
                    exports.push(name.clone());
                } else if let ifa_types::ast::Statement::ExportDefault { .. } = stmt {
                    exports.push("default".to_string());
                }
            }

            let compiler = Compiler::new(path.to_string_lossy().as_ref());
            let bytecode = compiler.compile(&program)?;

            let cached = CachedModule {
                hash,
                bytecode,
                exports,
            };

            self.cache.insert(path.to_path_buf(), cached.clone());
            Ok(cached)
        })();

        self.import_guard.exit(&module_key);
        result
    }

    #[cfg(not(feature = "compiler"))]
    pub fn load_from_source(&mut self, _path: &Path, _source: &str) -> IfaResult<CachedModule> {
        Err(IfaError::Runtime(
            "Compiler feature disabled; cannot load source".to_string(),
        ))
    }
}
