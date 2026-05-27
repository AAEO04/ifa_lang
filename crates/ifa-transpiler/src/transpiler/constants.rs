//! # Odù Domain Constants
//!
//! Constants for Odù domain and method names to replace magic strings.
//! Each domain has both Yoruba and ASCII/English aliases, strictly aligned with `odu_metadata.rs`.

/// Ọ̀gbè (System/Lifecycle) methods
pub mod ogbe {
    pub const VERSION: &[&str] = &["bere", "version"];
    pub const ARGS: &[&str] = &["args", "àwọn_àríyànjú"];
    pub const ENV: &[&str] = &["env", "ayika"];
    pub const CWD: &[&str] = &["cwd", "ibi_isisiyi"];
    pub const EXIT: &[&str] = &["jade", "exit"];
}

/// Ọ̀yẹ̀kú (Exit/Death) methods
pub mod oyeku {
    pub const EXIT: &[&str] = &["jade", "exit", "quit", "halt"];
    pub const SLEEP: &[&str] = &["sun", "sleep", "wait"];
}

/// Ìwòrì (Time/Iteration) methods
pub mod iwori {
    pub const NOW: &[&str] = &["bayi", "now", "current"];
    pub const NOW_MS: &[&str] = &["akoko", "timestamp"];
    pub const ITERATE: &[&str] = &["iterate", "yipo"];
}

/// Òdí (File I/O) methods
pub mod odi {
    pub const READ: &[&str] = &["ka", "read"];
    pub const WRITE: &[&str] = &["ko", "write"];
    pub const EXISTS: &[&str] = &["wa", "exists"];
    pub const DELETE: &[&str] = &["pa", "delete"];
}

/// Ọ̀wọ́nrín (Random) methods
pub mod owonrin {
    pub const RANDOM: &[&str] = &["nọmba", "random", "rand"];
    pub const BOOL: &[&str] = &["yan_bool", "random_bool"];
    pub const RANGE: &[&str] = &["yan_laarin", "range"];
}

/// Ọ̀bàrà (Math) methods
pub mod obara {
    pub const ADD: &[&str] = &["fikun", "add", "plus"];
    pub const MUL: &[&str] = &["isodipupo", "mul", "times"];
    pub const POW: &[&str] = &["agbara", "pow", "power"];
    pub const SQRT: &[&str] = &["gbongbo", "sqrt"];
}

/// Ọ̀kànràn (Assertion/Boolean) methods
pub mod okanran {
    pub const ASSERT: &[&str] = &["sise", "assert", "verify", "check"];
    pub const DIE: &[&str] = &["ta", "throw", "panic", "raise", "ku"];
    pub const EQUALS: &[&str] = &["dogba", "equals"];
}

/// Ògúndá (String/Collection) methods
pub mod ogunda {
    pub const APPEND: &[&str] = &["fi", "push", "append"];
    pub const POP: &[&str] = &["mu", "pop"];
    pub const MAP: &[&str] = &["yi_pada", "yipada", "map", "maapu"];
    pub const FILTER: &[&str] = &["yan", "filter", "sajo", "ṣàjọ"];
    pub const REDUCE: &[&str] = &["seku", "ṣẹ́kù", "fold", "reduce", "din"];
    pub const LENGTH: &[&str] = &["iwon", "len", "count", "apapo"];
}

/// Ọ̀sá (Async) methods
pub mod osa {
    pub const SLEEP: &[&str] = &["sun", "sleep"];
    pub const ALL: &[&str] = &["gbogbo", "all"];
    pub const SPAWN: &[&str] = &["sa", "spawn"];
    pub const AWAIT: &[&str] = &["duro", "await"];
}

/// Ìká (Comparison) methods
pub mod ika {
    pub const LEN: &[&str] = &["gigun", "len"];
    pub const SLICE: &[&str] = &["ge", "slice"];
    pub const CONCAT: &[&str] = &["so", "concat"];
    pub const HTML_TITLE: &[&str] = &["oruko_html", "html_title"];
    pub const STRIP_HTML: &[&str] = &["tumo_html", "strip_html"];
}

/// Òtúúrúpọ̀n (Modulo) methods
pub mod oturupon {
    pub const SUB: &[&str] = &["yokuro", "sub", "minus"];
    pub const DIV: &[&str] = &["pipin", "div", "divide"];
}

/// Òtúrá (Network/HTTP) methods
pub mod otura {
    pub const GET: &[&str] = &["gba", "get", "fetch"];
    pub const POST: &[&str] = &["ran", "post"];
    pub const LISTEN: &[&str] = &["de", "listen"];
    pub const CONNECT: &[&str] = &["soro", "connect"];
}

/// Ìrẹtẹ̀ (Crypto/Hash) methods
pub mod irete {
    pub const HASH: &[&str] = &["hash", "sha256"];
    pub const HMAC: &[&str] = &["hmac", "hmac_sha256"];
    pub const BASE64: &[&str] = &["base64", "base64_encode"];
    pub const DECODE: &[&str] = &["decode", "base64_decode"];
    pub const COMPRESS: &[&str] = &["funpo", "compress"];
    pub const DECOMPRESS: &[&str] = &["tu", "decompress"];
}

/// Ọ̀ṣẹ́ (Debug/Graphics) methods
pub mod ose {
    pub const INIT: &[&str] = &["bere", "init"];
    pub const END: &[&str] = &["pari", "end"];
    pub const READ_KEY: &[&str] = &["gbile", "read_key"];
    pub const BOX: &[&str] = &["apoti", "box"];
    pub const SECTION: &[&str] = &["ipinro", "section"];
    pub const DRAW: &[&str] = &["ya", "draw"];
    pub const DEBUG: &[&str] = &["wo", "debug"];
}

/// Òfún (Type/Reflection) methods
pub mod ofun {
    pub const CAN: &[&str] = &["le", "can"];
    pub const TYPE_OF: &[&str] = &["iru", "type_of", "typeof"];
    pub const IS_ALIVE: &[&str] = &["laaye", "is_alive"];
}

/// Ìrosù (I/O) methods
pub mod irosu {
    pub const PRINT: &[&str] = &["so", "print"];
    pub const PRINTLN: &[&str] = &["fo", "println"];
    pub const LISTEN: &[&str] = &["gbo", "listen"];
    pub const CLEAR: &[&str] = &["mo", "clear"];
    pub const FLUSH: &[&str] = &["san", "flush"];
    pub const ERROR: &[&str] = &["kigbe", "error"];
    pub const READ: &[&str] = &["ka", "read", "input"];
}

/// Check if a method name matches any of the aliases
#[inline]
pub fn matches_method(method: &str, aliases: &[&str]) -> bool {
    aliases.iter().any(|&alias| method == alias)
}
