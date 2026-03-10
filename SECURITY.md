# Security Policy

## Supported Versions

| Version | Supported          |
| :------ | :----------------- |
| 1.2.x   | :white_check_mark: |
| 1.1.x   | :x:                |
| < 1.0   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in Ifá-Lang, please **DO NOT** open a public issue.

### Process
1. Email `allioladapo@gmail.com
2. Describe the vulnerability with a reproduction script (`.ifa` or `.ifab`).
3. We will acknowledge within 48 hours.
4. Embargo period is typically 90 days.

## Threat Model

### Intepreter Sandbox
The `ifa-core` VM is designed to be sandboxed.
- **Memory Safety**: Guaranteed by Rust (mostly safe code).
- **Resource Limits**: `Opon` enforces memory caps. `Ebo` enforces handle limits.
- **Denial of Service**: `aale` (limits) prevents infinite loops/recursion if configured.

### Embedded
- No OS protections.
- Use `ifa-embedded` with `critical-section` for interrupt safety.
- **WARNING**: Reading from untrusted memory pointers (`Load`/`Store` opcodes) in unprotected mode allows arbitrary memory access. Verify bytecode signatures before execution on hardware.
