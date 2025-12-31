# Ifá-Lang Examples

Organized examples demonstrating all features of Ifá-Lang.

> **Note:** All examples use **BOTH Yoruba and English syntax** side-by-side to show that both work interchangeably!

---

##  Structure

```
examples/
├── 01_basics/        # Getting started
├── 02_features/      # Language features
├── 03_compounds/     # Compound Odù (256/4096)
└── 04_apps/          # Real-world applications
```

---

## 01_basics/ — Getting Started

| File | Description |
|------|-------------|
| `hello.ifa` | Hello World (dual syntax) |
| `demo.ifa` | Quick math demo (dual syntax) |
| `math.ifa` | Complete math operations (dual syntax) |
| `yoruba_style.ifa` | Full Yoruba-only syntax |
| `english_style.ifa` | Full English-only syntax |
| `mixed_syntax.ifa` | Both syntaxes combined |

---

## 02_features/ — Language Features

| File | Description |
|------|-------------|
| `oop_test.ifa` | Classes & objects with `odù` / `class` |
| `match_test.ifa` | Pattern matching with `yàn` / `match` |
| `lambda_test.ifa` | Arrow functions with `padà` / `return` |
| `debug_test.ifa` | Flight recorder debugging |
| `crash_test.ifa` | Panic handler demo |
| `phase2_test.ifa` | Regex, dates, env vars |

---

## 03_compounds/ — Compound Odù

| File | Description |
|------|-------------|
| `compound_256.ifa` | 2-component (Parent_Child) |
| `compound_4096.ifa` | 3-component (Grandparent_Parent_Child) |
| `user_compound.ifa` | Creating custom compounds |

---

## 04_apps/ — Real Applications

| File | Description |
|------|-------------|
| `crypto_bot.ifa` | Cryptocurrency trading bot (dual syntax) |
| `web_server.ifa` | TCP echo server (dual syntax) |
| `file_processor.ifa` | File I/O + JSON (dual syntax) |

---

## Running Examples

```bash
# Run any example
ifa run examples/01_basics/hello.ifa

# Compile to bytecode
ifa bytecode examples/01_basics/math.ifa

# Or compile to native binary
ifa build examples/04_apps/crypto_bot.ifa -o bot
```

---

## Dual Syntax Quick Reference

| Yoruba | English | Purpose |
|--------|---------|---------|
| `ìbà` | `import` | Import module |
| `ayanmọ` | `let` | Declare variable |
| `ese` | `fn` | Define function |
| `odù` | `class` | Define class |
| `ti` | `if` | Conditional |
| `bibẹkọ` | `else` | Else branch |
| `nigba` | `while` | While loop |
| `padà` | `return` | Return value |
| `àṣẹ` | `end` | End program |

---

**Àṣẹ!**

