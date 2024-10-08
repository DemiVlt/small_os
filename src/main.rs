#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(small_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use small_os::{hlt_loop, println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}{}", "!", "!");

    #[cfg(test)]
    test_main();

    small_os::init();
    println!("It didn't crash!");

    hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    small_os::test_panic_handler(info)
}
