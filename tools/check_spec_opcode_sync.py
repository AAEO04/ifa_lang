#!/usr/bin/env python3
"""
Checks that `IFA_LANG_RUNTIME_SPEC.md` §17.2 opcode table is in sync with
`crates/ifa-bytecode/src/lib.rs`'s OpCode byte assignments.

Exit codes:
  0  - in sync
  2  - mismatch
  3  - could not parse expected sections
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Mismatch:
    kind: str
    detail: str


def parse_code_opcodes(code_text: str) -> dict[str, int]:
    pairs = re.findall(
        r"^\s*([A-Za-z0-9_]+)\s*=\s*0x([0-9A-Fa-f]{2})\s*,\s*$",
        code_text,
        flags=re.M,
    )
    return {name: int(h, 16) for name, h in pairs}


def extract_spec_17_2(spec_text: str) -> str:
    start = spec_text.find("### 17.2 Opcode Table")
    if start == -1:
        raise ValueError("Missing '### 17.2 Opcode Table' heading")
    end = spec_text.find("### 17.3", start)
    if end == -1:
        raise ValueError("Missing '### 17.3' heading after 17.2")
    return spec_text[start:end]


def parse_spec_opcodes(spec_17_2: str) -> dict[str, int]:
    # Rows like: | `Push` | `0x01` | ...
    rows = re.findall(
        r"^\|\s*`([^`]+)`\s*\|\s*`0x([0-9A-Fa-f]{2})`\s*\|",
        spec_17_2,
        flags=re.M,
    )
    return {name: int(h, 16) for name, h in rows}


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    spec_path = repo_root / "IFA_LANG_RUNTIME_SPEC.md"
    code_path = repo_root / "crates" / "ifa-bytecode" / "src" / "lib.rs"

    try:
        spec_text = spec_path.read_text(encoding="utf-8")
        code_text = code_path.read_text(encoding="utf-8")
        spec_17_2 = extract_spec_17_2(spec_text)
        spec_ops = parse_spec_opcodes(spec_17_2)
        code_ops = parse_code_opcodes(code_text)
    except Exception as exc:
        print(f"[spec-opcode-sync] parse error: {exc}", file=sys.stderr)
        return 3

    mismatches: list[Mismatch] = []

    missing_in_spec = sorted(set(code_ops) - set(spec_ops))
    missing_in_code = sorted(set(spec_ops) - set(code_ops))
    if missing_in_spec:
        mismatches.append(
            Mismatch("missing_in_spec", ", ".join(missing_in_spec[:50]))
        )
    if missing_in_code:
        mismatches.append(
            Mismatch("missing_in_code", ", ".join(missing_in_code[:50]))
        )

    byte_mism = sorted(
        (
            (name, code_ops[name], spec_ops[name])
            for name in (set(code_ops) & set(spec_ops))
            if code_ops[name] != spec_ops[name]
        ),
        key=lambda t: t[0],
    )
    for name, code_byte, spec_byte in byte_mism:
        mismatches.append(
            Mismatch(
                "byte_mismatch",
                f"{name}: code=0x{code_byte:02X} spec=0x{spec_byte:02X}",
            )
        )

    if mismatches:
        print("[spec-opcode-sync] FAIL", file=sys.stderr)
        for mm in mismatches:
            print(f"  - {mm.kind}: {mm.detail}", file=sys.stderr)
        print(
            f"  - counts: spec={len(spec_ops)} code={len(code_ops)}",
            file=sys.stderr,
        )
        return 2

    print(
        f"[spec-opcode-sync] OK (spec={len(spec_ops)} code={len(code_ops)})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

