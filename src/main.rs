#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

// _start is the function that is looked for and ran/called after boot
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World");
    loop {}
}

// A handler is required when the system emits a panic.
// The panic-strategy is set to "abort" in the target x86_64-briz_os.json
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
