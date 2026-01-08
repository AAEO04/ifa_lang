# Ifá-Lang Legacy Python Implementation

> ⚠️ **DEPRECATED**: This is the original Python implementation of Ifá-Lang.
> The active development has moved to Rust in the `/crates` directory.

## Status

This code is preserved for reference but is no longer maintained.
All new features and bug fixes are implemented in the Rust version.

## Structure

```
legacy/
├── src/          # Python interpreter, VM, lexer, parser
├── lib/          # Python standard library (16 Odù domains)
├── tests/        # Python test suite
└── benchmarks/   # Python performance benchmarks
```

## Running (Legacy)

```bash
cd legacy
pip install -r requirements.txt
python -m src.interpreter examples/hello.ifa
```

## Migration Notes

The Rust implementation provides:
- Zero-cost abstractions (vs Python's runtime overhead)
- Compile-time safety (vs Python's runtime errors)
- Cross-compilation to embedded targets
- 10-100x performance improvement

See `/crates` for the new implementation.
