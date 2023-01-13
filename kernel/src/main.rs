#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
// #![feature(asm)]

mod vga_buffer;
use core::panic::PanicInfo;

// static HELLO: &[u8] = b"Hello World!\nTest new line";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("     ________________
    < Hello bitches! >
     ----------------
            \\
              \\ 
                _~^~^~_
            \\) /  o o  \\ (/
              '_   -   _'
              / '-----' \\");
    panic!("Some panic message");
    loop {}
}

// #[no_mangle] // don't mangle the name of this function
// pub extern "C" fn _start() -> ! {
//     // this function is the entry point, since the linker looks for a function
//     // named `_start` by default
//     loop {}
// }

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
