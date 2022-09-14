use alloc::boxed::Box;
use core::time::Duration;

use crate::console::CONSOLE;
use crate::process::{Process, State};
use crate::traps::TrapFrame;
use crate::SCHEDULER;
use kernel_api::*;
use pi::timer::{current_time, Timer};

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sys_sleep(ms: u32, tf: &mut TrapFrame) {
    let start_time = current_time();

    let sleep_fn = Box::new(move |p: &mut Process| {
        let elapsed = (current_time() - start_time).as_millis() as u32;
        if elapsed > ms {
            p.context.x[0] = elapsed as u64;
            p.context.x[7] = 1;
            true
        } else {
            false
        }
    });

    let new_state = State::Waiting(sleep_fn);
    SCHEDULER.switch(new_state, tf);
}

/// Returns current time.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns two
/// parameter:
///  - current time as seconds
///  - fractional part of the current time, in nanoseconds.
pub fn sys_time(tf: &mut TrapFrame) {
    unimplemented!("sys_time()");
}

/// Kills current process.
///
/// This system call does not take any parameters and does not return any values.
pub fn sys_exit(tf: &mut TrapFrame) {
    unimplemented!("sys_exit()");
}

/// Write to console.
///
/// This system call takes one parameter: a u8 character to print.
///
/// It only returns the usual status value.
pub fn sys_write(b: u8, tf: &mut TrapFrame) {
    unimplemented!("sys_write()");
}

/// Returns current process's ID.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns a
/// parameter: the current process's ID.
pub fn sys_getpid(tf: &mut TrapFrame) {
    unimplemented!("sys_getpid()");
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    use crate::console::kprintln;
    match num {
        1 => sys_sleep(tf.x[0] as u32, tf),
        _ => {}
    }
}
