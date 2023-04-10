// Adapted from https://github.com/rust-embedded/msp430.
//
// See also src/imp/msp430.rs.

#[cfg(not(portable_atomic_no_asm))]
use core::arch::asm;

pub(super) use super::super::msp430 as atomic;

#[derive(Clone, Copy)]
pub(super) struct State(u16);

/// Disables interrupts and returns the previous interrupt state.
#[inline]
pub(super) fn disable() -> State {
    let r: u16;
    // SAFETY: reading the status register and disabling interrupts are safe.
    // (see module-level comments of interrupt/mod.rs on the safety of using privileged instructions)
    unsafe {
        // Do not use `nomem` and `readonly` because prevent subsequent memory accesses from being reordered before interrupts are disabled.
        // Do not use `preserves_flags` because DINT modifies the GIE (global interrupt enable) bit of the status register.
        // Refs: https://mspgcc.sourceforge.net/manual/x951.html
        #[cfg(not(portable_atomic_no_asm))]
        asm!(
            "mov R2, {0}",
            "dint {{ nop",
            out(reg) r,
            options(nostack),
        );
        #[cfg(portable_atomic_no_asm)]
        {
            llvm_asm!("mov R2, $0" : "=r"(r) ::: "volatile");
            llvm_asm!("dint { nop" ::: "memory" : "volatile");
        }
    }
    State(r)
}

/// Restores the previous interrupt state.
#[inline]
pub(super) unsafe fn restore(State(r): State) {
    // SAFETY: the caller must guarantee that the state was retrieved by the previous `disable`,
    unsafe {
        // This clobbers the entire status register, but we never explicitly modify
        // flags within a critical session, and the only flags that may be changed
        // within a critical session are the arithmetic flags that are changed as
        // a side effect of arithmetic operations, etc., which LLVM recognizes,
        // so it is safe to clobber them here.
        // See also the discussion at https://github.com/taiki-e/portable-atomic/pull/40.
        //
        // Do not use `nomem` and `readonly` because prevent preceding memory accesses from being reordered after interrupts are enabled.
        // Do not use `preserves_flags` because MOV modifies the status register.
        #[cfg(not(portable_atomic_no_asm))]
        asm!("nop {{ mov {0}, R2 {{ nop", in(reg) r, options(nostack));
        #[cfg(portable_atomic_no_asm)]
        llvm_asm!("nop { mov $0, R2 { nop" :: "r"(r) : "memory" : "volatile");
    }
}
