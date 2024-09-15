#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(small_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use small_os::println;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    small_os::test_panic_handler(info)
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}
