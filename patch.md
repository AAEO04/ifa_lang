🔴 "Your has_capability function is security theater" — ofun.rs:76-86
"has_capability" | "can" => {
    let has_cap = cap_type != "Unknown";  // NEVER checks actual CapabilitySet!
    Ok(IfaValue::bool(has_cap))
}
It always returns true for any known capability name. The _capabilities parameter is completely ignored. ofun.request("admin") unconditionally succeeds. And the one centralized check_capability() method is #[allow(dead_code)] — deliberately disabled.
The enforcement path is correct. The user-facing query path is a lie. Anyone reading if Ofun.le("network") { Otura.gba(...) } trusts a function that never checks.
🔴 "Your path security has a .. shaped hole" — odi.rs + capability.rs:57
(Ofun::ReadFiles { root: g }, Ofun::ReadFiles { root: r }) => r.starts_with(g),
Neither path is canonicalized. /tmp/../../../etc/shadow passes. The sandbox crate has a correct can_access_file() that canonicalizes and rejects symlinks — but odi.rs doesn't use it.
🔴 "Your FFI bind system is void* with extra steps" — ffi.rs:617-626
Every C function is resolved as unsafe extern "C" fn(), cast to *mut c_void, then cast back at call time with zero ABI validation. Unknown type strings are silently dropped by filter_map, shifting the signature. And blocked symbols is a blacklist of six entries — missing execv, execvp, posix_spawn, syscall.
🔴 "Your cache key is an Arc pointer — that's a use-after-free" — vm_ikin.rs:93
let key = std::sync::Arc::as_ptr(arc) as *const u8 as usize;
When the Arc<str> drops and a new allocation reuses the address, the cache returns the old string's length. Inline strings bypass the cache entirely. Linus would call this "a caching layer that's wrong half the time and absent the other half."
🔴 "Actors have a resource registry, just not the right one" — actor.rs:193-216
transfer_resources() updates the parent's registry, but actors get fresh empty registries. Resources sent to actors are dangling. The isolation is correct in principle, broken in practice.
🟡 "Why do 14 of 16 domain handlers receive capabilities they ignore?"
Every handler takes _capabilities and never calls enforce_crossroads(). Only odi.rs, otura.rs, and irete.rs actually check. That's 14 blind spots in your security model.