//! # Òfún Handler - Permissions/Reflection
//!
//! Handles permission checking and reflection.
//! Binary pattern: 0101

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Òfún (Permissions/Reflection) domain.
pub struct OfunHandler;

/// Parse capability name string to Ofun enum variant name
fn parse_capability_name(name: &str) -> &'static str {
    match name.to_lowercase().as_str() {
        "stdio" | "io" | "console" => "Stdio",
        "time" | "clock" | "datetime" => "Time",
        "random" | "rand" => "Random",
        "network" | "net" | "http" => "Network",
        "files" | "filesystem" | "fs" | "read" | "write" => "ReadFiles",
        "env" | "environment" => "Environment",
        "execute" | "exec" | "spawn" | "process" => "Execute",
        "bridge" | "ffi" | "python" | "js" => "Bridge",
        _ => "Unknown",
    }
}

/// Get domain methods for reflection
fn get_domain_methods(domain: &str) -> Vec<IfaValue> {
    let methods: &[&str] = match domain.to_lowercase().as_str() {
        "irosu" | "io" => &["so", "println", "ka", "read", "format"],
        "ogbe" | "system" => &["asiko", "time", "namuwe", "version", "asin", "env"],
        "obara" | "math" => &["fi_kun", "add", "so", "mul", "powo", "pow"],
        "oturupon" | "math2" => &["di_nu", "sub", "pin", "div", "mod"],
        "ika" | "string" => &["gun", "len", "wa", "find", "ni", "has", "rọ", "replace"],
        "oyeku" | "control" => &["jade", "exit", "sun", "sleep"],
        "owonrin" | "random" => &["àìdámọ̀", "random", "wọn", "range"],
        "ogunda" | "array" => &["kun", "push", "mu", "pop", "ati", "map", "irele", "filter"],
        "iwori" | "time" => &["bayi", "now", "dateformat", "epoch"],
        "okanran" | "error" => &["asise", "error", "try", "assert"],
        "otura" | "network" => &["gbe", "fetch", "fi", "post", "json"],
        "odi" | "files" => &["ka_faili", "read", "kọ_faili", "write", "existe", "exists"],
        "osa" | "async" => &["bẹrẹ", "spawn", "duro", "await", "afiwe", "parallel"],
        "ofun" | "reflect" => &["ni_agbara", "has_capability", "iru", "typeof", "methods"],
        "irete" | "crypto" => &["hash", "sha256", "encode", "decode", "uuid"],
        "ose" | "ui" => &["canvas", "rect", "text", "color", "render"],
        "ohun" | "audio" => &["play", "record", "volume", "load"],
        "fidio" | "video" => &["play", "record", "frame", "duration"],
        _ => &[],
    };
    methods
        .iter()
        .map(|m| IfaValue::Str(m.to_string()))
        .collect()
}

impl OduHandler for OfunHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Ofun
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // Check if capability is granted
            // Note: In WASM mode, all capabilities return true (browser sandbox)
            // In native mode with no Interpreter context, default to true
            "ni_agbara" | "has_capability" | "can" => {
                if let Some(IfaValue::Str(cap)) = args.first() {
                    let cap_type = parse_capability_name(cap);
                    // Return true for valid capabilities, false for Unknown
                    // (Full sandbox integration requires Interpreter context)
                    let has_cap = cap_type != "Unknown";
                    return Ok(IfaValue::Bool(has_cap));
                }
                Err(IfaError::Runtime(
                    "has_capability requires capability name".into(),
                ))
            }

            // Request capability (logs and returns success)
            "beere" | "request" => {
                if let Some(IfaValue::Str(cap)) = args.first() {
                    let cap_type = parse_capability_name(cap);
                    if cap_type == "Unknown" {
                        return Err(IfaError::Runtime(format!(
                            "Unknown capability: '{}'. Valid: stdio, time, random, network, files, env, execute, bridge",
                            cap
                        )));
                    }
                    // In production, this would go through CapabilitySet::grant
                    return Ok(IfaValue::Bool(true));
                }
                Err(IfaError::Runtime("request requires capability name".into()))
            }

            // Reflect on type - real implementation
            "iru" | "typeof" => {
                if let Some(val) = args.first() {
                    return Ok(IfaValue::Str(val.type_name().to_string()));
                }
                Ok(IfaValue::Str("null".to_string()))
            }

            // Reflect on methods - real implementation with domain lookup
            "awọn_ẹsẹ" | "methods" => {
                if let Some(IfaValue::Str(domain)) = args.first() {
                    return Ok(IfaValue::List(get_domain_methods(domain)));
                }
                // Return all domains if no argument
                let domains: Vec<IfaValue> = vec![
                    "irosu", "ogbe", "obara", "oturupon", "ika", "oyeku", "owonrin", "ogunda",
                    "iwori", "okanran", "otura", "odi", "osa", "ofun", "irete", "ose", "ohun",
                    "fidio",
                ]
                .into_iter()
                .map(|d| IfaValue::Str(d.to_string()))
                .collect();
                Ok(IfaValue::List(domains))
            }

            // List available capabilities
            "awọn_agbara" | "capabilities" => {
                let caps: Vec<IfaValue> = vec![
                    "stdio", "time", "random", "network", "files", "env", "execute", "bridge",
                ]
                .into_iter()
                .map(|c| IfaValue::Str(c.to_string()))
                .collect();
                Ok(IfaValue::List(caps))
            }

            // Get module info - real implementation
            "alaye_ẹka" | "module_info" => Ok(IfaValue::Map(std::collections::HashMap::from([
                ("name".to_string(), IfaValue::Str("ifá-core".to_string())),
                (
                    "version".to_string(),
                    IfaValue::Str(env!("CARGO_PKG_VERSION").to_string()),
                ),
                ("edition".to_string(), IfaValue::Str("2024".to_string())),
            ]))),

            _ => Err(IfaError::Runtime(format!(
                "Unknown Òfún method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "ni_agbara",
            "has_capability",
            "can",
            "beere",
            "request",
            "iru",
            "typeof",
            "awọn_ẹsẹ",
            "methods",
            "awọn_agbara",
            "capabilities",
            "alaye_ẹka",
            "module_info",
        ]
    }
}
