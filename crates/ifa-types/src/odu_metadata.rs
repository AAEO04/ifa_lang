use crate::domain::OduDomain;

#[derive(Debug, Clone)]
pub struct OduMethodInfo {
    pub yoruba: &'static str,
    pub english: &'static str,
    pub description: &'static str,
}

pub static ODU_METHODS: &[(OduDomain, &[&str])] = &[
    (OduDomain::Ogbe, &["type", "iru", "len", "gigun", "assert", "jẹri", "format", "ṣẹda", "parse_int", "parse_float"]),
    (OduDomain::Oyeku, &["jade", "exit", "sun", "sleep", "sun_sẹkọndi", "sleep_sec"]),
    (OduDomain::Iwori, &["bayi", "now", "timestamp", "bayi_ms", "now_ms", "aago", "elapsed", "ṣe_ọjọ", "format", "iso", "ka_ọjọ", "parse", "laarin", "range"]),
    (OduDomain::Odi, &["ka", "read", "kọ", "write", "fikun", "append", "wa", "exists", "pa", "delete", "remove", "ṣe_akojọ", "list", "ls", "ṣe_folda", "mkdir"]),
    (OduDomain::Irosu, &["fo", "sọ", "so", "print", "println", "ka", "input", "listen", "gbo", "kigbe", "error"]),
    (OduDomain::Owonrin, &["nọmba", "random", "rand", "laarin", "range", "ida", "float", "boolean", "bool", "aruwo", "shuffle", "yan", "choice"]),
    (OduDomain::Obara, &["fikun", "add", "isodipupo", "mul", "multiply", "agbara", "pow", "power", "abs", "max", "min"]),
    (OduDomain::Okanran, &["ta", "throw", "error", "jagun", "panic", "jẹri", "assert", "jẹri_asọ", "assert_msg", "jẹri_bakan", "assert_eq", "jẹri_yato", "assert_ne", "ko_ṣee_de", "unreachable"]),
    (OduDomain::Ogunda, &["da", "create", "new", "gigun", "len", "iwon", "fikun", "push", "append", "yọ", "pop", "mu", "yipada", "reverse", "akọkọ", "first", "ikẹhin", "last", "gba", "get", "ni", "contains", "ge", "slice", "maapu", "map", "yi_pada", "ṣàjọ", "filter", "yan", "ṣẹ́kù", "seku", "din", "reduce", "fold"]),
    (OduDomain::Osa, &["awọn_okun", "threads", "num_threads", "afikun_afiwe", "parallel_sum", "sum", "isoro_afiwe", "parallel_product", "product", "kekere_afiwe", "parallel_min", "min", "tobi_afiwe", "parallel_max", "max", "tọ_afiwe", "parallel_sort", "sort", "eyikeyi_afiwe", "parallel_any", "any", "gbogbo_afiwe", "parallel_all", "all", "sun", "sleep", "bẹrẹ", "spawn", "duro", "await", "fi", "send", "gba", "recv"]),
    (OduDomain::Ika, &["so", "concat", "dapo", "join", "gigun", "len", "pin", "split", "trim", "nla", "uppercase", "upper", "kekere", "lowercase", "lower", "ni", "contains", "has", "ropo", "replace", "sub", "substring", "slice"]),
    (OduDomain::Oturupon, &["yọkuro", "sub", "subtract", "pin", "div", "divide", "iyoku", "mod", "modulo", "floor_div", "neg", "negate", "sqrt"]),
    (OduDomain::Otura, &["http_get", "gba", "get", "http_post", "fi", "post", "serve", "sin", "listen", "ws_connect", "asopọ_ws", "fetch_json", "gba_json", "url_encode", "koodu_url"]),
    (OduDomain::Irete, &["sha256", "fọwọsi", "encode_base64", "si_base64", "decode_base64", "lati_base64", "random_bytes", "awọn_baiti_laileto", "uuid", "id_alailẹgbẹ", "hmac", "ṣayẹwo", "hash_password", "fọwọsi_ọrọigbaniwọle", "verify_password", "rii_daju_ọrọigbaniwọle"]),
    (OduDomain::Ose, &["nu", "clear", "lọ_si", "goto", "move_to", "awọ", "color", "apoti", "box", "kọ_si", "print_at", "fia_kasọta", "hide_cursor", "ṣafihan_kasọta", "show_cursor", "iwọn", "size"]),
    (OduDomain::Ofun, &["ni_agbara", "has_capability", "can", "beere", "request", "iru", "typeof", "awọn_ẹsẹ", "methods", "awọn_agbara", "capabilities", "alaye_ẹka", "module_info", "dbg", "debug"]),
];

pub fn is_valid_odu_method(domain: &OduDomain, method: &str) -> bool {
    ODU_METHODS
        .iter()
        .find(|(d, _)| d == domain)
        .map(|(_, methods)| methods.contains(&method))
        .unwrap_or(false)
}

pub fn odu_methods(domain: &OduDomain) -> &'static [&'static str] {
    ODU_METHODS
        .iter()
        .find(|(d, _)| d == domain)
        .map(|(_, methods)| *methods)
        .unwrap_or(&[])
}

pub fn all_odu_domains_with_methods() -> &'static [(OduDomain, &'static [&'static str])] {
    ODU_METHODS
}
