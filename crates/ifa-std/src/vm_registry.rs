//! # StdRegistry - VM OduRegistry Implementation
//!
//! Bridges the ifa-std domain structs to the VM's OduRegistry trait,
//! enabling `CallOdu` opcodes to dispatch to the standard library.

use ifa_vm::IfaValue;
use ifa_vm::error::{IfaError, IfaResult};
use ifa_vm::native::{OduRegistry, VmContext};
use ifa_types::value_union::FutureState;
use std::sync::OnceLock;

#[cfg(feature = "crypto")]
use crate::odu::irete::Irete;
use crate::esu::Esu;
use crate::odu::irosu::Irosu;
use crate::odu::odi::Odi;
#[cfg(all(feature = "backend", feature = "full"))]
use crate::odu::otura::Otura;
use crate::sandbox_shim::CapabilitySet;

// ---------------------------------------------------------------------------
// StdRegistry
// ---------------------------------------------------------------------------

/// Standard library registry for the bytecode VM.
pub struct StdRegistry {
    irosu: OnceLock<Irosu>,
    odi: OnceLock<Odi>,
    #[cfg(all(feature = "backend", feature = "full"))]
    otura: OnceLock<Otura>,
    #[cfg(feature = "crypto")]
    irete: OnceLock<Irete>,
    storage: crate::hardware::storage::StorageWorker,
    esu: Esu,
}

impl StdRegistry {
    pub fn new() -> Self {
        Self::new_with_caps(CapabilitySet::new())
    }

    fn new_with_caps(caps: CapabilitySet) -> Self {
        let esu = Esu::new(caps.clone());
        Self {
            irosu: OnceLock::new(),
            odi: OnceLock::new(),
            #[cfg(all(feature = "backend", feature = "full"))]
            otura: OnceLock::new(),
            #[cfg(feature = "crypto")]
            irete: OnceLock::new(),
            storage: crate::hardware::storage::StorageWorker::new(),
            esu,
        }
    }

    pub fn set_capabilities(&mut self, caps: CapabilitySet) {
        *self = Self::new_with_caps(caps);
    }

    fn irosu(&self) -> &Irosu {
        self.irosu
            .get_or_init(|| Irosu::new(self.esu.world_state()))
    }

    fn odi(&self) -> &Odi {
        self.odi.get_or_init(|| Odi::new(self.esu.clone()))
    }

    #[cfg(all(feature = "backend", feature = "full"))]
    fn otura(&self) -> &Otura {
        self.otura.get_or_init(|| Otura::new(self.esu.clone()))
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
    fn clone_registry(&self) -> Box<dyn OduRegistry> {
        Box::new(StdRegistry {
            irosu: OnceLock::new(),
            odi: OnceLock::new(),
            #[cfg(all(feature = "backend", feature = "full"))]
            otura: OnceLock::new(),
            #[cfg(feature = "crypto")]
            irete: OnceLock::new(),
            storage: self.storage.clone(),
            esu: Esu::new(self.esu.world_state()),
        })
    }

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
            4 => self.dispatch_irosu(method_name, args, ctx),
            5 => dispatch_owonrin(method_name, args),
            6 => dispatch_obara(method_name, args),
            7 => dispatch_okanran(method_name, args),
            8 => dispatch_ogunda(method_name, args, ctx),
            9 => dispatch_osa(method_name, args, ctx),
            10 => dispatch_ika(method_name, args),
            11 => dispatch_oturupon(method_name, args),
            #[cfg(feature = "backend")]
            12 => self.dispatch_otura(method_name, args),
            #[cfg(feature = "crypto")]
            13 => self.dispatch_irete(method_name, args),
            14 => dispatch_ose(method_name, args, ctx),
            15 => dispatch_ofun(method_name, args),
            18 => crate::hardware::cpu::dispatch(method_name, args, ctx),
            19 => {
                #[cfg(feature = "gpu")]
                {
                    match method_name {
                        "init" => crate::hardware::gpu::handle_init(args, ctx),
                        "dispatch_pipeline" | "dispatch" => crate::hardware::gpu::handle_dispatch_pipeline(args, ctx),
                        "sync" => crate::hardware::gpu::handle_sync(args, ctx),
                        "alloc_buffer" => crate::hardware::gpu::handle_alloc_buffer(args, ctx),
                        "read_buffer" => crate::hardware::gpu::handle_read_buffer(args, ctx),
                        "write_buffer" => crate::hardware::gpu::handle_write_buffer(args, ctx),
                        _ => Err(IfaError::Custom(format!("Unknown gpu method: {}", method_name))),
                    }
                }
                #[cfg(not(feature = "gpu"))]
                {
                    Err(IfaError::Runtime("GPU disabled".into()))
                }
            },
            20 => crate::hardware::storage::dispatch(&self.storage, method_name, args),
            29 => crate::hardware::sys::dispatch(method_name, args),
            _ => Err(IfaError::Custom(format!(
                "Unknown Odù domain ID: {}",
                domain_id
            ))),
        }
    }

    #[inline]
    fn call_fast(
        &self,
        domain_id: u8,
        method_id: u16,
        args: Vec<IfaValue>,
        ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        let low = (method_id & 0xFF) as u8;
        match domain_id {
            0 => match low {
                0x01 => dispatch_ogbe("bere", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            1 => match low {
                0x01 => dispatch_oyeku("jade", args),
                0x02 => dispatch_oyeku("sun", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            2 => match low {
                0x01 => dispatch_iwori("bayi", args),
                0x02 => dispatch_iwori("akoko", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            3 => match low {
                0x01 => self.dispatch_odi("ka", args),
                0x02 => self.dispatch_odi("ko", args),
                0x03 => self.dispatch_odi("wa", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            4 => match low {
                0x01 => self.dispatch_irosu("fo", args, ctx),
                0x02 => self.dispatch_irosu("so", args, ctx),
                0x03 => self.dispatch_irosu("gbo", args, ctx),
                0x04 => self.dispatch_irosu("gbo_nomba", args, ctx),
                0x05 => self.dispatch_irosu("mo", args, ctx),
                0x06 => self.dispatch_irosu("san", args, ctx),
                0x07 => self.dispatch_irosu("kigbe", args, ctx),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            5 => match low {
                0x01 => dispatch_owonrin("pese", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            6 => match low {
                0x01 => dispatch_obara("fikun", args),
                0x02 => dispatch_obara("isodipupo", args),
                0x03 => dispatch_obara("agbara", args),
                0x04 => dispatch_obara("gbongbo", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            7 => match low {
                0x01 => dispatch_okanran("sise", args),
                0x02 => dispatch_okanran("kigbe", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            8 => match low {
                0x01 => dispatch_ogunda("iwon", args, ctx),
                0x02 => dispatch_ogunda("fi", args, ctx),
                0x03 => dispatch_ogunda("mu", args, ctx),
                0x04 => dispatch_ogunda("yi_pada", args, ctx),
                0x05 => dispatch_ogunda("yan", args, ctx),
                0x06 => dispatch_ogunda("seku", args, ctx),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            9 => match low {
                0x01 => dispatch_osa("sun", args, ctx),
                0x02 => dispatch_osa("gbogbo", args, ctx),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            10 => match low {
                0x01 => dispatch_ika("gigun", args),
                0x02 => dispatch_ika("ge", args),
                0x03 => dispatch_ika("so", args),
                0x04 => dispatch_ika("oruko_html", args),
                0x05 => dispatch_ika("tumo_html", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            11 => match low {
                0x01 => dispatch_oturupon("yokuro", args),
                0x02 => dispatch_oturupon("pipin", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            12 => match low {
                0x01 => { #[cfg(feature = "backend")] { self.dispatch_otura("gba", args) } #[cfg(not(feature = "backend"))] { Err(IfaError::Runtime(format!("Otura requires backend feature")) ) } },
                0x02 => { #[cfg(feature = "backend")] { self.dispatch_otura("ran", args) } #[cfg(not(feature = "backend"))] { Err(IfaError::Runtime(format!("Otura requires backend feature")) ) } },
                0x03 => { #[cfg(feature = "backend")] { self.dispatch_otura("de", args) } #[cfg(not(feature = "backend"))] { Err(IfaError::Runtime(format!("Otura requires backend feature")) ) } },
                0x04 => { #[cfg(feature = "backend")] { self.dispatch_otura("soro", args) } #[cfg(not(feature = "backend"))] { Err(IfaError::Runtime(format!("Otura requires backend feature")) ) } },
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            13 => match low {
                0x01 => { #[cfg(feature = "crypto")] { self.dispatch_irete("hash", args) } #[cfg(not(feature = "crypto"))] { Err(IfaError::Runtime(format!("Irete requires crypto feature")) ) } },
                0x02 => { #[cfg(feature = "crypto")] { self.dispatch_irete("hmac", args) } #[cfg(not(feature = "crypto"))] { Err(IfaError::Runtime(format!("Irete requires crypto feature")) ) } },
                0x03 => { #[cfg(feature = "crypto")] { self.dispatch_irete("base64", args) } #[cfg(not(feature = "crypto"))] { Err(IfaError::Runtime(format!("Irete requires crypto feature")) ) } },
                0x04 => { #[cfg(feature = "crypto")] { self.dispatch_irete("decode", args) } #[cfg(not(feature = "crypto"))] { Err(IfaError::Runtime(format!("Irete requires crypto feature")) ) } },
                0x05 => { #[cfg(feature = "crypto")] { self.dispatch_irete("funpo", args) } #[cfg(not(feature = "crypto"))] { Err(IfaError::Runtime(format!("Irete requires crypto feature")) ) } },
                0x06 => { #[cfg(feature = "crypto")] { self.dispatch_irete("tu", args) } #[cfg(not(feature = "crypto"))] { Err(IfaError::Runtime(format!("Irete requires crypto feature")) ) } },
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            14 => match low {
                0x01 => dispatch_ose("bere", args, ctx),
                0x02 => dispatch_ose("pari", args, ctx),
                0x03 => dispatch_ose("gbile", args, ctx),
                0x04 => dispatch_ose("apoti", args, ctx),
                0x05 => dispatch_ose("ipinro", args, ctx),
                0x06 => dispatch_ose("ya", args, ctx),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            15 => match low {
                0x01 => dispatch_ofun("le", args),
                0x02 => dispatch_ofun("dbg", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            18 => {
                let name = ifa_types::methods::method_name_from_id(18, method_id)
                    .ok_or_else(|| IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id)))?;
                crate::hardware::cpu::dispatch(name, args, ctx)
            },
            19 => {
                #[cfg(feature = "gpu")]
                {
                    match low {
                        0x01 => crate::hardware::gpu::handle_init(args, ctx),
                        0x02 => crate::hardware::gpu::handle_dispatch_pipeline(args, ctx),
                        0x03 => crate::hardware::gpu::handle_sync(args, ctx),
                        0x04 => crate::hardware::gpu::handle_alloc_buffer(args, ctx),
                        0x05 => crate::hardware::gpu::handle_read_buffer(args, ctx),
                        0x06 => crate::hardware::gpu::handle_write_buffer(args, ctx),
                        _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
                    }
                }
                #[cfg(not(feature = "gpu"))]
                {
                    Err(IfaError::Runtime("GPU disabled".into()))
                }
            },
            20 => match low {
                0x01 => crate::hardware::storage::dispatch(&self.storage, "open", args),
                0x02 => crate::hardware::storage::dispatch(&self.storage, "get", args),
                0x03 => crate::hardware::storage::dispatch(&self.storage, "set", args),
                0x04 => crate::hardware::storage::dispatch(&self.storage, "delete", args),
                0x05 => crate::hardware::storage::dispatch(&self.storage, "compact", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            29 => match low {
                0x01 => crate::hardware::sys::dispatch("num_cores", args),
                0x02 => crate::hardware::sys::dispatch("total_memory", args),
                0x03 => crate::hardware::sys::dispatch("available_memory", args),
                0x04 => crate::hardware::sys::dispatch("uptime", args),
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            },
            _ => Err(IfaError::Custom(format!("Unknown Odù domain ID: {}", domain_id))),
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
    #[cfg(feature = "backend")]
    fn dispatch_otura(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        #[cfg(feature = "full")]
        {
            match method {
                "gba" | "get" | "fetch" => {
                    let url = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|e| IfaError::Runtime(format!("Otura runtime init failed: {e}")))?;
                    let result = rt.block_on(self.otura().gba(&url))?;
                    Ok(IfaValue::str(result))
                }
                "ran" | "post" => {
                    let url = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let body = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|e| IfaError::Runtime(format!("Otura runtime init failed: {e}")))?;
                    let result = rt.block_on(self.otura().ran(&url, &body))?;
                    Ok(IfaValue::str(result))
                }
                "de" | "listen" => Err(IfaError::Runtime(
                    "Otura.de returns a TcpListener and is not exposed through the VM registry".into(),
                )),
                "soro" | "connect" => Err(IfaError::Runtime(
                    "Otura.soro returns a TcpStream and is not exposed through the VM registry".into(),
                )),
                _ => Err(IfaError::Custom(format!(
                    "Otura: unknown method '{}'",
                    method
                ))),
            }
        }

        #[cfg(not(feature = "full"))]
        {
            let _ = args;
            Err(IfaError::Runtime(format!(
                "Otura requires the 'full' feature (method: {})",
                method
            )))
        }
    }

    #[cfg(not(feature = "backend"))]
    fn dispatch_otura(&self, method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        Err(IfaError::Runtime(format!(
            "Otura requires the 'backend' feature (method: {})",
            method
        )))
    }

    #[cfg(feature = "crypto")]
    fn dispatch_irete(&self, method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        match method {
            "hash" | "sha256" => {
                let data = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::str(self.irete().sha256_hex(data.as_bytes())?))
            }
            "hmac" | "hmac_sha256" => {
                let key = args.first().map(|v| v.to_string()).unwrap_or_default();
                let data = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::list(
                    self.irete().hmac_sha256(key.as_bytes(), data.as_bytes())?
                        .into_iter()
                        .map(|b| IfaValue::int(b as i64))
                        .collect(),
                ))
            }
            "base64" | "base64_encode" => {
                let data = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(IfaValue::str(self.irete().base64_encode(data.as_bytes())?))
            }
            "decode" | "base64_decode" => {
                let data = args.first().map(|v| v.to_string()).unwrap_or_default();
                let bytes = self.irete().base64_decode(&data)?;
                Ok(IfaValue::list(
                    bytes.into_iter().map(|b| IfaValue::int(b as i64)).collect(),
                ))
            }
            "funpo" | "compress" => Err(IfaError::Runtime(
                "Irete.funpo requires a byte-oriented dispatch path".into(),
            )),
            "tu" | "decompress" => Err(IfaError::Runtime(
                "Irete.tu requires a byte-oriented dispatch path".into(),
            )),
            _ => Err(IfaError::Custom(format!(
                "Irete: unknown method '{}'",
                method
            ))),
        }
    }

    #[cfg(not(feature = "crypto"))]
    #[allow(dead_code)]
    fn dispatch_irete(&self, method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        Err(IfaError::Runtime(format!(
            "Irete requires the 'crypto' feature (method: {})",
            method
        )))
    }

    fn dispatch_irosu(&self, method: &str, args: Vec<IfaValue>, _ctx: &mut VmContext) -> IfaResult<IfaValue> {
        match method {
            "fo" | "println" => {
                if let Some(val) = args.first() {
                    self.irosu().fo(val);
                }
                Ok(IfaValue::null())
            }
            "so" | "print" => {
                if let Some(val) = args.first() {
                    self.irosu().so(val);
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
            match args.first() {
                Some(IfaValue::Str(cs)) => Ok(IfaValue::int(cs.char_len() as i64)),
                Some(v) => {
                    let s = v.to_string();
                    Ok(IfaValue::int(IfaValue::unicode_string_len(&s) as i64))
                }
                None => Ok(IfaValue::int(0)),
            }
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
                .unwrap_or(IfaValue::unicode_string_len(&s));
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
        // H2: Osa.egbe(handler) — spawn an isolated actor VM on a new OS thread.
        // `handler` must be a Fn or Closure. Returns an Actor handle value.
        "egbe" | "ẹgbẹ" | "spawn_actor" => {
            let handler = args
                .into_iter()
                .next()
                .ok_or_else(|| IfaError::ArgumentError("Osa.egbe expects a handler function".into()))?;
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
        // H2: Osa.ran(actor, value) — send a message to an actor's inbox.
        // Non-blocking. Returns null on success, errors on full inbox or dead actor.
        "ran" | "send" => {
            let actor = args.get(0).cloned().ok_or_else(|| {
                IfaError::ArgumentError("Osa.ran expects (actor, value)".into())
            })?;
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

fn dispatch_ogunda(method: &str, mut args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    match method {
        "iwọn" | "iwon" | "len" | "count" | "apapo" => {
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
        "yi_pada" | "yipada" | "map" | "maapu" => {
            if args.len() < 2 {
                return Err(IfaError::ArgumentError("map expects (list, closure)".into()));
            }
            let closure = args[1].clone();
            let list = match &args[0] {
                IfaValue::List(l) => l.clone(),
                other => return Err(IfaError::TypeError { expected: "List".into(), got: other.type_name().into() }),
            };
            let mut results = Vec::with_capacity(list.len());
            for item in list.iter() {
                let mapped = ctx.call_value(closure.clone(), vec![item.clone()])?;
                results.push(mapped);
            }
            Ok(IfaValue::List(std::sync::Arc::new(results)))
        }
        "yan" | "filter" | "sajo" | "ṣàjọ" => {
            if args.len() < 2 {
                return Err(IfaError::ArgumentError("filter expects (list, closure)".into()));
            }
            let closure = args[1].clone();
            let list = match &args[0] {
                IfaValue::List(l) => l.clone(),
                other => return Err(IfaError::TypeError { expected: "List".into(), got: other.type_name().into() }),
            };
            let mut results = Vec::new();
            for item in list.iter() {
                let keep = ctx.call_value(closure.clone(), vec![item.clone()])?;
                if keep.is_truthy() {
                    results.push(item.clone());
                }
            }
            Ok(IfaValue::List(std::sync::Arc::new(results)))
        }
        "ṣẹ́kù" | "seku" | "din" | "reduce" | "fold" => {
            if args.len() < 2 {
                return Err(IfaError::ArgumentError("reduce expects (list, closure) or (list, initial, closure)".into()));
            }
            let list = match &args[0] {
                IfaValue::List(l) => l.clone(),
                other => return Err(IfaError::TypeError { expected: "List".into(), got: other.type_name().into() }),
            };
            
            if args.len() == 2 {
                let closure = args[1].clone();
                if list.is_empty() {
                    return Err(IfaError::Custom("Cannot reduce empty list with no initial value".into()));
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
            return Err(IfaError::Exit(code));
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

fn dispatch_ose(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    #[cfg(feature = "game")]
    {
        crate::odu::ose::Ose::dispatch(method, args, ctx)
    }
    #[cfg(not(feature = "game"))]
    {
        let _ = args;
        let _ = ctx;
        // Ose (Graphics/UI) - stub implementations for kiri browser
        match method {
            "bere" | "init" => Ok(IfaValue::str("terminal")),
            "pari" | "end" => Ok(IfaValue::null()),
            "gbile" | "read_key" => Ok(IfaValue::str("q")), // Auto-quit for now
            "apoti" | "box" => Ok(IfaValue::null()),
            "ipinro" | "section" => Ok(IfaValue::null()),
            "ya" | "draw" => Ok(IfaValue::null()),
            _ => Err(IfaError::Custom(format!(
                "Ose: unknown method '{}'",
                method
            ))),
        }
    }
}

fn dispatch_ofun(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
    match method {
        "le" | "can" => Ok(IfaValue::bool(true)),
        "dbg" | "debug" => {
            if let Some(val) = args.first() {
                let debug_str = format!("{:?}", val);
                eprintln!("[dbg] {}", debug_str);
                Ok(val.clone())
            } else {
                Err(IfaError::ArgumentError("Ofun.dbg expects 1 argument".into()))
            }
        }
        _ => Err(IfaError::Custom(format!(
            "Ofun: unknown method '{}'",
            method
        ))),
    }
}

// ---------------------------------------------------------------------------
// F1: Kernel / Sys (Domain 29)
// ---------------------------------------------------------------------------



// ---------------------------------------------------------------------------
// F5: Storage (Domain 20) — FutureCell via StorageWorker
// ---------------------------------------------------------------------------

impl StdRegistry {
}

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
        let buf_val = crate::hardware::cpu::dispatch("alloc_buffer", vec![IfaValue::Int(3)], &mut ctx)
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
        let buf_val = crate::hardware::cpu::dispatch("alloc_buffer", vec![IfaValue::Int(3)], &mut ctx)
            .expect("alloc_buffer should succeed");

        // 2. Write [1.5, 2.5, 3] into the buffer
        crate::hardware::cpu::dispatch(
            "write_buffer",
            vec![
                buf_val.clone(),
                IfaValue::list(vec![IfaValue::float(1.5), IfaValue::float(2.5), IfaValue::Int(3)]),
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

