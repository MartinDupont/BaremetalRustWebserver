use core::fmt;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    pub ELR: u64,   // Exception Link Register
    pub SPSR: u64,  // Saved Program Status Register
    pub SP: u64,    // Stack Pointer
    pub TPIDR: u64, // Thread ID Register
    pub TTBR0: u64, // Translation Table Base Register 0
    pub TTBR1: u64, // Translation Table Base Register 1
    pub q: [u128; 32],
    pub x: [u64; 30],
    pub lr: u64,
    _xzr: u64,
}

