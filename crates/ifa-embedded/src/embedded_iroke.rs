//! # Embedded Iroke - The Polling Tapper
//!
//! Iroke adapted for constrained, no_std environments.
//! Instead of signals/interrupts, it uses cooperative polling via HAL traits.
//!
//! "In the absence of the King, the Chiefs must consult regularly." - Cooperative Multitasking.

use crate::VmExit;
use crate::{EmbeddedError, EmbeddedVm};

/// Polling Trait for Hardware Abstraction
pub trait IrokePoller {
    /// Check if the system needs to yield (e.g., interrupt pending, time slice expired)
    fn should_yield(&mut self) -> bool;
}

/// The Tapper - Drives the Embedded VM Cycle with Polling
///
/// This function runs the VM loop but injects a polling hook to check the HAL
/// for interrupts or yield conditions on every instruction cycle.
#[inline(always)]
pub fn tap<P, const OPON_SIZE: usize, const STACK_SIZE: usize>(
    vm: &mut EmbeddedVm<OPON_SIZE, STACK_SIZE>,
    code: &[u8],
    poller: &mut P,
) -> Result<VmExit, EmbeddedError>
where
    P: IrokePoller,
{
    vm.resume_with_hook(code, || poller.should_yield())
}

/// Simple implementation for closures
impl<F> IrokePoller for F
where
    F: FnMut() -> bool,
{
    fn should_yield(&mut self) -> bool {
        (self)()
    }
}
