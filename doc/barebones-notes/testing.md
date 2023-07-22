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
## Exiting QEMU
### I/O Ports
### Using the Exit Device
### Success Exit Code
