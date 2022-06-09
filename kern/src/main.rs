#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(raw_vec_internals)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;
pub mod allocator;

use console::kprintln;

use pi::uart::uart_io;
use shim::io::Write;
use shim::io::Read;
use allocator::Allocator;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

fn kmain() -> ! {
    unsafe {
        ALLOCATOR.initialize();
    }
    shell::shell("> ");
}
