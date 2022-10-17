#![allow(non_snake_case)]

use alloc::boxed::Box;
use alloc::string::String;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::{c_void};
use core::slice;
use core::time::Duration;

use pi::interrupt::{Controller, Interrupt};
use pi::timer::{current_ticks, spin_sleep};
use smoltcp::wire::EthernetAddress;

use crate::mutex::Mutex;
use crate::net::Frame;
use crate::traps::irq::IrqHandlerRegistry;
use crate::{ALLOCATOR, FIQ, GLOBAL_IRQ};

const DEBUG_USPI: bool = false;

pub macro uspi_trace {
() => (if DEBUG_USPI { trace!("\n") } ),
($fmt:expr) => (if DEBUG_USPI { trace!(concat!($fmt, "\n")) }),
($fmt:expr, $($arg:tt)*) => (if DEBUG_USPI { trace!(concat!($fmt, "\n"), $($arg)*) })
}

pub type TKernelTimerHandle = u64;
pub type TKernelTimerHandler = Option<
    unsafe extern "C" fn(hTimer: TKernelTimerHandle, pParam: *mut c_void, pContext: *mut c_void),
>;
pub type TIRQHandler = Option<unsafe extern "C" fn(pParam: *mut c_void)>;

mod inner {
    use core::convert::TryInto;
    use core::ptr;
    use core::time::Duration;
    use core::ffi::{c_void};


    use super::{TKernelTimerHandle, TKernelTimerHandler};
    use crate::net::Frame;
    use crate::param::USPI_TIMER_HZ;

    #[allow(non_camel_case_types)]
    type c_uint = usize;
    pub type TNetDeviceSpeed = c_uint;


    #[repr(C)]
    #[derive(Debug)]
    pub struct CMACAddress {
        pub m_bValid: bool,
        pub m_Address: [u8; 6usize],
    }


    pub struct USPi(());

    extern "C" {
        pub fn USPiInitialize() -> bool;
        pub fn USPiGetMACAddress() -> *const CMACAddress;
        pub fn USPiIsSendFrameAdvisable() -> bool;
        pub fn USPiSendFrame(
            pBuffer: *const u8,
            nLength: c_uint,
        ) -> bool;
        pub fn USPiReceiveFrame(
            pBuffer: *mut u8,
            pResultLength: *mut c_uint,
        ) -> bool;
        pub fn USPiIsLinkUp() -> bool;
        pub fn USPiGetLinkSpeed() -> TNetDeviceSpeed;
        pub fn USPiUpdatePHY() -> bool;
    }

    impl ! Sync for USPi {}

    impl USPi {
        /// The caller should assure that this function is called only once
        /// during the lifetime of the kernel.
        pub unsafe fn initialize() -> Self {
            assert!(USPiInitialize() != false);
            USPi(())
        }

        /// Returns whether ethernet is available on RPi
        pub fn is_eth_available(&mut self) -> bool {
            unsafe { USPiIsLinkUp() != false }
        } // TODO: Double check!!!!

        /// Returns MAC address of RPi
        pub fn get_mac_address(&mut self, buf: &mut [u8; 6]) {
            unsafe {
                let address = USPiGetMACAddress();
                buf.copy_from_slice(&(*address).m_Address[..])
            }
        }

        /// Checks whether RPi ethernet link is up or not
        pub fn is_eth_link_up(&mut self) -> bool {
            unsafe { USPiIsLinkUp() != false }
        }

        /// Sends an ethernet frame using USPiSendFrame
        pub fn send_frame(&mut self, frame: &Frame) -> Option<i32> {
            trace!("Send frame {:?}", frame);
            let result = unsafe { USPiSendFrame(frame.as_ptr(), frame.len() as c_uint) };
            match result {
                false => None,
                n => Some(0),
            }
        }

        /// Receives an ethernet frame using USPiRecvFrame
        pub fn recv_frame<'a>(&mut self, frame: &mut Frame) -> Option<i32> {
            let mut result_len = 0;
            trace!("Recv frame {:?}", frame);
            let result = unsafe { USPiReceiveFrame(frame.as_mut_ptr(), &mut result_len) };
            frame.set_len(result_len as u32);
            match result {
                false => None,
                _ => Some(0),
            }
        }

/*        /// A wrapper function to `TimerStartKernelHandler`.
        pub fn start_kernel_timer(&mut self, delay: Duration, handler: TKernelTimerHandler) {
            trace!(
                "Core {}, delay {:?}, handler {:?}",
                aarch64::affinity(),
                &delay,
                handler.map(|v| v as usize as *mut u8)
            );

            let divisor = (1000 / USPI_TIMER_HZ) as u128;
            let delay_as_hz = (delay.as_millis() + divisor - 1) / divisor;

            if let Ok(c_delay) = delay_as_hz.try_into() {
                unsafe {
                    TimerStartKernelTimer(
                        TimerGet(),
                        c_delay,
                        handler,
                        ptr::null_mut(),
                        ptr::null_mut(),
                    );
                }
            }
        }*/
    }
}

pub use inner::USPi;

unsafe fn layout(size: usize) -> Layout {
    Layout::from_size_align_unchecked(size + 16, 16)
}

#[no_mangle]
unsafe fn malloc(size: u32) -> *mut c_void {
    let layout = unsafe { layout(size as usize) };
    let pointer = ALLOCATOR.alloc(layout);

    *(pointer as *mut usize) = layout.size();
    // Return the allocated memory but shifted forward by 16. So, when we free the memory
    // at that pointer address, we know that we need to walk back by 16 to get the size
    (pointer as usize + 16) as *mut c_void
}

#[no_mangle]
unsafe fn free(ptr: *mut c_void) {
    let size_pointer = ptr as usize - 16;
    let size = unsafe { *(size_pointer as *mut usize) };
    let layout = Layout::from_size_align_unchecked(size, 16);
    ALLOCATOR.dealloc(size_pointer as *mut u8, layout)
}

#[no_mangle]
pub fn TimerSimpleMsDelay(nMilliSeconds: u32) {
    let time = Duration::from_millis(nMilliSeconds as u64);
    spin_sleep(time)
}

#[no_mangle]
pub fn TimerSimpleusDelay(nMicroSeconds: u32) {
    let time = Duration::from_millis(nMicroSeconds as u64 / 1000);
    spin_sleep(time)
}

#[no_mangle]
pub fn MsDelay(nMilliSeconds: u32) {
    let time = Duration::from_millis(nMilliSeconds as u64);
    spin_sleep(time)
}

#[no_mangle]
pub fn usDelay(nMicroSeconds: u32) {
    let time = Duration::from_millis(nMicroSeconds as u64 / 1000);
    spin_sleep(time)
}

#[no_mangle]
pub fn GetMicrosecondTicks() -> u32 {
    return current_ticks();
}

struct VoidHandle(*mut c_void);

unsafe impl Send for VoidHandle {}

unsafe impl Sync for VoidHandle {}

/// Registers `pHandler` to the kernel's IRQ handler registry.
/// When the next time the kernel receives `nIRQ` signal, `pHandler` handler
/// function should be invoked with `pParam`.
///
/// If `nIRQ == Interrupt::Usb`, register the handler to FIQ interrupt handler
/// registry. Otherwise, register the handler to the global IRQ interrupt handler.
#[no_mangle]
pub unsafe fn ConnectInterrupt(nIRQ: u32, pHandler: TIRQHandler, pParam: *mut c_void) {
    let interrupt = Interrupt::from(nIRQ as usize);
    let param = VoidHandle(pParam);
    let actual_p_handler = pHandler.expect("The handler should exist");

    if interrupt == Interrupt::Usb {
        FIQ.register((), Box::new(move |tf| {
            actual_p_handler(param.0)
        }));
    } else if interrupt == Interrupt::Timer3 {
        GLOBAL_IRQ.register(interrupt, Box::new(move |tf| {
            let actual_p_handler = pHandler.expect("The handler should exist");
            actual_p_handler(param.0)
        }))
    } else { panic!("interrupt irq number should be either USB or Timer3") }
}

/// Writes a log message from USPi using `uspi_trace!` macro.
#[no_mangle]
pub unsafe fn DoLogWrite(_pSource: *const u8, _Severity: u32, pMessage: *const u8) {
    unsafe {
        extern "C" {
            fn strlen(s: *const u8) -> usize;
        }
        let len = strlen(pMessage);
        let pMessage = pMessage as *const u8;
        uspi_trace!("{:?}", slice::from_raw_parts(pMessage, len as usize + 1))
    }
}

#[no_mangle]
pub fn DebugHexdump(_pBuffer: *const c_void, _nBufLen: u32, _pSource: *const u8) {
    unimplemented!("You don't have to implement this")
}

#[no_mangle]
pub unsafe fn uspi_assertion_failed(pExpr: *const u8, pFile: *const u8, nLine: u32) {
    unsafe {
        extern "C" {
            fn strlen(s: *const u8) -> usize;
        }
        let len = strlen(pExpr);
        let pExpr = pExpr as *const u8;
        uspi_trace!("{}, {:?}", nLine,  slice::from_raw_parts(pExpr, len as usize + 1));

        let len2 = strlen(pFile);
        let pFile = pFile as *const u8;
        uspi_trace!("{}, {:?}", nLine,  slice::from_raw_parts(pFile, len2 as usize + 1));
    }
}

pub struct Usb(pub Mutex<Option<USPi>>);

impl Usb {
    pub const fn uninitialized() -> Usb {
        Usb(Mutex::new(None))
    }

    pub fn initialize(&self) {
        let mut inner = self.0.lock();
        if let None = *inner {
            *inner = Some(unsafe { USPi::initialize() });
        }
    }

    pub fn is_eth_available(&self) -> bool {
        self.0
            .lock()
            .as_mut()
            .expect("USB not initialized")
            .is_eth_available()
    }

    pub fn get_eth_addr(&self) -> EthernetAddress {
        let mut buf = [0; 6];
        self.0
            .lock()
            .as_mut()
            .expect("USB not initialized")
            .get_mac_address(&mut buf);
        return EthernetAddress::from_bytes(&buf);
    }

    pub fn is_eth_link_up(&self) -> bool {
        self.0
            .lock()
            .as_mut()
            .expect("USB not initialized")
            .is_eth_link_up()
    }

    pub fn send_frame(&self, frame: &Frame) -> Option<i32> {
        self.0
            .lock()
            .as_mut()
            .expect("USB not initialized")
            .send_frame(frame)
    }

    pub fn recv_frame(&self, frame: &mut Frame) -> Option<i32> {
        self.0
            .lock()
            .as_mut()
            .expect("USB not initialized")
            .recv_frame(frame)
    }

    pub fn start_kernel_timer(&self, delay: Duration, handler: TKernelTimerHandler) {
        // self.0
        //     .lock()
        //     .as_mut()
        //     .expect("USB not initialized")
        //     .start_kernel_timer(delay, handler)
    }
}
