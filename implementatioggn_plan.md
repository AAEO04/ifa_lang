# Implementation Plan — Bug Fixes & Otura Socket Support

This plan addresses the verified bugs and unwired features from the codebase audit, with a focus on making `Otura` networking work and completing the other critical bug fixes.

---

## Proposed Changes

### VM Correctness & Sandboxing

#### [MODIFY] [vm.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs)
- Remove `#[allow(dead_code)]` from `CachedModule` since the field warning is no longer applicable.
- Safely handle `NativeFutureState::Ready(bytes)` by deserializing with bincode and mapping error to `IfaError::Runtime` instead of calling `.unwrap()` and risking a VM process panic.

#### [MODIFY] [lib.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/lib.rs)
- Update the **ARCHITECTURAL STATUS** header to reflect that `Add` is no longer overloaded and `Concat` (`0x27`) is the dedicated string op.
- Remove references to the archived `interpreter` module.

#### [DELETE] [vm.rs.tmp](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs.tmp)
- Delete the stale scratch file.

---

### Otura (Networking) & Odu Registry

#### [MODIFY] [otura.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-std/src/odu/otura.rs)
- Define `TcpListenerResource` and `TcpStreamResource` wrapping `tokio::net::TcpListener` and `tokio::net::TcpStream` in `Mutex` to satisfy `Send + Sync` constraints for resource registry registration.

#### [MODIFY] [vm_registry.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-std/src/vm_registry.rs)
- Change `dispatch_otura` signature to accept `ctx: &mut VmContext`.
- Update `call` and `call_fast` method dispatches for Otura (domain 12) to pass `ctx`.
- Implement socket-based dispatch logic in `dispatch_otura`:
  - `de`/`listen`: Bind to address, register the `TcpListener` resource, and return its `ResourceToken` wrapper.
  - `soro`/`connect`: Connect to address, register the `TcpStream` resource, and return its `ResourceToken` wrapper.
  - `gba`/`get`/`fetch`: If argument is a `TcpListener` resource, accept a connection and return a new `TcpStream` resource. If argument is a `TcpStream` resource, read available bytes into a buffer and return as string. Otherwise, fall back to HTTP GET.
  - `ran`/`post`: If first argument is a `TcpStream` resource, write the second argument's string data to the stream. Otherwise, fall back to HTTP POST.
  - `pa`/`close`: Close/de-register the resource (listener or stream) from the registry.
- Complete fast-path dispatches in `call_fast` for:
  - **Osa** (domain 9): `sa` (0x03), `duro` (0x04), `egbe` (0x05), and `ran` (0x06).
  - **Owonrin** (domain 5): `yan_bool` (0x02), `yan_laarin` (0x03), `paaro` (0x04), and `uuid` (0x05).
- Add `"duro" | "await"` implementation to `dispatch_osa` to await future cells.

#### [MODIFY] [odu_metadata.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-types/src/odu_metadata.rs)
- Add metadata records for `Osa.egbe` (0x0905) and `Osa.ran` (0x0906).
- Add metadata record for `Owonrin.uuid` (0x0505).

---

### Ika & HTML Correctness

#### [MODIFY] [ika.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-std/src/odu/ika.rs)
- Modify `tumo` to return a `Result` type (`IfaValue::err`) describing that DOM parse handles are not implemented, avoiding silent `Null` values that cause unexpected downstream crashes.

---

### Tooling & CLI Gaps

#### [MODIFY] [main.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-cli/src/main.rs)
- **`Commands::Test`**: Parse expect comments (`# expect:` / `// # expect:`) from each test source file. Validate that successful executions printed the expected string (captured from `opon` history under `"Ìrosù"` / `"fọ̀"`), and that error results contained the expected error message.
- **`Commands::Run` (WASM sandbox mode)**: Enforce the standard 30s timeout on VM execution by running it in a scoped thread and waiting with a timeout.

---

## Verification Plan

### Automated Tests
- Run `cargo test` in `ifa-std`, `ifa-vm`, `ifa-sandbox`, `ifa-cli` workspace.
- Execute standard test runner: `cargo run -- test tests/conformance/vm`.
- Check conformance test suite to ensure existing tests pass.

### Manual Verification
- Write a test `.ifa` script verifying TCP client-server roundtrip using `Otura.de`, `Otura.soro`, `Otura.gba`, `Otura.ran`, and `Otura.pa`.
