# VGA Text Mode
- Simple way to print text to the screen
## The VGA Text Buffer
- To print a character to the screen in VGA text mode, one has to write it to the text buffer of the VGA hardware
- Two-dimensional array with typically 25 rows and 80 columns
- Each array entry describes a single screen character with the following format
<table>
  <tr>
    <th>Bit(s)</th>
    <th>Value</th>
  </tr>
  <tr>
    <td>0-7</td>
    <td>ASCII code point</td>
  </tr>
  <tr>
    <td>8-11</td>
    <td>Foreground color</td>
  </tr>
  <tr>
    <td>12-14</td>
    <td>Background color</td>
  </tr>
  <tr>
    <td>15</td>
    <td>Blink</td>
  </tr>
</table>

- First byte represents the character that should be printed in ASCII encoding
- Not exactly ASCII, but a character set named "code page 437" with more characters and modifications
- The second byte defines how the character is displayed
- First four bits define foreground color, next three bits is the background color, and the last bit determines whether the character should blink
- The fourth bit for the foreground color is called the "bright" bit, turning blue for example into light blue
- VGA text buffer is accessible throught `memory-mapped I/O` at address `0xb8000`
- This means any access does not access RAM directly, but accesses the text buffer on the VGA hardware directly
- **NOTE**: memory-mapped hardware might not support all normal RAM ops
## A Rust Module
- We add the module `vga_buffer` in `kernel/src/main.rs`
```rust
mod vga_buffer;
```
- We then create a file for the buffer called `kernel/src/vga_buffer.rs`
### Colors
- Add the enum to the file:
```rust
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    White = 15,
}
```
- Have to use 8 bit uint even though 4 bits is enough since thats not a supported type
- The `#[allow(dead_code)]` attribute disables warnings of unused variants
- Deriving those traits in the `derive` attribute enables [copy semantics](https://doc.rust-lang.org/1.30.0/book/first-edition/ownership.html#copy-types) for the type and make it printable and comparable
- We then implement a new type to represent a ful color code:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}
```
- The `ColorCode` struct contains the full color byte, containing foreground and background color
- Use the `repr(transparent)` attribute to ensure the struct has the same data layout as `u8`
### Text Buffer
- Now we need structures for representing a screen character and the buffer
- Inside of `kernel/src/vga_buffer`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
```
- The `repr(C)` attribute guarantees that the struct's fields are laid out exactly like in C, guaranteeing correct field ordering
- Using the `repr(transparent)` attribute for the Buffer struct ensures it has the same memory layout as its single field
- For actually writing to the screen, we have a writer type:
```rust
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}
```
- Will always write to the last line and shifts the line up when a line is full or the new line character `\n`
- `column_position` keeps track of the current position in the last row
- Current foreground and background colors are specified by `color_code`
- The `'static` for the buffer describes an "explicit lifetime", which lets the reference be valid for the whole program
### Printing
- We then add a method to write a single ASCII byte:
```rust
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
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {/* TODO */}
}
```
- This prints out all characters except for the new line character, which calls `new_line` written later on.
- When printing, it checks if the line is full
    - If so it also calls the soon to be implemented function `new_line`
    - It then writes a new `ScreenChar` to the buffer
    - Finally, the current column position is advanced
- We implement the method to print whole strings by converting them to bytes and printing them one by one:
```rust
// Put inside impl Writer
pub fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
        match byte {
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' => self.write_byte(byte),
            // not part of printable ASCII range
            _ => self.write_byte(0xfe),
        }

    }
}
```
- Since the VGA text buffer only supports ASCII but Rust strings are UTF-8 by default, a `match` is used to differentiate printable ASCII bytes and unprintable bytes
- We print ■ for unprintable bytes
- **NOTE:** I will be skipping the "Try it out" part since the function is only temporary.
### Volatile
- We need to specify these prints as volatile to let the compiler know that our prints have side effects and should not be optimized away.
- This is because the compiler doesn't know we access VGA buffer memory and doesn't know the side effect of characters appearing on screen. 
- We use the volatile library to wrap our read and write methods.
- This internally uses the `read_volatile` and `write_volatile` functions of the core library
- Add the dependency in `kernel/Cargo.toml`:
```toml
[dependencies]
volatile = "0.2.6"
```
- **NOTE:** We need to use the version "0.2.6" to work with our OS
- We then update the Buffer type with:
```rust
use volatile::Volatile;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
```
- This also means we use a `Volatile<ScreenChar>` instead of just `ScreenChar` so that we can't accidentally write to it "normally.
- Then we have to use the `write` method instead of the `=` operator, changing the `write_byte` method:
```rust
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
            self.buffer.chars[row][col] = ScreenChar {
                ascii_character: byte,
                color_code,
            };
            self.column_position += 1;
        }
    }
}
```
### Formatting Macros
- Nice to support Rust's formatting macros so that we can easily print different types like integers and floats
- We need to implement the `core::fmt::Write` trait, only needing to implement the `write_str` method to our own.
- In `kernel/vga_buffer.rs`:
```rust
use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
```
- The `Ok(())` is just a `Ok` Result containing the `()` type.
### Newlines
- Now we need to actually implement the `new_line` function:
```rust
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

fn clear_row(&mut self, row: usize) {/* TODO */}
```
- Iterates through all of the screen characters and moves each one up a row
- The upper bounds of the range notation `( .. )` is exclusive
- We also omit the 0th row because its the row that is shifted off screen.
- Also need to implement the `clear_row` function, which clears a row by overwiting all of its characters with a space character
```rust
fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
        ascii_character: b' ',
        color_code: self.color_code,
    };
    for col in 0..BUFFER_WIDTH {
        self.buffer.chars[row][col].write(blank);
    }
}
```
## Global Interface
- Need to make a static `WRITER` to be able to use a global interface in any other modules:
```rust
pub static WRITER: Writer = Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
};
```
- This causes an error because Rust's const evaluator is not able to convert raw pointers to references at compile time
- This has to do with the fact that statics are not like variables, which are initialized at runtime, but are initialized at compile time
### Lazy Statics
- The `lazy_static` crate provides a `lazy_static!` macro that defines a lazily initialized static.
- The static lazily initializes itself when accessed for the first time instead of at compile time
- Add the crate to `kernel/Cargo.toml`
```toml
[dependencies.lazy_static]
version = "1.0"
features = ["spin_no_std"]
```
- Use the `spin_no_std` feature to not link the standard library
- Now we can write WRITER without any problems
```rust
use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER: Writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };
}
```
- Can't write anything to this tho since all the write methods takes `&mut self`
- This leads us to spinlocks
### Spinlocks
- Don't have access to the standard library feature: `Mutex`
- We can use the [`spin`](https://crates.io/crates/spin) crate instead:
```toml
[dependencies]
spin = "0.5.2"
```
- We then can use the spinning mutex tp add safe interior mutability to our static WRITER:
```rust
use spin::Mutex;
...
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
```
- We now need to import the `fmt::Write` trait in order to be able to use it
### Safety
- Note that we only have a single unsafe block in our code, which is needed to create a Buffer reference pointing to 0xb8000. 
- Afterwards, all operations are safe. 
- Rust uses bounds checking for array accesses by default, so we can’t accidentally write outside the buffer. 
- Thus, we encoded the required conditions in the type system and are able to provide a safe interface to the outside.
### A `println` Macro
- Can add a `println` macro that can be used from anywhere in the codebase
- Below is the source of the `println!` macro in the standard library:
```rust
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}
```
- Macros are defined through one or more rules
- `println!` has two rules:
    1. Invocations without arguments will print just a newline
    2. For invocations with arguments, a newline character is added to the end of the string
- The `#[macro_export]` attributes makes the macro available to the whole crate and external crates
- It also places the macro at the crate root, meaning we need to import the macro through `std::println` instead of `std::macros::println`
- `print!` is defined as:
```rust
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}
```
- It expands to a call of the `_print` function in the `io` module
- `$crate` variable ensures that the macro also works from outside the `std` crate by expanding to `std`
- We just need to copy the `println!` and `print!` macros and modify them to use our own `_print` function:
```rust
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
```
- The change we added is that the invocations of the `print!` macro are prefixed with `$crate` too
- Make the macros global with the attribute `#[macro_export]`
- `_print` function locks our static WRITER and calls the `write_fmt` method on it
- Need to import this from the `Write` trait
- Need to make the `_print` function public, but add `#[doc(hidden)]` attribute to hide it from generated documentation
### Hello World using `println`
- Make our `_start` function in `kernel/src/main.rs` to use the new macro:
```rust
#[no_mangle]
pub extern "C" fn _start() {
    println!("Hello World{}", "!");

    loop {}
}
```
### Printing Panic Messages
- We can also use this to print the panic info:
```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
```
