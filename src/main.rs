#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(small_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use small_os::{hlt_loop, init, println};
use x86_64::registers::control::Cr3;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    init();

    let (lvl_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", lvl_4_page_table.start_address());

    #[cfg(test)]
    test_main();

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
