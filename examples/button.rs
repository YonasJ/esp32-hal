#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use esp32_hal::prelude::*;
use esp32_hal::println;
use esp32_hal::console::Console;

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };
    esp32_hal::watchdog_disabler::disable();
    Console::begin(19200);

    let gpios = dp.GPIO.split();
    let mut blinky2 = gpios.gpio2.into_push_pull_output();
    let button =  gpios.gpio15.into_pull_down_input();


    println!("Monitoring button. Press it the led will follow it, and it will print 'button pushed.' on the serial.");
    let mut last_val:bool = button.is_high().unwrap();
    println!("Button value is: {}", last_val);
    loop {
        let val = button.is_high().unwrap();
        if last_val != val {
            if val {
                println!("Button pushed.");
                blinky2.set_high().unwrap();
            } else {
                println!("Button released.");
                blinky2.set_low().unwrap();
            }
            last_val = val;
        }
    }
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }