//! # Òdí Handler - Files/Database
//!
//! Handles file I/O operations by delegating to ifa-std::odu::Odi.
//! Binary pattern: 1001

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;
use super::{EnvRef, OduHandler};

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
        _env: &EnvRef,
        _output: &mut Vec<String>,
        capabilities: &ifa_types::capability::CapabilitySet,
    ) -> IfaResult<IfaValue> {
        #[cfg(feature = "std")]
        {
            let esu = ifa_std::esu::Esu::new(capabilities.clone());
            let odi = ifa_std::odu::odi::Odi::new(esu);

            let arg0 = args.get(0);
            let arg1 = args.get(1);

            match method {
                "ka" | "read" => {
                    if let Some(IfaValue::Str(path)) = arg0 {
                        let content = odi.ka(path)?;
                        Ok(IfaValue::str(content))
                    } else {
                        Err(IfaError::Runtime("read requires file path".into()))
                    }
                }
                "ko" | "write" => {
                    if let (Some(IfaValue::Str(path)), Some(IfaValue::Str(content))) = (arg0, arg1) {
                        odi.ko(path, content)?;
                        Ok(IfaValue::bool(true))
                    } else {
                        Err(IfaError::Runtime("write requires path and content".into()))
                    }
                }
                "fikun" | "append" => {
                    if let (Some(IfaValue::Str(path)), Some(IfaValue::Str(content))) = (arg0, arg1) {
                        odi.fi(path, content)?;
                        Ok(IfaValue::bool(true))
                    } else {
                        Err(IfaError::Runtime("append requires path and content".into()))
                    }
                }
                "wa" | "exists" => {
                    if let Some(IfaValue::Str(path)) = arg0 {
                        Ok(IfaValue::bool(odi.wa(path)))
                    } else {
                        Err(IfaError::Runtime("exists requires path".into()))
                    }
                }
                "pa" | "delete" | "remove" => {
                    if let Some(IfaValue::Str(path)) = arg0 {
                        odi.pa_faili(path)?;
                        Ok(IfaValue::bool(true))
                    } else {
                        Err(IfaError::Runtime("delete requires path".into()))
                    }
                }
                "ṣe_akojọ" | "list" | "ls" => {
                    if let Some(IfaValue::Str(path)) = arg0 {
                        let files = odi.akojo(path)?;
                        let list = files.into_iter().map(IfaValue::str).collect();
                        Ok(IfaValue::list(list))
                    } else {
                        Err(IfaError::Runtime("list requires directory path".into()))
                    }
                }
                "ṣe_folda" | "mkdir" => {
                    if let Some(IfaValue::Str(path)) = arg0 {
                        odi.seda_apoti(path)?;
                        Ok(IfaValue::bool(true))
                    } else {
                        Err(IfaError::Runtime("mkdir requires path".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!("Unknown Òdí method: {}", method))),
            }
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = args;
            let _ = capabilities;
            Err(IfaError::Runtime("std library not enabled".into()))
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
