#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::Vec;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<64>::new();

    let mut eof: bool = false;

    const SIZE: usize = 1000;
    let mut left: Vec<i32, SIZE> = Vec::new();
    let mut right: Vec<i32, SIZE> = Vec::new();

    while !eof {
        delay.delay(1.millis());

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();
                let mut parts = line.split(' ');

                let l = i32::from_str_radix(parts.next().unwrap(), 10).unwrap();
                left.push(l).unwrap();
                let r = i32::from_str_radix(parts.next_back().unwrap(), 10).unwrap();
                right.push(r).unwrap();

                println!("{} {}", l, r);
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading!"),
        }
        reader.clear();
    }

    left.sort_unstable();
    right.sort_unstable();

    // Part 1
    let mut sum: i32 = 0;
    for (l, r) in left.iter().zip(right.iter()) {
        sum += (l - r).abs();
    }
    println!("Sum of distances: {}", sum);

    // Part 2
    let mut similarity: i32 = 0;
    for num in &left {
        let count = right.iter().filter(|x| *x == num).count() as i32;
        similarity += num * count;
    }
    println!("Total similarity: {}", similarity);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}
