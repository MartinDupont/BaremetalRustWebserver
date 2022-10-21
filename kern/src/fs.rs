pub use fat32::traits;
use fat32::vfat::{Dir, Entry, File, VFat, VFatHandle};
use shim::io;
use shim::newioerr;
use shim::path::Path;

use alloc::rc::Rc;
use core::fmt::{self, Debug};

use crate::mutex::Mutex;

use self::sd::Sd;

pub mod sd;

#[derive(Clone)]
pub struct PiVFatHandle(Rc<Mutex<VFat<Self>>>);

// These impls are *unsound*. We should use `Arc` instead of `Rc` to implement
// `Sync` and `Send` trait for `PiVFatHandle`. However, `Arc` uses atomic memory
// access, which requires MMU to be initialized on ARM architecture. Since we
// have enabled only one core of the board, these unsound impls will not cause
// any immediate harm for now. We will fix this in the future.
unsafe impl Send for PiVFatHandle {}
unsafe impl Sync for PiVFatHandle {}

impl Debug for PiVFatHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "PiVFatHandle")
    }
}

impl VFatHandle for PiVFatHandle {
    fn new(val: VFat<PiVFatHandle>) -> Self {
        PiVFatHandle(Rc::new(Mutex::new(val)))
    }

    fn lock<R>(&self, f: impl FnOnce(&mut VFat<PiVFatHandle>) -> R) -> R {
        f(&mut self.0.lock())
    }
}

pub struct FileSystem(Mutex<Option<PiVFatHandle>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file system failed to initialize.
    pub unsafe fn initialize(&self) {
        let sd_device = Sd::new().expect("No SD card found");

        let handle = VFat::<PiVFatHandle>::from(sd_device).expect("Could not initialize filesystem from SD device");
        *self.0.lock() = Some(handle);
    }
}

impl fat32::traits::FileSystem for &FileSystem {
    type File = File<PiVFatHandle>;
    type Dir = Dir<PiVFatHandle>;
    type Entry = Entry<PiVFatHandle>;


    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        self.0.lock().as_ref().unwrap().open(path)
    }

    fn open_file<P: AsRef<Path>>(self, path: P) -> io::Result<Self::File> {
        use fat32::traits::Entry;
        let thing = self.open(path)?;
        thing.into_file().ok_or(newioerr!(NotFound, "Is a directory, not a file"))
    }

    fn open_dir<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Dir> {
        use fat32::traits::Entry;
        let thing = self.open(path)?;
        thing.into_dir().ok_or(newioerr!(NotFound, "Is a file, not a directory"))
    }
}
