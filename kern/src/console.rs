use core::fmt;
use pi::uart::MiniUart;
use shim::io;
use shim::io::Write;
use shim::io::Read;

use crate::mutex::Mutex;

/// A global singleton allowing read/write access to the console.
pub struct Console {
    inner: Option<MiniUart>,
}

impl Console {
    /// Creates a new instance of `Console`.
    pub(crate) const fn new() -> Console {
        Console { inner: None }
    }

    /// Initializes the console if it's not already initialized.
    #[inline]
    fn initialize(&mut self) {
        self.inner = Some(MiniUart::new());
    }

    /// Returns a mutable borrow to the inner `MiniUart`, initializing it as
    /// needed.
    fn inner(&mut self) -> &mut MiniUart {
        if let None = self.inner {
            self.initialize()
        }
        return match &mut self.inner {
            Some(x) => x,
            None => panic!(),
        }
    }

    /// Reads a byte from the UART device, blocking until a byte is available.
    pub fn read_byte(&mut self) -> u8 {
        let inner = self.inner();
        let mut buf = [0u8];
        inner.read(&mut buf).unwrap();
        buf[0]
    }

    /// Writes the byte `byte` to the UART device.
    pub fn write_byte(&mut self, byte: u8) {
        self.inner().write(&[byte]);
    }
}

impl io::Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner().read(buf)
    }
}

impl io::Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner().write_str(s)
    }
}

/// Global `Console` singleton.
pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

/// Internal function called by the `kprint[ln]!` macros.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    #[cfg(not(test))]
    {
        use core::fmt::Write;
        let mut console = CONSOLE.lock();
        core::fmt::Write::write_fmt(&mut *console, args).unwrap();
    }

    #[cfg(test)]
    {
        print!("{}", args);
    }
}

/// Like `println!`, but for kernel-space.
pub macro kprintln {
() => (kprint!("\n")),
($fmt:expr) => (kprint!(concat!($fmt, "\n"))),
($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro kprint($($arg:tt)*) {
_print(format_args!($($arg)*))
}
