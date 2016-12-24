//! Provides utility methods for interacting with the VGA text buffer, which is located in physical memory at
//! 0xB8000. Much of this code was inspired by Phillip Oppermann's lovely blog, the link for which can be found
//! in the README.

use core::fmt;
use volatile::Volatile;
use core::ptr::Unique;
use spin::Mutex;

/// The default VGA text buffer width, in characters.
const BUFFER_WIDTH: usize = 80;

/// The default VGA text buffer height, in characters.
const BUFFER_HEIGHT: usize = 25;

/// The number of spaces that 1 tab is equivalent to.
const TAB_SIZE: usize = 4;

// This is very temporary. I promise. Temporary.

/// The static writer instance used for writing to the VGA text buffer.
pub static VGA_WRITER: Mutex<VGAWriter> = Mutex::new(VGAWriter {
    row: 0,
    column: 0,
    color: ColorCode::new(Color::Green, Color::Black),
    buffer: unsafe { Unique::new(0xB8000 as *mut _) }
});

/// Represents the possible VGA text colors.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
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
    White = 15
}

/// A VGA color code, which contains both a foreground and background color,
/// and a "bright" bit and "blink" bit.
#[derive(Debug, Clone, Copy)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Creates a new color code from a foreground and background color.
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}


/// Represents a VGA screen character, which is a combination of an ASCII character and
/// associated color code.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    /// The character being displayed.
    character: u8,

    /// The color the character is being displayed as.
    color: ColorCode
}

/// An in-memory representation of the VGA text buffer.
struct TextBuffer {
    /// The actual array of screen characters.
    characters: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

/// Provides a sink to write ASCII characters to. Implements the standard
/// library's Write trait.
pub struct VGAWriter {

    /// The current row we're on.
    row: usize,

    /// The current column we're on.
    column: usize,

    /// The color we're printing as.
    color: ColorCode,

    /// The underlying raw VGA buffer we're writing to.
    buffer: Unique<TextBuffer>
}

// Provides standard manipulation.
impl VGAWriter {
    /// Writes an ASCII character to the underlying text buffer at the given
    /// row, column, and with the given color.
    pub fn write_char(&mut self, character: u8) {
        // We'll assume before hand that the positions are always valid, as
        // we control the row and column.
        match character {
            b'\r' => self.column = 0,
            b'\n' => {
                self.row += 1;
                self.column = 0;

                if self.row >= BUFFER_HEIGHT {
                    self.shift_buffer_up();
                }
            },
            b'\t' => {
                for _ in 0 .. TAB_SIZE {
                    self.write_char(b' ');
                }
            },
            _ => {
                // TODO: Rust non-lexical borrowing pls
                let row = self.row;
                let column = self.column;
                let color = self.color;

                self.buffer().characters[row][column].write(ScreenChar {
                    character: character, color: color
                });

                self.column += 1;

                // If we've gone off the edge, move down one line.
                // A great way to do it, I'm sure you'll agree.
                if self.column >= BUFFER_WIDTH {
                    self.write_char(b'\n');
                }
            }
        }
    }

    /// Moves everything in the buffer, including the cursor, up one line.
    /// If the cursor is already at the top of the buffer, it is not moved.
    pub fn shift_buffer_up(&mut self) {
        // Iterate row-wise then column wise to copy everything up.
        for row in 0 .. BUFFER_HEIGHT - 1 {
            for col in 0 .. BUFFER_WIDTH {
                let old_char = self.buffer().characters[row + 1][col].read();
                self.buffer().characters[row][col].write(old_char);
            }
        }

        // Then clear the bottom row.
        for col in 0 .. BUFFER_WIDTH {
            let color = self.color;

            self.buffer().characters[BUFFER_HEIGHT - 1][col].write(ScreenChar { 
                character: b' ', color: color
            })
        }

        // Move the row up only if we're not already at the top.
        if self.row > 0 {
            self.row -= 1;
        }
    }

    /// Obtain the default color used by this text writer.
    pub fn color(&self) -> ColorCode {
        self.color
    }

    /// Update the default color used by the text writer to the given color.
    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color
    }

    /// Obtain a mutable reference to the underlying buffer.
    fn buffer(&mut self) -> &mut TextBuffer {
        // UNSAFE: Safe, as this text writer uniquely owns this buffer
        // and this method is private.
        unsafe { self.buffer.get_mut() }
    }
}

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        // We make the lovely assumption that everything is ASCII! How bold of us.
        for ascii_char in string.bytes() {
            self.write_char(ascii_char)
        }

        Ok(())
    }
}

// Macro definitions, mostly stolen from the standard libary. Much appreciated, stdlib.

/// Prints a line to the VGA buffer, appending a newline at the end. Uses the default output color.
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Prints a line to the VGA buffer, appending a newline at the end. Uses the provided foreground color.
macro_rules! color_println {
    ($color:expr, $fmt:expr) => (color_print!($color, concat!($fmt, "\n")));
    ($color:expr, $fmt:expr, $($arg:tt)*) => (color_print!($color, concat!($fmt, "\n"), $($arg)*));
}

/// Prints characters to the VGA buffer. Uses the default output color.
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = $crate::vga::VGA_WRITER.lock();
        writer.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

/// Prints characters to the VGA buffer. Uses the provided foreground color.
macro_rules! color_print {
    ($color:expr, $($arg:tt)*) => ({
        use core::fmt::Write;
        use $crate::vga;
        let mut writer = vga::VGA_WRITER.lock();

        let old_color = writer.color();

        writer.set_color(vga::ColorCode::new($color, vga::Color::Black));
        writer.write_fmt(format_args!($($arg)*)).unwrap();
        writer.set_color(old_color);
    });
}