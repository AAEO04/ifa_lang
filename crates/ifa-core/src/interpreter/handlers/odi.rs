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
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {

        let arg0 = args.get(0);
        let arg1 = args.get(1);

        match method {
            // Read file
            "ka" | "read" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(path) = val {
                         match std::fs::read_to_string(&**path) {
                            Ok(content) => Ok(IfaValue::str(content)),
                            Err(e) => Err(IfaError::Runtime(format!("Cannot read file: {}", e))),
                        }
                    } else {
                         Err(IfaError::Runtime("read requires file path".into()))
                    }
                } else {
                    Err(IfaError::Runtime("read requires file path".into()))
                }
            }

            // Write file
            "kọ" | "write" => {
                if let (Some(path_val), Some(content_val)) = (arg0, arg1) {
                    if let (IfaValue::Str(path), IfaValue::Str(content)) = (path_val, content_val) {
                        match std::fs::write(&**path, content.as_bytes()) {
                            Ok(_) => return Ok(IfaValue::bool(true)),
                            Err(e) => {
                                return Err(IfaError::Runtime(format!("Cannot write file: {}", e)));
                            }
                        }
                    } else {
                         return Err(IfaError::Runtime("write requires path and content".into()));
                    }
                }
                Err(IfaError::Runtime("write requires path and content".into()))
            }

            // Append to file
            "fikun" | "append" => {
                 if let (Some(path_val), Some(content_val)) = (arg0, arg1) {
                    if let (IfaValue::Str(path), IfaValue::Str(content)) = (path_val, content_val) {
                        use std::io::Write;
                        let mut file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&**path)
                            .map_err(|e| IfaError::Runtime(format!("Cannot open file: {}", e)))?;
                        file.write_all(content.as_bytes())
                            .map_err(|e| IfaError::Runtime(format!("Cannot append: {}", e)))?;
                        return Ok(IfaValue::bool(true));
                    }
                 }
                Err(IfaError::Runtime("append requires path and content".into()))
            }

            // Check if file exists
            "wa" | "exists" => {
                if let Some(val) = arg0 {
                    if let IfaValue::Str(path) = val {
                         return Ok(IfaValue::bool(PathBuf::from(&**path).exists()));
                    }
                }
                Err(IfaError::Runtime("exists requires path".into()))
            }

            // Delete file
            "pa" | "delete" | "remove" => {
               if let Some(val) = arg0 {
                    if let IfaValue::Str(path) = val {
                         match std::fs::remove_file(&**path) {
                            Ok(_) => return Ok(IfaValue::bool(true)),
                            Err(e) => return Err(IfaError::Runtime(format!("Cannot delete: {}", e))),
                        }
                    } else {
                        Err(IfaError::Runtime("delete requires path".into()))
                    }
                } else {
                    Err(IfaError::Runtime("delete requires path".into()))
                }
            }

            // List directory
            "ṣe_akojọ" | "list" | "ls" => {
               if let Some(val) = arg0 {
                    if let IfaValue::Str(path) = val {
                        match std::fs::read_dir(&**path) {
                            Ok(entries) => {
                                let files: Vec<IfaValue> = entries
                                    .filter_map(|e| e.ok())
                                    .map(|e| {
                                        IfaValue::str(e.file_name().to_string_lossy())
                                    })
                                    .collect();
                                return Ok(IfaValue::list(files));
                            }
                            Err(e) => return Err(IfaError::Runtime(format!("Cannot list: {}", e))),
                        }
                    } else {
                        Err(IfaError::Runtime("list requires directory path".into()))
                    }
                } else {
                     Err(IfaError::Runtime("list requires directory path".into()))
                }
            }

            // Create directory
            "ṣe_folda" | "mkdir" => {
                if let Some(IfaValue::Str(path)) = arg0 {
                    match std::fs::create_dir_all(&**path) {
                        Ok(_) => Ok(IfaValue::bool(true)),
                        Err(e) => Err(IfaError::Runtime(format!("Cannot mkdir: {}", e))),
                    }
                } else {
                    Err(IfaError::Runtime("mkdir requires path".into()))
                }
            }

            _ => Err(IfaError::Runtime(format!("Unknown Òdí method: {}", method))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "ka",
            "read",
            "kọ",
            "write",
            "fikun",
            "append",
            "wa",
            "exists",
            "pa",
            "delete",
            "remove",
            "ṣe_akojọ",
            "list",
            "ls",
            "ṣe_folda",
            "mkdir",
        ]
    }
}
