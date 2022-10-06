#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(raw_vec_internals)]
#![feature(panic_info_message)]

#[cfg(not(test))]
mod init;

extern crate alloc;
pub mod console;
pub mod mutex;
pub mod shell;
pub mod allocator;
pub mod fs;

use console::kprintln;

use pi::uart::uart_io;
use shim::io::Write;
use shim::io::Read;
use allocator::Allocator;
use fs::FileSystem;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();

fn kmain() -> ! {
    unsafe {
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
    }
    shell::shell("> ");
}
