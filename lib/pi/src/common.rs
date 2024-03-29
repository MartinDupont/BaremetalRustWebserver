/// The address where I/O peripherals are mapped to.
pub const IO_BASE: usize = 0xFE00_0000;
pub const CLOCK_HZ: u64 = 500 * 1000 * 1000;
pub const IO_BASE_END: usize = 0x1_0000_0000;

/// The base address of the `GPIO` registers
pub const GPIO_BASE: usize = IO_BASE + 0x200000;

pub const EMMC_OFFSET: usize = 0x0034_0000;

pub const EMMC_START: usize = IO_BASE + EMMC_OFFSET;

/// The number of cores in Rpi4
pub const NCORES: usize = 4;

// TODO: Not checked
/// The base of physical addresses that each core is spinning on
pub const SPINNING_BASE: *mut usize = 0xd8 as *mut usize;


/// Generates `pub enums` with no variants for each `ident` passed in.
pub macro states($($name:ident),*) {
    $(
        /// A possible state.
        #[doc(hidden)]
        pub enum $name {  }
    )*
}
