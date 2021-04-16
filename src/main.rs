#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

static HELLO: &[u8] = b"Hello Briz OS!";

// _start is the function that is looked for an ran after boot
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8; // memory mapped I/0 address for the VGA text buffer.
    let color = 0xb;
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = color;
        }
    }

    vga_buffer::print_something();

    loop {}
}

// A handler is required when the system emits a panic. 
// The panic-strategy is set to "abort" in the target x86_64-briz_os.json
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
