//! # Òtúrá Handler - Networking
//!
//! Handles HTTP and WebSocket operations.
//! Binary pattern: 1011
//!
//! When `network` feature is enabled, uses `ureq` for real HTTP requests.
//! Otherwise, returns simulated responses for development.

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Òtúrá (Networking) domain.
pub struct OturaHandler;

// Real HTTP implementation when network feature is enabled
#[cfg(feature = "network")]
fn http_get(url: &str) -> IfaResult<String> {
    match ureq::get(url).call() {
        Ok(response) => response
            .into_body()
            .read_to_string()
            .map_err(|e| IfaError::Runtime(format!("Failed to read response: {}", e))),
        Err(e) => Err(IfaError::Runtime(format!("HTTP GET failed: {}", e))),
    }
}

#[cfg(feature = "network")]
fn http_post(url: &str, body: &str) -> IfaResult<String> {
    match ureq::post(url)
        .header("Content-Type", "application/json")
        .send(body.as_bytes())
    {
        Ok(response) => response
            .into_body()
            .read_to_string()
            .map_err(|e| IfaError::Runtime(format!("Failed to read response: {}", e))),
        Err(e) => Err(IfaError::Runtime(format!("HTTP POST failed: {}", e))),
    }
}

// Fallback when network feature is not enabled
#[cfg(not(feature = "network"))]
fn http_get(url: &str) -> IfaResult<String> {
    Err(IfaError::Runtime(format!(
        "Network disabled. Enable 'network' feature to make real HTTP requests. URL: {}",
        url
    )))
}

#[cfg(not(feature = "network"))]
fn http_post(url: &str, _body: &str) -> IfaResult<String> {
    Err(IfaError::Runtime(format!(
        "Network disabled. Enable 'network' feature to make real HTTP requests. URL: {}",
        url
    )))
}

impl OduHandler for OturaHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Otura
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {
        match method {
            // HTTP GET - real or simulated based on feature
            "http_get" | "gba" | "get" => {
                if let Some(IfaValue::Str(url)) = args.first() {
                    let response = http_get(url)?;
                    Ok(IfaValue::Str(response))
                } else {
                    Err(IfaError::Runtime("http_get requires URL".into()))
                }
            }

            // HTTP POST - real or simulated based on feature
            "http_post" | "fi" | "post" => {
                if args.len() >= 2 {
                    if let (IfaValue::Str(url), body) = (&args[0], &args[1]) {
                        let body_str = match body {
                            IfaValue::Str(s) => s.clone(),
                            _ => serde_json::to_string(body)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?,
                        };
                        let response = http_post(url, &body_str)?;
                        return Ok(IfaValue::Str(response));
                    }
                }
                Err(IfaError::Runtime("http_post requires URL and body".into()))
            }

            // Fetch JSON - parse response as JSON
            "fetch_json" | "gba_json" => {
                if let Some(IfaValue::Str(url)) = args.first() {
                    let response = http_get(url)?;
                    // Parse JSON response
                    let json: serde_json::Value = serde_json::from_str(&response)
                        .map_err(|e| IfaError::Runtime(format!("JSON parse error: {}", e)))?;
                    // Convert to IfaValue::Map
                    fn json_to_ifa(v: serde_json::Value) -> IfaValue {
                        match v {
                            serde_json::Value::Null => IfaValue::Null,
                            serde_json::Value::Bool(b) => IfaValue::Bool(b),
                            serde_json::Value::Number(n) => {
                                if let Some(i) = n.as_i64() {
                                    IfaValue::Int(i)
                                } else {
                                    IfaValue::Float(n.as_f64().unwrap_or(0.0))
                                }
                            }
                            serde_json::Value::String(s) => IfaValue::Str(s),
                            serde_json::Value::Array(arr) => {
                                IfaValue::List(arr.into_iter().map(json_to_ifa).collect())
                            }
                            serde_json::Value::Object(obj) => IfaValue::Map(
                                obj.into_iter().map(|(k, v)| (k, json_to_ifa(v))).collect(),
                            ),
                        }
                    }
                    return Ok(json_to_ifa(json));
                }
                Err(IfaError::Runtime("fetch_json requires URL".into()))
            }

            // Start HTTP server (placeholder - would need tokio/axum)
            "serve" | "sin" | "listen" => {
                if let Some(IfaValue::Int(port)) = args.first() {
                    // Server functionality requires async runtime
                    return Err(IfaError::Runtime(format!(
                        "HTTP server on port {} requires async runtime. Use ifa-std backend stack.",
                        port
                    )));
                }
                Err(IfaError::Runtime("serve requires port number".into()))
            }

            // WebSocket connect (placeholder - would need tungstenite)
            "ws_connect" | "asopọ_ws" => {
                if let Some(IfaValue::Str(url)) = args.first() {
                    return Err(IfaError::Runtime(format!(
                        "WebSocket to {} requires websocket library. Use ifa-std backend stack.",
                        url
                    )));
                }
                Err(IfaError::Runtime("ws_connect requires URL".into()))
            }

            // URL encode
            "url_encode" | "koodu_url" => {
                if let Some(IfaValue::Str(text)) = args.first() {
                    let encoded: String = text
                        .chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
                                c.to_string()
                            } else {
                                format!("%{:02X}", c as u32)
                            }
                        })
                        .collect();
                    return Ok(IfaValue::Str(encoded));
                }
                Err(IfaError::Runtime("url_encode requires string".into()))
            }

            _ => Err(IfaError::Runtime(format!(
                "Unknown Òtúrá method: {}",
                method
            ))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "http_get",
            "gba",
            "get",
            "http_post",
            "fi",
            "post",
            "serve",
            "sin",
            "listen",
            "ws_connect",
            "asopọ_ws",
            "fetch_json",
            "gba_json",
            "url_encode",
            "koodu_url",
        ]
    }
}
