use shim::io;
use shim::path::{Path, PathBuf};

use stack_vec::StackVec;

use pi::atags::Atags;
use pi::timer;

use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry};

use crate::console::{kprint, kprintln, CONSOLE};
use crate::ALLOCATOR;

use shim::io::Write;
use shim::io::Read;

use core::str;
use core::fmt;
use aarch64::{current_el, DAIF, SPSel};
use pi::armlocal::ArmLocalController;
use crate::param::TICK;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns.
pub fn shell(prefix: &str) -> () {
    const CMD_LEN: usize = 512;
    const ARG_LEN: usize = 64;
    kprintln!();
    kprintln!("======================================================================");
    kprintln!("                           Welcome to my OS                           ");
    kprintln!("======================================================================");
    kprintln!();
    'outer: loop {
        let mut cmd_buf = [0u8; CMD_LEN];
        let mut arg_buf = [""; ARG_LEN];

        kprint!("{}", prefix);

        let mut i = 0;
        'cmd: loop {
            if i == CMD_LEN {
                kprintln!();
                kprintln!("command length exceeds {}", CMD_LEN);
                break 'cmd;
            }

            let byte = CONSOLE.lock().read_byte();
            if byte == b'\n' || byte == b'\r' {
                kprint!("\n");
                let cmd_result = str::from_utf8(&cmd_buf[0..i]);
                if let Ok(cmd) = cmd_result {
                    match Command::parse(cmd, &mut arg_buf) { // enter
                        Err(Error::Empty) => {}
                        Err(Error::TooManyArgs) => {
                            kprintln!("error: too many arguments");
                        }
                        Ok(cmd) => {
                            let result = process_command(cmd);
                            if result.is_none() {
                                break 'outer
                            }
                        }
                    }
                    break 'cmd;
                } else {
                    kprintln!("Could not parse input bytes into string");
                    kprint!("\u{7}");
                    kprintln!("");
                    cmd_buf = [0u8; CMD_LEN];
                    arg_buf = [""; ARG_LEN];
                }
            } else if byte == 8 || byte == 127 { // backspace
                if i > 0 {
                    kprint!("\u{8} \u{8}");
                    i -= 1
                }
            } else {
                cmd_buf[i] = byte;
                CONSOLE.lock().write_byte(byte);
                i += 1;
            }
        }
    }
}


fn process_command(cmd: Command) -> Option<()> {
    let arg1 = cmd.path();
    match arg1 {
        "echo" => {
            let mut count = 0;
            for arg in cmd.args {
                if count > 0 {
                    kprint!("{} ", arg);
                }
                count += 1
            }
            kprint!("\n");
        }
        "exit" => {
            return None;
        }
        "mem" => {
            if cmd.args.len() != 2 {
                kprintln!("Accepts exactly one argument");
                return Some(())
            }
            let mem = cmd.args[1];
            let my_int = u64::from_str_radix(mem.trim_start_matches("0x"), 16);
            match my_int {
                Ok(mem_address) => {
                    let value = unsafe { &mut *(mem_address as *mut [u32; 8]) };
                    kprintln!("{:X?}", value)
                },
                Err(e) => {
                    kprintln!("{}", e)
                }

            }
        }
        "ack" => unsafe {
            timer::ack();
            kprintln!("Acked!")
        }
        "tick" => unsafe {
            timer::tick_in(TICK);
            kprintln!("Tick set")
        }
        "daif" => unsafe {
            let v = DAIF.get();
            kprintln!("{:X?}", v)
        }
        "brk" => unsafe {
            kprintln!("Brking!");
            asm!("brk 1" :::: "volatile");
            kprintln!("Brked.");
        }
        "current_el" => {
            kprintln!("current_el: {}", unsafe { current_el() });
        }
        "clear_local" => {
            let mut arm_local_controller = ArmLocalController::new();
            arm_local_controller.clear();
            kprintln!("cleared")
        }
        "set_timeout" => {
            if cmd.args.len() != 2 {
                kprintln!("Accepts exactly one argument");
                return Some(())
            }
            let mem = cmd.args[1];
            let my_int = u32::from_str_radix(mem.trim_start_matches("0x"), 16);
            match my_int {
                Ok(value) => {
                    let mut arm_local_controller = ArmLocalController::new();
                    arm_local_controller.set_timeout(value);
                },
                Err(e) => {
                    kprintln!("{}", e)
                }

            }
        }
        "set_peri" => {
            if cmd.args.len() != 2 {
                kprintln!("Accepts exactly one argument");
                return Some(())
            }
            let mem = cmd.args[1];
            let my_int = u32::from_str_radix(mem.trim_start_matches("0x"), 16);
            match my_int {
                Ok(value) => {
                    let mut arm_local_controller = ArmLocalController::new();
                    arm_local_controller.set_peri(value);
                },
                Err(e) => {
                    kprintln!("{}", e)
                }

            }
        }
        "" => {
            kprintln!();
        }
        _ => {
            kprintln!("Unknown command: {}", arg1);
        }
    };
    return Some(());
}
