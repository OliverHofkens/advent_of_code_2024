#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::Vec;

// Part 1: false, Part 2: true
const PROBLEM_DAMPENER: bool = true;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<64>::new();

    let mut eof: bool = false;

    let mut safe: usize = 0;

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();
                let numbers: Vec<i32, 64> = line
                    .split(' ')
                    .map(|x| i32::from_str_radix(x, 10).unwrap())
                    .collect();

                match unsafe_at(&numbers) {
                    None => safe += 1,
                    Some(ix) if PROBLEM_DAMPENER => {
                        for rem in ix.saturating_sub(1)..=ix.saturating_add(1) {
                            // Remove x and try again:
                            let mut retry = numbers.clone();
                            retry.remove(rem);
                            if let None = unsafe_at(&retry) {
                                safe += 1;
                                println!("Problem dampened: {}", line);
                                break;
                            }
                        }
                    }
                    _ => continue,
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading! {}", e),
        }
    }

    println!("Safe: {}", safe);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn unsafe_at(levels: &[i32]) -> Option<usize> {
    let mut dir = 0;

    for (i, window) in levels.windows(2).enumerate() {
        match window[1] - window[0] {
            x if x > 0 && x <= 3 && dir >= 0 => dir = 1,
            x if x < 0 && x >= -3 && dir <= 0 => dir = -1,
            _ => return Some(i),
        };
    }
    None
}
