//! # Odù Domain Constants
//!
//! Constants for Odù domain and method names to replace magic strings.
//! Each domain has both Yoruba and ASCII/English aliases.

/// Odù domain identifiers (normalized to lowercase ASCII)
pub mod domains {
    pub const OGBE: &str = "ogbe";
    pub const OYEKU: &str = "oyeku";
    pub const IWORI: &str = "iwori";
    pub const ODI: &str = "odi";
    pub const OWONRIN: &str = "owonrin";
    pub const OBARA: &str = "obara";
    pub const OKANRAN: &str = "okanran";
    pub const OGUNDA: &str = "ogunda";
    pub const OSA: &str = "osa";
    pub const IKA: &str = "ika";
    pub const OTURUPON: &str = "oturupon";
    pub const OTURA: &str = "otura";
    pub const IRETE: &str = "irete";
    pub const OSE: &str = "ose";
    pub const OFUN: &str = "ofun";
    pub const IROSU: &str = "irosu";
}

/// Ọ̀gbè (System/Environment) methods
pub mod ogbe {
    pub const ARGS: &[&str] = &["args", "àwọn_àríyànjú"];
    pub const ENV: &[&str] = &["env", "ayika", "àyíká"];
    pub const SET_ENV: &[&str] = &["set_env", "ṣeto_ayika"];
    pub const CWD: &[&str] = &["cwd", "ibi_isisiyi"];
    pub const EXIT: &[&str] = &["jade", "exit"];
}

/// Ọ̀yẹ̀kú (Exit/Death) methods
pub mod oyeku {
    pub const EXIT: &[&str] = &["jade", "exit"];
    pub const SLEEP: &[&str] = &["sun", "sleep"];
}

/// Ìwòrì (Time/Iteration) methods
pub mod iwori {
    pub const NOW: &[&str] = &["bayi", "now"];
    pub const NOW_MS: &[&str] = &["bayi_ms", "now_ms"];
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
    pub const RANDOM: &[&str] = &["yan", "random"];
    pub const RANGE: &[&str] = &["yan_laarin", "range"];
    pub const BOOL: &[&str] = &["yan_bool", "random_bool"];
}

/// Ọ̀bàrà (Math) methods
pub mod obara {
    pub const POWER: &[&str] = &["agbara", "pow", "power"];
    pub const SQRT: &[&str] = &["sqrt", "gbongbo"];
    pub const FLOOR: &[&str] = &["isale", "floor"];
    pub const CEIL: &[&str] = &["oke", "ceil"];
    pub const ABS: &[&str] = &["pipe", "abs"];
    pub const SIN: &[&str] = &["sin"];
    pub const COS: &[&str] = &["cos"];
}

/// Ọ̀kànràn (Assertion/Boolean) methods
pub mod okanran {
    pub const ASSERT: &[&str] = &["beeni", "assert"];
    pub const EQUALS: &[&str] = &["dogba", "equals"];
    pub const DIE: &[&str] = &["ku", "die", "panic"];
}

/// Ògúndá (String/Collection) methods
pub mod ogunda {
    pub const APPEND: &[&str] = &["fi_kun", "append", "push"];
    pub const SPLIT: &[&str] = &["ge", "split"];
    pub const JOIN: &[&str] = &["so_po", "join"];
    pub const LENGTH: &[&str] = &["gigun", "len", "length"];
}

/// Ọ̀sá (Async) methods
pub mod osa {
    pub const SPAWN: &[&str] = &["sa", "spawn"];
    pub const SLEEP: &[&str] = &["sun", "sleep_async"];
    pub const AWAIT: &[&str] = &["duro", "await"];
}

/// Ìká (Comparison) methods
pub mod ika {
    pub const MAX: &[&str] = &["nla", "max"];
    pub const MIN: &[&str] = &["kekere", "min"];
    pub const SORT: &[&str] = &["to", "sort"];
    pub const CLAMP: &[&str] = &["pin", "clamp"];
}

/// Òtúúrúpọ̀n (Modulo) methods
pub mod oturupon {
    pub const MOD: &[&str] = &["iyoku", "mod", "remainder"];
}

/// Òtúrá (Network/HTTP) methods
pub mod otura {
    pub const GET: &[&str] = &["gba", "get", "fetch"];
    pub const POST: &[&str] = &["fi", "post"];
}

/// Ìrẹtẹ̀ (Crypto/Hash) methods
pub mod irete {
    pub const HASH: &[&str] = &["hash", "ṣe_hash"];
    pub const BASE64: &[&str] = &["base64_encode", "base64"];
}

/// Ọ̀ṣẹ́ (Debug/Graphics) methods
pub mod ose {
    pub const DEBUG: &[&str] = &["wo", "debug"];
}

/// Òfún (Type/Reflection) methods
pub mod ofun {
    pub const IS_ALIVE: &[&str] = &["laaye", "is_defined", "is_alive"];
    pub const TYPE_OF: &[&str] = &["iru", "type_of", "typeof"];
}

/// Ìrosù (I/O) methods
pub mod irosu {
    pub const PRINT: &[&str] = &["fo", "sọ", "print"];
    pub const PRINTLN: &[&str] = &["fọ", "println"];
    pub const READ: &[&str] = &["ka", "read", "input"];
}

/// Check if a method name matches any of the aliases
#[inline]
pub fn matches_method(method: &str, aliases: &[&str]) -> bool {
    aliases.iter().any(|&alias| method == alias)
}
