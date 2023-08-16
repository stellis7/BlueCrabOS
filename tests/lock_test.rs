#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blue_crab_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blue_crab_os::println;
use core::panic::PanicInfo;
use spinning_top::Spinlock;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blue_crab_os::test_panic_handler(info)
}

#[test_case]
fn spinning_top_spinlock_test() {    
    let data = String::from("Hello"); // Make string
    let spinlock = Spinlock::new(data); // Wrap string in spinlock
    make_uppercase(&spinlock); // only pass a shared reference
    // We have ownership of the spinlock, so we can extract the data without locking
    // Note: this consumes the spinlock
    let data = spinlock.into_inner();
    assert_eq!(data.as_str(), "HELLO");
}

// function will try to make data uppercase
fn make_uppercase(spinlock: &Spinlock<String>) {
    // Lock the spinlock to get a mutable reference to the data
    let mut locked_data = spinlock.lock();
    assert_eq!(locked_data.as_str(), "Hello");
    locked_data.make_ascii_uppercase();

    // the lock is automatically freed at the end of the scope
}