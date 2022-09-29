use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use core::borrow::BorrowMut;
use core::{fmt, mem};
use pi::timer::tick_in;
use alloc::vec::Vec;

use core::ffi::c_void;
use core::time::Duration;

use aarch64::*;
use pi::interrupt::{Controller, Interrupt};
use pi::armlocal::ArmLocalController;
use pi::local_interrupt::LocalInterrupt;
use smoltcp::time::Instant;

use crate::mutex::Mutex;
use crate::net::uspi::TKernelTimerHandle;
use crate::param::*;
use crate::percore::{get_preemptive_counter, is_mmu_ready, local_irq};
use crate::process::{Id, Process, State};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;
use crate::{GLOBAL_IRQ, process, shell, VMM};
use crate::SCHEDULER;

use crate::console::{kprint, kprintln, CONSOLE};
use crate::{ETHERNET, USB};

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Box<Scheduler>>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Enters a critical region and execute the provided closure with a mutable
    /// reference to the inner scheduler.
    pub fn critical<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut Scheduler) -> R,
    {
        let mut guard = self.0.lock();
        f(guard.as_mut().expect("scheduler uninitialized"))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.critical(move |scheduler| scheduler.add(process))
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::schedule_out()` and `Scheduler::switch_to()`.
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Id {
        self.critical(|scheduler| scheduler.schedule_out(new_state, tf));
        self.switch_to(tf)
    }

    /// Loops until it finds the next process to schedule.
    /// Call `wfi()` in the loop when no process is ready.
    /// For more details, see the documentation on `Scheduler::switch_to()`.
    ///
    /// Returns the process's ID when a ready process is found.
    pub fn switch_to(&self, tf: &mut TrapFrame) -> Id {
        loop {
            let rtn = self.critical(|scheduler| scheduler.switch_to(tf));
            if let Some(id) = rtn {
                trace!(
                    "[core-{}] switch_to {:?}, pc: {:x}, lr: {:x}, x29: {:x}, x28: {:x}, x27: {:x}",
                    affinity(),
                    id,
                    tf.ELR,
                    tf.lr,
                    tf.x[29],
                    tf.x[28],
                    tf.x[27]
                );
                return id;
            }

            aarch64::wfi();
        }
    }

    /// Kills currently running process and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::kill()`.
    #[must_use]
    pub fn kill(&self, tf: &mut TrapFrame) -> Option<Id> {
        self.critical(|scheduler| scheduler.kill(tf))
    }

    /// Starts executing processes in user space using timer interrupt based
    /// preemptive scheduling. This method should not return under normal
    /// conditions.
    pub fn start(&self) -> ! {

        let mut tf = Box::new(TrapFrame::default());
        self.critical(|scheduler| scheduler.switch_to(&mut tf));


        unsafe {
            asm!("mov x0, $0
                  mov sp, x0"
                 :: "r"(tf)
                 :: "volatile");
            asm!("bl context_restore" :::: "volatile");
            asm!("adr x0, _start
                  mov sp, x0"
                 :::: "volatile");
            asm!("mov x0, #0" :::: "volatile");
            eret();
        }

        loop {}

    }

    /// # Lab 4
    /// Initializes the global timer interrupt with `pi::timer`. The timer
    /// should be configured in a way that `Timer1` interrupt fires every
    /// `TICK` duration, which is defined in `param.rs`.
    ///
    /// # Lab 5
    /// Registers a timer handler with `Usb::start_kernel_timer` which will
    /// invoke `poll_ethernet` after 1 second.
    pub fn initialize_global_timer_interrupt(&self) {
        // Setup timer interrupt
        GLOBAL_IRQ.register(
            Interrupt::Timer1,
            Box::new(|tf| {
                let id = SCHEDULER.switch(State::Ready, tf);
                tick_in(TICK);
            }),
        );
        let mut controller = Controller::new();
        controller.enable(Interrupt::Timer1);
        tick_in(TICK);
    }

    /// Initializes the per-core local timer interrupt with `pi::local_interrupt`.
    /// The timer should be configured in a way that `CntpnsIrq` interrupt fires
    /// every `TICK` duration, which is defined in `param.rs`.
    pub fn initialize_local_timer_interrupt(&self) {
        // Lab 5 2.C
        unimplemented!("initialize_local_timer_interrupt()")
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler.
    pub unsafe fn initialize(&self) {
        let mut process1 = Process::new().expect("new process");
        let mut tf = &mut process1.context;
        tf.ELR = USER_IMG_BASE as *const u64 as u64;
        //tf.ELR = 0xffff_ffff_c000_0000 as *const u64 as u64;
        tf.SPSR = (SPSR_EL1::M & 0b0000) | SPSR_EL1::F | SPSR_EL1::A | SPSR_EL1::D;
        tf.SP = process1.stack.top().as_u64();
        tf.TTBR0 = crate::VMM.get_baddr().as_u64();
        tf.TTBR1 = process1.vmap.get_baddr().as_u64();
        self.test_phase_3(&mut process1);

        // let mut process2 = Process::new().expect("new process");
        // let mut tf = &mut process2.context;
        // tf.ELR = USER_IMG_BASE as *const u64 as u64;
        // tf.SPSR = (SPSR_EL1::M & 0b0000) | SPSR_EL1::F | SPSR_EL1::A | SPSR_EL1::D;
        // tf.SP = process2.stack.top().as_u64();
        // tf.TTBR0 = crate::VMM.get_baddr().as_u64();
        // tf.TTBR1 = process2.vmap.get_baddr().as_u64();
        // self.test_phase_3(&mut process2);

        let mut scheduler = Scheduler::new();
        scheduler.add(process1);
        //scheduler.add(process2);
        *self.0.lock() = Some(Box::new(scheduler));
    }

    // The following method may be useful for testing Phase 3:
    //
    // * A method to load a extern function to the user process's page table.
    //
    pub fn test_phase_3(&self, proc: &mut Process){
        use crate::vm::{VirtualAddr, PagePerm};

        let mut page = proc.vmap.alloc(
            VirtualAddr::from(USER_IMG_BASE as u64), PagePerm::RWX);

        let text = unsafe {
            core::slice::from_raw_parts(test_user_process as *const u8, 24)
        };

        page[0..24].copy_from_slice(text);
    }
}

#[no_mangle]
pub extern "C" fn start_shell1() {
    loop {shell::shell("1>")}
}

#[no_mangle]
pub extern "C" fn start_shell2() {
    loop {shell::shell("2>")}
}


/// Poll the ethernet driver and re-register a timer handler using
/// `Usb::start_kernel_timer`.
extern "C" fn poll_ethernet(_: TKernelTimerHandle, _: *mut c_void, _: *mut c_void) {
    // Lab 5 2.B
    unimplemented!("poll_ethernet")
}

/// Internal scheduler struct which is not thread-safe.
pub struct Scheduler {
    processes: VecDeque<Process>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        Scheduler {
            processes: VecDeque::new(),
            last_id: None,
        }
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        let new_id = self.last_id.unwrap_or(0) + 1;

        self.last_id = Some(new_id);
        let mut tf = &mut process.context;
        tf.TPIDR = new_id;

        self.processes.push_back(process);

        Some(new_id)
    }

    /// Finds the currently running process, sets the current process's state
    /// to `new_state`, prepares the context switch on `tf` by saving `tf`
    /// into the current process, and push the current process back to the
    /// end of `processes` queue.
    ///
    /// If the `processes` queue is empty or there is no current process,
    /// returns `false`. Otherwise, returns `true`.
    fn schedule_out(&mut self, new_state: State, tf: &mut TrapFrame) -> bool {
        match self.processes.pop_front() {
            None => false,
            Some(mut old_process) => {
                old_process.state = new_state;
                old_process.context = Box::new(*tf);
                self.processes.push_back(old_process);
                true
            }
        }
    }

    /// Finds the next process to switch to, brings the next process to the
    /// front of the `processes` queue, changes the next process's state to
    /// `Running`, and performs context switch by restoring the next process`s
    /// trap frame into `tf`.
    ///
    /// If there is no process to switch to, returns `None`. Otherwise, returns
    /// `Some` of the next process`s process ID.
    fn switch_to(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        let mut i = 0;
        while let Some(mut process) = self.processes.swap_remove_front(i) {
            if process.is_ready() {
                process.state = State::Running;
                *tf = *process.context;
                let id = process.context.TPIDR;
                self.processes.push_front(process);
                return Some(id);
            }
            self.processes.push_front(process);
            i += 1;
        }
        return None;
    }

    /// Kills currently running process by scheduling out the current process
    /// as `Dead` state. Releases all process resources held by the process,
    /// removes the dead process from the queue, drops the dead process's
    /// instance, and returns the dead process's process ID.
    fn kill(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        self.schedule_out(State::Dead, tf);
        let process = self.processes.pop_back()?;
        Some((*process.context).TPIDR)
    }

    /// Releases all process resources held by the current process such as sockets.
    fn release_process_resources(&mut self, tf: &mut TrapFrame) {
        // Lab 5 2.C
        unimplemented!("release_process_resources")
    }

    /// Finds a process corresponding with tpidr saved in a trap frame.
    /// Panics if the search fails.
    pub fn find_process(&mut self, tf: &TrapFrame) -> &mut Process {
        for i in 0..self.processes.len() {
            if self.processes[i].context.TPIDR == tf.TPIDR {
                return &mut self.processes[i];
            }
        }
        panic!("Invalid TrapFrame");
    }
}

impl fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.processes.len();
        write!(f, "  [Scheduler] {} processes in the queue\n", len)?;
        for i in 0..len {
            write!(
                f,
                "    queue[{}]: proc({:3})-{:?} \n",
                i, self.processes[i].context.TPIDR, self.processes[i].state
            )?;
        }
        Ok(())
    }
}

pub extern "C" fn test_user_process() -> ! {
    loop {
        let ms = 10000;
        let error: u64;
        let elapsed_ms: u64;

        unsafe {
            asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
                 : "=r"(elapsed_ms), "=r"(error)
                 : "r"(ms)
                 : "x0", "x7"
                 : "volatile");
        }
    }
}
