use core::panic::PanicInfo;
use crate::console::{kprint, kprintln, CONSOLE};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("                            ");
    kprintln!(" Raspberry Pi has panicked  ");
    kprintln!("                            ");
    kprintln!("        _____________       ");
    kprintln!("      /              \\     ");
    kprintln!("     |                |     ");
    kprintln!("    |                  |    ");
    kprintln!("    |  X            X  |    ");
    kprintln!("     \\       /\\        /    ");
    kprintln!("       |   ++++++   |       ");
    kprintln!("       |   ++++++   |       ");
    kprintln!("        \\          /        ");
    kprintln!("          --------          ");
    kprintln!("                            ");

    if let Some(location) = _info.location() {
        kprintln!("FILE: {}", location.file());
        kprintln!("LINE: {}", location.line());
    }
    if let Some(s) = _info.payload().downcast_ref::<&str>() {
        kprintln!("MESSAGE: {}", s);
    }


    loop {}
}
