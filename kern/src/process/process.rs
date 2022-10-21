use aarch64;
use aarch64::SPSR_EL1;
use fat32::traits::{Entry, File, FileSystem};
use kernel_api::{OsError, OsResult};
use shim::io;
use shim::io::Read;
use shim::path::Path;
use smoltcp::socket::SocketHandle;

use alloc::boxed::Box;
use core::mem;

use crate::allocator::util::{align_down};
use crate::FILESYSTEM;
use crate::param::*;
use crate::process::{Stack, State};
use crate::traps::TrapFrame;
use crate::vm::*;

/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub context: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The page table describing the Virtual Memory of the process
    pub vmap: Box<UserPageTable>,
    /// The scheduling state of the process.
    pub state: State,
    // Lab 5 2.C
    // Socket handles held by the current process
    // pub sockets: Vec<SocketHandle>,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> OsResult<Process> {
        let stack = Stack::new().ok_or(OsError::NoMemory)?;

        Ok(Process {
            context: Box::new(Default::default()),
            stack: stack,
            state: State::Ready,
            vmap: Box::new(UserPageTable::new()),
        })
    }

    /// Loads a program stored in the given path by calling `do_load()` method.
    /// Sets trapframe `context` corresponding to its page table.
    /// `sp` - the address of stack top
    /// `elr` - the address of image base.
    /// `ttbr0` - the base address of kernel page table
    /// `ttbr1` - the base address of user page table
    /// `spsr` - `F`, `A`, `D` bit should be set.
    ///
    /// Returns Os Error if do_load fails.
    pub fn load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        use crate::VMM;

        let mut p = Process::do_load(pn)?;

        let mut tf = &mut p.context;
        tf.ELR = Self::get_image_base().as_u64();
        tf.SPSR = (SPSR_EL1::M & 0b0000) | SPSR_EL1::F | SPSR_EL1::A | SPSR_EL1::D;
        tf.SP = Self::get_stack_top().as_u64();
        tf.TTBR0 = VMM.get_baddr().as_u64();
        tf.TTBR1 = p.vmap.get_baddr().as_u64();

        Ok(p)
    }

    /// Creates a process and open a file with given path.
    /// Allocates one page for stack with read/write permission, and N pages with read/write/execute
    /// permission to load file's contents.
    fn do_load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        let mut p = Process::new()?;

        let entry = (&FILESYSTEM).open(pn)?;
        let mut file = entry.into_file().ok_or(OsError::NoEntry)?;

        p.vmap.alloc(VirtualAddr::from(Process::get_stack_base()), PagePerm::RW);

        let size = file.size() as usize;
        let mut addr = USER_IMG_BASE;
        let end_addr = addr + size;

        while addr < end_addr {
            let bytes = p.vmap.alloc(VirtualAddr::from(addr), PagePerm::RWX);
            file.read(bytes)?;
            addr += PAGE_SIZE;
        }

        Ok(p)
    }

    /// Returns the highest `VirtualAddr` that is supported by this system.
    pub fn get_max_va() -> VirtualAddr {
        VirtualAddr::from(USER_MAX_VM_SIZE - 1) + Process::get_image_base()
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// memory space.
    pub fn get_image_base() -> VirtualAddr {
        VirtualAddr::from(USER_IMG_BASE)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// process's stack.
    pub fn get_stack_base() -> VirtualAddr {
        // Set the stack base to be the address of the last page. Make sure the result is aligned
        // by the page_size, even though it should already be aligned by hard coded values.
        Process::get_max_va() - VirtualAddr::from(PAGE_SIZE) + VirtualAddr::from(1) &
            VirtualAddr::from(!(PAGE_SIZE - 1))
    }

    /// Returns the `VirtualAddr` represents the top of the user process's
    /// stack.
    pub fn get_stack_top() -> VirtualAddr {
        VirtualAddr::from(align_down(USER_MAX_VA, 16))
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occurred. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        let mut state = mem::replace(&mut self.state, State::Ready);
        match state {
            State::Ready => true,
            State::Waiting(ref mut event_poll_fn) => {
                if event_poll_fn(self) {
                    true
                } else {
                    self.state = state;
                    false
                }
            }
            _ => {
                self.state = state;
                false
            }
        }
    }
}
