# Blue Crab OS (BCOS)
The purpose of this project is mainly to act as a personal project to further our skills in low level programming, but also to test what an OS built with Rust would be like. Our end goal is to make a usable OS that we can at the very least dual boot and use for other development. For our base OS, we started by following the guide [here](https://os.phil-opp.com), but will be continuing on our own. 
# Contributors
- Nick Battista
- Seth Ellis
- Michael Gay
- John Hair
- Joe Lewis
# How to compile the kernel
- Need to make sure you have [QEMU](https://www.qemu.org/) and bootimage installed:
```cargo install bootimage```
- Also need to make sure you have a nightly version of rust:
```rustup update nightly --force```
## Building
- Build the project with:
```cargo build```
- Create a bootable disk image from the compiled kernel:
```cargo bootimage```
- This will create a bootable disk image in kernel/target/x86_64-blue_crab_os/debug
## Running
- Make sure to be in the `kernel/` directory in order to be able to run the rust commands for building and running
- In order to get a basic window, you can do `cargo run`
- To run the tests, invoke `cargo test`
# Progress
- Followed the tutorial for how to create a Rust bare metal OS, sitting in the dev branch
- Notes of followed posts in `doc/` directory