//! # StdRegistry - VM OduRegistry Implementation
//!
//! Bridges the ifa-std domain structs to the VM's OduRegistry trait,
//! enabling `CallOdu` opcodes to dispatch to the standard library.

use ifa_core::IfaValue;
use ifa_core::error::{IfaError, IfaResult};
use ifa_core::native::{OduRegistry, VmContext};

use crate::irosu::Irosu;
use crate::odi::Odi;
use crate::sandbox_shim::CapabilitySet;

/// Standard library registry for the bytecode VM.
pub struct StdRegistry {
    irosu: Irosu,
    odi: Odi,
}

impl StdRegistry {
    pub fn new() -> Self {
        let caps = CapabilitySet::new();
        Self {
            irosu: Irosu::new(caps.clone()),
            odi: Odi::new(caps),
        }
    }

    pub fn set_capabilities(&mut self, caps: CapabilitySet) {
        self.irosu = Irosu::new(caps.clone());
        self.odi = Odi::new(caps);
    }
}

impl Default for StdRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OduRegistry for StdRegistry {
    fn call(
        &self,
        domain_id: u8,
        method_name: &str,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        match domain_id {
            0 => dispatch_ogbe(method_name, args),
            1 => dispatch_oyeku(method_name, args),
            2 => dispatch_iwori(method_name, args),
            3 => self.dispatch_odi(method_name, args),
            4 => self.dispatch_irosu(method_name, args),
            5 => dispatch_owonrin(method_name, args),
            6 => dispatch_obara(method_name, args),
            7 => dispatch_okanran(method_name, args),
            8 => dispatch_ogunda(method_name, args),
            9 => dispatch_osa(method_name, args, ctx),
            10 => dispatch_ika(method_name, args),
            11 => dispatch_oturupon(method_name, args),
            14 => dispatch_ose(method_name, args),
            15 => dispatch_ofun(method_name, args),
            _ => Err(IfaError::Custom(format!(
                "Unknown Odù domain ID: {}",
                domain_id
            ))),
        }
    }

    fn import(&self, path: &str) -> IfaResult<IfaValue> {
        let key = path.replace('\\', "/");
        let domain = key
            .strip_prefix("std.")
            .or_else(|| key.strip_prefix("std/"))
            .unwrap_or(&key);
        let name = domain.split('.').last().unwrap_or(domain);
        let id = match name.to_lowercase().as_str() {
            "ogbe" => 0,
            "oyeku" => 1,
            "iwori" => 2,
            "odi" => 3,
            "irosu" => 4,
            "owonrin" => 5,
            "obara" => 6,
            "okanran" => 7,
            "ogunda" => 8,
            "osa" => 9,
            "ika" => 10,
            "oturupon" => 11,
            "otura" => 12,
            "irete" => 13,
            "ose" => 14,
            "ofun" => 15,
            "coop" => 16,
            "opele" => 17,
            "cpu" => 18,
            "gpu" => 19,
            "storage" => 20,
            "backend" => 21,
            "frontend" => 22,
            "crypto" => 23,
            "ml" => 24,
            "gamedev" => 25,
            "iot" => 26,
            "ohun" => 27,
            "fidio" => 28,
            "sys" => 29,
            _ => return Err(IfaError::Custom(format!("Unknown std module: {}", name))),
        };
        Ok(IfaValue::str(format!("__odu_mod__:{id}")))
    }
}

impl StdRegistry {
    fn dispatch_irosu(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        match method {
            "fo" | "println" => {
                if let Some(val) = args.first() {
                    self.irosu.fo(val);
                }
                Ok(IfaValue::null())
            }
            "so" | "print" => {
                if let Some(val) = args.first() {
                    self.irosu.so(val);
                }
                Ok(IfaValue::null())
            }
            "gbo" | "listen" => {
                let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::str(self.irosu.gbo(&prompt)))
            }
            "gbo_nomba" => {
                let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::int(self.irosu.gbo_nomba(&prompt)))
            }
            "mo" | "clear" => {
                self.irosu.mo();
                Ok(IfaValue::null())
            }
            "san" | "flush" => {
                self.irosu.san();
                Ok(IfaValue::null())
            }
            "kigbe" | "error" => {
                let text = args.first().map(|v| v.to_string()).unwrap_or_default();
                self.irosu.kigbe(&text);
                Ok(IfaValue::null())
            }
            _ => Err(IfaError::Custom(format!(
                "Irosu: unknown method '{}'",
                method
            ))),
        }
    }

    fn dispatch_odi(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        match method {
            "ka" | "read" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                self.odi.ka(&path).map(IfaValue::str)
            }
            "ko" | "write" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                let content = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                self.odi.ko(&path, &content).map(|_| IfaValue::null())
            }
            "wa" | "exists" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::bool(self.odi.wa(&path)))
            }
            _ => Err(IfaError::Custom(format!(
                "Odi: unknown method '{}'",
                method
            ))),
        }
    }
}

// Stateless dispatchers (no struct instance needed)

fn dispatch_ika(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "gigun" | "len" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::int(s.chars().count() as i64))
        }
        "ge" | "slice" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let start = args
                .get(1)
                .and_then(|v| {
                    if let IfaValue::Int(i) = v {
                        Some(*i as usize)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            let end = args
                .get(2)
                .and_then(|v| {
                    if let IfaValue::Int(i) = v {
                        Some(*i as usize)
                    } else {
                        None
                    }
                })
                .unwrap_or(s.chars().count());
            let result: String = s
                .chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .collect();
            Ok(IfaValue::str(result))
        }
        "so" | "concat" => {
            let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            Ok(IfaValue::str(parts.join("")))
        }
        "oruko_html" | "html_title" => {
            let raw = args.first().map(|v| v.to_string()).unwrap_or_default();
            let title = if let Some(start) = raw.find("<title>") {
                if let Some(end) = raw[start..].find("</title>") {
                    raw[start + 7..start + end].to_string()
                } else {
                    "Untitled".into()
                }
            } else {
                "Untitled".into()
            };
            Ok(IfaValue::str(title))
        }
        "tumo_html" | "strip_html" => {
            let raw = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut result = String::new();
            let mut in_tag = false;
            for ch in raw.chars() {
                if ch == '<' {
                    in_tag = true;
                    continue;
                }
                if ch == '>' {
                    in_tag = false;
                    continue;
                }
                if !in_tag {
                    result.push(ch);
                }
            }
            Ok(IfaValue::str(result))
        }
        _ => Err(IfaError::Custom(format!(
            "Ika: unknown method '{}'",
            method
        ))),
    }
}

fn extract_num(v: &IfaValue) -> f64 {
    match v {
        IfaValue::Int(i) => *i as f64,
        IfaValue::Float(f) => *f,
        _ => 0.0,
    }
}

fn dispatch_obara(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    let a = args.first().map(extract_num).unwrap_or(0.0);
    let b = args.get(1).map(extract_num).unwrap_or(0.0);
    match method {
        "fikun" | "add" | "plus" => Ok(IfaValue::float(a + b)),
        "isodipupo" | "mul" | "times" => Ok(IfaValue::float(a * b)),
        "agbara" | "pow" => Ok(IfaValue::float(a.powf(b))),
        "gbongbo" | "sqrt" => Ok(IfaValue::float(a.sqrt())),
        _ => Err(IfaError::Custom(format!(
            "Obara: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_osa(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    match method {
        "ise" | "spawn" | "sa" | "bẹrẹ" => {
            let task = args
                .get(0)
                .cloned()
                .ok_or_else(|| IfaError::ArgumentError("Osa.ise expects a task".into()))?;
            let task_args = args
                .get(1)
                .and_then(|v| {
                    if let IfaValue::List(list) = v {
                        Some(list.to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            ctx.spawn_task(task, task_args)
        }
        "sun" | "sleep" => {
            if let Some(IfaValue::Int(ms)) = args.get(0) {
                std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                Ok(IfaValue::future_ready(IfaValue::null()))
            } else {
                Err(IfaError::ArgumentError(
                    "Osa.sun expects milliseconds".into(),
                ))
            }
        }
        "gbogbo" | "all" => {
            if let Some(IfaValue::List(list)) = args.get(0) {
                let mut results = Vec::new();
                for item in list.iter() {
                    match item {
                        IfaValue::Future(cell) => {
                            let result = ctx.await_future(cell)?;
                            results.push(result);
                        }
                        other => {
                            results.push(other.clone());
                        }
                    }
                }
                Ok(IfaValue::list(results))
            } else {
                Err(IfaError::ArgumentError(
                    "Osa.gbogbo expects list of futures".into(),
                ))
            }
        }
        _ => Err(IfaError::Custom(format!(
            "Osa: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_oturupon(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    let a = args.first().map(extract_num).unwrap_or(0.0);
    let b = args.get(1).map(extract_num).unwrap_or(0.0);
    match method {
        "yokuro" | "sub" | "minus" => Ok(IfaValue::float(a - b)),
        "pipin" | "div" | "divide" => {
            if b == 0.0 {
                return Err(IfaError::Custom("Division by zero".into()));
            }
            Ok(IfaValue::float(a / b))
        }
        _ => Err(IfaError::Custom(format!(
            "Oturupon: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_owonrin(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "pese" | "random" => {
            let min = args
                .first()
                .and_then(|v| {
                    if let IfaValue::Int(i) = v {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            let max = args
                .get(1)
                .and_then(|v| {
                    if let IfaValue::Int(i) = v {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .unwrap_or(100);
            // Simple time-based random (matches existing Owonrin behavior)
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64;
            let range = (max - min + 1).max(1);
            Ok(IfaValue::int(min + (seed.abs() % range)))
        }
        _ => Err(IfaError::Custom(format!(
            "Owonrin: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_ogunda(method: &str, mut args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "iwọn" | "len" | "count" | "apapo" => {
            if let Some(IfaValue::List(list)) = args.first() {
                Ok(IfaValue::int(list.len() as i64))
            } else {
                Ok(IfaValue::int(0))
            }
        }
        "fi" | "push" | "append" => {
            if args.len() < 2 {
                return Err(IfaError::ArgumentError("push/fi expects (list, item)".into()));
            }
            let val = args[1].clone();
            if let IfaValue::List(ref mut list_arc) = args[0] {
                let vec = std::sync::Arc::make_mut(list_arc);
                vec.push(val);
                Ok(IfaValue::Null)
            } else {
                Err(IfaError::TypeError { expected: "List".into(), got: args[0].type_name().into() })
            }
        }
        "mu" | "pop" => {
            if let IfaValue::List(ref mut list_arc) = args[0] {
                let vec = std::sync::Arc::make_mut(list_arc);
                Ok(vec.pop().unwrap_or(IfaValue::Null))
            } else {
                Err(IfaError::TypeError { expected: "List".into(), got: args[0].type_name().into() })
            }
        }
        _ => Err(IfaError::Custom(format!(
            "Ogunda: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_ogbe(method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "bere" | "version" => Ok(IfaValue::str("Ifá-Lang v1.2.2")),
        _ => Err(IfaError::Custom(format!(
            "Ogbe: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_oyeku(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "jade" | "exit" | "quit" | "halt" => {
            let code = args
                .first()
                .and_then(|v| {
                    if let IfaValue::Int(i) = v {
                        Some(*i as i32)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            std::process::exit(code);
        }
        "sun" | "sleep" | "wait" => {
            let ms = args
                .first()
                .and_then(|v| {
                    if let IfaValue::Int(i) = v {
                        Some(*i as u64)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(IfaValue::null())
        }
        _ => Err(IfaError::Custom(format!(
            "Oyeku: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_iwori(method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "bayi" | "now" | "current" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            Ok(IfaValue::int(now))
        }
        "akoko" | "timestamp" => {
             let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            Ok(IfaValue::int(now))
        }
        _ => Err(IfaError::Custom(format!(
            "Iwori: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_okanran(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "sise" | "assert" | "verify" | "check" => {
            let null_val = IfaValue::null();
            let val = args.first().unwrap_or(&null_val);
            if val.is_truthy() {
                Ok(IfaValue::bool(true))
            } else {
                let msg = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| "Assertion failed".into());
                Err(IfaError::Runtime(format!("[Okanran.assert] {msg}")))
            }
        }
        "kigbe" | "throw" | "panic" | "raise" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_else(|| "Manually triggered error".into());
            Err(IfaError::Runtime(format!("[Okanran.throw] {msg}")))
        }
        _ => Err(IfaError::Custom(format!(
            "Okanran: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_ose(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    crate::ose::Ose::dispatch(method, args)
}

fn dispatch_ofun(method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "le" | "can" => Ok(IfaValue::bool(true)),
        _ => Err(IfaError::Custom(format!(
            "Ofun: unknown method '{}'",
            method
        ))),
    }
}
