# Standard library

The standard library is organized into 16 Odù domains, plus infrastructure and stack modules. Each domain is a struct with methods that can be called from Ifá-Lang scripts.

## Domain reference

### Domain dispatch IDs

The VM dispatches domain calls by numeric ID; these are fixed values:

| ID | Domain | Binary | English name |
|----|--------|--------|-------------|
| 0 | **Ọ̀gbè** | 1111 | Ogbe |
| 1 | **Ọ̀yẹ̀kú** | 0000 | Oyeku |
| 2 | **Ìwòrì** | 0110 | Iwori |
| 3 | **Òdí** | 1001 | Odi |
| 4 | **Ìrosù** | 1100 | Irosu |
| 5 | **Ọ̀wọ́nrín** | 0011 | Owonrin |
| 6 | **Ọ̀bàrà** | 1000 | Obara |
| 7 | **Ọ̀kànràn** | 0001 | Okanran |
| 8 | **Ògúndá** | 1110 | Ogunda |
| 9 | **Ọ̀sá** | 0111 | Osa |
| 10 | **Ìká** | 0100 | Ika |
| 11 | **Òtúúrúpọ̀n** | 0010 | Oturupon |
| 12 | **Òtúrá** | 1011 | Otura |
| 13 | **Ìrẹtẹ̀** | 1101 | Irete |
| 14 | **Ọ̀ṣẹ́** | 1010 | Ose |
| 15 | **Òfún** | 0101 | Ofun |

Infrastructure domains use higher IDs: Cpu=18, Gpu=19, Storage=20, Sys=29.

### Feature gates

| Domain | Requires feature | Notes |
|--------|-----------------|-------|
| Odi | `backend` | Files, SQLite database |
| Osa | `backend` | Async, concurrency, tokio |
| Otura | `backend` | Networking, HTTP |
| Irete | `crypto` | Hashing, encryption, compression |
| Ose | `game` | Terminal UI, graphics |

All other domains are available by default.

---

## Core domains (always available)

### Ọ̀gbè — System (ID: 0, Binary: 1111)

System information, CLI arguments, environment variables.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `awon_ohun` | `args` | `() -> List` | CLI arguments |
| `ohun` | `arg` | `(Int) -> Str` | Argument at index |
| `iye_ohun` | `arg_count` | `() -> Int` | Number of arguments |
| `ayika` | `get_env` | `(Str) -> Str` | Environment variable |
| `ayika_tabi` | `get_env_or` | `(Str, Str) -> Str` | Env var with default |
| `ile` | `home_dir` | `() -> Str` | Home directory |
| `oju_ona` | `cwd` | `() -> Str` | Current working directory |
| `eto` | `os` | `() -> Str` | Operating system name |
| `apẹrẹ` | `arch` | `() -> Str` | CPU architecture |

### Ọ̀yẹ̀kú — Exit (ID: 1, Binary: 0000)

Process exit, sleep, and lifecycle management.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `ku` | `exit` | `(Int) -> never` | Exit with code |
| `ku_daadaa` | `exit_ok` | `() -> never` | Exit successfully |
| `ku_buruku` | `exit_err` | `() -> never` | Exit with error |
| `sun_ms` | `sleep_ms` | `(Int) -> Null` | Sleep milliseconds |
| `sun` | `sleep` | `(Float) -> Null` | Sleep seconds |
| `duro` | `wait` | `(Int) -> Null` | Wait/pause (ms) |
| `da_duro` | `abort` | `() -> never` | Abort immediately |

### Ìwòrì — Time (ID: 2, Binary: 0110)

Date, time, and iteration utilities.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `isisinyi` | `now` | `() -> Str` | Current local time |
| `utc` | `utc_now` | `() -> Str` | Current UTC time |
| `akoko` | `timestamp` | `() -> Int` | Unix timestamp (seconds) |
| `akoko_ms` | `timestamp_ms` | `() -> Int` | Unix timestamp (ms) |
| `ojo` | `format` | `(Str) -> Str` | Format date by pattern |
| `ka_ojo` | `parse_date` | `(Str, Str) -> Date` | Parse date string |
| `odun` | `year` | `() -> Int` | Current year |
| `osu` | `month` | `() -> Int` | Current month (1-12) |
| `ojo_osu` | `day` | `() -> Int` | Current day of month |
| `wakati` | `hour` | `() -> Int` | Current hour (0-23) |
| `iseju` | `minute` | `() -> Int` | Current minute |
| `aaya` | `second` | `() -> Int` | Current second |
| `ojo_ose` | `day_of_week` | `() -> Int` | Day of week (0=Sunday) |
| `odun_abule` | `is_leap` | `(Int) -> Bool` | Is leap year |
| `fikun` | `add_duration` | `(Int, Int, Int, Int) -> Date` | Add duration to now |
| `iye_ojo` | `days_between` | `(Date, Date) -> Int` | Days between dates |

### Ìrosù — Console I/O (ID: 4, Binary: 1100)

Printing, user input, terminal control.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `fo` | `println` | `(Any) -> Null` | Print with newline |
| `so` | `print` | `(Any) -> Null` | Print without newline |
| `gbo` | `listen` | `(Str) -> Str` | Read line from stdin |
| `gbo_nomba` | `read_int` | `(Str) -> Int` | Read integer |
| `gbo_odidi` | `read_float` | `(Str) -> Float` | Read float |
| `awo` | `color` | `(Str, Str) -> Null` | Colored output |
| `mo` | `clear` | `() -> Null` | Clear screen |
| `san` | `flush` | `() -> Null` | Flush output |
| `kigbe` | `error` | `(Str) -> Null` | Print to stderr |
| `ilosiwaju` | `progress` | `(Int, Int, Int) -> Null` | Progress bar |

### Ọ̀wọ́nrín — Random (ID: 5, Binary: 0011)

Cryptographically secure random generation.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `pese` | `random` | `(Int, Int) -> Int` | Random int in range |
| `pese_odidi` | `random_float` | `() -> Float` | Random float [0, 1) |
| `pese_larin` | `random_range` | `(Float, Float) -> Float` | Random float in range |
| `boya` | `maybe` | `(Float) -> Bool` | Random boolean |
| `yan` | `choose` | `(List) -> Any` | Random element |
| `dapo` | `shuffle` | `(List) -> Null` | Shuffle list in place |
| `awon_bytes` | `bytes` | `(Int) -> List` | Random bytes |
| `hex` | `random_hex` | `(Int) -> Str` | Random hex string |
| `uuid` | `uuid_v4` | `() -> Str` | UUID v4 |
| `yan_iwuwo` | `weighted_choice` | `(List) -> Any` | Weighted random choice |

### Ọ̀bàrà — Math (ID: 6, Binary: 1000)

Addition, multiplication, powers, trigonometry, statistics.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `fikun` | `add` | `(Num, Num) -> Num` | Add |
| `isodipupo` | `mul` | `(Num, Num) -> Num` | Multiply |
| `agbara` | `pow` | `(Num, Num) -> Num` | Power |
| `gbongbo` | `sqrt` | `(Num) -> Num` | Square root |
| `abs` | `abs` | `(Num) -> Num` | Absolute value |
| `apapo` | `sum` | `(List) -> Num` | Sum of list |
| `ile` | `floor` | `(Num) -> Num` | Floor |
| `orule` | `ceil` | `(Num) -> Num` | Ceiling |
| `yika` | `round` | `(Num, Int) -> Num` | Round to decimals |
| `iyoku` | `mod` | `(Num, Num) -> Num` | Modulo |
| `sin` | `sin` | `(Num) -> Num` | Sine |
| `cos` | `cos` | `(Num) -> Num` | Cosine |
| `tan` | `tan` | `(Num) -> Num` | Tangent |
| `asin` | `asin` | `(Num) -> Num` | Arcsine |
| `acos` | `acos` | `(Num) -> Num` | Arccosine |
| `atan` | `atan` | `(Num) -> Num` | Arctangent |
| `log` | `ln` | `(Num) -> Num` | Natural log |
| `log10` | `log10` | `(Num) -> Num` | Base-10 log |
| `exp` | `exp` | `(Num) -> Num` | e^x |
| `aropin` | `mean` | `(List) -> Num` | Mean/average |
| `nla_julo` | `max` | `(List) -> Num` | Maximum |
| `kere_julo` | `min` | `(List) -> Num` | Minimum |
| `pi` | `pi` | `() -> Float` | Pi constant |
| `e` | `e` | `() -> Float` | Euler's number |

### Ọ̀kànràn — Errors (ID: 7, Binary: 0001)

Assertions, error throwing, debugging.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `beeni` | `assert` | `(Bool, Str) -> Result` | Assert true |
| `dogba` | `assert_eq` | `(Any, Any) -> Result` | Assert equal |
| `yato` | `assert_ne` | `(Any, Any) -> Result` | Assert not equal |
| `beko` | `assert_false` | `(Bool, Str) -> Result` | Assert false |
| `ko_si` | `assert_not_null` | `(Any, Str) -> Result` | Assert not null |
| `ta` | `throw` | `(Str) -> Result` | Throw recoverable error |
| `owe` | `proverb` | `(Error) -> Str` | Get proverb for error |
| `gbiyanju` | `try_with_default` | `(Fn, Any) -> Any` | Try with default |
| `wo` | `debug` | `(Str, Any) -> Null` | Debug print |

### Ògúndá — Arrays (ID: 8, Binary: 1110)

List and map operations, process execution.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `seda` | `new` | `() -> List` | Create new list |
| `fi` | `push` | `(List, Any) -> Null` | Push element |
| `mu` | `pop` | `(List) -> Any` | Pop element |
| `iwon` | `len` | `(List) -> Int` | List length |
| `sofo` | `is_empty` | `(List) -> Bool` | Check empty |
| `pada` | `reverse` | `(List) -> List` | Reverse |
| `to` | `sort` | `(List) -> List` | Sort |
| `dapo` | `concat` | `(List, List) -> List` | Concatenate |
| `yan` | `filter` | `(List, Fn) -> List` | Filter |
| `yi_pada` | `map` | `(List, Fn) -> List` | Map |
| `wa` | `find` | `(List, Fn) -> Any` | Find first |
| `eyikeyi` | `any` | `(List, Fn) -> Bool` | Any match |
| `gbogbo` | `all` | `(List, Fn) -> Bool` | All match |
| `ge` | `slice` | `(List, Int, Int) -> List` | Slice |
| `awon_kokoro` | `keys` | `(Map) -> List` | Map keys |
| `awon_iye` | `values` | `(Map) -> List` | Map values |
| `awon_nkan` | `items` | `(Map) -> List` | Map entries |
| `yo` | `remove` | `(Map, Str) -> Any` | Remove key |
| `sise` | `run` | `(Str, List) -> Result` | Run process |
| `sise_ka` | `run_stdout` | `(Str, List) -> Str` | Run, capture stdout |
| `bere` | `spawn` | `(Str, List) -> Int` | Spawn process |

### Ìká — Strings (ID: 10, Binary: 0100)

String manipulation, regex, serialization.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `so` | `concat` | `(List) -> Str` | Concatenate strings |
| `gigun` | `len` | `(Str) -> Int` | Character count |
| `nla` | `upper` | `(Str) -> Str` | Uppercase |
| `kekere` | `lower` | `(Str) -> Str` | Lowercase |
| `wa` | `find` | `(Str, Str) -> Int` | Find position |
| `ni` | `contains` | `(Str, Str) -> Bool` | Contains |
| `pin` | `split` | `(Str, Str) -> List` | Split |
| `dapo` | `join` | `(List, Str) -> Str` | Join |
| `yi_pada` | `replace` | `(Str, Str, Str) -> Str` | Replace all |
| `ge` | `trim` | `(Str) -> Str` | Trim whitespace |
| `pada` | `reverse` | `(Str) -> Str` | Reverse |
| `ge_lara` | `substring` | `(Str, Int, Int) -> Str` | Substring |
| `tun` | `repeat` | `(Str, Int) -> Str` | Repeat |
| `bere` | `starts_with` | `(Str, Str) -> Bool` | Starts with |
| `pari` | `ends_with` | `(Str, Str) -> Bool` | Ends with |
| `yi_si_json` | `to_json` | `(Any) -> Str` | Serialize to JSON |
| `yi_pada_json` | `from_json` | `(Str) -> Any` | Parse from JSON |
| `ba_mu` | `regex_match` | `(Str, Str) -> Bool` | Regex match |
| `wa_akoko` | `regex_find` | `(Str, Str) -> Str` | First regex match |
| `wa_gbogbo` | `regex_find_all` | `(Str, Str) -> List` | All regex matches |
| `ropo` | `regex_replace` | `(Str, Str, Str) -> Str` | Regex replace all |
| `bo_asiri_url` | `url_encode` | `(Str) -> Str` | URL encode |
| `titu_asiri_url` | `url_decode` | `(Str) -> Str` | URL decode |

### Òtúúrúpọ̀n — Math II (ID: 11, Binary: 0010)

Subtraction, division, checked arithmetic.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `din` | `sub` | `(Int, Int) -> Result` | Checked subtraction |
| `pin` | `div` | `(Int, Int) -> Result` | Checked division (float) |
| `pin_odidi` | `div_int` | `(Int, Int) -> Result` | Integer division |
| `din_f` | `sub_f` | `(Float, Float) -> Float` | Float subtraction |
| `pin_f` | `div_f` | `(Float, Float) -> Result` | Float division |
| `ku` | `mod` | `(Int, Int) -> Result` | Modulo |
| `dake` | `neg` | `(Int) -> Result` | Checked negate |
| `idakeji` | `reciprocal` | `(Float) -> Result` | Reciprocal |

### Òfún — Permissions (ID: 15, Binary: 0101)

Capability checking and type reflection.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `le` | `can` | `(Str) -> Bool` | Check capability |
| `ju` | `revoke` | `(Str) -> Null` | Drop capability |
| `awon_agbara` | `capabilities` | `() -> List` | Get current capabilities |
| `iru` | `type_of` | `(Any) -> Str` | Get type name |
| `je` | `is_type` | `(Any, Str) -> Bool` | Type check |
| `afiwe` | `to_debug` | `(Any) -> Str` | Debug representation |

---

## Feature-gated domains

### Òdí — Files (ID: 3, Binary: 1001) – requires `backend`

File I/O and SQLite database operations.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `ka` | `read` | `(Str) -> Result` | Read file |
| `ka_bytes` | `read_bytes` | `(Str) -> Result` | Read file as bytes |
| `ka_ila` | `read_lines` | `(Str) -> Result` | Read file lines |
| `ko` | `write` | `(Str, Str) -> Result` | Write file |
| `fi` | `append` | `(Str, Str) -> Result` | Append to file |
| `wa` | `exists` | `(Str) -> Bool` | Check file exists |
| `pa_faili` | `delete` | `(Str) -> Result` | Delete file |
| `seda_apoti` | `mkdir` | `(Str) -> Result` | Create directory |
| `akojo` | `list_dir` | `(Str) -> Result` | List directory |
| `iwon` | `size` | `(Str) -> Result` | File size |
| `so_db` | `open_db` | `(Str) -> Result` | Open SQLite database |
| `so_db_iranti` | `open_memory_db` | `() -> Result` | Open in-memory database |

### Ọ̀sá — Concurrency (ID: 9, Binary: 0111) – requires `backend`

Async task spawning, channels, synchronization.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `sa` | `spawn` | `(Fn) -> Task` | Spawn async task |
| `sun` | `sleep` | `(Int) -> Null` | Async sleep (ms) |
| `oju_ona` | `channel` | `(Int) -> (Chan, Chan)` | Create mpsc channel |
| `oju_ona_kan` | `oneshot` | `() -> (Chan, Chan)` | Create oneshot channel |
| `titipe` | `mutex` | `(Any) -> Mutex` | Create async mutex |
| `kaka` | `rwlock` | `(Any) -> RwLock` | Create async rwlock |
| `pẹlu_akoko` | `with_timeout` | `(Fn, Int) -> Any` | Future with timeout |
| `jeki` | `yield_now` | `() -> Null` | Yield to scheduler |

### Òtúrá — Networking (ID: 12, Binary: 1011) – requires `backend`

HTTP client with SSRF protection.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `ṣàyẹ̀wò` | `check_url` | `(Str) -> Bool` | Validate URL safety |
| `gba` | `get` | `(Str) -> Result` | HTTP GET |
| `ran` | `post` | `(Str, Str) -> Result` | HTTP POST |
| `de` | `listen` | `(Str) -> Result` | TCP listen |
| `soro` | `connect` | `(Str) -> Result` | TCP connect |

### Ìrẹtẹ̀ — Crypto (ID: 13, Binary: 1101) – requires `crypto`

Hashing, encryption, signing, compression.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `sha256` | `sha256` | `(Bytes) -> Bytes` | SHA-256 hash |
| `sha256_hex` | `sha256_hex` | `(Bytes) -> Str` | SHA-256 hex |
| `sha512` | `sha512` | `(Bytes) -> Bytes` | SHA-512 hash |
| `hmac_sha256` | `hmac` | `(Bytes, Bytes) -> Bytes` | HMAC-SHA256 |
| `base64_encode` | `to_base64` | `(Bytes) -> Str` | Base64 encode |
| `base64_decode` | `from_base64` | `(Str) -> Result` | Base64 decode |
| `hex_encode` | `to_hex` | `(Bytes) -> Str` | Hex encode |
| `hex_decode` | `from_hex` | `(Str) -> Result` | Hex decode |
| `chacha20_encrypt` | `encrypt` | `(Bytes, Bytes, Bytes) -> Result` | ChaCha20-Poly1305 encrypt |
| `chacha20_decrypt` | `decrypt` | `(Bytes, Bytes, Bytes) -> Result` | ChaCha20-Poly1305 decrypt |
| `ed25519_generate` | `generate_keypair` | `() -> Result` | Ed25519 keypair |
| `ed25519_sign` | `sign` | `(Bytes, Bytes) -> Result` | Ed25519 sign |
| `ed25519_verify` | `verify` | `(Bytes, Bytes, Bytes) -> Bool` | Ed25519 verify |
| `random_bytes` | `secure_bytes` | `(Int) -> Result` | Secure random bytes |
| `funpo` | `compress` | `(Bytes, Int) -> Result` | zstd compress |
| `tu` | `decompress` | `(Bytes) -> Result` | zstd decompress |

### Ọ̀ṣẹ́ — Graphics (ID: 14, Binary: 1010) – requires `game`

Terminal canvas for ASCII/Unicode graphics.

| Method | English alias | Signature | Description |
|--------|---------------|-----------|-------------|
| `bere` | `init` | `() -> Null` | Initialize terminal UI |
| `pari` | `end` | `() -> Null` | Restore terminal |
| `tobi` | `resize` | `(Int, Int) -> Null` | Resize canvas |
| `nu` | `clear` | `(Str) -> Null` | Fill canvas |
| `ya` | `draw` | `(Int, Int, Str) -> Null` | Draw character at |
| `ko` | `write` | `(Int, Int, Str) -> Null` | Write text at |
| `ila` | `line` | `(Int, Int, Int, Int, Str) -> Null` | Draw line |
| `onigun` | `rect` | `(Int, Int, Int, Int, Str) -> Null` | Draw rectangle |
| `onigun_kun` | `fill_rect` | `(Int, Int, Int, Int, Str) -> Null` | Filled rectangle |
| `iyokoto` | `circle` | `(Int, Int, Int, Str) -> Null` | Draw circle |
| `han` | `render` | `() -> Null` | Render to screen |

---

## Stack modules (feature-gated)

| Stack | Feature | Description |
|-------|---------|-------------|
| `crypto` | `crypto` | SecretStore, Password hashing, constant-time comparison |
| `backend` | `backend` | HTTP server, ORM client, middleware, request/response |
| `frontend` | `frontend` | HTML elements, safe HTML, localStorage, router, fetch |
| `gamedev` | `game` | Vec2, AABB, ECS, animation, audio, collision, input |
| `ml` | `ml` | Tensor operations, linear layers, SGD optimizer |
| `iot` | `iot` | GPIO, I2C, SPI, serial, timers |
| `fusion` | `fusion` | Fullstack IPC runtime, context, role management |

## FFI (Polyglot)

The `ffi` module (gated by `native_ffi`) provides bidirectional calling between Ifá-Lang, JavaScript (via Boa), Python (via PyO3), and native C libraries (via libloading).

Key types:

| Component | Description |
|-----------|-------------|
| `IfaFfi` | Primary FFI bridge — load and call foreign code |
| `SecureFfi` | Hardened wrapper with symbol allowlists |
| `IfaApi` | Export Ifá-Lang functions for external callers |
| `IfaRpcServer` | JSON-RPC 2.0 HTTP server for Ifá-Lang APIs |

## Infrastructure modules

| Module | Description |
|--------|-------------|
| `Cpu` | Parallel iterators, task graphs, core count (ID: 18) |
| `Gpu` | WGPU compute, shader loading, matrix operations (ID: 19) |
| `Storage` | Key-value store, compaction (ID: 20) |
| `Sys` | Kernel info, memory stats, uptime (ID: 29) |
