use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};
use shim::const_assert_size;


pub const ARM_LOCAL_BASE: usize = 0xFF800000;

// 0x00 ARM_CONTROL ARM Timer and AXI Error IRQ control
// 0x0c CORE_IRQ_CONTROL VideoCore Interrupt Control
// 0x10 PMU_CONTROL_SET PMU Bit Set
// 0x14 PMU_CONTROL_CLR PMU Bit Clear
// 0x24 PERI_IRQ_ROUTE0 Peripheral Interrupt Routing (Bank 0)
// 0x30 AXI_QUIET_TIME AXI Outstanding Transaction Time and IRQ Control
// 0x34 LOCAL_TIMER_CONTROL Local Timer Control
// 0x38 LOCAL_TIMER_IRQ


#[allow(non_snake_case)]
#[repr(C)]
struct ArmLocalRegisters {
    ARM_CONTROL: Volatile<u32>,
    _res0: [Volatile<u32>; 2],
    CORE_IRQ_CONTROL: Volatile<u32>,
    PMU_CONTROL_SET: Volatile<u32>,
    PMU_CONTROL_CLR: Volatile<u32>,
    _res1: [Volatile<u32>; 3],
    PERI_IRQ_ROUTE0: Volatile<u32>,
    _res2: [Volatile<u32>; 2],
    AXI_QUIET_TIME: Volatile<u32>,
    LOCAL_TIMER_CONTROL: Volatile<u32>,
    LOCAL_TIMER_IRQ: Volatile<u32>,
}

const_assert_size!(ArmLocalRegisters, 0x3C - 0x0);


/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct ArmLocalController {
    registers: &'static mut ArmLocalRegisters,
}

impl ArmLocalController {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> ArmLocalController {
        ArmLocalController {
            registers: unsafe { &mut *((ARM_LOCAL_BASE) as *mut ArmLocalRegisters) },
        }
    }

    pub fn setup(&mut self) {
        let enable = (0b11 << 28);
        let timeout = 0xfffff;
        self.registers.LOCAL_TIMER_CONTROL.write(enable | timeout);
        self.registers.PERI_IRQ_ROUTE0.write(0x01000000)
    }

    pub fn clear(&mut self) {
        self.registers.LOCAL_TIMER_IRQ.write(0xc0000000)
    }

    pub fn set_timeout(&mut self, val: u32) {
        let enable = (0b11 << 28);
        self.registers.LOCAL_TIMER_CONTROL.write(enable | val)
    }

    pub fn set_peri(&mut self, val: u32) {
        self.registers.PERI_IRQ_ROUTE0.write(val)
    }
}

