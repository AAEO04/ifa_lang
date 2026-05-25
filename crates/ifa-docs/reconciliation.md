# Documentation Reconciliation

This file documents discrepancies found between `docs/` (root-level HTML website) and the crate source code in `crates/`. All fixes applied are listed below.

## CLI reference (`docs/tools/cli.html`)

**Status: Rewritten**

| Issue | Fix |
|-------|-----|
| `ifa compile` listed instead of `ifa bytecode` | Corrected command name |
| Missing `ifa build`, `ifa runb`, `ifa deploy`, `ifa debug`, `ifa doc`, `ifa flash` | Added all missing commands |
| `ifa new` listed (doesn't exist; should be `ifa oja init`) | Removed, replaced with `ifa oja` section |
| No `--unstable` flag for `ifa fmt` | Added with note that it hard-errors without it |
| `.ifab` bytecode standalone execution implied | Added security gate note (requires `.ifa` source) |
| No capability flags documented (`--allow-*`) | Full flag tables added |
| Implicit `run` not mentioned | Added note |

## Irosu domain (`docs/domains/irosu.html`)

**Status: Fixed**

| Issue | Source (crates/ifa-std/src/irosu.rs) |
|-------|--------|
| Binary pattern said `0011` (Owonrin's pattern) | Corrected to `1100` |
| Listed `fo_inline` (doesn't exist) | Replaced with `so` (print without newline) |
| Listed `ka` (doesn't exist for input) | Replaced with `gbo` (listen/read_line) |
| Listed `ka_airi` (doesn't exist) | Replaced with `gbo_nomba`, `gbo_odidi` |
| Missing `mo` (clear screen) | Added |
| Missing `san` (flush output) | Added |
| Missing `kigbe` (print to stderr) | Added |
| Missing `ilosiwaju` (progress bar) | Added |

## Ogbe domain (`docs/domains/ogbe.html`)

**Status: Rewritten**

The page listed methods from other domains: `jade` (Oyeku), `duro` (Oyeku), `gba` (not an Ogbe method), `fi` (Odi), `bere` (Odi/Ogunda), `pa` (Odi).

Actual Ogbe methods from `crates/ifa-std/src/ogbe.rs`:
- `awon_ohun` / `args` — CLI arguments
- `ohun` / `arg` — argument at index
- `iye_ohun` / `arg_count` — argument count
- `ayika` / `get_env` — environment variable
- `ayika_tabi` / `get_env_or` — env var with default
- `ile` / `home_dir` — home directory
- `oju_ona` / `cwd` — current working directory
- `eto` / `os` — OS name
- `apẹrẹ` / `arch` — CPU architecture

## API reference (`docs/api/api.html`)

**Status: Fixed**

| Issue | Fix |
|-------|-----|
| "17 Odù domains" | Corrected to "16" |
| Ogbe methods wrong | Updated to actual method list |
| Irosu methods incomplete | Added all missing methods |
| Oyeku methods incomplete | Added exit variants and sleep methods |
| Ohun/Fidio listed as separate domains | Updated: Ohun = Irosu feature-gated methods (not a separate domain); Fidio = not implemented, planned for future |
| Stack links broken (`../deployment/stacks/` not `../stacks/`) | Fixed all 7 links |
| Added reconciliation banner | Links to crates/ifa-docs/ as canonical truth |

## Ohun (Audio) page (`docs/domains/ohun.html`)

**Status: Banner added**

Audio does not exist as a separate domain in `crates/ifa-std/src/`. Audio playback is available as feature-gated methods on Irosu (`Irosu.siro`, `Irosu.play`, `Irosu.siro_duro`, `Irosu.play_blocking`, `Irosu.kigbe_orin`) behind the `audio` feature. The existing page documents a planned future API. A reconciliation banner was added.

## Fidio (Video) page (`docs/domains/fidio.html`)

**Status: Reconciliation banner — not implemented, scheduled for removal**

Fidio is not implemented in the VM. The handler source file at `crates/ifa-interpreter/src/interpreter/handlers/fidio.rs` is dead code scheduled for removal. Planned for a future release.

## Remaining unfixed discrepancies

The following were identified but not individually fixed (18 domain pages × partial method mismatches). They link to `crates/ifa-docs/std-library.md` as the canonical source of truth:

| File | Known discrepancy |
|------|-------------------|
| `docs/domains/oyeku.html` | Methods named `exit`/`sleep` instead of crate names `ku`/`sun_ms`/`sun`/`duro`/`da_duro`. Missing exit variant methods. |
| `docs/domains/owonrin.html` | Method names differ from crate API |
| `docs/domains/obara.html` | Missing many math methods (trig, stats, constants) |
| `docs/domains/okanran.html` | Method names differ from crate API |
| `docs/domains/ogunda.html` | Method names differ; missing process operations |
| `docs/domains/ika.html` | Missing regex, JSON, URL methods |
| `docs/domains/oturupon.html` | Method names differ from crate API |
| `docs/domains/odi.html` | Missing SQLite database methods |
| `docs/domains/osa.html` | Method names differ from crate async API |
| `docs/domains/otura.html` | Method names differ; missing SSRF protection details |
| `docs/domains/irete.html` | Missing many crypto methods |
| `docs/domains/ose.html` | Method names differ from crate terminal UI API |
| `docs/domains/ofun.html` | Method names differ from crate reflection API |
| `docs/domains/iwori.html` | Method names differ from crate API |
| `docs/reference/grammar.html` | May contain syntax drift from actual `grammar.pest` |

The `crates/ifa-docs/` directory is the authoritative, crate-verified reference for all above domains.
