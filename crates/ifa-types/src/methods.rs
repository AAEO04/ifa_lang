use crate::domain::OduDomain;

/// Reverse-lookup: given a domain_id byte and a full method_id u16, return the
/// canonical method name.  Only used by the OduRegistry::call_fast default fallback;
/// the fast path in vm_registry.rs overrides this entirely with an integer match.
pub fn method_name_from_id(domain_id: u8, method_id: u16) -> Option<&'static str> {
    let low = (method_id & 0xFF) as u8;
    match domain_id {
        0 => match low { 0x01 => Some("bere"), _ => None },
        1 => match low { 0x01 => Some("jade"), 0x02 => Some("sun"), _ => None },
        2 => match low { 0x01 => Some("bayi"), 0x02 => Some("akoko"), _ => None },
        3 => match low { 0x01 => Some("ka"), 0x02 => Some("ko"), 0x03 => Some("wa"), _ => None },
        4 => match low {
            0x01 => Some("fo"), 0x02 => Some("so"), 0x03 => Some("gbo"),
            0x04 => Some("gbo_nomba"), 0x05 => Some("mo"), 0x06 => Some("san"),
            0x07 => Some("kigbe"), _ => None,
        },
        5 => match low { 0x01 => Some("pese"), _ => None },
        6 => match low { 0x01 => Some("fikun"), 0x02 => Some("isodipupo"), 0x03 => Some("agbara"), 0x04 => Some("gbongbo"), _ => None },
        7 => match low { 0x01 => Some("sise"), 0x02 => Some("kigbe"), _ => None },
        8 => match low {
            0x01 => Some("iwon"),
            0x02 => Some("fi"),
            0x03 => Some("mu"),
            0x04 => Some("yi_pada"),
            0x05 => Some("yan"),
            0x06 => Some("seku"),
            _ => None,
        },
        9 => match low { 0x01 => Some("sun"), 0x02 => Some("gbogbo"), _ => None },
        10 => match low { 0x01 => Some("gigun"), 0x02 => Some("ge"), 0x03 => Some("so"), 0x04 => Some("oruko_html"), 0x05 => Some("tumo_html"), _ => None },
        11 => match low { 0x01 => Some("yokuro"), 0x02 => Some("pipin"), _ => None },
        12 => match low { 0x01 => Some("gba"), 0x02 => Some("ran"), 0x03 => Some("de"), 0x04 => Some("soro"), _ => None },
        13 => match low { 0x01 => Some("hash"), 0x02 => Some("hmac"), 0x03 => Some("base64"), 0x04 => Some("decode"), 0x05 => Some("funpo"), 0x06 => Some("tu"), _ => None },
        14 => match low { 0x01 => Some("bere"), 0x02 => Some("pari"), 0x03 => Some("gbile"), 0x04 => Some("apoti"), 0x05 => Some("ipinro"), 0x06 => Some("ya"), _ => None },
        15 => match low { 0x01 => Some("le"), _ => None },
        18 => match low {
            0x01 => Some("square"), 0x02 => Some("cube"), 0x03 => Some("double"),
            0x04 => Some("increment"), 0x05 => Some("decrement"), 0x06 => Some("neg"),
            0x07 => Some("abs"), 0x08 => Some("sqrt"), 0x09 => Some("positive"),
            0x0A => Some("negative"), 0x0B => Some("nonzero"), 0x0C => Some("even"),
            0x0D => Some("odd"), 0x0E => Some("sum"), 0x0F => Some("product"),
            0x10 => Some("min"), 0x11 => Some("max"), 0x12 => Some("configure"),
            0x13 => Some("threads"), 0x14 => Some("par_sum"), 0x15 => Some("par_map"),
            0x16 => Some("par_filter"), 0x17 => Some("par_reduce"), 0x18 => Some("par_sort"),
            0x19 => Some("alloc_buffer"), 0x1A => Some("read_buffer"), 0x1B => Some("write_buffer"),
            _ => None,
        },
        19 => match low { 0x01 => Some("init"), 0x02 => Some("dispatch_pipeline"), 0x03 => Some("sync"), 0x04 => Some("alloc_buffer"), 0x05 => Some("read_buffer"), 0x06 => Some("write_buffer"), _ => None },
        20 => match low { 0x01 => Some("open"), 0x02 => Some("get"), 0x03 => Some("set"), 0x04 => Some("delete"), 0x05 => Some("compact"), _ => None },
        29 => match low { 0x01 => Some("num_cores"), 0x02 => Some("total_memory"), 0x03 => Some("available_memory"), 0x04 => Some("uptime"), _ => None },
        _ => None,
    }
}



/// Stable method IDs for all built-in Odù domains.
/// 
/// The format is a u16 where the high byte is the domain ID (if < 255)
/// and the low byte is the method index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OduMethodId(pub u16);

impl OduMethodId {
    pub fn new(domain_id: u8, method_idx: u8) -> Self {
        OduMethodId(((domain_id as u16) << 8) | (method_idx as u16))
    }
}

/// Resolves a method name string to a static method ID for a given domain.
pub fn resolve_method_id(domain: OduDomain, method: &str) -> Option<u16> {
    match domain {
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
            "kigbe" | "throw" | "panic" | "raise" => Some(0x0702),
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
            "par_sum" => Some(0x1214), // alias "sum" omitted to avoid collision
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
        },
        _ => None,
    }
}
