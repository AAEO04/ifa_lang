import os

# 1. value.rs
path = "crates/ifa-types/src/value.rs"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace("_ => IfaValue::Null,", '_ => panic!("Arithmetic type mismatch: cannot perform operation on incompatible types"),')

with open(path, "w", encoding="utf-8") as f:
    f.write(content)

# 2. parser.rs
path = "crates/ifa-parser/src/parser.rs"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# For parser, we need to replace `_ => {}` with returning an error, but the variable name inside the loop varies (e.g. `p`, `inner`).
# Let's use a regex
import re
content = re.sub(
    r'_ => \{\}',
    r'_ => return Err(IfaError::Parse(format!("Unexpected token: {:?}", p.as_rule()))),',
    content
)
# Some loops use `item` or `decl_inner` instead of `p` but let's check
# I will just write `return Err(IfaError::Parse("Unexpected token in PEG match".into())),`
content = content.replace(
    '_ => return Err(IfaError::Parse(format!("Unexpected token: {:?}", p.as_rule()))),',
    '_ => return Err(IfaError::Parse("Unexpected token in PEG match".into())),'
)

with open(path, "w", encoding="utf-8") as f:
    f.write(content)

# 3. ifa-wasm/Cargo.toml
path = "crates/ifa-wasm/Cargo.toml"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# Change ifa-std features
# currently it has `features = ["wasm"]` but the WASM feature in ifa-std enables GPU deps?
# We want to explicitly disable gpu and bytemuck if they are there, or just use `default-features = false, features = ["rsa_math"]` or whatever it needs.
# The user issue said: "ifa-wasm/Cargo.toml enables ifa-std/wasm which enables gpu feature -> pulls wgpu, bytemuck"
# Let's fix ifa-std/Cargo.toml WASM feature instead, since that's what's broken! Wait, the issue says: "ifa-wasm enables GPU deps in WASM builds"
# In ifa-std/Cargo.toml: wasm = ["rsa_math", "dep:futures-channel"]
# Wait, I didn't see `gpu` in the `wasm` feature in ifa-std! Let's check `ifa-wasm/Cargo.toml`
