import re

methods_rs = """
        OduDomain::Ogbe => match method {
            "bere" | "version" => Some(0x0001),
            _ => None,
        },
        OduDomain::Oyeku => match method {
            "jade" | "exit" | "quit" | "halt" => Some(0x0101),
            "sun" | "sleep" | "wait" => Some(0x0102),
            _ => None,
        },
        OduDomain::Iwori => match method {
            "bayi" | "now" | "current" => Some(0x0201),
            "akoko" | "timestamp" => Some(0x0202),
            _ => None,
        },
        OduDomain::Odi => match method {
            "ka" | "read" => Some(0x0301),
            "ko" | "write" => Some(0x0302),
            "wa" | "exists" => Some(0x0303),
            _ => None,
        },
        OduDomain::Irosu => match method {
            "fo" | "println" => Some(0x0401),
            "so" | "print" => Some(0x0402),
            "gbo" | "listen" => Some(0x0403),
            "gbo_nomba" => Some(0x0404),
            "mo" | "clear" => Some(0x0405),
            "san" | "flush" => Some(0x0406),
            "kigbe" | "error" => Some(0x0407),
            _ => None,
        },
        OduDomain::Owonrin => match method {
            "pese" | "random" => Some(0x0501),
            _ => None,
        },
        OduDomain::Obara => match method {
            "fikun" | "add" | "plus" => Some(0x0601),
            "isodipupo" | "mul" | "times" => Some(0x0602),
            "agbara" | "pow" => Some(0x0603),
            "gbongbo" | "sqrt" => Some(0x0604),
            _ => None,
        },
        OduDomain::Okanran => match method {
            "sise" | "assert" | "verify" | "check" => Some(0x0701),
            "kigbe" | "throw" | "panic" | "raise" | "ta" => Some(0x0702),
            _ => None,
        },
        OduDomain::Ogunda => match method {
            "iwon" | "len" | "count" | "apapo" => Some(0x0801),
            "fi" | "push" | "append" => Some(0x0802),
            "mu" | "pop" => Some(0x0803),
            "yi_pada" | "yipada" | "map" | "maapu" => Some(0x0804),
            "yan" | "filter" | "sajo" | "ṣàjọ" => Some(0x0805),
            "ṣẹ́kù" | "seku" | "din" | "reduce" | "fold" => Some(0x0806),
            _ => None,
        },
        OduDomain::Osa => match method {
            "sun" | "sleep" => Some(0x0901),
            "gbogbo" | "all" => Some(0x0902),
            _ => None,
        },
        OduDomain::Ika => match method {
            "gigun" | "len" => Some(0x0A01),
            "ge" | "slice" => Some(0x0A02),
            "so" | "concat" => Some(0x0A03),
            "oruko_html" | "html_title" => Some(0x0A04),
            "tumo_html" | "strip_html" => Some(0x0A05),
            _ => None,
        },
        OduDomain::Oturupon => match method {
            "yokuro" | "sub" | "minus" => Some(0x0B01),
            "pipin" | "div" | "divide" => Some(0x0B02),
            _ => None,
        },
        OduDomain::Otura => match method {
            "gba" | "get" | "fetch" => Some(0x0C01),
            "ran" | "post" => Some(0x0C02),
            "de" | "listen" => Some(0x0C03),
            "soro" | "connect" => Some(0x0C04),
            _ => None,
        },
        OduDomain::Irete => match method {
            "hash" | "sha256" => Some(0x0D01),
            "hmac" | "hmac_sha256" => Some(0x0D02),
            "base64" | "base64_encode" => Some(0x0D03),
            "decode" | "base64_decode" => Some(0x0D04),
            "funpo" | "compress" => Some(0x0D05),
            "tu" | "decompress" => Some(0x0D06),
            _ => None,
        },
        OduDomain::Ose => match method {
            "bere" | "init" => Some(0x0E01),
            "pari" | "end" => Some(0x0E02),
            "gbile" | "read_key" => Some(0x0E03),
            "apoti" | "box" => Some(0x0E04),
            "ipinro" | "section" => Some(0x0E05),
            "ya" | "draw" => Some(0x0E06),
            _ => None,
        },
        OduDomain::Ofun => match method {
            "le" | "can" => Some(0x0F01),
            _ => None,
        },
        OduDomain::Cpu => match method {
            "square" => Some(0x1201),
            "cube" => Some(0x1202),
            "double" => Some(0x1203),
            "increment" | "inc" => Some(0x1204),
            "decrement" | "dec" => Some(0x1205),
            "neg" | "negate" => Some(0x1206),
            "abs" => Some(0x1207),
            "sqrt" => Some(0x1208),
            "positive" => Some(0x1209),
            "negative" => Some(0x120A),
            "nonzero" => Some(0x120B),
            "even" => Some(0x120C),
            "odd" => Some(0x120D),
            "sum" => Some(0x120E),
            "product" | "prod" => Some(0x120F),
            "min" => Some(0x1210),
            "max" => Some(0x1211),
            "configure" => Some(0x1212),
            "threads" | "num_threads" => Some(0x1213),
            "par_sum" => Some(0x1214),
            "par_map" | "map" => Some(0x1215),
            "par_filter" | "filter" => Some(0x1216),
            "par_reduce" | "reduce" => Some(0x1217),
            "par_sort" | "sort" => Some(0x1218),
            "alloc_buffer" => Some(0x1219),
            "read_buffer" => Some(0x121A),
            "write_buffer" => Some(0x121B),
            _ => None,
        },
        OduDomain::Gpu => match method {
            "init" => Some(0x1301),
            "dispatch_pipeline" | "dispatch" => Some(0x1302),
            "sync" => Some(0x1303),
            "alloc_buffer" => Some(0x1304),
            "read_buffer" => Some(0x1305),
            "write_buffer" => Some(0x1306),
            _ => None,
        },
        OduDomain::Storage => match method {
            "open" => Some(0x1401),
            "get" => Some(0x1402),
            "set" => Some(0x1403),
            "delete" | "del" => Some(0x1404),
            "compact" => Some(0x1405),
            _ => None,
        },
        OduDomain::Sys => match method {
            "num_cores" | "cores" => Some(0x1D01),
            "total_memory" | "mem_total" => Some(0x1D02),
            "available_memory" | "mem_available" => Some(0x1D03),
            "uptime" => Some(0x1D04),
            _ => None,
        }
"""

domains = []
curr_domain = None
curr_methods = []

for line in methods_rs.splitlines():
    line = line.strip()
    if line.startswith("OduDomain::"):
        domain_name = line.split(" ")[0].replace("OduDomain::", "")
        if curr_domain:
            domains.append((curr_domain, curr_methods))
        curr_domain = domain_name
        curr_methods = []
    elif "=> Some(" in line:
        parts = line.split("=> Some(")
        aliases_str = parts[0].strip()
        idx_str = parts[1].replace("),", "").strip()
        aliases = [a.strip().strip('"') for a in aliases_str.split("|")]
        
        yoruba = aliases[0]
        # Find english alias
        english = aliases[-1] if len(aliases) > 1 else yoruba
        # Sometimes the english is the second one, sometimes there are multiple yoruba ones.
        
        # Override rules based on user feedback
        # Owonrin pese/random -> from user: Owonrin -> nọmba/random/rand, yan/choice
        
        curr_methods.append({
            "idx": idx_str,
            "aliases": aliases,
            "yoruba": yoruba,
            "english": english
        })
if curr_domain:
    domains.append((curr_domain, curr_methods))

out_rs = """use crate::domain::OduDomain;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct OduMethodInfo {
    pub domain: OduDomain,
    pub method_id: u16,
    pub yoruba: &'static str,
    pub english: &'static str,
    pub aliases: &'static [&'static str],
}

pub static ODU_METHODS: &[(OduDomain, &[OduMethodInfo])] = &[
"""

for domain, methods in domains:
    out_rs += f"    (OduDomain::{domain}, &[\n"
    for m in methods:
        idx = m['idx']
        aliases_rs = ", ".join(f'"{a}"' for a in m['aliases'])
        yoruba = m['yoruba']
        english = m['english']
        
        # Adjust some specific inconsistencies based on user feedback
        if domain == "Okanran":
            if yoruba == "kigbe": yoruba = "ta"
        if domain == "Owonrin":
            if yoruba == "pese": yoruba = "nọmba"
        
        out_rs += f"""        OduMethodInfo {{
            domain: OduDomain::{domain},
            method_id: {idx},
            yoruba: "{yoruba}",
            english: "{english}",
            aliases: &[{aliases_rs}],
        }},\n"""
    out_rs += "    ]),\n"
out_rs += "];\n\n"

out_rs += """
static METHOD_MAP: OnceLock<HashMap<(OduDomain, String), u16>> = OnceLock::new();
static METHOD_ID_MAP: OnceLock<HashMap<(u8, u16), &'static str>> = OnceLock::new();

fn init_maps() {
    let mut map = HashMap::new();
    let mut id_map = HashMap::new();
    for (domain, methods) in ODU_METHODS {
        for method in *methods {
            for alias in method.aliases {
                map.insert((*domain, alias.to_string()), method.method_id);
            }
            if let Some(domain_id) = domain.dispatch_id() {
                id_map.insert((domain_id, method.method_id), method.yoruba);
            }
        }
    }
    let _ = METHOD_MAP.set(map);
    let _ = METHOD_ID_MAP.set(id_map);
}

pub fn resolve_method_id(domain: OduDomain, method: &str) -> Option<u16> {
    if METHOD_MAP.get().is_none() { init_maps(); }
    METHOD_MAP.get().unwrap().get(&(domain, method.to_string())).copied()
}

pub fn method_name_from_id(domain_id: u8, method_id: u16) -> Option<&'static str> {
    if METHOD_ID_MAP.get().is_none() { init_maps(); }
    METHOD_ID_MAP.get().unwrap().get(&(domain_id, method_id)).copied()
}

pub fn is_valid_odu_method(domain: &OduDomain, method: &str) -> bool {
    resolve_method_id(*domain, method).is_some()
}

pub fn odu_methods(domain: &OduDomain) -> Vec<&'static str> {
    ODU_METHODS
        .iter()
        .find(|(d, _)| d == domain)
        .map(|(_, methods)| methods.iter().flat_map(|m| m.aliases.iter().copied()).collect())
        .unwrap_or_default()
}

pub fn get_method_info(domain: &OduDomain, method: &str) -> Option<OduMethodInfo> {
    ODU_METHODS
        .iter()
        .find(|(d, _)| d == domain)
        .and_then(|(_, methods)| {
            methods.iter().find(|m| m.aliases.contains(&method)).cloned()
        })
}

pub fn all_odu_domains_with_methods() -> &'static [(OduDomain, &'static [OduMethodInfo])] {
    ODU_METHODS
}
"""

with open(r"c:\Users\allio\Desktop\ifa_lang\crates\ifa-types\src\odu_metadata.rs", "w", encoding="utf-8") as f:
    f.write(out_rs)

with open(r"c:\Users\allio\Desktop\ifa_lang\crates\ifa-types\src\methods.rs", "w", encoding="utf-8") as f:
    f.write("""use crate::domain::OduDomain;
pub use crate::odu_metadata::{resolve_method_id, method_name_from_id};

/// Stable method IDs for all built-in Odù domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OduMethodId(pub u16);

impl OduMethodId {
    pub fn new(domain_id: u8, method_idx: u8) -> Self {
        OduMethodId(((domain_id as u16) << 8) | (method_idx as u16))
    }
}
""")
