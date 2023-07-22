#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blue_crab_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blue_crab_os::println;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    #[cfg(not(test))]
    panic!("SOMEONE CALL 911, SHAWTY FIRE BURNING ON THE DANCE FLO', WHOOOAAA OOHHH OOHHH");
    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blue_crab_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}