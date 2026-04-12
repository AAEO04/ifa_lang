param(
  [switch]$Fast
)

$ErrorActionPreference = "Stop"

Write-Host "== Spec checks =="
python tools/check_spec_opcode_sync.py

if (-not $Fast) {
  Write-Host "== Bytecode crate tests =="
  cargo test -p ifa-bytecode
}

Write-Host "OK"

