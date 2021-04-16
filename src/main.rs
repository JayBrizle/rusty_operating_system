#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"] // Remap the test runner entry call to something other than the "main" function since this is a #![no_main] environment

use core::panic::PanicInfo;
mod vga_buffer;
mod serial;

pub trait Testable {
    fn run(&self) -> ();
}

/// Avoids adding serial_print(s) to all test_cases
/// test_runner updated to then expect this type
impl<T> Testable for T 
where 
    T: Fn(), 
{
    fn run(&self) { // Assuming #[test_case] must supply an underlying run() for the function marked with it
        serial_print!("{}...\t", core::any::type_name::<T>());
        self(); // We know this is a Fn()
        serial_println!("[ok]");
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) { // Set by #![test_runner(crate::test_runner)] as the test runner for the current environment
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] // Since iosize is set to 4 bytes in Cargo.toml 
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}


pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}



// _start is the function that is looked for and ran/called after boot
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World");

    #[cfg(test)] // Conditional compilation
    test_main(); // Name provided to [reexport_test_harness_main]

    loop {}
}

// A handler is required when the system emits a panic.
// The panic-strategy is set to "abort" in the target x86_64-briz_os.json
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}


// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}