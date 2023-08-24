#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blue_crab_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
    let data = 3;
    // Wrap some data in a spinlock
    let spinlock = Spinlock::new(data);

    // Lock the spinlock to get a mutex guard for the data
    let mut locked_data = spinlock.lock();
    
    assert_eq!(*locked_data, 3);
    *locked_data += 7;
    assert_eq!(*locked_data, 10);

    // the guard automatically frees the lock at the end of the scope
}

#[test_case]
fn spinning_top_spinlock_test_multi_function() {    
    let data = 2; // Make string
    let spinlock = Spinlock::new(data); // Wrap string in spinlock
    plus_one(&spinlock); // only pass a shared reference
    // We have ownership of the spinlock, so we can extract the data without locking
    // Note: this consumes the spinlock
    let data = spinlock.into_inner();
    assert_eq!(data, 3);

    // the guard automatically frees the lock at the end of the scope
}

// function will try to make data uppercase
fn plus_one(spinlock: &Spinlock<i64>) {
    // Lock the spinlock to get a mutable reference to the data
    let mut locked_data = spinlock.lock();
    assert_eq!(*locked_data, 2);
    *locked_data = 3;

    // the lock is automatically freed at the end of the scope
}

#[test_case]
fn 