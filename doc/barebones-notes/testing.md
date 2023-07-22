# Testing
- Implement structure for running unit and integration testing
## Testing in Rust
- Rust has a buil-in test framework, just need to add the `#[test]` attribute and run `cargo test` instead of `cargo run`
- However, its more complicated since we don't have `std`, which the `test` crate relies on
### Custom Test Frameworks
- We can replace the test framework with the unstable `custom_test_frameworks` feature
- This means we won't have the advanced features the default test framework has (like `should_panic` tests)
- This is an advantage as tests like `should_panic` use other features we don't have like stack unwinding
- Add the following to `kernel/src/main.rs` to implement a custom test framework:
```rust
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
```
- **NOTE:** This is later removed as we create a library for tests, no longer needing a `test_runner` function
- This just prints a short debug message and then calls each test function in the list
- The argument type `&[&dyn Fn()]` is a slice of trait object references of the Fn() trait
- We can add the `#![rexport_test_harness_main = "test_main"]` to call the renamed function from `_start`:
```rust
#![reexport_test_harness_main = "test_main"]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}
```
- Use conditional compilation (the `#[cfg(test)]` attribute) to add the call to `test_main` only in test contexts
- Test added to `main.rs` not included here, but what we take from it is that we want to exit the loop once tests are done, unlike our normal system
## Exiting QEMU
- Just have an endless loop at the end of `_start`
- Could offer support for either [APM](https://wiki.osdev.org/APM) or [ACPI](https://wiki.osdev.org/ACPI) but this can be pretty complicated
- For now, we can use QEMU's `isa-debug-exit` device
- Just need to pass the `-device` argument to QEMU, in `kernel/Cargo.toml`:
```toml
[package.metadata.bootimage]
test-args = ["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
```
- The `iobase` and `iosize` parameters specify the I/O port that through which the device can be reached from our kernel
### I/O Ports
- Two different approaches for communicating between the CPU and peripheral hardware for x86:
    - Memory-mapped I/O
    - Port-mapped I/O
- Used memory-mapped I/O for accessing the VA text buffer
- In contrast to memory-mapped, port-mapped has a separate I/O bus for communication
- Use the `in` and `out` commands to communicate with an I/O port
- The `isa-debug-exit` device uses port-mapped I/O
    - `iobase` specifies on which port address teh device should live, 0xf4 is generally unused
    - `iosize` specifies the port size, this being 4 bytes
### Using the Exit Device
- For the `isa-debug-exit` device, when a value is written to the I/O port specified by `iobase`, it causes QEMU to exit with exit status `(value << 1) | 1`
- So if we write:
    - 0 -> (0 << 1) | 1 = 1
    - 1 -> (1 << 1) | 1 = 3
- Can just use the abstractions from the `x86_64` create instead of using the assembly `in` and `out`:
```toml
# in Cargo.toml

[dependencies]
x86_64 = "0.14.2"
```
- Can use the `Port` type to make an `exit_qemu` function in `kernel/src/main.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
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
```
- Creates a new Port at our `iobase`: `0xf4`
- Use `u32` since thats the `iosize` (4 bytes)
- Both operations are deemed unsafe since writing to an I/O port can generally result in arbitrary behavior
- Use 2 and 3 as success and failed exit codes since 33 ((0x10 << 1) | 1) and 35 ((0x11 << 1) | 1)would be easier to distinguish between instead of 1, which is QEMU's default exit code
- Now need to update the test_runner:
```rust
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    /// new
    exit_qemu(QemuExitCode::Success);
}
```
- Need to now fix it so that QEMU can consider our Success code as a success since it defaults to all other exit codes as failures
### Success Exit Code
- Can use `bootimage` `test-success-exit-code` config key:
```toml
# in Cargo.toml

[package.metadata.bootimage]
test-args = […]
test-success-exit-code = 33         # (0x10 << 1) | 1
```
- This will map our success exit code to 0, which QEMU will consider as a success now
- Now we need to print the results to the console since the QEMU window only opens for a split second
## Printing to the Console
- Need to send data over serial port from our kernel to our host computer
### Serial Port
- Chips implementing a serial interface are called UARTs
- Common UARTs are all compatible with the 16550 UART, which we can use for our testing framework
- Use the `uart_16550` crate, adding it to `kernel/Cargo.toml`:
```toml
[dependencies]
uart_16550 = "0.2.0"
```
- This crate contains a `SerialPort` struct that represents the UART registers, but we still need to construct an instance of it ourselves
- We need to make a `serial` module
```rust
// in src/main.rs

mod serial;
```
- New file `kernel/src/serial.rs`
```rust
use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}
```
- Use `lazy_static` to create a static writer instance, like with the VGA text buffer
- Using `lazy_static` ensures that the `init` method is called exactly once on its first use
- Need to program the UART with multiple I/O ports for programming different device registers
- We can just pass the first I/O port `0x3F8`, which is the standard port number for the first serial interface
- It will calculate the other ports that we need
- We can add `serial_print!` and `serial_println!` macros in `kernel/src/serial.rs`:
```rust
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
```
- Now we can write to the console instead of the VGA text buffer in `kernel/src/main.rs`:
```rust
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    […]
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}
```
### QEMU Arguments
- Need to redirect the serial output to `stdout` using the `-serial` argument
```toml
# in Cargo.toml

[package.metadata.bootimage]
test-args = [
    "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-serial", "stdio"
]
```
- For cases in which a test fails, we need to be able to print an error message on panic too as this will still write to the VGA buffer
### Print an Error Message on Panic
- Use a different panic handler when testing with conditional compilation:
```rust
// our existing panic handler
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
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
```
- Still use a loop since the compiler doesn't know we use `isa-debug-exit` to cause a program exit
- Now that we print to the console, we don't even need the QEMU window to pop up
### Hiding QEMU
- Can use the `-display none` argument to QEMU to hide it:
```toml
# in Cargo.toml

[package.metadata.bootimage]
test-args = [
    "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-serial", "stdio",
    "-display", "none"
]
```
- This also lets our test framework to run in environments without a GUI
### Timeouts
- Have a problem since `cargo test` waits until test runner is done, which won't happen if a function never returns
- Certain cases that cause infinte loops:
    - Bootloader fails to load our kernel, meaning it will reboot endlessly
    - BIOS/UEFI fails to boot the bootloader
    - CPU enters a `loop {}` statement, which is at the end of some of our functions
    - Hardware causes a system reset, like when a CPU exception is not caught (something we need to look into)
- Can add a timeout feature provided by `bootimage`, which can be configured with:
```toml
[package.metadata.bootimage]
test-timeout = 300          # (in seconds)
```
### Insert Print Automatically
- Instead of needing to write prints for every test function, we can just update `test_runner` to print
- Do that by adding a new `Testable` trait:
```rust
pub trait Testable {
    fn run(&self) -> ();
}
```
- Now implement this trait for all types `T` that implement the `Fn()` trait:
```rust
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
```
- This implements the `run` function by first printing the name of the function, then call it with the `self();`, and then print `[ok]` to show it did not panic
- The last thing to do is update `test_runner` to use this new `Testable` trait:
```rust
#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) { // new
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run(); // new
    }
    exit_qemu(QemuExitCode::Success);
}
```
- The changes are changing the type of `&[&dyn Fn()]` to `&[&dyn Testable]` and that we now call `test.run()` instead of just `test()`
- Now adding print statements to show the test status is no longer necessary
## Testing the VGA Buffer
- Now we can create test for the VGA Buffer
- Test to print to the VGA Buffer, making sure it doesn't panic:
```rust
// in src/vga_buffer.rs

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}
```
- Test to check that multiple lines printed doesn't cause a panic:
```rust
// in src/vga_buffer.rs

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}
```
- Test to verify that printed lines actually appear on the screen:
```rust
// in src/vga_buffer.rs

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
```
- What this does is it prints a test string, with a newline, so then the test checks the buffer height + 2
- We use c to compare with whats actually the screenchar in the buffer
- Now we will focus on integration tests
## Integration Tests
- Convention for integration tests is to put them in a separate `tests/` directory
- Our test framework will automatically execute all of the tests in this directory
- Make our first test called `basic_boot` (not gonna include it as its quite long)
- This can be found in `kernel/tests/basic_boot.rs`
- This being a separate executable means we need to provide all of the same crate attributes as in `kernel/src/main.rs`
- In order to get the functions needed four our integration tests, we need a library from our `main.rs`
### Create a library
- We create `kernel/src/lib.rs`, moving all of the test functions and attributes from `main.rs` into `lib.rs`
- Did not add the new code for `lib.rs`, but now the `test_runner` function does not have the conditional compilation and is made public
- We also need a `_start` function since its tested independently from `main.rs`
- Using the `cfg_attr` create attribute conditionally enables the `no_main` attribute
- Moving the `serial` and `vga_buffer` modules to `lib.rs` makes it public outside of the library:
```rust
// in src/lib.rs

pub mod serial;
pub mod vga_buffer;
```
- Now we update `main.rs` to use the library:
```rust
#![test_runner(blog_os::test_runner)]
```
- Library is used like a normal create, called `blue_crab_os`
### Completing the Integration Test
- We now add the new library for our `basic_boot` test as well
- This lets use the library's `test_panic_handler()`:
```rust
#![test_runner(blog_os::test_runner)]

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}
```
- Implementing the `println` test inside `basic_boot` without calling any initialization routines in `_start` means we can ensure that it works right after booting
- This is important for things like printing panic messages
### Future Tests
- Integration tests are important since they are treated as separate executables, giving complete control over the environment
- Future things we will need to implement and furthermore test:
    - CPU Exceptions
    - Page Tables
    - Userspace Programs
### Tests that Should Panic
- Test framwork of the standard library supports a `#[should_panic]` attribute which allows tests that should fail
- We can make our own `should_panic` in `kernel/tests/should_panic.rs`
- This new test doesn't reuse the library's `test_runner`, but uses its own that will exit witha failure exit code when a test returns without panicking
- Otherwise the runner exits with a success error code
- Below is the function that will actually fail:
```rust
use blog_os::serial_print;

#[test_case]
fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}
```
- It purposefully fails by asserting that 0 equals 1
### No Harness Tests
- Don't need a test runner if we only have one test, like the `should_panic` test
- This means we can just go directly to the `_start` function, meaning we need to disable the `harness` flag for the test in `Cargo.toml`
- This defines if a test runner is used for an integration test:
```rust
[[test]]
name = "should_panic"
harness = false
```
- This lets us remove the `test_runner` related code in `should_panic`, calling `should_fail()` within `_start`
- Disabling the `harness` flag can be usefull for complicated tests where individual test functions hav side effects and need to be run in a specified order