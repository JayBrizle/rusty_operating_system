use core::fmt;
use volatile::Volatile; // prevents compiler optimizations as needed

#[allow(dead_code)] // Suppress compiler warning for unused variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Enable "copy semantics" and make printable and comparable
#[repr(u8)] // Stores each attribute as u8
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Ensures this type has the exact same layout as the u8 type
struct ColorCode(u8); // Will contain the color byte

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // To guarentee field order like a C struct. Rust has no order guarentee.
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)] // Ensures same memory layout as its single field
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize, // keeps track of the current position in the last row
    color_code: ColorCode,  // Specifies the current foreground and background colors
    buffer: &'static mut Buffer, // Set reference lifetime to lifetime of the program, since the text buffer is available at all times
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                // The chars property ScreenChar is wrapped in Volatile so we use the underlying write method to prevent compiler optimizating it away.
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or a newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}


use lazy_static::lazy_static;
use spin::Mutex;
// Raw pointer items and non const calls like the ColorCode::new can't happen in statics
// Compiler can't resolve them because they are runtime dependent. 
// lazy_static wraps and initializes static after first access at runtime.
lazy_static! {
    // Mutex to provide interior mutability 
    // Hacky work around due to language features and the nature of what is being built here with a kernel (No Rust std, threads, etc., available)
    pub static ref WRITER: Mutex<Writer> = Mutex::new(
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}, // need unsafe to allow the reference to the memory mapped VGA buffer address on the hardware
        }
    );
}


#[macro_export] // Places in root namespace of current crate and is then available everywhere
macro_rules! print {
    ($($args:tt)*) => ($crate::vga_buffer::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($args:tt)*) => ($crate::print!("{}\n", format_args!($($args)*)))
}

#[doc(hidden)] // Hide from generated documentation
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println_simple() {
    println!("println! works.");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}