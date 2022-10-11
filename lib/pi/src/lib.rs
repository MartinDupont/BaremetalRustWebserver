#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(duration_float)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(never_type)]
#![no_std]
#![feature(ptr_offset_from)]
#![feature(ptr_cast)]
#![feature(type_alias_enum_variants)]
#![feature(optin_builtin_traits)]



pub mod atags;
pub mod common;
pub mod gpio;
pub mod interrupt;
pub mod local_interrupt;
pub mod timer;
pub mod uart;
pub mod armlocal;
pub mod emmc;