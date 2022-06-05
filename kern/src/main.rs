#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use console::kprintln;

use pi::uart::uart_io;
use shim::io::Write;
use shim::io::Read;
// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.

fn kmain() -> ! {
    let mut mini_uart = uart_io::MiniUart::new();
    let mut buf = [b'a'; 1];
    loop {
        mini_uart.write(&buf);
    }
}
