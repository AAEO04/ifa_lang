//! # Òdí Handler - Files/Database
//!
//! Handles file I/O operations.
//! Binary pattern: 1001

use std::path::PathBuf;

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Òdí (Files/Database) domain.
pub struct OdiHandler;

impl OduHandler for OdiHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Odi
    }
    
    fn call(
        &self, 
        method: &str, 
        args: Vec<IfaValue>, 
        _env: &mut Environment
    ) -> IfaResult<IfaValue> {
        match method {
            // Read file
            "ka" | "read" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    match std::fs::read_to_string(path) {
                        Ok(content) => Ok(IfaValue::Str(content)),
                        Err(e) => Err(IfaError::Runtime(format!("Cannot read file: {}", e))),
                    }
                } else {
                    Err(IfaError::Runtime("read requires file path".into()))
                }
            }
            
            // Write file
            "kọ" | "write" => {
                if args.len() >= 2 {
                    if let (IfaValue::Str(path), IfaValue::Str(content)) = (&args[0], &args[1]) {
                        match std::fs::write(path, content) {
                            Ok(_) => return Ok(IfaValue::Bool(true)),
                            Err(e) => return Err(IfaError::Runtime(format!("Cannot write file: {}", e))),
                        }
                    }
                }
                Err(IfaError::Runtime("write requires path and content".into()))
            }
            
            // Append to file
            "fikun" | "append" => {
                if args.len() >= 2 {
                    if let (IfaValue::Str(path), IfaValue::Str(content)) = (&args[0], &args[1]) {
                        use std::io::Write;
                        let mut file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(path)
                            .map_err(|e| IfaError::Runtime(format!("Cannot open file: {}", e)))?;
                        file.write_all(content.as_bytes())
                            .map_err(|e| IfaError::Runtime(format!("Cannot append: {}", e)))?;
                        return Ok(IfaValue::Bool(true));
                    }
                }
                Err(IfaError::Runtime("append requires path and content".into()))
            }
            
            // Check if file exists
            "wa" | "exists" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    return Ok(IfaValue::Bool(PathBuf::from(path).exists()));
                }
                Err(IfaError::Runtime("exists requires path".into()))
            }
            
            // Delete file
            "pa" | "delete" | "remove" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    match std::fs::remove_file(path) {
                        Ok(_) => return Ok(IfaValue::Bool(true)),
                        Err(e) => return Err(IfaError::Runtime(format!("Cannot delete: {}", e))),
                    }
                }
                Err(IfaError::Runtime("delete requires path".into()))
            }
            
            // List directory
            "ṣe_akojọ" | "list" | "ls" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    match std::fs::read_dir(path) {
                        Ok(entries) => {
                            let files: Vec<IfaValue> = entries
                                .filter_map(|e| e.ok())
                                .map(|e| IfaValue::Str(e.file_name().to_string_lossy().to_string()))
                                .collect();
                            return Ok(IfaValue::List(files));
                        }
                        Err(e) => return Err(IfaError::Runtime(format!("Cannot list: {}", e))),
                    }
                }
                Err(IfaError::Runtime("list requires directory path".into()))
            }
            
            // Create directory
            "ṣe_folda" | "mkdir" => {
                if let Some(IfaValue::Str(path)) = args.first() {
                    match std::fs::create_dir_all(path) {
                        Ok(_) => return Ok(IfaValue::Bool(true)),
                        Err(e) => return Err(IfaError::Runtime(format!("Cannot mkdir: {}", e))),
                    }
                }
                Err(IfaError::Runtime("mkdir requires path".into()))
            }
            
            _ => Err(IfaError::Runtime(format!(
                "Unknown Òdí method: {}",
                method
            ))),
        }
    }
    
    fn methods(&self) -> &'static [&'static str] {
        &["ka", "read", "kọ", "write", "fikun", "append", 
          "wa", "exists", "pa", "delete", "remove",
          "ṣe_akojọ", "list", "ls", "ṣe_folda", "mkdir"]
    }
}
