# Posts
1. [Freestanding Rust Binary](#freestanding-rust-binary)
2. [Minimal Rust Kernel](#minimal-rust-kernel)
3. [VGA Text Mode](#vga-text-mode)
4. [Testing](#testing)
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
```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```
- **NOTE**: This is later commented out in another post since it is added in our target json file
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
- Two different firmware standards for x86 (UEFI and BIOS)
- Only support for BIOS
### BIOS Boot
- Almost all x86 systems have support for BIOS booting
- This wide compatibility comes with disadvantages since its put in 16-bit compability, called real mode so that older bootloaders can work
- When you turn on a computer
    1. It loads the BIOS from special flash memory on the mobo
    2. The BIOS runs self-test and initialization routines, which is a 512 byte portion of executable code stored at the disk's beginning
    3. Most bootloaders are much larger, so they are split into a small first stage and a second stage.
    4. The bootloader has to determine the location of the kernel image and load it into memory
    5. Also needs to switch from real mode to 32-bit protected mode
    6. Then switches to 64-bit long mode, where 64-bi registers and the complete main memory are available
    7. Its next job is to query info from the BIOS and pass it to the OS kernel
- Since writing a bootloader is quite cumbersome, they have provided a tool called bootimage that automatically prepends a bootloader to our kernel
#### The Multiboot Standard
- The Free Software Foundation created an open bootloader standard called Multiboot to handle the issue of every OS needing to implement its own bootloader
- Standard defines an interface between the bootloader and the OS
- Reference implementation is [GNU GRUB](https://en.wikipedia.org/wiki/GNU_GRUB)
- Just need to add the Multiboot header to be multiboot compliant
- Some problems:
    - Only support protected mode, meaning that you still have to do the CPU config to switch to long mode
    - Designed to make bootloader simple instead of the kernel
    - Both GRUB and multiboot standard are only sparsely documented
    - GRUB needs to be installed on the host system to create a bootable disk image from the kernel file
- This means the bootimage tool does not support the multiboot standard, but supports multiboot 2
## A Minimal Kernel
By using `cargo` to build the binary, it builds for the host system which is something we don't want since the kernel would then run on top of the host OS. We instead want to compile to a clearly defined target system.
### Installing Rust Nightly
- Rust has three release channels:
    - Stable
    - Beta
    - Nightly
- Using `rustup` allows us to install nightly, beta, and stable compilers side-by-side and makes it easy to update them
- In order to use a nightly compiler, we have a file in `kernel/rust-toolchain` with just the content `nightly`
- `rustc --version` shows version as `rustc 1.69.0-nightly (d7948c843 2023-01-26)`
- We can use feature flags at the top of our file with the nightly compiler, like the experimental `asm!` macro for inline assembly
- Since they are unstable, they are only used if absolutely necessary
### Target Specification
- The target triple describes the architecture, vendor, the OS, and the ABI (Applications Binary Interface)
- We need our own special config params so we can create a json file in order to define our own target, which is at `kernel/x86_64-blue_crab_os.json`
- Most fields are required by LLVM to generate code for the platform
- Other fields are used for conditonal compilation by rust
- The third kind of fields are those that define how the crate should be built
- Below is the full file:
```json
{
    "llvm-target": "x86_64-unknown-none",
    "data-layout": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
    "arch": "x86_64",
    "target-endian": "little",
    "target-pointer-width": "64",
    "target-c-int-width": "32",
    "os": "none",
    "executables": true,
    "linker-flavor": "ld.lld",
    "linker": "rust-lld",
    "panic-strategy": "abort",
    "disable-redzone": true,
    "features": "-mmx,-sse,+soft-float"
}
```
- The OS field is set to none in `llvm-target` since it will run on bare metal
- We use th cross-platform LLD linker with the `linker-flavor` and `linker` field
- The `panic-strategy` specifics that the target does not support stack unwinding on panic so it should abort directly
    - This allows us to remove the `panic = "abort"` in Cargo.toml
    - This also applies to compiling the `core` library a little later on
- In order to handle interrupts safely, we need to disable the "red zone" stack pointer optimization since it would cause stack corruption with `"disable-redzone": true,`
- The `features` field enables and disables target features
    - The `mmx` and `sse` features are disabled by prefixing them with a minus and enable the `soft-float` with a plus
    - `mmx` and `sse` determine support for SIMD (Single Instruction Multiple Data), which is good for speeding up programs but using large SIMD registers in our OS kernel leads to performance problems
    - This is because registers are restored to their original state before continuing an interrupted program
    - We need the `soft-float` feature to solve the problem of the floating point operations requiring SIMD registers by default
    - This feature instead emulates all fp ops through software functions based on normal ints
    - More info [here](https://os.phil-opp.com/disable-simd/)
### Building our Kernel
- Need to define our entry point as `_start` since its LLVM uses Linux conventions
- When building with `cargo build --target x86_64-blog_os.json` leads to a core library issue since the core library is only valid for supported host triples since it is a precompiled library
#### The `build-std` Option
- The `build-standard` feature lets us recompile `core` and other standard library crates on demand
- This feature is unstable and only available for the nightly release
- We need to make the `kernel/.cargo/config.toml` file and add:
```toml
[unstable]
build-std = ["core", "compiler_builtins"]
```
- This tells cargo to recompile `core` and `compiler_builtins`
- **NOTE**: The unstable.build-std configuration key requires at least the Rust nightly from 2020-07-15.
#### Memory-Related Intrinsics
- Compiler assumes that a certain set of functions are available for all systems, which most are povided by `compiler_builtins`
- There are some memory-related functions that are not enabled by default
- These include: 
    - `memset`
    - `memcpy`
    - `memcmp`
- We need to enable the `compiler_builtins` implementation of these functions since they are disabled by default to conflict with the C library, which we can't link to
- We enable it by adding the line below to `kernel/.cargo/config.toml`:
```toml
build-std-features = ["compiler-builtins-mem"]
```
- This flag enables the `mem` feature of the `compiler_builtins` create
#### Set a Default Target
- We can add the line below to `kernel/.cargo/config.toml` to avoid passing the `--target` parameter when building
```toml
[build]
target = "x86_64-blog_os.json"
```
- Now to print something to the screen from `_start`
### Printing to Screen
- I'm gonna leave this part out since its better covered in [VGA Text Mode](#vga-text-mode)
## Running our Kernel
- With our executable, we now need to turn it into a bootable disk image by linking it with a bootloader
- Then run the disk image with QEMU or boot it on real hardware
- **NOTE** from John: I have only tested with QEMU's virtual machine
### Creating a Bootimage
- Need to link with a bootloader to turn the compiled kernel into a bootable disk image
- Use the `bootloader` create, which implements a basic BIOS bootloader without any C dependencies
- We add the dependency in `kernel/Cargo.toml`:
```toml
[dependencies]
bootloader = "0.9.23"
```
- We also need to link our kernel with the bootloader after compilation, which cargo has no support for with no post-build scripts
- We then use the tool `bootimage` that will first compile the kernel and bootloader, then link them together to make the bootable image
- Install the tool with:
```
cargo install bootimage
```
- Then need to install llvm-tools-preview
```
rustup component add llvm-tools-preview
```
- Running `cargo bootimage` now creates the bootable image inside of the target directory (hence why it is added to .gitignore)
#### How does it work?
- Bootimage tool does the following steps:
    1. Compiles our kernel to an ELF (Executable and Linking Format) file
    2. Compiles the bootloader dependency as a standalone executable
    3. Links the bytes of the kernel ELF to the bootloader
- When booted, the bootloader:
    1. First reads and parses the appended ELF file
    2. Then maps the program segments to virtual addresses in the page tables
    3. Zeroes the `.bss` (block starting symbol) section
    4. Sets up a stack
    5. Finally, reads the entry point address (`_start` function) and jumps to it
### Booting it in QEMU
- Run the following command to boot in QEMU:
```
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blue_crab_os/debug bootimage-blue_crab_os.bin
```
- **NOTE**: Not needed anymore with the `cargo run` command setup [later](#using-cargo-run)
### Real Machine
- To boot it on a real machine by puting it on a USB stick:
```
dd if=target/x86_64-blue_crab_os/debug/bootimage-blue_crab_os.bin of=/dev/sdX && sync
```
- `sdX` is the device name of the USB stick
- **NOTE**: This has not been tested
- After writing to the USB, you can run it on real hardware, as long as it uses BIOS
### Using `cargo run`
- We can set the `runner` config key in `kernel/.cargo/config.toml`:
```toml
[target.'cfg(target_os = "none")']
runner = "bootimage runner"
```
- This table applies to all targets who has the `"os"` field set to `"none"`
- The `runner` key specifies the `cargo run` command, which is run after a successful build
- It runs `bootimage runner`, which will link the given executable with the project's bootloader dependency and then launch QEMU
- Readme of `bootimage` found [here](https://github.com/rust-osdev/bootimage)
# VGA Text Mode
- Simple way to print text to the screen
## The VGA Text Buffer
- To print a character to the screen in VGA text mode, one has to write it to the text buffer of the VGA hardware
- Two-dimensional array with typically 25 rows and 80 columns
- Each array entry describes a single screen character with the following format
<br/>
| foo | bar |
| --- | --- |
| baz | bim |

# Testing
