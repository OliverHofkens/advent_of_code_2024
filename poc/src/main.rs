#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use embedded_io::Read;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<64>::new();

    println!("Initialized");

    loop {
        delay.delay(100.millis());

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();
                println!("Read: {:?}", line);
            }
            Ok(false) => println!("Nothing"),
            Err(e) => println!("Error reading! {}", e),
        }
    }
}
