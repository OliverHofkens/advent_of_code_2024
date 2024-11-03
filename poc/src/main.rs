#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use embedded_io::Read;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};

const INPUT_BUF_SIZE_B: usize = 22 * 1000;

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    esp_println::logger::init_logger_from_env();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let reader = io::LineReader::<64>::new();

    loop {
        delay.delay(500.millis());
        let line = reader.line();
        defmt::println!("Read: {}", line);
    }
}
