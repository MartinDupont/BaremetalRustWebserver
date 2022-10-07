use core::time::Duration;
use shim::io;
use shim::ioerr;

use crate::console::{CONSOLE, Console, kprint, kprintln};

use fat32::traits::BlockDevice;
use pi::emmc::{EMMCController, SdResult};

const ERR_TIMEOUT: i32 = -1;
const ERR_SENDING_CMD: i32 = -2;

use pi::timer::spin_sleep;

// Define a `#[no_mangle]` `wait_micros` function for use by `libsd`.
// The `wait_micros` C signature is: `void wait_micros(unsigned int);`
#[no_mangle]
fn wait_micros(micros: u32) {
    spin_sleep(Duration::from_micros(micros as u64 * 1000));
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

// TODO: REMOVE Console from EMMC controller after I'm finished debugging!
pub static EMMC_CONT: EMMCController<Console> =
    unsafe { EMMCController::new(Console::new()) };

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization. We can enforce the requirement in safe Rust code
    /// with atomic memory access, but we can't use it yet since we haven't
    /// written the memory management unit (MMU).
    pub unsafe fn new() -> Result<Sd, io::Error> {
        match &EMMC_CONT.emmc_init_card() {
            pi::emmc::SdResult::EMMC_OK => {
                kprintln!("EMMC2 driver initialized...\n");
                Ok(Sd {})
            }
            _ => {
                ioerr!(BrokenPipe, "sending command")
            }
        }
    }
}

#[repr(align(4))]
struct Sector([u8; 512]);

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        // kprintln!("DBG Read sector {}", n);
        if buf.len() < 512 {
            return ioerr!(InvalidInput, "buf.len() < 512");
        }
        if n > 0x7fffffff {
            return ioerr!(InvalidInput, "n > 0x7fffffff");
        }
        // if core::mem::align_of_val(buf) < 4 {
        //     kprintln!("align: {}", core::mem::align_of_val(buf));
        //     return ioerr!(InvalidInput, "align_of_val(buf) < 4");
        // }
        let res = &EMMC_CONT.emmc_transfer_blocks(n as u32, 1, buf, false);

        return match res {
            SdResult::EMMC_OK => {
                Ok(512)
            },
            SdResult::EMMC_TIMEOUT => ioerr!(BrokenPipe, "timeout"),
            SdResult::EMMC_ERROR_APP_CMD => ioerr!(TimedOut, "error sending commsnt"),
            r => ioerr!(Other, "unknown error"),
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
