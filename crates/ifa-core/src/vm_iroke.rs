//! # Iroke - The Tapper (Dispatcher)
//!
//! Iroke implements the high-performance dispatch loop for the Ifá-Lang VM.
//! It replaces the previous "BatchSorter" design with a direct, inline dispatcher.
//!
//! "We tap the board to invoke the presence." - Cultural Metaphor for Instruction Fetch.

use crate::bytecode::{Bytecode, OpCode};
use crate::error::{IfaError, IfaResult};
use crate::vm::IfaVM;

/// The Tapper - Drives the VM Cycle
///
/// It "taps" (fetches) the next instruction from the bytecode.
/// This function is marked `always_inline` to ensure it is embedded directly into the loop.
#[inline(always)]
pub fn tap(vm: &mut IfaVM, bytecode: &Bytecode) -> IfaResult<OpCode> {
    // 1. Check for Interrupts (GC, Signals) - "Clearing the Board"
    // We check every 1024 ticks to amortize the cost.
    if vm.ticks & 1023 == 0 {
        // checks_interrupts(vm)?; // Placeholder for future checking
    }
    vm.ticks = vm.ticks.wrapping_add(1);

    // 2. Fetch (Tap) the next instruction
    // Security: Check bounds. In a JIT, we might omit this, but for Interpreter, safety first.
    if vm.ip >= bytecode.code.len() {
        return Err(IfaError::Runtime(
            "Instruction Pointer Out of Bounds".into(),
        ));
    }

    // Safe indexing: bounds already checked at line 27 above.
    // The optimizer elides the redundant check in release builds.
    let byte = bytecode.code[vm.ip];
    vm.ip += 1;

    // 3. Decode
    OpCode::from_u8(byte)
        .ok_or_else(|| IfaError::Runtime(format!("Invalid OpCode: 0x{:02X}", byte)))
}
