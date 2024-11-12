#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
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

    let mut sum: u32 = 0;

    loop {
        delay.delay(10.millis());

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();

                let mut iter = line.iter().filter_map(|d| (*d as char).to_digit(10));
                let first_digit = iter.next().unwrap();
                let last_digit = iter.next_back().unwrap_or(first_digit);
                let number = first_digit * 10 + last_digit;
                println!("Number: {}", number);
                sum += number;
            }
            Ok(false) => continue,
            Err(e) => println!("Error reading! {}", e),
        }
        println!("Result: {}", sum);
        reader.clear();
    }
}
