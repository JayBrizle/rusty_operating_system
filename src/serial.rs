use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let standard_serial_port = 0x3F8;
        let mut serial_port = unsafe { SerialPort::new(standard_serial_port) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Print to serial port failed.");
}

/// Prints to the host through the serial interface.
/// SerialPort already implements the fmt::Write interface.
#[macro_export]
macro_rules! serial_print {
    ($($args:tt)*) => {
        $crate::serial::_print(format_args!($($args)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($args:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($args)*));
}