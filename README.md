# Blue Crab OS (BCOS)
The purpose of this project is mainly to act as a personal project to further our skills in low level programming, but also to test what an OS built with Rust would be like. Our end goal is to make a usable OS that we can at the very least dual boot and use for other development. For our base OS, we started by following the guide [here](https://os.phil-opp.com), but will be continuing on our own. 
## Unstable Features
Our OS requires the use of the nightly release channel for Rust in order to use unstable features that help us use features of Rust that would normally not be provided with `no_std`:
- `build-std`: Used in the config.toml to allow us to recompile core and other standard libraries since those aren't normally linked with our standalone binary
- `custom_test_frameworks`: Gives us Rust's test framework without the need for `std`
# Contributors
- Nick Battista
- Seth Ellis
- Michael Gay
- **John Hair**
- Joe Lewis
# How to compile the kernel
## Need to install
- Add necessary rust components
```bash
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview
```
- bootimage (In order to create the bootimage from the binary): cargo install
```cargo install bootimage```
- Also need to make sure you have a nightly version of rust:
```rustup update nightly --force```
- Download [QEMU](https://www.qemu.org/) 
- **FOR WINDOWS**: add it to your environment variables:
    - Download the installer and run it
    - Find where it sits, default is `C:\Program Files\qemu`
    - Go to windows search bar and search "environment variables"
    - Click "Environment Variables" near the bottom
    - Under user variables, click the variable "Path" and click "Edit..."
    - Then click "New" on the top right and paste in the file location of QEMU
    - If you have any open cmd prompt sessions, make sure to close them before trying to run the commands to run the QEMU VM
## Building
- Only need to build if you'd liked to boot to a flash drive with the following tools
- Build the project with:
```cargo build```
- Create a bootable disk image from the compiled kernel:
```cargo bootimage```
- This will create a bootable disk image in kernel/target/x86_64-blue_crab_os/debug
## Running
- This will also build, so no need to first build the project
- Make sure to be in the `kernel/` directory in order to be able to run the rust commands for building and running
- In order to get a basic window, you can do `cargo run`
- To run the tests, invoke `cargo test`
# Progress
- Followed the tutorial for how to create a Rust bare metal OS, sitting in the dev branch
- Notes on followed posts and Rust in `doc/` directory