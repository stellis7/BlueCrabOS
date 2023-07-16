# Posts
- [Freestanding Rust Binary](#freestanding-rust-binary)
- [Minimal Rust Kernel](#minimal-rust-kernel)
- [VGA Text Mode](#vga-text-mode)
- [Testing](#testing)
# Freestanding Rust Binary
## Introduction
- Need to first create rust executable that does not link the standard library so that it can be run bare metal
- This also means can't rely on most of Rust's features since it requires an underlying OS
- Features we do get:
    - [Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
    - [Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
    - [Pattern matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
    - [Option](https://doc.rust-lang.org/core/option/) and [Result](https://doc.rust-lang.org/core/result/)
    - [String formatting](https://doc.rust-lang.org/core/macro.write.html)
    - **[Ownership system](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)**
## Disabling the Standard Library
- All Rust creates link the standard library, which we can't have.
- Create new project with `cargo new blue_crab_os --bin --edition 2021`
    - Creates a new project called "blue_crab_os"
    - `--bin` flag specifies that we want to create an executable
    - We are using the 2021 edition even though
    - All of the project's information can be found in `kernel/Cargo.toml`
- In order to not implicitly link the standard library, you must add the `#![no_std]` attribute at the top of the `main.rs` file
- This means we now need to implement a panic handler function and a language item for the compiler since we can't use the one provided by the standard library
## Panic Implementation
- The panic handler is invoked when a panic occurs
- Define the panic handler function with the following code in main
```rust
use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```
- This function takes in a `PanicInfo` parameter, which has the information of where the panic happened
- This is a diverging function which means it returns the "never" type `!`, meaning it never returns
## The `eh_personality` Language Item
- Language items are special functions and types required internally by the compiler
- Providing custom implementations of language items is doable, but should be last resort, since they are highly unstable implementation details and aren't type checked.
- The `eh_personality` language item marks a function that is used for implement stack unwinding, which is a complicated process that requires OS-specific libraries
    - `libunwind` for Linux or `structure exception handling` for Windows
    - It is used by Rust by default to run the destructors of all live stack variables in case of a panic
    - This makes sure that all memory is freed and allows the parent thread to catch the panic
### Disabling Unwinding
- By adding the following code to `Cargo.toml`, we can easily disable unwinding:
```rust
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```
- **NOTE**: This is later commented out in another post
- It sets panic strat to `abort` for both the dev and release profile
- Now the `start` language item is needed
## The start attribute
- The `main` function is indeed not the first function called when you run a program, where most languages have a runtime system, which needs to be called before `main`
- This is responsible for things like garbage collection and software threads
- Typically a Rust binary, with the standard library linked, starts execution in a C runtime libary called `crt0`.
- We don't have access to the Rust runtime and `crt0` so we need to make our own entry point
- We need to overwrite the entry point since simply implementing the `start` language item still requires `crt0`
### Overwriting the Entry Point
- We can add the `#![no_main]` attribute to `src/main.rs` to tell the rust compiler not to use the normal entry point
- This also means there is no point in having the `main` function, since there is no standard entry point that will call it, so it is removed
- We instead have our own `_start` function for our own entry point, as follows below:
```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
```
- This also disables name mangling, which means the compiler will aactually output the function name `_start` instead of some randomly generated symbol
- The `extern "C"` tells the compiler to use the C calling convention for this function.
- Naming it `_start` is good since it is usually the default entry point for most systems
- This is a diverging function since this function is not called by any other function
- Might end later with resulting in the system shutting down by the end of the function
- This leads to linker errors by running `cargo build`, leading to the next section
## Linker Errors
- GENERAL: The linker is a program that combines the generated code into an executable
- Each system has its own linker, meaning that each one throws a different error
- The cause is genreally the same, the default config of the linker assumes that our program depends on the C runtime, which it does not
### Building for a Bare Metal Target
- Rust by default tries to build an executable for your current system environment
- This environment is called your "host" system
- A string called a **target triple** is used to described different environments.
- `rustc --version --verbose` can be invoked to see your host system's target triple
- If we were to build to our host triple, it would lead to linker errors since it assumes there is an underlying OS using the C runtime by default
- In order to avoid this we can compile for a different environment with no underlying OS
- We compile for the `thumbv7em-none-eabihf` target, which is an embedded ARM system by doing `rustup target add thumbv7em-none-eabihf`
- This downloads a copy of the standard and core library for the system, which we use to build our freestanding exe for this target `cargo build --target thumbv7em-none-eabihf`
- Passing the `--target` flag lets us cross compile our executable for a bare metal target system
- WIth the target system having no OS, the linker does not try to link the C runtime, meaning no link errors on build
- We use a custom target, which is described in the [Minimal Rust Kernel](#minimal-rust-kernel)
- This will be built for an x86_64 bare metal environment
### Linker Arguments
- This section was optional since it was not used in the main tutorial
- It showed how to resolve the linker errors instead of building toward a bare metal system
# Minimal Rust Kernel
Create a minimal 64-bit Rust kernel for x86 architecture
## The Boot Process
- Within turning on your computer, it begins firmware code that is on the mobo ROM
- This perfroms [power-on self-test](https://en.wikipedia.org/wiki/Power-on_self-test), detects available RAM, and pre-initializes CPU and hardware
- It then looks for a bootable disk and starts booting the OS kernel
- Two different firmware standards for x86
# VGA Text Mode
# Testing