Code Duplication & Dead Code Analysis
Here is the complete consolidated report. 16 issues found across the codebase.
🔴 Critical (broken functionality)
1. Registry call_fast is broken for all 16 domains
The odu_registry_dispatch! macro in vm_registry.rs assigns sequential method IDs to each alias string, but odu_metadata.rs groups aliases into semantic entries. The IDs are misaligned on every single domain — call_fast dispatches wrong methods (e.g. metadata says method_id 0x02 = "version", macro routes it to "args").
Fix: Rebuild the macro so method IDs match metadata's canonical mapping, or remove call_fast entirely until the macro is fixed.
2. Ogbe.eto() was unwired (just fixed)
Already caught and fixed in the previous round.
3. check_effect() only gates 4 of 16 domains
Only Odi (files), Irosu (stdio), Osa (concurrency), Ogunda (collections) check capabilities. Otura (net), Irete (crypto), Storage, Iwori (time), Owonrin (random), Oyeku (exit) — all pass through unchecked.
Fix: Add Ofun mappings for remaining sensitive domains.
🟠 High (dead code / maintenance burden)
4. Old value.rs — 789 lines of dead code
Still compiled via pub mod value; in ifa-types/src/lib.rs:35 but never used anywhere. The active IfaValue lives in value_union.rs. The entire module is wasted compilation.
Fix: Remove pub mod value; (or gate it behind a legacy feature flag).
5. PrincipalOdu enum duplicates OduDomain enum
ifa-std/src/opele.rs:183-200 defines the same 16 Odù with the same binary patterns already in ifa-types/src/domain.rs. Adding or reordering an Odù requires edits in two places.
Fix: Either derive opele.rs from domain.rs or merge.
6. OduDomain enum (ifa-types) vs OduDomain trait (ifa-std)
Naming collision between ifa-types::domain::OduDomain (enum of 16 domains) and ifa-std::traits::OduDomain (trait for domain structs). Importing both is ambiguous.
Fix: Rename the trait to OduDomainImpl or DomainBehavior.
7. 16 identical "unknown method" catch-all arms
Every dispatch function ends with the same error pattern:
_ => Err(IfaError::Custom(format!("{domain}: unknown method '{method}'", ...)))
Fix: Create fn unknown_method(domain, method) -> IfaError helper.
8. Legacy opcodes handled but emitted by compiler
- PushList (0x76), PushMap (0x77) — superseded by BuildList/BuildMap
- Push (0x01) — never emitted (compiler uses PushNull/True/False/Int/Float/Str/Fn)
- SetField (0x79) — never emitted (compiler emits GetField only)
- Load32/64, Store32/64 — only for embedded, never in standard path
Fix: Remove or deprecate dead opcode handlers from vm.rs.
🟡 Medium (boilerplate)
9. ~150+ repetitive IfaValue::str/int/float/list/Null calls
In vm_registry.rs (~283 call sites) and vm.rs (~252 call sites). Heavily repeated pattern.
Fix: Add helpers like fn ok_str(s), fn ok_int(n), fn ok_float(f), fn ok_list(v) to IfaResult.
10. ~30× repeated args.first().map(|v| v.to_string()).unwrap_or_default()
Same extraction pattern across all dispatch functions.
Fix: Add fn arg_str(args: &[IfaValue], idx: usize) -> String helper to vm_registry.rs.
11. Arithmetic type-coercion repeated 4× in vm.rs
Add/Sub/Mul/Div each repeat the same 4-arm pattern (Int-Int, Float-Float, Int-Float, Float-Int).
Fix: Factor into fn binary_arith<F: Fn(f64, f64) -> f64>(a, b, int_op).
12. import() hardcodes domain name→ID mapping
Line 268-302 matches strings like "ogbe", "oyeku", etc. directly instead of using OduDomain::from_str() or domain constants.
Fix: Use OduDomain enum's name resolution instead.
🔵 Low (minor)
13. PathError defined twice in ifa-installer-core
Both windows.rs and unix.rs define enum PathError { ... } — same name, same crate, different module files.
14. ~18 crate-specific error types despite IfaError being "canonical"
The error.rs file explicitly says "THE canonical error type" but 18+ other error types exist. Most are low-impact, but OponError, StorageError, CapabilityError could map to existing IfaError variants.
15. Dispatch functions are inconsistent — free functions vs methods
dispatch_ika, dispatch_obara, dispatch_okanran, dispatch_osa, dispatch_oturupon, dispatch_owonrin are free functions; the rest are self.dispatch_* methods.
16. "bere" alias means 3 different things across domains
- Ogbe: "version"
- Ose: "init" (terminal)
- Ofun: "spawn" (process)
Same Yoruba word, 3 English meanings. Intentional but confusing.
Recommended Priority Order for Fixing
Priority	Issue
1	Fix call_fast macro (re-align method IDs)
2	Remove dead value.rs module
3	Add check_effect for remaining domains
4	Factor 16 "unknown method" errors into helper
5	Factor args.first().map(...) into helper
6	Merge PrincipalOdu / OduDomain
7	Rename OduDomain trait to avoid collision
8	Factor IfaValue:: boilerplate helpers
9	Factor arithmetic coercion in vm.rs
10	Clean up legacy opcodes in VM
11	Use OduDomain in import()
12	Inline dispatch method consistency