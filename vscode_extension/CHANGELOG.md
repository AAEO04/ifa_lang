# Changelog

All notable changes to the Ifá-Lang VS Code extension.

## [1.2.0] - 2026-01-08

### Added
- 18 new snippets for advanced features:
  - **Ẹbọ (RAII)**: `ebo`, `ebo-scope` - Resource cleanup guards
  - **Àjọṣe (Reactive)**: `signal`, `effect`, `computed` - Reactive programming
  - **Èèwọ̀ (Taboo)**: `ewọ` - Architectural boundary declarations
  - **Ìwà Pẹ̀lẹ́**: `iwa-pele`, `iwa-retry` - Graceful error handling
  - **Sandbox**: `sandbox-wasm`, `sandbox-native` - Sandbox execution modes
  - **Permissions**: `allow-read`, `allow-net` - CLI permission flags
  - **Security**: `caps`, `password-hash`, `password-verify`, `hmac-sign`
  - **OpeleChain**: Tamper-evident audit logging

### Changed
- Updated keywords to include `sandbox`, `capability`, `security`

## [1.1.0] - 2026-01-07

### Added
- 30+ code snippets for common Odù patterns
- FFI syntax highlighting for new types
- Stack highlighting (IoT, ML, Crypto)
- Commands: Run file (Ctrl+Shift+R), REPL, Format
- LSP client with restart command

## [0.1.1] - 2024-12-30

### Added
- Babalawo Debugger with breakpoint support
- LSP integration for intellisense
- Dual Yoruba/English syntax highlighting

### Fixed
- Improved keyword recognition for all 16 Odù domains

## [0.1.0] - 2024-12-01

### Added
- Initial release
- Syntax highlighting for `.ifa` files
- Basic language configuration (brackets, comments)
- TextMate grammar for Ifá syntax
