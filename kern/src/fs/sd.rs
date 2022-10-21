use fat32::traits::BlockDevice;
use pi::emmc::{EMMCController, SdResult};
use shim::io;
use shim::ioerr;

use crate::console::{kprintln};
use crate::mutex::Mutex;

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

pub static EMMC_CONT: Mutex<EMMCController> = unsafe { Mutex::new(EMMCController::new()) };


impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization. We can enforce the requirement in safe Rust code
    /// with atomic memory access, but we can't use it yet since we haven't
    /// written the memory management unit (MMU).
    pub unsafe fn new() -> Result<Sd, io::Error> {
        match EMMC_CONT.lock().emmc_init_card() {
            SdResult::EMMC_OK => {
                kprintln!("EMMC2 driver initialized...\n");
                Ok(Sd {})
            }
            _ => {
                ioerr!(BrokenPipe, "sending command")
            }
        }
    }
}

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
        if buf.len() < 512 {
            return ioerr!(InvalidInput, "buf.len() < 512");
        }
        if n > 0x7fffffff {
            return ioerr!(InvalidInput, "n > 0x7fffffff");
        }
        let res = EMMC_CONT.lock().emmc_transfer_blocks(n as u32, 1, buf, false);

        return match res {
            SdResult::EMMC_OK => {
                Ok(512)
            },
            SdResult::EMMC_TIMEOUT => ioerr!(BrokenPipe, "timeout"),
            SdResult::EMMC_ERROR_APP_CMD => ioerr!(TimedOut, "error sending commsnt"),
            _ => ioerr!(Other, "unknown error"),
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
