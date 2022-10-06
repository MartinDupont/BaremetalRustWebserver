#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(never_type)]
#![no_std]
#![feature(ptr_offset_from)]
#![feature(ptr_cast)]
#![feature(type_alias_enum_variants)]



pub mod atags;
pub mod common;
pub mod gpio;
pub mod timer;
pub mod uart;
pub mod emmc;
mod blockdevice;