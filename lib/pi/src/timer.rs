use crate::common::IO_BASE;
use core::time::Duration;

use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile};

/// The base address for the ARM system timer registers.
const TIMER_REG_BASE: usize = IO_BASE + 0x3000;

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    CS: Volatile<u32>,
    CLO: ReadVolatile<u32>,
    CHI: ReadVolatile<u32>,
    COMPARE: [Volatile<u32>; 4],
}

/// The Raspberry Pi ARM system timer.
pub struct Timer {
    registers: &'static mut Registers,
}

impl Timer {
    /// Returns a new instance of `Timer`.
    pub fn new() -> Timer {
        Timer {
            registers: unsafe { &mut *(TIMER_REG_BASE as *mut Registers) },
        }
    }

    /// Reads the system timer's counter and returns Duration.
    /// `CLO` and `CHI` together can represent the number of elapsed microseconds.
    pub fn read(&self) -> Duration {
        let clo = self.registers.CLO.read();
        let chi = self.registers.CHI.read();
        let sec: u64 = ((chi as u64) << 32) + clo as u64;
        Duration::from_micros(sec)
    }

    /// Reads the system timer's counter and returns Duration.
    /// `CLO` and `CHI` together can represent the number of elapsed microseconds.
    pub fn read_ticks(&self) -> u32 {
        let clo = self.registers.CLO.read();
        return clo
    }

    /// Sets up a match in timer 1 to occur `t` duration from now. If
    /// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
    /// interrupt will be issued in `t` duration.
    pub fn tick_in(&mut self, t: Duration) {
        let cs = &mut self.registers.CS;
        let register = &mut self.registers.COMPARE[1];

        let clo = self.registers.CLO.read();
        let micros = t.as_micros() as u32;
        cs.write(0b10);
        register.write(clo + micros);
    }

    pub fn ack(&mut self) {
        let cs = &mut self.registers.CS;
        cs.write(0b10);
    }
}

/// Returns current time.
pub fn current_time() -> Duration {
    Timer::new().read()
}

pub fn current_ticks() -> u32 {
    Timer::new().read_ticks()
}

/// Spins until `t` duration have passed.
pub fn spin_sleep(t: Duration) {
    let timer = Timer::new();
    let current_time = timer.read();
    let target_time = current_time.checked_add(t).expect("Duration addition failed");
    while timer.read() <= target_time {}
}

/// Sets up a match in timer 1 to occur `t` duration from now. If
/// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
/// interrupt will be issued in `t` duration.
pub fn tick_in(t: Duration) {
    let mut timer = Timer::new();
    timer.tick_in(t);
}

pub fn ack() {
    let mut timer = Timer::new();
    timer.ack();
}