//! # Odù Domain Transpilation
//!
//! Transpiles Odù domain calls (Irosu.fo, Odi.ka, etc.) to Rust code.

use super::constants::matches_method;
use super::constants::{
    irete, irosu, iwori, odi, ofun, ogbe, okanran, osa, ose, otura, owonrin, oyeku,
};
use super::core::RustTranspiler;
use crate::ast::OduCall;

impl RustTranspiler {
    /// Transpile an Odù domain call to Rust
    pub fn transpile_odu_call(&mut self, call: &OduCall) -> String {
        let args: Vec<String> = call
            .args
            .iter()
            .map(|a| self.transpile_expression(a))
            .collect();
        let domain_str = format!("{:?}", call.domain).to_lowercase();
        let method = call.method.to_lowercase();

        match domain_str.as_str() {
            // ═══════════════════════════════════════════════════════════════════
            // ỌGBÈ (1111) - System, CLI Args, Lifecycle
            // ═══════════════════════════════════════════════════════════════════
            "ogbe" if matches_method(&method, ogbe::ARGS) => {
                "IfaValue::List(std::env::args().skip(1).map(|s| IfaValue::Str(s)).collect())".to_string()
            }
            "ogbe" if matches_method(&method, ogbe::ENV) => {
                if let Some(key) = args.first() {
                    format!("match std::env::var(if let IfaValue::Str(s) = {} {{ s }} else {{ String::new() }}) {{ Ok(v) => IfaValue::Str(v), Err(_) => IfaValue::Nil }}", key)
                } else {
                    "IfaValue::Nil".to_string()
                }
            }
            "ogbe" if matches_method(&method, ogbe::EXIT) => {
                if let Some(arg) = args.first() {
                    format!("std::process::exit(match {} {{ IfaValue::Int(n) => n as i32, _ => 0 }})", arg)
                } else {
                    "std::process::exit(0)".to_string()
                }
            }
            "ogbe" if matches_method(&method, ogbe::CWD) => {
                "IfaValue::Str(std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default())".to_string()
            }

            // ═══════════════════════════════════════════════════════════════════
            // ỌYÈKÚ (0000) - Exit/Sleep
            // ═══════════════════════════════════════════════════════════════════
            "oyeku" if matches_method(&method, oyeku::EXIT) => {
                if let Some(arg) = args.first() {
                    format!("std::process::exit(match {} {{ IfaValue::Int(n) => n as i32, _ => 0 }})", arg)
                } else {
                    "std::process::exit(0)".to_string()
                }
            }
            "oyeku" if matches_method(&method, oyeku::SLEEP) => {
                if let Some(arg) = args.first() {
                    format!("std::thread::sleep(std::time::Duration::from_millis(match {} {{ IfaValue::Int(n) => n as u64, _ => 1000 }}))", arg)
                } else {
                    "std::thread::sleep(std::time::Duration::from_secs(1))".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ÌWÒRÌ (0110) - Time, Iteration
            // ═══════════════════════════════════════════════════════════════════
            "iwori" if matches_method(&method, iwori::NOW) => {
                "IfaValue::Int(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0))".to_string()
            }
            "iwori" if matches_method(&method, iwori::NOW_MS) => {
                "IfaValue::Int(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0))".to_string()
            }
            "iwori" if matches_method(&method, iwori::ITERATE) => {
                if args.len() >= 2 {
                    format!("IfaValue::List((match {} {{ IfaValue::Int(s) => s, _ => 0 }}..match {} {{ IfaValue::Int(e) => e, _ => 0 }}).map(|i| IfaValue::Int(i)).collect())", args[0], args[1])
                } else {
                    "IfaValue::List(vec![])".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ÒDÍ (1001) - Files
            // ═══════════════════════════════════════════════════════════════════
            "odi" if matches_method(&method, odi::READ) => {
                if let Some(path) = args.first() {
                    format!("match std::fs::read_to_string(if let IfaValue::Str(s) = {} {{ s }} else {{ String::new() }}) {{ Ok(c) => IfaValue::Str(c), Err(_) => IfaValue::Nil }}", path)
                } else {
                    "IfaValue::Nil".to_string()
                }
            }
            "odi" if matches_method(&method, odi::WRITE) => {
                if args.len() >= 2 {
                    format!("{{ if let (IfaValue::Str(p), IfaValue::Str(c)) = ({}, {}) {{ std::fs::write(&p, &c).ok(); }} IfaValue::Bool(true) }}", args[0], args[1])
                } else {
                    "IfaValue::Bool(false)".to_string()
                }
            }
            "odi" if matches_method(&method, odi::EXISTS) => {
                if let Some(path) = args.first() {
                    format!("IfaValue::Bool(if let IfaValue::Str(p) = {} {{ std::path::Path::new(&p).exists() }} else {{ false }})", path)
                } else {
                    "IfaValue::Bool(false)".to_string()
                }
            }
            "odi" if matches_method(&method, odi::DELETE) => {
                if let Some(path) = args.first() {
                    format!("IfaValue::Bool(if let IfaValue::Str(p) = {} {{ std::fs::remove_file(&p).is_ok() }} else {{ false }})", path)
                } else {
                    "IfaValue::Bool(false)".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ÌROSÙ - Console I/O
            // ═══════════════════════════════════════════════════════════════════
            "irosu" if matches_method(&method, irosu::PRINT) || matches_method(&method, irosu::PRINTLN) => {
                format!("println!(\"{{}}\", {})", args.join(", "))
            }
            "irosu" if matches_method(&method, irosu::READ) => {
                "{ let mut s = String::new(); std::io::stdin().read_line(&mut s).ok(); IfaValue::Str(s.trim().to_string()) }".to_string()
            }

            // ═══════════════════════════════════════════════════════════════════
            // ỌWỌNRÍN - Random
            // ═══════════════════════════════════════════════════════════════════
            "owonrin" if matches_method(&method, owonrin::RANDOM) => {
                self.needs_rand = true;
                "IfaValue::Float(rand::random::<f64>())".to_string()
            }
            "owonrin" if matches_method(&method, owonrin::RANGE) => {
                self.needs_rand = true;
                if args.len() >= 2 {
                    format!("IfaValue::Int(rand::Rng::gen_range(&mut rand::thread_rng(), match {} {{ IfaValue::Int(n) => n, _ => 0 }}..match {} {{ IfaValue::Int(n) => n, _ => 100 }}))", args[0], args[1])
                } else {
                    "IfaValue::Int(rand::Rng::gen_range(&mut rand::thread_rng(), 0..100))".to_string()
                }
            }
            "owonrin" if matches_method(&method, owonrin::BOOL) => {
                self.needs_rand = true;
                "IfaValue::Bool(rand::random::<bool>())".to_string()
            }

            // ═══════════════════════════════════════════════════════════════════
            // ỌKÀNRÀN - Errors, Assertions
            // ═══════════════════════════════════════════════════════════════════
            "okanran" if matches_method(&method, okanran::ASSERT) => {
                if let Some(cond) = args.first() {
                    let msg = args.get(1).cloned().unwrap_or_else(|| "\"Assertion failed\"".to_string());
                    format!("if !({}).is_truthy() {{ panic!(\"Ọ̀kànràn: {{}}\", {}); }}", cond, msg)
                } else {
                    "/* assert with no condition */".to_string()
                }
            }
            "okanran" if matches_method(&method, okanran::EQUALS) => {
                if args.len() >= 2 {
                    format!("if {} != {} {{ panic!(\"Ọ̀kànràn: Not equal\"); }}", args[0], args[1])
                } else {
                    "/* assert_eq requires 2 args */".to_string()
                }
            }
            "okanran" if matches_method(&method, okanran::DIE) => {
                if let Some(msg) = args.first() {
                    format!("panic!(\"Ọ̀kànràn: {{}}\", {})", msg)
                } else {
                    "panic!(\"Ọ̀kànràn: Fatal error\")".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ỌSÁ - Async
            // ═══════════════════════════════════════════════════════════════════
            "osa" if matches_method(&method, osa::SPAWN) => {
                self.has_async = true;
                self.needs_tokio = true;
                if let Some(expr) = args.first() {
                    format!("tokio::spawn(async move {{ {} }})", expr)
                } else {
                    "tokio::spawn(async {})".to_string()
                }
            }
            "osa" if matches_method(&method, osa::SLEEP) => {
                self.has_async = true;
                self.needs_tokio = true;
                if let Some(arg) = args.first() {
                    format!("tokio::time::sleep(tokio::time::Duration::from_millis(match {} {{ IfaValue::Int(n) => n as u64, _ => 1000 }})).await", arg)
                } else {
                    "tokio::time::sleep(tokio::time::Duration::from_secs(1)).await".to_string()
                }
            }
            "osa" if matches_method(&method, osa::AWAIT) => {
                self.has_async = true;
                if let Some(arg) = args.first() {
                    format!("{}.await", arg)
                } else {
                    "/* await requires expression */".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ÒTÚRÁ - Networking
            // ═══════════════════════════════════════════════════════════════════
            "otura" if matches_method(&method, otura::GET) => {
                self.has_async = true;
                self.needs_reqwest = true;
                self.needs_tokio = true;
                if let Some(url) = args.first() {
                    format!("match reqwest::get(if let IfaValue::Str(s) = {} {{ s }} else {{ String::new() }}).await {{ Ok(r) => IfaValue::Str(r.text().await.unwrap_or_default()), Err(_) => IfaValue::Nil }}", url)
                } else {
                    "IfaValue::Nil".to_string()
                }
            }
            "otura" if matches_method(&method, otura::POST) => {
                self.has_async = true;
                self.needs_reqwest = true;
                self.needs_tokio = true;
                if args.len() >= 2 {
                    format!("match reqwest::Client::new().post(if let IfaValue::Str(s) = {} {{ s }} else {{ String::new() }}).body(if let IfaValue::Str(s) = {} {{ s }} else {{ String::new() }}).send().await {{ Ok(r) => IfaValue::Str(r.text().await.unwrap_or_default()), Err(_) => IfaValue::Nil }}", args[0], args[1])
                } else {
                    "IfaValue::Nil".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ÌRẸTẸ̀ - Crypto
            // ═══════════════════════════════════════════════════════════════════
            "irete" if matches_method(&method, irete::HASH) => {
                if let Some(arg) = args.first() {
                    format!("IfaValue::Str({{ use std::collections::hash_map::DefaultHasher; use std::hash::{{Hash, Hasher}}; let mut h = DefaultHasher::new(); if let IfaValue::Str(s) = {} {{ s.hash(&mut h); }} format!(\"{{:x}}\", h.finish()) }})", arg)
                } else {
                    "IfaValue::Str(String::new())".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ỌṢẸ́ - Debug
            // ═══════════════════════════════════════════════════════════════════
            "ose" if matches_method(&method, ose::DEBUG) => {
                if let Some(arg) = args.first() {
                    format!("{{ eprintln!(\"[Ọ̀ṣẹ́ DEBUG] {{:?}}\", {}); {} }}", arg, arg)
                } else {
                    "IfaValue::Nil".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // ÒFÚN - Reflection
            // ═══════════════════════════════════════════════════════════════════
            "ofun" if matches_method(&method, ofun::IS_ALIVE) => {
                "IfaValue::Bool(true)".to_string()
            }
            "ofun" if matches_method(&method, ofun::TYPE_OF) => {
                if let Some(arg) = args.first() {
                    format!("IfaValue::Str(match {} {{ IfaValue::Int(_) => \"Int\", IfaValue::Float(_) => \"Float\", IfaValue::Str(_) => \"Str\", IfaValue::Bool(_) => \"Bool\", IfaValue::List(_) => \"List\", IfaValue::Map(_) => \"Map\", IfaValue::Nil => \"Nil\", }}.to_string())", arg)
                } else {
                    "IfaValue::Str(\"Nil\".to_string())".to_string()
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // Unknown domain/method — hard failure in generated code.
            // If you see this panic at runtime, add a transpilation arm above.
            // ═══════════════════════════════════════════════════════════════════
            _ => {
                format!(
                    "unimplemented!(\"ifa-transpiler: {}.{}({}) has no Rust mapping\")",
                    domain_str,
                    method,
                    args.join(", ")
                )
            }
        }
    }
}
