
use core::time::Duration;
use aarch64::{CNTFRQ_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use shim::const_assert_size;

use volatile::prelude::*;
use volatile::Volatile;

// The ARM_LOCAL register base address is 0x4c0000000. Note that, unlike other peripheral addresses in this document, this
// is an ARM-only address and not a legacy master address. If Low Peripheral mode is enabled this base address becomes
// 0xff80_0000.
const INT_BASE: usize = 0xFF80_0000;

/// Core interrupt sources (QA7: 4.10)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocalInterrupt {
    CNT_PS_IRQ,
    CNT_PNS_IRQ,
    CNT_HP_IRQ,
    CNT_V_IRQ,
    MAILBOX_IRQ0,
    MAILBOX_IRQ1,
    MAILBOX_IRQ2,
    MAILBOX_IRQ3,
    CORE_IRQ,
    PMU_IRQ,
    AXI_QUIET,
    TIMER_IRQ,
}

impl LocalInterrupt {
    pub const MAX: usize = 12;

    pub fn iter() -> impl Iterator<Item = LocalInterrupt> {
        (0..LocalInterrupt::MAX).map(|n| LocalInterrupt::from(n))
    }
}

impl From<usize> for LocalInterrupt {
    fn from(irq: usize) -> LocalInterrupt {
        match irq {
            11 => LocalInterrupt::TIMER_IRQ,
            10 => LocalInterrupt::AXI_QUIET,
            9 => LocalInterrupt::PMU_IRQ,
            8 => LocalInterrupt::CORE_IRQ,
            7 => LocalInterrupt::MAILBOX_IRQ3,
            6 => LocalInterrupt::MAILBOX_IRQ2,
            5 => LocalInterrupt::MAILBOX_IRQ1,
            4 => LocalInterrupt::MAILBOX_IRQ0,
            3 => LocalInterrupt::CNT_V_IRQ,
            2 => LocalInterrupt::CNT_HP_IRQ,
            1 => LocalInterrupt::CNT_PNS_IRQ,
            0 => LocalInterrupt::CNT_PS_IRQ,
            _ => panic!("invalid value in LocalInterrupt")
        }
    }
}

/// BCM2711 ARM local Registers
#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    ARM_CONTROL: Volatile<u32>,
    __res0 : [Volatile<u32>; 2],
    CORE_IRQ_CONTROL: Volatile<u32>,
    PMU_CONTROL_SET: Volatile<u32>,
    PMU_CONTROL_CLR: Volatile<u32>,
    __res1 : [Volatile<u32>; 3],
    PERI_IRQ_ROUTE0: Volatile<u32>,
    __res2 : [Volatile<u32>; 2],
    AXI_QUIET_TIME: Volatile<u32>,
    LOCAL_TIMER_CONTROL: Volatile<u32>,
    LOCAL_TIMER_IRQ: Volatile<u32>,
    __res3 : [Volatile<u32>; 1],
    TIMER_CNTRL: [Volatile<u32>; 4],
    MAILBOX_CNTRL: [Volatile<u32>; 4],
    IRQ_SOURCE: [Volatile<u32>; 4],
    FIQ_SOURCE: [Volatile<u32>; 4],
}
const_assert_size!(Registers, 0x80);

pub struct LocalController {
    core: usize,
    registers: &'static mut Registers,
}

impl LocalController {
    /// Returns a new handle to the interrupt controller.
    pub fn new(core: usize) -> LocalController {
        LocalController {
            core: core,
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    pub fn enable_local_timer(&mut self) {
        self.registers.TIMER_CNTRL[self.core].write(0b10);
        unsafe {
            CNTP_CTL_EL0.set(CNTP_CTL_EL0.get() | CNTP_CTL_EL0::ENABLE);
            CNTP_CTL_EL0.set(CNTP_CTL_EL0.get() & !CNTP_CTL_EL0::IMASK);
        }
    }

    pub fn is_pending(&self, int: LocalInterrupt) -> bool {
        let register = & self.registers.IRQ_SOURCE[self.core];
        let mask = 1 << int as u32;
        register.has_mask(mask)
    }

    pub fn tick_in(&mut self, t: Duration) {
        let freq = unsafe { CNTFRQ_EL0.get() };
        let ticks = freq as u128 * t.as_micros() / 1000000;
        unsafe { CNTP_TVAL_EL0.set(ticks as u64) };
    }
}

pub fn local_tick_in(core: usize, t: Duration) {
    LocalController::new(core).tick_in(t);
}
