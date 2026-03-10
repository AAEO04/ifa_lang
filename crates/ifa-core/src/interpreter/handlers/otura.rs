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

        let arg0 = args.first();
        let arg1 = args.get(1);

        match method {
            // HTTP GET
            "http_get" | "gba" | "get" => {
                if let Some(IfaValue::Str(url)) = arg0 {
                        let response = http_get(url)?;
                        Ok(IfaValue::str(response))
                    } else {
                        Err(IfaError::Runtime("http_get requires URL".into()))
                    }
            }

            // HTTP POST
            "http_post" | "fi" | "post" => {
                if let (Some(IfaValue::Str(url)), Some(body_val)) = (arg0, arg1) {
                         let body_str = match body_val {
                             IfaValue::Str(s) => s.to_string(),
                             // Use serde_json to stringify other types
                             _ => serde_json::to_string(body_val)
                                 .map_err(|e| IfaError::Runtime(e.to_string()))?,
                         };
                        let response = http_post(url, &body_str)?;
                        return Ok(IfaValue::str(response));
                }
                Err(IfaError::Runtime("http_post requires URL and body".into()))
            }

            // Fetch JSON
            "fetch_json" | "gba_json" => {
                 if let Some(IfaValue::Str(url)) = arg0 {
                         let response = http_get(url)?;
                        // Parse JSON response
                        let json: serde_json::Value = serde_json::from_str(&response)
                            .map_err(|e| IfaError::Runtime(format!("JSON parse error: {}", e)))?;
                        
                        fn json_to_ifa(v: serde_json::Value) -> IfaValue {
                            match v {
                                serde_json::Value::Null => IfaValue::null(),
                                serde_json::Value::Bool(b) => IfaValue::bool(b),
                                serde_json::Value::Number(n) => {
                                    if let Some(i) = n.as_i64() {
                                        IfaValue::int(i)
                                    } else {
                                        IfaValue::float(n.as_f64().unwrap_or(0.0))
                                    }
                                }
                                serde_json::Value::String(s) => IfaValue::str(s),
                                serde_json::Value::Array(arr) => {
                                    IfaValue::list(arr.into_iter().map(json_to_ifa).collect())
                                }
                                serde_json::Value::Object(obj) => IfaValue::map(
                                    obj.into_iter()
                                        .map(|(k, v)| (k, json_to_ifa(v)))
                                        .collect(),
                                ),
                            }
                        }
                        return Ok(json_to_ifa(json));
                    }
                 Err(IfaError::Runtime("fetch_json requires URL".into()))
            }

            // Start HTTP server
            "serve" | "sin" | "listen" => {
                 if let Some(IfaValue::Int(port)) = arg0 {
                         return Err(IfaError::Runtime(format!(
                            "HTTP server on port {} requires async runtime. Use ifa-std backend stack.",
                            port
                        )));
                    }
                Err(IfaError::Runtime("serve requires port number".into()))
            }

            // WebSocket connect
            "ws_connect" | "asopọ_ws" => {
                 if let Some(IfaValue::Str(url)) = arg0 {
                         return Err(IfaError::Runtime(format!(
                            "WebSocket to {} requires websocket library. Use ifa-std backend stack.",
                            url
                        )));
                    }
                Err(IfaError::Runtime("ws_connect requires URL".into()))
            }

            // URL encode
            "url_encode" | "koodu_url" => {
                 if let Some(IfaValue::Str(text)) = arg0 {
                        let encoded: String = text
                            .chars()
                            .map(|c: char| {
                                if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
                                    c.to_string()
                                } else {
                                    format!("%{:02X}", c as u32)
                                }
                            })
                            .collect();
                        return Ok(IfaValue::str(encoded));
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
