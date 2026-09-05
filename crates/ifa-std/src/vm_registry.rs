//! # StdRegistry - VM OduRegistry Implementation
//!
//! Bridges the ifa-std domain structs to the VM's OduRegistry trait,
//! enabling `CallOdu` opcodes to dispatch to the standard library.

use ifa_vm::IfaValue;
use ifa_vm::error::{IfaError, IfaResult};
use ifa_vm::native::{OduRegistry, VmContext};
use std::sync::OnceLock;

#[cfg(feature = "tui")]
use ratatui::{Terminal, backend::CrosstermBackend};
#[cfg(feature = "tui")]
use std::io;
#[cfg(feature = "tui")]
use std::sync::Mutex;

use crate::esu::Esu;
#[cfg(feature = "crypto")]
use crate::odu::irete::Irete;
use crate::odu::irosu::Irosu;
use crate::odu::odi::Odi;
#[cfg(feature = "network")]
use crate::odu::otura::Otura;
use crate::sandbox_shim::{CapabilitySet, Ofun};

// ---------------------------------------------------------------------------
// Domain ID constants — single authoritative source for all numeric domain IDs.
// ---------------------------------------------------------------------------
pub mod domain {
    // Odù standard library (0-15)
    pub const OGBE: u8 = 0; // System lifecycle
    pub const OYEKU: u8 = 1; // Process control
    pub const IWORI: u8 = 2; // Time
    pub const ODI: u8 = 3; // File I/O
    pub const IROSU: u8 = 4; // Console / I/O
    pub const OWONRIN: u8 = 5; // Randomness
    pub const OBARA: u8 = 6; // Math
    pub const OKANRAN: u8 = 7; // Error handling
    pub const OGUNDA: u8 = 8; // Collections
    pub const OSA: u8 = 9; // Async / actors
    pub const IKA: u8 = 10; // Strings
    pub const OTURUPON: u8 = 11; // Parsing
    pub const OTURA: u8 = 12; // Networking
    pub const IRETE: u8 = 13; // Crypto
    pub const OSE: u8 = 14; // Graphics / TUI
    pub const OFUN: u8 = 15; // Permissions
    // Hardware domains (18+)
    pub const CPU: u8 = 18;
    pub const GPU: u8 = 19;
    pub const STORAGE: u8 = 20;
    pub const SYS: u8 = 29;
}

// ---------------------------------------------------------------------------
// StdRegistry
// ---------------------------------------------------------------------------

/// Standard library registry for the bytecode VM.
pub struct StdRegistry {
    irosu: OnceLock<Irosu>,
    odi: OnceLock<Odi>,
    #[cfg(feature = "network")]
    otura: OnceLock<Otura>,
    #[cfg(feature = "crypto")]
    irete: OnceLock<Irete>,
    storage: crate::hardware::storage::StorageWorker,
    /// Persistent tokio runtime for networking.
    #[cfg(feature = "network")]
    net_runtime: OnceLock<tokio::runtime::Runtime>,
    esu: Esu,
}

impl StdRegistry {
    ifa_macros::odu_registry_dispatch! {
        domain 0 => self.dispatch_ogbe, {
            0x01 => "bere",
            0x02 => "version",
            0x03 => "args",
            0x04 => "àwọn_àríyànjú",
            0x05 => "env",
            0x06 => "ayika",
            0x07 => "cwd",
            0x08 => "ibi_isisiyi"
        },
        domain 1 => self.dispatch_oyeku, {
            0x01 => "jade",
            0x02 => "sun",
            0x03 => "pid",
            0x04 => "is_alive"
        },
        domain 2 => self.dispatch_iwori, {
            0x01 => "sise",
            0x02 => "duro"
        },
        domain 3 => self.dispatch_odi, {
            0x01 => "ka",
            0x02 => "ko",
            0x03 => "so_po",
            0x04 => "pa_re",
            0x05 => "is_file",
            0x06 => "is_dir"
        },
        domain 4 ctx => self.dispatch_irosu, {
            0x01 => "te",
            0x02 => "gba"
        },
        domain 5 => dispatch_owonrin, {
            0x01 => "tuka",
            0x02 => "yan"
        },
        domain 6 => dispatch_obara, {
            0x01 => "ropo",
            0x02 => "pin",
            0x03 => "sin",
            0x04 => "cos"
        },
        domain 7 ctx => dispatch_okanran, {
            0x01 => "dajo",
            0x02 => "se"
        },
        domain 8 ctx => self.dispatch_ogunda, {
            0x01 => "da",
            0x02 => "pa",
            0x03 => "len",
            0x04 => "push"
        },
        domain 9 ctx => dispatch_osa, {
            0x01 => "sa",
            0x02 => "di",
            0x03 => "spawn"
        },
        domain 10 ctx => dispatch_ika, {
            0x01 => "ka",
            0x02 => "ko",
            0x03 => "split"
        },
        domain 11 => dispatch_oturupon, {
            0x01 => "ru",
            0x02 => "pon",
            0x03 => "json_parse"
        },
        domain 12 ctx => self.dispatch_otura, {
            0x01 => "tu",
            0x02 => "ra",
            0x03 => "get",
            0x04 => "post",
            0x05 => "de",
            0x06 => "soro",
            0x07 => "ka",
            0x08 => "ko",
            0x09 => "pa",
            0x0a => "gba_sile",
            0x0b => "url_encode",
            0x0c => "fetch_json",
            0x0d => "check_host"
        },
        domain 13 => self.dispatch_irete, {
            0x01 => "te",
            0x02 => "re",
            0x03 => "hash"
        },
        domain 14 ctx => dispatch_ose, {
            0x01 => "se",
            0x02 => "la",
            0x03 => "draw"
        },
        domain 15 ctx => self.dispatch_ofun, {
            0x01 => "fun",
            0x02 => "da",
            0x03 => "grant"
        }
    }
    pub fn new() -> Self {
        Self::new_with_caps(CapabilitySet::new())
    }

    fn new_with_caps(caps: CapabilitySet) -> Self {
        let esu = Esu::new(caps.clone());
        Self {
            irosu: OnceLock::new(),
            odi: OnceLock::new(),
            #[cfg(feature = "network")]
            otura: OnceLock::new(),
            #[cfg(feature = "crypto")]
            irete: OnceLock::new(),
            storage: crate::hardware::storage::StorageWorker::new(),
            #[cfg(feature = "network")]
            net_runtime: OnceLock::new(),
            esu,
        }
    }

    /// Update the active capability set.
    ///
    /// Preserves the `StorageWorker` so open storage connections survive.
    /// Must be called before any domain is first accessed — all OnceLocks are reset.
    pub fn set_capabilities(&mut self, caps: CapabilitySet) {
        let storage = self.storage.clone();
        *self = Self::new_with_caps(caps);
        self.storage = storage;
    }

    fn irosu(&self) -> &Irosu {
        self.irosu
            .get_or_init(|| Irosu::new(self.esu.world_state()))
    }

    fn odi(&self) -> &Odi {
        self.odi.get_or_init(|| Odi::new(self.esu.clone()))
    }

    #[cfg(feature = "network")]
    fn otura(&self) -> &Otura {
        self.otura.get_or_init(|| Otura::new(self.esu.clone()))
    }

    /// Get or create the shared networking runtime.
    #[cfg(feature = "network")]
    fn net_runtime(&self) -> &tokio::runtime::Runtime {
        self.net_runtime.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("ifa-net")
                .build()
                .expect("Failed to create networking runtime")
        })
    }

    #[cfg(feature = "crypto")]
    fn irete(&self) -> &Irete {
        self.irete.get_or_init(|| Irete::new(self.esu.clone()))
    }
}

impl Default for StdRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OduRegistry for StdRegistry {
    /// Forks this registry: creates a fresh instance sharing only the `StorageWorker`
    /// and capability state. All OnceLock domain structs are lazily re-initialized on
    /// first use. Intentionally a fork, not a deep clone.
    fn clone_registry(&self) -> Box<dyn OduRegistry> {
        Box::new(StdRegistry {
            irosu: OnceLock::new(),
            odi: OnceLock::new(),
            #[cfg(feature = "network")]
            otura: OnceLock::new(),
            #[cfg(feature = "crypto")]
            irete: OnceLock::new(),
            storage: self.storage.clone(),
            #[cfg(feature = "network")]
            net_runtime: OnceLock::new(),
            esu: Esu::new(self.esu.world_state()),
        })
    }

    #[inline]
    fn check_effect(&self, domain_id: u8) -> IfaResult<()> {
        let cap = match domain_id {
            // Irosu: Print, Input
            4 => Ofun::Stdio,
            // Odi: Import (Module loading requires broad read access)
            3 => Ofun::ReadFiles {
                root: std::path::PathBuf::from("/"),
            },
            // Otura: Network access
            12 => Ofun::Network {
                domains: vec!["*".to_string()],
            },
            // Osa, Ogunda, Cpu are internal VM state manipulation, no external capability needed
            9 | 8 | 18 => return Ok(()),
            _ => return Ok(()),
        };
        self.esu.enforce_crossroads(&cap, "direct VM opcode")
    }

    fn call(
        &self,
        domain_id: u8,
        method_name: &str,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        self.generated_call(domain_id, method_name, args, ctx)
    }

    fn call_fast(
        &self,
        domain_id: u8,
        method_id: u16,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        self.generated_call_fast(domain_id, method_id, args, ctx)
    }
    fn import(&self, path: &str) -> IfaResult<IfaValue> {
        let key = path.replace('\\', "/");
        let domain = key
            .strip_prefix("std.")
            .or_else(|| key.strip_prefix("std/"))
            .unwrap_or(&key);
        let name = domain.split('.').next_back().unwrap_or(domain);
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
            "cpu" => 18,
            "gpu" => 19,
            "storage" => 20,
            "sys" => 29,
            // Note: Audio methods (ohun) route through irosu (domain 4).
            // Stacks (crypto, ml, gamedev, backend, frontend, iot, fidio)
            // are external packages — resolved via 'iba' imports, not domain IDs.
            _ => return Err(IfaError::Custom(format!("Unknown std module: {}", name))),
        };
        Ok(IfaValue::str(format!("__odu_mod__:{id}")))
    }
}

impl StdRegistry {
    fn dispatch_ogbe(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let ogbe = crate::odu::ogbe::Ogbe::new(self.esu.world_state());
        match method {
            "bere" | "version" => Ok(IfaValue::str("1.3.0")),
            "args" | "àwọn_àríyànjú" => {
                let args_vec = ogbe.awon_ohun();
                Ok(IfaValue::list(
                    args_vec.into_iter().map(IfaValue::str).collect(),
                ))
            }
            "env" | "ayika" => {
                let key = args.first().map(|v| v.to_string()).unwrap_or_default();
                if let Some(val) = ogbe.ayika(&key) {
                    Ok(IfaValue::str(val))
                } else {
                    Ok(IfaValue::Null)
                }
            }
            "cwd" | "ibi_isisiyi" => {
                if let Some(val) = ogbe.oju_ona() {
                    Ok(IfaValue::str(val))
                } else {
                    Ok(IfaValue::Null)
                }
            }
            "bi" | "init" => Ok(IfaValue::null()),
            "eto" | "os" => Ok(IfaValue::str(ogbe.eto())),
            _ => Err(IfaError::Custom(format!(
                "Ogbe: unknown method '{}'",
                method
            ))),
        }
    }

    fn dispatch_oyeku(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let oyeku = crate::odu::oyeku::Oyeku::new(self.esu.world_state());
        match method {
            "jade" | "exit" | "quit" | "halt" => {
                let code = args.first().and_then(as_int).unwrap_or(0) as i32;
                oyeku.ku(code);
            }
            "sun" | "sleep" | "wait" => {
                let seconds = args.first().map(extract_num).unwrap_or(0.0);
                oyeku.sun(seconds);
                Ok(IfaValue::null())
            }
            "duro" | "stop" => {
                let ms = args.first().and_then(as_int).unwrap_or(0) as u64;
                oyeku.duro(ms);
                Ok(IfaValue::null())
            }
            "gbale" | "gc" => Ok(IfaValue::null()),
            _ => Err(IfaError::Custom(format!(
                "Oyeku: unknown method '{}'",
                method
            ))),
        }
    }

    fn dispatch_iwori(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let iwori = crate::odu::iwori::Iwori;
        match method {
            "bayi" | "now" | "current" => Ok(IfaValue::str(iwori.isisinyi().to_rfc3339())),
            "akoko" | "timestamp" => Ok(IfaValue::int(iwori.akoko())),
            "yipo" | "iterate" => {
                let start = args.first().and_then(as_int).unwrap_or(0);
                let end = args.get(1).and_then(as_int).unwrap_or(0);
                let list: Vec<IfaValue> = (start..=end).map(IfaValue::int).collect();
                Ok(IfaValue::list(list))
            }
            "pada" | "return" => Ok(args.first().cloned().unwrap_or(IfaValue::Null)),
            _ => Err(IfaError::Custom(format!(
                "Iwori: unknown method '{}'",
                method
            ))),
        }
    }

        fn dispatch_otura(&self, method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
        #[cfg(feature = "network")]
        {
            match method {
                "gba" | "get" | "fetch" => {
                    let url = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let result = self.net_runtime().block_on(self.otura().gba(&url))?;
                    Ok(IfaValue::str(result))
                }
                "ran" | "post" => {
                    let url = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let body = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                    let result = self.net_runtime().block_on(self.otura().ran(&url, &body))?;
                    Ok(IfaValue::str(result))
                }
                "de" | "listen" => {
                    let addr = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let listener = self.net_runtime().block_on(self.otura().de(&addr))?;
                    let resource = crate::odu::otura::TcpListenerResource { listener };
                    let token = ctx.resource_registry().register(resource);
                    Ok(IfaValue::Resource(std::sync::Arc::new(token)))
                }
                "soro" | "connect" => {
                    let addr = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let stream = self.net_runtime().block_on(self.otura().soro(&addr))?;
                    let resource = crate::odu::otura::TcpStreamResource {
                        stream: std::sync::Mutex::new(stream),
                    };
                    let token = ctx.resource_registry().register(resource);
                    Ok(IfaValue::Resource(std::sync::Arc::new(token)))
                }
                "gba_sile" | "accept" => {
                    let token = match args.first() {
                        Some(IfaValue::Resource(r)) => **r,
                        _ => return Err(IfaError::ArgumentError(
                            "otura.accept: first arg must be TcpListener resource".into(),
                        )),
                    };
                    let registry = ctx.resource_registry();
                    let resource = registry
                        .get::<crate::odu::otura::TcpListenerResource>(token)
                        .ok_or_else(|| IfaError::Runtime("TcpListener handle not found".into()))?;
                    let (stream, _addr) = self.net_runtime()
                        .block_on(resource.listener.accept())
                        .map_err(|e| IfaError::ConnectionFailed(format!("Accept failed: {e}")))?;
                    let stream_resource = crate::odu::otura::TcpStreamResource {
                        stream: std::sync::Mutex::new(stream),
                    };
                    let stream_token = ctx.resource_registry().register(stream_resource);
                    Ok(IfaValue::Resource(std::sync::Arc::new(stream_token)))
                }
                "ka" | "read" => {
                    let token = match args.first() {
                        Some(IfaValue::Resource(r)) => **r,
                        _ => return Err(IfaError::ArgumentError(
                            "otura.read: first arg must be TcpStream resource".into(),
                        )),
                    };
                    let max_bytes = args.get(1).and_then(|v| match v {
                        IfaValue::Int(i) => Some(*i as usize),
                        _ => None,
                    }).unwrap_or(4096);
                    let registry = ctx.resource_registry();
                    let resource = registry
                        .get::<crate::odu::otura::TcpStreamResource>(token)
                        .ok_or_else(|| IfaError::Runtime("TcpStream handle not found".into()))?;
                    use tokio::io::AsyncReadExt;
                    let mut buf = vec![0u8; max_bytes];
                    let n = {
                        let mut stream = resource.stream.lock().map_err(|_| {
                            IfaError::Runtime("TcpStream lock poisoned".into())
                        })?;
                        self.net_runtime().block_on(stream.read(&mut buf))
                            .map_err(|e| IfaError::IoError(format!("Read failed: {e}")))?
                    };
                    buf.truncate(n);
                    match String::from_utf8(buf) {
                        Ok(s) => Ok(IfaValue::str(s)),
                        Err(e) => Ok(IfaValue::list(
                            e.as_bytes().iter().map(|b| IfaValue::int(*b as i64)).collect(),
                        )),
                    }
                }
                "ko" | "write" => {
                    let token = match args.first() {
                        Some(IfaValue::Resource(r)) => **r,
                        _ => return Err(IfaError::ArgumentError(
                            "otura.write: first arg must be TcpStream resource".into(),
                        )),
                    };
                    let data = args.get(1).map(|v| v.to_string().into_bytes()).unwrap_or_default();
                    let registry = ctx.resource_registry();
                    let resource = registry
                        .get::<crate::odu::otura::TcpStreamResource>(token)
                        .ok_or_else(|| IfaError::Runtime("TcpStream handle not found".into()))?;
                    use tokio::io::AsyncWriteExt;
                    let n = {
                        let mut stream = resource.stream.lock().map_err(|_| {
                            IfaError::Runtime("TcpStream lock poisoned".into())
                        })?;
                        self.net_runtime().block_on(stream.write_all(&data))
                            .map_err(|e| IfaError::IoError(format!("Write failed: {e}")))?;
                        data.len()
                    };
                    Ok(IfaValue::int(n as i64))
                }
                "pa" | "close" => {
                    let _token = match args.first() {
                        Some(IfaValue::Resource(r)) => **r,
                        _ => return Err(IfaError::ArgumentError(
                            "otura.close: first arg must be a network resource".into(),
                        )),
                    };
                    Ok(IfaValue::null())
                }
                "sayewo" | "check_host" => {
                    let host = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(IfaValue::bool(self.otura().ṣàyẹ̀wò(&host)))
                }
                _ => Err(IfaError::Custom(format!("Otura: unknown method '{}'", method))),
            }
        }
        #[cfg(not(feature = "network"))]
        {
            let _ = (args, ctx);
            Err(IfaError::Runtime(format!("Otura requires the network feature (method: {})", method)))
        }
    }

#[cfg(feature = "crypto")]
    fn dispatch_irete(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let to_bytes = |val: &IfaValue| -> Vec<u8> {
            match val {
                IfaValue::Str(s) => s.as_bytes().to_vec(),
                IfaValue::List(l) => l
                    .iter()
                    .map(|v| match v {
                        IfaValue::Int(i) => *i as u8,
                        IfaValue::Float(f) => *f as u8,
                        _ => 0,
                    })
                    .collect(),
                other => other.to_string().into_bytes(),
            }
        };

        let to_value = |bytes: Vec<u8>| -> IfaValue {
            IfaValue::list(bytes.into_iter().map(|b| IfaValue::int(b as i64)).collect())
        };

        match method {
            "hash" | "sha256" => {
                let data = args.first().map(to_bytes).unwrap_or_default();
                let hashed = self.irete().sha256(&data)?;
                Ok(to_value(hashed))
            }
            "sha256_hex" => {
                let data = args.first().map(to_bytes).unwrap_or_default();
                Ok(IfaValue::str(self.irete().sha256_hex(&data)?))
            }
            "sha512" => {
                let data = args.first().map(to_bytes).unwrap_or_default();
                let hashed = self.irete().sha512(&data)?;
                Ok(to_value(hashed))
            }
            "hmac" | "hmac_sha256" => {
                let key = args.first().map(to_bytes).unwrap_or_default();
                let data = args.get(1).map(to_bytes).unwrap_or_default();
                let hmac_res = self.irete().hmac_sha256(&key, &data)?;
                Ok(to_value(hmac_res))
            }
            "hmac_verify" => {
                let key = args.first().map(to_bytes).unwrap_or_default();
                let data = args.get(1).map(to_bytes).unwrap_or_default();
                let sig = args.get(2).map(to_bytes).unwrap_or_default();
                let verified = self.irete().hmac_verify(&key, &data, &sig)?;
                Ok(IfaValue::bool(verified))
            }
            "base64" | "base64_encode" | "encode_base64" => {
                let data = args.first().map(to_bytes).unwrap_or_default();
                Ok(IfaValue::str(self.irete().base64_encode(&data)?))
            }
            "decode" | "base64_decode" | "decode_base64" => {
                let data = args.first().map(|v| v.to_string()).unwrap_or_default();
                let bytes = self.irete().base64_decode(&data)?;
                if let Ok(s) = String::from_utf8(bytes.clone()) {
                    Ok(IfaValue::str(s))
                } else {
                    Ok(to_value(bytes))
                }
            }
            "random_bytes" => {
                let count = args
                    .first()
                    .and_then(|v| match v {
                        IfaValue::Int(n) => Some(*n as usize),
                        IfaValue::Float(f) => Some(*f as usize),
                        _ => None,
                    })
                    .unwrap_or(16);
                let bytes = self.irete().random_bytes(count)?;
                Ok(IfaValue::str(self.irete().hex_encode(&bytes)?))
            }
            "hex_encode" | "hex" => {
                let data = args.first().map(to_bytes).unwrap_or_default();
                Ok(IfaValue::str(self.irete().hex_encode(&data)?))
            }
            "hex_decode" | "unhex" => {
                let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                let decoded = self.irete().hex_decode(&s)?;
                Ok(to_value(decoded))
            }
            "chacha20_encrypt" | "encrypt" | "di_pa" => {
                let key = args.first().map(to_bytes).unwrap_or_default();
                let nonce = args.get(1).map(to_bytes).unwrap_or_default();
                let data = args.get(2).map(to_bytes).unwrap_or_default();
                let encrypted = self.irete().chacha20_encrypt(&key, &nonce, &data)?;
                Ok(to_value(encrypted))
            }
            "chacha20_decrypt" | "decrypt" | "tu_pa" => {
                let key = args.first().map(to_bytes).unwrap_or_default();
                let nonce = args.get(1).map(to_bytes).unwrap_or_default();
                let enc_data = args.get(2).map(to_bytes).unwrap_or_default();
                let decrypted = self.irete().chacha20_decrypt(&key, &nonce, &enc_data)?;
                Ok(to_value(decrypted))
            }
            "ed25519_generate" | "keypair" => {
                let (priv_key, pub_key) = self.irete().ed25519_generate()?;
                Ok(IfaValue::list(vec![to_value(priv_key), to_value(pub_key)]))
            }
            "ed25519_sign" | "sign" | "fi_o" => {
                let priv_key = args.first().map(to_bytes).unwrap_or_default();
                let msg = args.get(1).map(to_bytes).unwrap_or_default();
                let sig = self.irete().ed25519_sign(&priv_key, &msg)?;
                Ok(to_value(sig))
            }
            "ed25519_verify" | "verify" | "yewo_fo" => {
                let pub_key = args.first().map(to_bytes).unwrap_or_default();
                let msg = args.get(1).map(to_bytes).unwrap_or_default();
                let sig = args.get(2).map(to_bytes).unwrap_or_default();
                let verified = self.irete().ed25519_verify(&pub_key, &msg, &sig)?;
                Ok(IfaValue::bool(verified))
            }
            "funpo" | "compress" => {
                let data = args.first().map(to_bytes).unwrap_or_default();
                let level = args.get(1).and_then(as_int).unwrap_or(3) as i32;
                let compressed = self.irete().funpo(&data, level)?;
                Ok(to_value(compressed))
            }
            "tu" | "decompress" => {
                let data = args.first().map(to_bytes).unwrap_or_default();
                let decompressed = self.irete().tu(&data)?;
                if let Ok(s) = String::from_utf8(decompressed.clone()) {
                    Ok(IfaValue::str(s))
                } else {
                    Ok(to_value(decompressed))
                }
            }
            "iwon_funpo" | "ratio" => {
                let orig_len = args.first().and_then(as_int).unwrap_or(0) as usize;
                let comp_len = args.get(1).and_then(as_int).unwrap_or(0) as usize;
                let ratio = self.irete().iwon_funpo(orig_len, comp_len)?;
                Ok(IfaValue::float(ratio))
            }
            _ => Err(IfaError::Custom(format!(
                "Irete: unknown method '{}'",
                method
            ))),
        }
    }

    #[cfg(not(feature = "crypto"))]
    fn dispatch_irete(&self, method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        Err(IfaError::Runtime(format!(
            "Irete requires the 'crypto' feature (method: {})",
            method
        )))
    }

    fn dispatch_irosu(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        match method {
            "fo" | "println" => {
                if let Some(val) = args.first() {
                    self.irosu().fo(val);
                    ctx.vm.opon.record("Ìrosù", "fọ̀ (spoke)", val);
                }
                Ok(IfaValue::null())
            }
            "so" | "print" => {
                if let Some(val) = args.first() {
                    self.irosu().so(val);
                    ctx.vm.opon.record("Ìrosù", "fọ̀ (spoke_raw)", val);
                }
                Ok(IfaValue::null())
            }
            "gbo" | "listen" => {
                let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::str(self.irosu().gbo(&prompt)))
            }
            "gbo_nomba" => {
                let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::int(self.irosu().gbo_nomba(&prompt)))
            }
            "mo" | "clear" => {
                self.irosu().mo();
                Ok(IfaValue::null())
            }
            "san" | "flush" => {
                self.irosu().san();
                Ok(IfaValue::null())
            }
            "kigbe" | "error" => {
                let text = args.first().map(|v| v.to_string()).unwrap_or_default();
                self.irosu().kigbe(&text);
                ctx.vm
                    .opon
                    .record("Ìrosù", "kígbe (screamed)", &IfaValue::str(&text));
                Ok(IfaValue::null())
            }
            "ka" | "read" | "input" => {
                let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::str(self.irosu().gbo(&prompt)))
            }
            #[cfg(feature = "audio")]
            "siro_duro" | "play_blocking" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                self.irosu().siro_duro(&path).map_err(IfaError::Custom)?;
                Ok(IfaValue::null())
            }
            #[cfg(feature = "audio")]
            "kigbe_orin" | "beep" => {
                self.irosu().kigbe_orin();
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
                self.odi().ka(&path).map(IfaValue::str)
            }
            "ko" | "write" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                let content = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                self.odi().ko(&path, &content).map(|_| IfaValue::null())
            }
            "wa" | "exists" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::bool(self.odi().wa(&path)))
            }
            "si" | "open" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::str(format!("File {} opened", path)))
            }
            "pa" | "close" => Ok(IfaValue::null()),
            "pa_iwe" | "delete" => {
                let path = args.first().map(|v| v.to_string()).unwrap_or_default();
                self.odi().pa_faili(&path).map(|_| IfaValue::null())
            }
            _ => Err(IfaError::Custom(format!(
                "Odi: unknown method '{}'",
                method
            ))),
        }
    }
}

// Stateless dispatchers (no struct instance needed)

fn as_int(val: &IfaValue) -> Option<i64> {
    match val {
        IfaValue::Int(i) => Some(*i),
        IfaValue::Float(f) => Some(*f as i64),
        _ => None,
    }
}

fn as_bool(val: &IfaValue) -> Option<bool> {
    match val {
        IfaValue::Bool(b) => Some(*b),
        _ => None,
    }
}

fn parse_rounding_mode(s: &str) -> crate::odu::oturupon::RoundingMode {
    match s.to_lowercase().as_str() {
        "halfeven" | "half_even" => crate::odu::oturupon::RoundingMode::HalfEven,
        "halfup" | "half_up" => crate::odu::oturupon::RoundingMode::HalfUp,
        "halfdown" | "half_down" => crate::odu::oturupon::RoundingMode::HalfDown,
        "down" | "truncate" => crate::odu::oturupon::RoundingMode::Down,
        "up" => crate::odu::oturupon::RoundingMode::Up,
        "ceil" | "ceiling" => crate::odu::oturupon::RoundingMode::Ceil,
        "floor" => crate::odu::oturupon::RoundingMode::Floor,
        _ => crate::odu::oturupon::RoundingMode::HalfEven,
    }
}

fn extract_floats(args: &[IfaValue]) -> Vec<f64> {
    if let Some(IfaValue::List(list)) = args.first() {
        list.iter().map(extract_num).collect()
    } else {
        args.iter().map(extract_num).collect()
    }
}

struct RopeResource(std::sync::Mutex<ropey::Rope>);

fn dispatch_ika(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    let ika = crate::odu::ika::Ika;
    match method {
        "gigun" | "len" => match args.first() {
            Some(IfaValue::Str(cs)) => Ok(IfaValue::int(cs.chars().count() as i64)),
            Some(v) => {
                let s = v.to_string();
                Ok(IfaValue::int(IfaValue::unicode_string_len(&s) as i64))
            }
            None => Ok(IfaValue::int(0)),
        },
        "ge" | "slice" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let start = args.get(1).and_then(as_int).unwrap_or(0) as usize;
            let end = args
                .get(2)
                .and_then(as_int)
                .unwrap_or(s.chars().count() as i64) as usize;
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
            Ok(IfaValue::str(ika.oruko_html(&raw)))
        }
        "tumo_html" | "strip_html" => {
            let raw = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.tumo_html(&raw)))
        }
        "mo" | "clean_html" => {
            let raw = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.mo(&raw)))
        }
        "wa_html" | "html_query" => {
            let html = args.first().map(|v| v.to_string()).unwrap_or_default();
            let selector = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::list(
                ika.wa_html(&html, &selector)
                    .into_iter()
                    .map(IfaValue::str)
                    .collect(),
            ))
        }
        "nla" | "uppercase" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.nla(&s)))
        }
        "kekere" | "lowercase" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.kekere(&s)))
        }
        "wa" | "find" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let needle = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match ika.wa(&s, &needle) {
                Some(idx) => Ok(IfaValue::int(idx as i64)),
                None => Ok(IfaValue::Null),
            }
        }
        "ni" | "has" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let needle = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::bool(ika.ni(&s, &needle)))
        }
        "pin" | "split" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let delim = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let parts = ika.pin(&s, &delim);
            Ok(IfaValue::list(
                parts.into_iter().map(IfaValue::str).collect(),
            ))
        }
        "dapo" | "join" => {
            let list_val = args
                .first()
                .ok_or_else(|| IfaError::ArgumentError("join expects (list, separator)".into()))?;
            let sep = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let parts: Vec<String> = match list_val {
                IfaValue::List(l) => l.iter().map(|v| v.to_string()).collect(),
                other => vec![other.to_string()],
            };
            let parts_ref: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
            Ok(IfaValue::str(ika.dapo(&parts_ref, &sep)))
        }
        "yi_pada" | "replace" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let from = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let to = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.yi_pada(&s, &from, &to)))
        }
        "pada" | "reverse" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.pada(&s)))
        }
        "ge_lara" | "substring" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let start = args.get(1).and_then(as_int).unwrap_or(0) as usize;
            let end = args
                .get(2)
                .and_then(as_int)
                .unwrap_or(s.chars().count() as i64) as usize;
            Ok(IfaValue::str(ika.ge_lara(&s, start, end)))
        }
        "tun" | "repeat" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let n = args.get(1).and_then(as_int).unwrap_or(1) as usize;
            Ok(IfaValue::str(ika.tun(&s, n)))
        }
        "bere" | "starts_with" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let prefix = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::bool(ika.bere(&s, &prefix)))
        }
        "pari" | "ends_with" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let suffix = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::bool(ika.pari(&s, &suffix)))
        }
        "ba_mu" | "regex_match" | "matches" => {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::bool(ika.ba_mu(&pattern, &text)?))
        }
        "wa_akoko" | "regex_find" => {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            match ika.wa_akoko(&pattern, &text)? {
                Some(m) => Ok(IfaValue::str(m)),
                None => Ok(IfaValue::Null),
            }
        }
        "wa_gbogbo" | "regex_find_all" => {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let matches = ika.wa_gbogbo(&pattern, &text)?;
            Ok(IfaValue::list(
                matches.into_iter().map(IfaValue::str).collect(),
            ))
        }
        "ropo" | "regex_replace" => {
            let pattern = args.first().map(|v| v.to_string()).unwrap_or_default();
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let repl = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.ropo(&pattern, &text, &repl)?))
        }
        "yi_si_json" | "encode" | "to_json" => {
            let val = args
                .first()
                .ok_or_else(|| IfaError::ArgumentError("encode expects a value".into()))?;
            Ok(IfaValue::str(ika.yi_si_json(val)?))
        }
        "yi_pada_json" | "decode" | "from_json" => {
            let json = args.first().map(|v| v.to_string()).unwrap_or_default();
            ika.yi_pada_json(&json)
        }
        "bo" | "url_encode" | "bo_asiri_url" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.bo_asiri_url(&s)))
        }
        "titu_asiri_url" | "url_decode" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.titu_asiri_url(&s)?))
        }
        "yi_si_csv" | "to_csv" => {
            let rows = args
                .first()
                .ok_or_else(|| IfaError::ArgumentError("to_csv expects rows".into()))?;
            Ok(IfaValue::str(ika.yi_si_csv(rows)?))
        }
        "yi_pada_csv" | "from_csv" => {
            let csv = args.first().map(|v| v.to_string()).unwrap_or_default();
            let has_headers = args.get(1).and_then(as_bool).unwrap_or(false);
            ika.yi_pada_csv(&csv, has_headers)
        }
        "rope_new" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let rope = ika.rope_new(&s);
            let token = ctx
                .resource_registry()
                .register(RopeResource(std::sync::Mutex::new(rope)));
            Ok(IfaValue::Resource(std::sync::Arc::new(token)))
        }
        "rope_insert" => {
            let token = match args.first() {
                Some(IfaValue::Resource(r)) => **r,
                _ => {
                    return Err(IfaError::ArgumentError(
                        "rope_insert expects a Rope resource".into(),
                    ));
                }
            };
            let idx = args.get(1).and_then(as_int).unwrap_or(0) as usize;
            let text = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            let res = ctx
                .resource_registry()
                .get::<RopeResource>(token)
                .ok_or_else(|| IfaError::Runtime("Rope handle not found".into()))?;
            let mut rope = res
                .0
                .lock()
                .map_err(|_| IfaError::Runtime("Rope lock poisoned".into()))?;
            ika.rope_insert(&mut rope, idx, &text);
            Ok(IfaValue::Null)
        }
        "rope_delete" => {
            let token = match args.first() {
                Some(IfaValue::Resource(r)) => **r,
                _ => {
                    return Err(IfaError::ArgumentError(
                        "rope_delete expects a Rope resource".into(),
                    ));
                }
            };
            let start = args.get(1).and_then(as_int).unwrap_or(0) as usize;
            let end = args.get(2).and_then(as_int).unwrap_or(0) as usize;
            let res = ctx
                .resource_registry()
                .get::<RopeResource>(token)
                .ok_or_else(|| IfaError::Runtime("Rope handle not found".into()))?;
            let mut rope = res
                .0
                .lock()
                .map_err(|_| IfaError::Runtime("Rope lock poisoned".into()))?;
            ika.rope_delete(&mut rope, start, end);
            Ok(IfaValue::Null)
        }
        "rope_slice" => {
            let token = match args.first() {
                Some(IfaValue::Resource(r)) => **r,
                _ => {
                    return Err(IfaError::ArgumentError(
                        "rope_slice expects a Rope resource".into(),
                    ));
                }
            };
            let start = args.get(1).and_then(as_int).unwrap_or(0) as usize;
            let end = args.get(2).and_then(as_int).unwrap_or(0) as usize;
            let res = ctx
                .resource_registry()
                .get::<RopeResource>(token)
                .ok_or_else(|| IfaError::Runtime("Rope handle not found".into()))?;
            let rope = res
                .0
                .lock()
                .map_err(|_| IfaError::Runtime("Rope lock poisoned".into()))?;
            let slice = ika.rope_slice(&rope, start, end);
            Ok(IfaValue::str(slice))
        }
        "rope_len" => {
            let token = match args.first() {
                Some(IfaValue::Resource(r)) => **r,
                _ => {
                    return Err(IfaError::ArgumentError(
                        "rope_len expects a Rope resource".into(),
                    ));
                }
            };
            let res = ctx
                .resource_registry()
                .get::<RopeResource>(token)
                .ok_or_else(|| IfaError::Runtime("Rope handle not found".into()))?;
            let rope = res
                .0
                .lock()
                .map_err(|_| IfaError::Runtime("Rope lock poisoned".into()))?;
            let len = ika.rope_len(&rope);
            Ok(IfaValue::int(len as i64))
        }
        "ge_trim" | "trim" => {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Ok(IfaValue::str(ika.ge(&s)))
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
    let obara = crate::odu::obara::Obara;
    let a = args.first().map(extract_num).unwrap_or(0.0);
    let b = args.get(1).map(extract_num).unwrap_or(0.0);
    match method {
        "fikun" | "add" | "plus" => Ok(IfaValue::float(obara.fikun(a, b))),
        "fikun_odidi" | "add_decimal" => {
            let a_val = args.first().and_then(as_int).unwrap_or(0);
            let b_val = args.get(1).and_then(as_int).unwrap_or(0);
            let scale = args.get(2).and_then(as_int).unwrap_or(0) as u32;
            let res = obara.fikun_odidi(a_val, b_val, scale)?;
            Ok(IfaValue::int(res))
        }
        "fikun_rounded" | "add_decimal_rounded" => {
            let a_val = args.first().and_then(as_int).unwrap_or(0);
            let b_val = args.get(1).and_then(as_int).unwrap_or(0);
            let scale = args.get(2).and_then(as_int).unwrap_or(0) as u32;
            let mode_str = args
                .get(3)
                .map(|v| v.to_string().to_lowercase())
                .unwrap_or_else(|| "half_even".into());
            let mode = parse_rounding_mode(&mode_str);
            let res = obara.fikun_rounded(a_val, b_val, scale, mode)?;
            Ok(IfaValue::int(res))
        }
        "isodipupo" | "mul" | "times" => Ok(IfaValue::float(obara.isodipupo(a, b))),
        "isodipupo_odidi" | "mul_decimal" => {
            let a_val = args.first().and_then(as_int).unwrap_or(0);
            let b_val = args.get(1).and_then(as_int).unwrap_or(0);
            let scale = args.get(2).and_then(as_int).unwrap_or(0) as u32;
            let res = obara.isodipupo_odidi(a_val, b_val, scale)?;
            Ok(IfaValue::int(res))
        }
        "agbara" | "pow" => Ok(IfaValue::float(obara.agbara(a, b))),
        "gbongbo" | "sqrt" => Ok(IfaValue::float(obara.gbongbo(a))),
        "abs" => Ok(IfaValue::float(obara.abs(a))),
        "apapo" | "sum" => {
            let items = extract_floats(&args);
            Ok(IfaValue::float(obara.apapo(&items)))
        }
        "ile" | "floor" => Ok(IfaValue::float(obara.ile(a))),
        "orule" | "ceil" => Ok(IfaValue::float(obara.orule(a))),
        "yika" | "round" => {
            let decimals = b as i32;
            Ok(IfaValue::float(obara.yika(a, decimals)))
        }
        "iyoku" | "modulo" | "mod" => Ok(IfaValue::float(obara.iyoku(a, b))),
        "sin" => Ok(IfaValue::float(obara.sin(a))),
        "cos" => Ok(IfaValue::float(obara.cos(a))),
        "tan" => Ok(IfaValue::float(obara.tan(a))),
        "asin" => Ok(IfaValue::float(obara.asin(a))),
        "acos" => Ok(IfaValue::float(obara.acos(a))),
        "atan" => Ok(IfaValue::float(obara.atan(a))),
        "log" | "ln" => Ok(IfaValue::float(obara.log(a))),
        "log10" => Ok(IfaValue::float(obara.log10(a))),
        "exp" => Ok(IfaValue::float(obara.exp(a))),
        "aropin" | "mean" | "avg" => {
            let items = extract_floats(&args);
            Ok(IfaValue::float(obara.aropin(&items)))
        }
        "nla_julo" | "max" => {
            let items = extract_floats(&args);
            Ok(IfaValue::float(obara.nla_julo(&items)))
        }
        "kere_julo" | "min" => {
            let items = extract_floats(&args);
            Ok(IfaValue::float(obara.kere_julo(&items)))
        }
        "pi" => Ok(IfaValue::float(obara.pi())),
        "e" => Ok(IfaValue::float(obara.e())),
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
                .first()
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
            if let Some(IfaValue::Int(ms)) = args.first() {
                std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                Ok(IfaValue::future_ready(IfaValue::null()))
            } else {
                Err(IfaError::ArgumentError(
                    "Osa.sun expects milliseconds".into(),
                ))
            }
        }
        "gbogbo" | "all" => {
            if let Some(IfaValue::List(list)) = args.first() {
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
        "egbe" | "ẹgbẹ" | "spawn_actor" => {
            let handler = args.into_iter().next().ok_or_else(|| {
                IfaError::ArgumentError("Osa.egbe expects a handler function".into())
            })?;
            match &handler {
                IfaValue::Fn(_) | IfaValue::Closure(_) => {}
                other => {
                    return Err(IfaError::TypeError {
                        expected: "Fn or Closure".into(),
                        got: other.type_name().into(),
                    });
                }
            }
            ctx.spawn_actor(handler)
        }
        "ran" | "send" => {
            let actor = args
                .first()
                .cloned()
                .ok_or_else(|| IfaError::ArgumentError("Osa.ran expects (actor, value)".into()))?;
            let value = args.into_iter().nth(1).unwrap_or(IfaValue::null());
            ctx.actor_send(&actor, value)?;
            Ok(IfaValue::null())
        }
        _ => Err(IfaError::Custom(format!(
            "Osa: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_oturupon(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    let oturupon = crate::odu::oturupon::Oturupon;
    let a_num = args.first().map(extract_num).unwrap_or(0.0);
    let b_num = args.get(1).map(extract_num).unwrap_or(0.0);
    let a_int = args.first().and_then(as_int).unwrap_or(0);
    let b_int = args.get(1).and_then(as_int).unwrap_or(0);
    match method {
        "yokuro" | "sub" | "minus" => Ok(IfaValue::float(a_num - b_num)),
        "pipin" | "div" | "divide" => {
            if b_num == 0.0 {
                return Err(IfaError::Custom("Division by zero".into()));
            }
            Ok(IfaValue::float(a_num / b_num))
        }
        "din" | "sub_checked" => {
            let res = oturupon.din(a_int, b_int)?;
            Ok(IfaValue::int(res))
        }
        "din_odidi" | "sub_decimal" => {
            let scale = args.get(2).and_then(as_int).unwrap_or(0) as u32;
            let res = oturupon.din_odidi(a_int, b_int, scale)?;
            Ok(IfaValue::int(res))
        }
        "pin" | "div_checked" => {
            let res = oturupon.pin(a_int, b_int)?;
            Ok(IfaValue::float(res))
        }
        "pin_odidi_scaled" | "div_decimal" => {
            let scale = args.get(2).and_then(as_int).unwrap_or(0) as u32;
            let mode_str = args
                .get(3)
                .map(|v| v.to_string().to_lowercase())
                .unwrap_or_else(|| "half_even".into());
            let mode = parse_rounding_mode(&mode_str);
            let res = oturupon.pin_odidi_scaled(a_int, b_int, scale, mode)?;
            Ok(IfaValue::int(res))
        }
        "pin_odidi" | "div_int" => {
            let res = oturupon.pin_odidi(a_int, b_int)?;
            Ok(IfaValue::int(res))
        }
        "din_f" | "sub_float" => Ok(IfaValue::float(oturupon.din_f(a_num, b_num))),
        "pin_f" | "div_float" => {
            let mode_str = args
                .get(2)
                .map(|v| v.to_string().to_lowercase())
                .unwrap_or_else(|| "half_even".into());
            let mode = parse_rounding_mode(&mode_str);
            let res = oturupon.pin_f(a_num, b_num, mode)?;
            Ok(IfaValue::float(res))
        }
        "ku" | "rem_checked" => {
            let res = oturupon.ku(a_int, b_int)?;
            Ok(IfaValue::int(res))
        }
        "ku_euclidean" | "rem_euclidean" => {
            let res = oturupon.ku_euclidean(a_int, b_int)?;
            Ok(IfaValue::int(res))
        }
        "dake" | "neg_checked" => {
            let res = oturupon.dake(a_int)?;
            Ok(IfaValue::int(res))
        }
        "idakeji" | "reciprocal" => {
            let res = oturupon.idakeji(a_num)?;
            Ok(IfaValue::float(res))
        }
        "iyoku" | "diff" => {
            let res = oturupon.iyoku(a_int, b_int)?;
            Ok(IfaValue::int(res))
        }
        _ => Err(IfaError::Custom(format!(
            "Oturupon: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_owonrin(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    match method {
        "pese" | "random" | "rand" | "nọmba" => {
            let min = args.first().and_then(as_int);
            let max = args.get(1).and_then(as_int);
            match (min, max) {
                (Some(mn), Some(mx)) => {
                    let r = if mn >= mx { mn } else { rng.gen_range(mn..=mx) };
                    Ok(IfaValue::int(r))
                }
                _ => Ok(IfaValue::float(rng.r#gen::<f64>())),
            }
        }
        "yan_bool" | "random_bool" => {
            let prob = args.first().map(extract_num).unwrap_or(0.5);
            Ok(IfaValue::bool(rng.gen_bool(prob.clamp(0.0, 1.0))))
        }
        "yan_laarin" | "range" => {
            let min = args.first().map(extract_num).unwrap_or(0.0);
            let max = args.get(1).map(extract_num).unwrap_or(1.0);
            let r = if min >= max {
                min
            } else {
                rng.gen_range(min..=max)
            };
            Ok(IfaValue::float(r))
        }
        "paaro" | "shuffle" => {
            if let Some(IfaValue::List(list_arc)) = args.first() {
                let mut list = list_arc.clone();
                let vec = ifa_types::gc::IfaGc::make_mut(&mut list);
                use rand::seq::SliceRandom;
                vec.shuffle(&mut rng);
                Ok(IfaValue::List(list))
            } else {
                Err(IfaError::ArgumentError("shuffle expects a list".into()))
            }
        }
        "uuid" => Ok(IfaValue::str(uuid::Uuid::new_v4().to_string())),
        _ => Err(IfaError::Custom(format!(
            "Owonrin: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_okanran(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    let okanran = crate::odu::okanran::Okanran;
    match method {
        "sise" | "assert" | "verify" | "check" => {
            let null_val = IfaValue::Null;
            let val = args.first().unwrap_or(&null_val);
            if val.is_truthy() {
                Ok(IfaValue::bool(true))
            } else {
                let msg = args
                    .get(1)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Assertion failed".into());
                Err(IfaError::Runtime(format!("[Okanran.assert] {msg}")))
            }
        }
        "ta" | "throw" | "raise" | "kigbe" => {
            let val = args.first().cloned().unwrap_or(IfaValue::Null);
            Err(IfaError::UserError(val.freeze()?))
        }
        "dogba" | "equals" => {
            let a = args.first().unwrap_or(&IfaValue::Null);
            let b = args.get(1).unwrap_or(&IfaValue::Null);
            okanran.dogba(a, b)?;
            Ok(IfaValue::bool(true))
        }
        "beeni" | "assert_true" => {
            let cond = args.first().map(|v| v.is_truthy()).unwrap_or(false);
            let msg = args
                .get(1)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "Expected true".into());
            okanran.beeni(cond, &msg)?;
            Ok(IfaValue::bool(true))
        }
        "yato" | "assert_not_equal" => {
            let a = args.first().unwrap_or(&IfaValue::Null);
            let b = args.get(1).unwrap_or(&IfaValue::Null);
            okanran.yato(a, b)?;
            Ok(IfaValue::bool(true))
        }
        "beko" | "assert_false" => {
            let cond = args.first().map(|v| v.is_truthy()).unwrap_or(false);
            let msg = args
                .get(1)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "Expected false".into());
            okanran.beko(cond, &msg)?;
            Ok(IfaValue::bool(true))
        }
        "ko_si" | "assert_not_null" => {
            let val = args.first().unwrap_or(&IfaValue::Null);
            let name = args
                .get(1)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "value".into());
            okanran.ko_si(val, &name)?;
            Ok(IfaValue::bool(true))
        }
        "ku_bi" | "fatal" => {
            let msg = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "fatal error".into());
            Err(IfaError::Custom(format!("[FATAL] {}", msg)))
        }
        "ko_le_de_bi" | "unreachable" => {
            okanran.ko_le_de_bi()?;
            Ok(IfaValue::Null)
        }
        "ko_ti_se_bi" | "not_implemented" => {
            let feat = args.first().map(|v| v.to_string()).unwrap_or_default();
            okanran.ko_ti_se_bi(&feat)?;
            Ok(IfaValue::Null)
        }
        "owe" | "proverb" => {
            let err_msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            let dummy_err = IfaError::Custom(err_msg);
            Ok(IfaValue::str(dummy_err.proverb()))
        }
        "gbiyanju" | "try_or" => {
            let action = args
                .first()
                .ok_or_else(|| IfaError::ArgumentError("try_or expects (fn, default)".into()))?
                .clone();
            let default_val = args.get(1).cloned().unwrap_or(IfaValue::Null);
            match ctx.call_value(action, vec![]) {
                Ok(res) => Ok(res),
                Err(IfaError::Exit(code)) => Err(IfaError::Exit(code)),
                Err(_) => Ok(default_val),
            }
        }
        "wo" | "debug" => {
            let val = args.first().unwrap_or(&IfaValue::Null);
            let label = args
                .get(1)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "value".into());
            okanran.wo(&label, val);
            Ok(val.clone())
        }
        "ku" | "panic" => {
            let msg = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "panic".into());
            Err(IfaError::Runtime(format!("[Ọ̀kànràn] {}", msg)))
        }
        "ko_le_de" | "unreachable_panic" => Err(IfaError::Runtime(
            "[Ọ̀kànràn] Unreachable code executed!".into(),
        )),
        "ko_ti_se" | "not_implemented_panic" => {
            let feat = args.first().map(|v| v.to_string()).unwrap_or_default();
            Err(IfaError::Runtime(format!(
                "Ẹ̀yà '{}' kò tíì ṣé (Feature '{}' is not yet implemented)",
                feat, feat
            )))
        }
        _ => Err(IfaError::Custom(format!(
            "Okanran: unknown method '{}'",
            method
        ))),
    }
}

fn dispatch_ose(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    #[cfg(feature = "tui")]
    {
        match method {
            "nu" | "clear" => {
                if let Some(IfaValue::Resource(token_arc)) = args.first() {
                    let token = **token_arc;
                    if let Some(terminal_mutex) =
                        ctx.resource_registry()
                            .get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token)
                    {
                        let mut terminal = terminal_mutex
                            .lock()
                            .map_err(|_| IfaError::Runtime("Terminal lock poisoned".into()))?;
                        terminal
                            .clear()
                            .map_err(|e| IfaError::Runtime(e.to_string()))?;
                    }
                }
                Ok(IfaValue::null())
            }
            "wo" | "debug" => {
                // Return value for inspection, do not pollute stdout
                Ok(args.first().cloned().unwrap_or(IfaValue::Null))
            }
            _ => crate::odu::ose::Ose::dispatch(method, args, ctx),
        }
    }
    #[cfg(not(feature = "tui"))]
    {
        let _ = args;
        let _ = ctx;
        match method {
            "bere" | "init" => Ok(IfaValue::str("terminal")),
            "pari" | "end" => Ok(IfaValue::null()),
            "gbile" | "read_key" => Ok(IfaValue::str("q")),
            "apoti" | "box" => Ok(IfaValue::null()),
            "ipinro" | "section" => Ok(IfaValue::null()),
            "ya" | "draw" => Ok(IfaValue::null()),
            "nu" | "clear" => Ok(IfaValue::null()),
            "wo" | "debug" => Ok(args.first().cloned().unwrap_or(IfaValue::Null)),
            "gboran" | "listen" => Ok(IfaValue::null()),
            "ipile" | "layout" => {
                let mut m = std::collections::HashMap::new();
                m.insert("x".into(), IfaValue::int(0));
                m.insert("y".into(), IfaValue::int(0));
                m.insert("width".into(), IfaValue::int(80));
                m.insert("height".into(), IfaValue::int(24));
                Ok(IfaValue::list(vec![IfaValue::map(m)]))
            }
            "ẹmí" | "mouse_on" => Ok(IfaValue::null()),
            "pari_ẹmí" | "mouse_off" => Ok(IfaValue::null()),
            "iwọn" | "size" => {
                let mut map = std::collections::HashMap::new();
                map.insert("cols".into(), IfaValue::int(80));
                map.insert("rows".into(), IfaValue::int(24));
                Ok(IfaValue::map(map))
            }
            "duro" | "wait" => {
                let mut map = std::collections::HashMap::new();
                map.insert("type".into(), IfaValue::str("timeout"));
                Ok(IfaValue::map(map))
            }
            _ => Err(IfaError::Custom(format!(
                "Ose: unknown method '{}'",
                method
            ))),
        }
    }
}

impl StdRegistry {
    fn dispatch_ogunda(
        &self,
        method: &str,
        mut args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        let ogunda = crate::odu::ogunda::Ogunda::new(self.esu.clone());
        match method {
            "iwọn" | "iwon" | "len" | "count" | "apapo" => {
                if let Some(IfaValue::List(list)) = args.first() {
                    Ok(IfaValue::int(ogunda.iwon(list) as i64))
                } else {
                    Ok(IfaValue::int(0))
                }
            }
            "fi" | "push" | "append" => {
                if args.len() < 2 {
                    return Err(IfaError::ArgumentError(
                        "push/fi expects (list, item)".into(),
                    ));
                }
                let val = args[1].clone();
                if let IfaValue::List(ref mut list_arc) = args[0] {
                    let vec = ifa_types::gc::IfaGc::make_mut(list_arc);
                    ogunda.fi(vec, val);
                    Ok(IfaValue::Null)
                } else {
                    Err(IfaError::TypeError {
                        expected: "List".into(),
                        got: args[0].type_name().into(),
                    })
                }
            }
            "mu" | "pop" => {
                if let IfaValue::List(ref mut list_arc) = args[0] {
                    let vec = ifa_types::gc::IfaGc::make_mut(list_arc);
                    Ok(ogunda.mu(vec).unwrap_or(IfaValue::Null))
                } else {
                    Err(IfaError::TypeError {
                        expected: "List".into(),
                        got: args[0].type_name().into(),
                    })
                }
            }
            "yi_pada" | "yipada" | "map" | "maapu" => {
                if args.len() < 2 {
                    return Err(IfaError::ArgumentError(
                        "map expects (list, closure)".into(),
                    ));
                }
                let closure = args[1].clone();
                let list = match &args[0] {
                    IfaValue::List(l) => l.clone(),
                    other => {
                        return Err(IfaError::TypeError {
                            expected: "List".into(),
                            got: other.type_name().into(),
                        });
                    }
                };
                let mut results = Vec::with_capacity(list.len());
                for item in list.iter() {
                    let mapped = ctx.call_value(closure.clone(), vec![item.clone()])?;
                    results.push(mapped);
                }
                Ok(IfaValue::list(results))
            }
            "yan" | "filter" | "sajo" | "ṣàjọ" => {
                if args.len() < 2 {
                    return Err(IfaError::ArgumentError(
                        "filter expects (list, closure)".into(),
                    ));
                }
                let closure = args[1].clone();
                let list = match &args[0] {
                    IfaValue::List(l) => l.clone(),
                    other => {
                        return Err(IfaError::TypeError {
                            expected: "List".into(),
                            got: other.type_name().into(),
                        });
                    }
                };
                let mut results = Vec::new();
                for item in list.iter() {
                    let keep = ctx.call_value(closure.clone(), vec![item.clone()])?;
                    if keep.is_truthy() {
                        results.push(item.clone());
                    }
                }
                Ok(IfaValue::list(results))
            }
            "ṣẹ́kù" | "seku" | "din" | "reduce" | "fold" => {
                if args.len() < 2 {
                    return Err(IfaError::ArgumentError(
                        "reduce expects (list, closure) or (list, initial, closure)".into(),
                    ));
                }
                let list = match &args[0] {
                    IfaValue::List(l) => l.clone(),
                    other => {
                        return Err(IfaError::TypeError {
                            expected: "List".into(),
                            got: other.type_name().into(),
                        });
                    }
                };

                if args.len() == 2 {
                    let closure = args[1].clone();
                    if list.is_empty() {
                        return Err(IfaError::Custom(
                            "Cannot reduce empty list with no initial value".into(),
                        ));
                    }
                    let mut acc = list[0].clone();
                    for item in list.iter().skip(1) {
                        acc = ctx.call_value(closure.clone(), vec![acc, item.clone()])?;
                    }
                    Ok(acc)
                } else {
                    let initial = args[1].clone();
                    let closure = args[2].clone();
                    let mut acc = initial;
                    for item in list.iter() {
                        acc = ctx.call_value(closure.clone(), vec![acc, item.clone()])?;
                    }
                    Ok(acc)
                }
            }
            "ge" | "alloc" => {
                let list = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "ge (alloc/slice) expects a list".into(),
                        ));
                    }
                };
                let start = args.get(1).and_then(as_int).unwrap_or(0) as usize;
                let end = args.get(2).and_then(as_int).unwrap_or(list.len() as i64) as usize;
                let sliced = ogunda.ge(list, start, end)?;
                Ok(IfaValue::list(sliced))
            }
            "da" | "create" | "seda" | "new_list" => {
                let mut list = ogunda.seda::<IfaValue>();
                for arg in args {
                    list.push(arg);
                }
                Ok(IfaValue::list(list))
            }
            "seda_agbara" | "with_capacity" => {
                let cap = args.first().and_then(as_int).unwrap_or(0) as usize;
                Ok(IfaValue::list(ogunda.seda_agbara::<IfaValue>(cap)))
            }
            "sofo" | "is_empty" => {
                let list = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => return Ok(IfaValue::bool(true)),
                };
                Ok(IfaValue::bool(ogunda.sofo(list)))
            }
            "pada" | "reverse" => {
                let list = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "pada (reverse) expects a list".into(),
                        ));
                    }
                };
                Ok(IfaValue::list(ogunda.pada(list)))
            }
            "to" | "sort" => {
                let list = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => return Err(IfaError::ArgumentError("to (sort) expects a list".into())),
                };
                let mut sorted = list.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Ok(IfaValue::list(sorted))
            }
            "dapo" | "concat" => {
                let a = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "dapo (concat) expects two lists".into(),
                        ));
                    }
                };
                let b = match args.get(1) {
                    Some(IfaValue::List(l)) => l,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "dapo (concat) expects two lists".into(),
                        ));
                    }
                };
                Ok(IfaValue::list(ogunda.dapo(a, b)))
            }
            "wa" | "find" => {
                let list = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "wa (find) expects (list, closure)".into(),
                        ));
                    }
                };
                let closure = args
                    .get(1)
                    .ok_or_else(|| {
                        IfaError::ArgumentError("wa (find) expects (list, closure)".into())
                    })?
                    .clone();
                for item in list.iter() {
                    let keep = ctx.call_value(closure.clone(), vec![item.clone()])?;
                    if keep.is_truthy() {
                        return Ok(item.clone());
                    }
                }
                Ok(IfaValue::Null)
            }
            "eyikeyi" | "any" => {
                let list = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "eyikeyi (any) expects (list, closure)".into(),
                        ));
                    }
                };
                let closure = args
                    .get(1)
                    .ok_or_else(|| {
                        IfaError::ArgumentError("eyikeyi (any) expects (list, closure)".into())
                    })?
                    .clone();
                for item in list.iter() {
                    let keep = ctx.call_value(closure.clone(), vec![item.clone()])?;
                    if keep.is_truthy() {
                        return Ok(IfaValue::bool(true));
                    }
                }
                Ok(IfaValue::bool(false))
            }
            "gbogbo" | "all" => {
                let list = match args.first() {
                    Some(IfaValue::List(l)) => l,
                    _ => {
                        return Err(IfaError::ArgumentError(
                            "gbogbo (all) expects (list, closure)".into(),
                        ));
                    }
                };
                let closure = args
                    .get(1)
                    .ok_or_else(|| {
                        IfaError::ArgumentError("gbogbo (all) expects (list, closure)".into())
                    })?
                    .clone();
                for item in list.iter() {
                    let keep = ctx.call_value(closure.clone(), vec![item.clone()])?;
                    if !keep.is_truthy() {
                        return Ok(IfaValue::bool(false));
                    }
                }
                Ok(IfaValue::bool(true))
            }
            "awon_kokoro" | "keys" => {
                let map = args
                    .first()
                    .ok_or_else(|| IfaError::ArgumentError("keys expects a map".into()))?;
                let keys = ogunda.awon_kokoro(map)?;
                Ok(IfaValue::list(
                    keys.into_iter().map(IfaValue::str).collect(),
                ))
            }
            "awon_iye" | "values" => {
                let map = args
                    .first()
                    .ok_or_else(|| IfaError::ArgumentError("values expects a map".into()))?;
                let values = ogunda.awon_iye(map)?;
                Ok(IfaValue::list(values))
            }
            "awon_nkan" | "items" => {
                let map = args
                    .first()
                    .ok_or_else(|| IfaError::ArgumentError("items expects a map".into()))?;
                let items = ogunda.awon_nkan(map)?;
                let list_items: Vec<IfaValue> = items.into_iter().map(IfaValue::list).collect();
                Ok(IfaValue::list(list_items))
            }
            "yo" | "remove" => {
                let key = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                let map = args
                    .first_mut()
                    .ok_or_else(|| IfaError::ArgumentError("remove expects a map".into()))?;
                ogunda.yo(map, &key)
            }
            "sise" | "run" => {
                let cmd = args.first().map(|v| v.to_string()).unwrap_or_default();
                let cmd_args_strs = if let Some(IfaValue::List(l)) = args.get(1) {
                    l.iter().map(|v| v.to_string()).collect::<Vec<_>>()
                } else {
                    args.iter()
                        .skip(1)
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                };
                let cmd_args_ref: Vec<&str> = cmd_args_strs.iter().map(|s| s.as_str()).collect();
                let output = ogunda.sise(&cmd, &cmd_args_ref)?;
                let mut res_map = std::collections::HashMap::new();
                res_map.insert(
                    "code".into(),
                    IfaValue::int(output.status.code().unwrap_or(0) as i64),
                );
                res_map.insert(
                    "stdout".into(),
                    IfaValue::str(String::from_utf8_lossy(&output.stdout).into_owned()),
                );
                res_map.insert(
                    "stderr".into(),
                    IfaValue::str(String::from_utf8_lossy(&output.stderr).into_owned()),
                );
                Ok(IfaValue::Map(res_map.into()))
            }
            "sise_ka" | "run_read" => {
                let cmd = args.first().map(|v| v.to_string()).unwrap_or_default();
                let cmd_args_strs = if let Some(IfaValue::List(l)) = args.get(1) {
                    l.iter().map(|v| v.to_string()).collect::<Vec<_>>()
                } else {
                    args.iter()
                        .skip(1)
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                };
                let cmd_args_ref: Vec<&str> = cmd_args_strs.iter().map(|s| s.as_str()).collect();
                let stdout = ogunda.sise_ka(&cmd, &cmd_args_ref)?;
                Ok(IfaValue::str(stdout))
            }
            "bere" | "spawn" => {
                let cmd = args.first().map(|v| v.to_string()).unwrap_or_default();
                let cmd_args_strs = if let Some(IfaValue::List(l)) = args.get(1) {
                    l.iter().map(|v| v.to_string()).collect::<Vec<_>>()
                } else {
                    args.iter()
                        .skip(1)
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                };
                let cmd_args_ref: Vec<&str> = cmd_args_strs.iter().map(|s| s.as_str()).collect();
                let child_id = ogunda.bere(&cmd, &cmd_args_ref)?;
                Ok(IfaValue::int(child_id as i64))
            }
            "ayika" | "get_env" => {
                let key = args.first().map(|v| v.to_string()).unwrap_or_default();
                match ogunda.ayika(&key)? {
                    Some(val) => Ok(IfaValue::str(val)),
                    None => Ok(IfaValue::Null),
                }
            }
            _ => Err(IfaError::Custom(format!(
                "Ogunda: unknown method '{}'",
                method
            ))),
        }
    }

    fn dispatch_ofun(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        let ofun = crate::odu::ofun::Ofun::with_capabilities(self.esu.world_state());
        match method {
            "le" | "can" => {
                let cap = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::bool(ofun.le(&cap)))
            }
            "da" | "create" => {
                let name = args
                    .first()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "dummy_sandbox_resource".to_string());
                let token = ctx.resource_registry().register(name);
                Ok(IfaValue::Resource(std::sync::Arc::new(token)))
            }
            "pa" | "delete" => {
                if let Some(IfaValue::Resource(token_arc)) = args.first() {
                    let closed = ctx.resource_registry().close(**token_arc);
                    Ok(IfaValue::bool(closed))
                } else {
                    Err(IfaError::ArgumentError(
                        "pa (delete) expects a Resource token".into(),
                    ))
                }
            }
            "iru" | "type_of" | "typeof" => {
                let val = args.first().unwrap_or(&IfaValue::Null);
                Ok(IfaValue::str(ofun.iru(val)))
            }
            "laaye" | "is_alive" => {
                if let Some(IfaValue::Resource(token_arc)) = args.first() {
                    Ok(IfaValue::bool(
                        ctx.resource_registry().contains(**token_arc),
                    ))
                } else {
                    Err(IfaError::ArgumentError(
                        "laaye (is_alive) expects a Resource token".into(),
                    ))
                }
            }
            "ju" | "drop" => {
                let cap_str = args.first().map(|v| v.to_string()).unwrap_or_default();
                match cap_str.as_str() {
                    "read" | "ka" => self
                        .esu
                        .ju_match(|c| matches!(c, crate::sandbox_shim::Ofun::ReadFiles { .. })),
                    "write" | "ko" => self
                        .esu
                        .ju_match(|c| matches!(c, crate::sandbox_shim::Ofun::WriteFiles { .. })),
                    "network" | "nẹtiwọki" => self
                        .esu
                        .ju_match(|c| matches!(c, crate::sandbox_shim::Ofun::Network { .. })),
                    "spawn" | "bere" => self
                        .esu
                        .ju_match(|c| matches!(c, crate::sandbox_shim::Ofun::Execute { .. })),
                    "env" | "ayika" => self
                        .esu
                        .ju_match(|c| matches!(c, crate::sandbox_shim::Ofun::Environment { .. })),
                    "crypto" | "irete" => self
                        .esu
                        .ju_match(|c| matches!(c, crate::sandbox_shim::Ofun::Crypto)),
                    s if s.starts_with("bridge:") => {
                        let lang = s[7..].to_string();
                        self.esu.ju_match(move |c| {
                            if let crate::sandbox_shim::Ofun::Bridge { language } = c {
                                language == &lang
                            } else {
                                false
                            }
                        });
                    }
                    _ => {}
                }
                Ok(IfaValue::Null)
            }
            "awon_agbara" | "capabilities" => {
                let list: Vec<IfaValue> = ofun
                    .awon_agbara()
                    .all()
                    .iter()
                    .map(|cap| {
                        let s = match cap {
                            crate::sandbox_shim::Ofun::ReadFiles { root } => {
                                format!("read:{}", root.display())
                            }
                            crate::sandbox_shim::Ofun::WriteFiles { root } => {
                                format!("write:{}", root.display())
                            }
                            crate::sandbox_shim::Ofun::Network { domains } => {
                                format!("network:{}", domains.join(","))
                            }
                            crate::sandbox_shim::Ofun::Execute { programs } => {
                                format!("execute:{}", programs.join(","))
                            }
                            crate::sandbox_shim::Ofun::Environment { keys } => {
                                format!("env:{}", keys.join(","))
                            }
                            crate::sandbox_shim::Ofun::Time => "time".to_string(),
                            crate::sandbox_shim::Ofun::Random => "random".to_string(),
                            crate::sandbox_shim::Ofun::Stdio => "stdio".to_string(),
                            crate::sandbox_shim::Ofun::Crypto => "crypto".to_string(),
                            crate::sandbox_shim::Ofun::Bridge { language } => {
                                format!("bridge:{}", language)
                            }
                        };
                        IfaValue::str(s)
                    })
                    .collect();
                Ok(IfaValue::list(list))
            }
            "je" | "is_type" => {
                if args.len() < 2 {
                    return Err(IfaError::ArgumentError(
                        "is_type expects (value, type_name)".into(),
                    ));
                }
                let val = &args[0];
                let expected_type = args[1].to_string();
                Ok(IfaValue::bool(ofun.je(val, &expected_type)))
            }
            "afiwe" | "debug" | "inspect" => {
                let val = args.first().unwrap_or(&IfaValue::Null);
                Ok(IfaValue::str(ofun.afiwe(val)))
            }
            "dbg" | "debug_print" => {
                if let Some(val) = args.first() {
                    Ok(val.clone())
                } else {
                    Err(IfaError::ArgumentError(
                        "Ofun.dbg expects 1 argument".into(),
                    ))
                }
            }
            _ => Err(IfaError::Custom(format!(
                "Ofun: unknown method '{}'",
                method
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// F1: Kernel / Sys (Domain 29)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// F5: Storage (Domain 20) — FutureCell via StorageWorker
// ---------------------------------------------------------------------------

impl StdRegistry {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_par_map_applies_named_numeric_operation() {
        let mut vm = ifa_vm::vm::IfaVM::new();
        let bytecode = ifa_vm::bytecode::Bytecode::new("test");
        let mut ctx = ifa_vm::native::VmContext {
            vm: &mut vm,
            bytecode: &bytecode,
        };

        // 1. Allocate buffer of size 3
        let buf_val =
            crate::hardware::cpu::dispatch("alloc_buffer", vec![IfaValue::Int(3)], &mut ctx)
                .expect("alloc_buffer should succeed");

        // 2. Write [1, 2, 3] into the buffer
        crate::hardware::cpu::dispatch(
            "write_buffer",
            vec![
                buf_val.clone(),
                IfaValue::list(vec![IfaValue::Int(1), IfaValue::Int(2), IfaValue::Int(3)]),
            ],
            &mut ctx,
        )
        .expect("write_buffer should succeed");

        // 3. par_map with square
        let mapped_buf = crate::hardware::cpu::dispatch(
            "par_map",
            vec![buf_val, IfaValue::str("square")],
            &mut ctx,
        )
        .expect("par_map should succeed");

        // 4. Read buffer back
        let result = crate::hardware::cpu::dispatch("read_buffer", vec![mapped_buf], &mut ctx)
            .expect("read_buffer should succeed");

        assert_eq!(
            result,
            IfaValue::list(vec![
                IfaValue::float(1.0),
                IfaValue::float(4.0),
                IfaValue::float(9.0),
            ])
        );
    }

    #[test]
    fn cpu_par_reduce_sums_numeric_lists() {
        let mut vm = ifa_vm::vm::IfaVM::new();
        let bytecode = ifa_vm::bytecode::Bytecode::new("test");
        let mut ctx = ifa_vm::native::VmContext {
            vm: &mut vm,
            bytecode: &bytecode,
        };

        // 1. Allocate buffer of size 3
        let buf_val =
            crate::hardware::cpu::dispatch("alloc_buffer", vec![IfaValue::Int(3)], &mut ctx)
                .expect("alloc_buffer should succeed");

        // 2. Write [1.5, 2.5, 3] into the buffer
        crate::hardware::cpu::dispatch(
            "write_buffer",
            vec![
                buf_val.clone(),
                IfaValue::list(vec![
                    IfaValue::float(1.5),
                    IfaValue::float(2.5),
                    IfaValue::Int(3),
                ]),
            ],
            &mut ctx,
        )
        .expect("write_buffer should succeed");

        // 3. par_reduce with sum
        let result = crate::hardware::cpu::dispatch(
            "par_reduce",
            vec![buf_val, IfaValue::str("sum")],
            &mut ctx,
        )
        .expect("par_reduce should succeed");

        assert_eq!(result, IfaValue::float(7.0));
    }
}
