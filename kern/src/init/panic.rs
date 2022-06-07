use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("        _____________       ");
    kprintln!("      /              \\     ");
    kprintln!("     |                |     ");
    kprintln!("    |                  |    ");
    kprintln!("    |  X            X  |    ");
    kprintln!("     \\      /\\       /    ");
    kprintln!("       |   ++++++   |       ");
    kprintln!("       |   ++++++   |       ");
    kprintln!("        \\         /        ");
    kprintln!("          --------          ");
    kprintln!("                            ");
    kprintln!(" Raspberry Pi has panicked  ");

    if let Some(location) = panic_info.location() {
        kprintln!("FILE: {}", location.file);
        kprintln!("LINE: {}", location.line());
    }
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        kprintln!("MESSAGE: {s:?}");
    }


    loop {}
}
