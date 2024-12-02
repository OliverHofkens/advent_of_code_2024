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

    let mut safe: usize = 0;

    'outer: while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();
                let numbers: Vec<i32, 64> = line
                    .split(' ')
                    .map(|x| i32::from_str_radix(x, 10).unwrap())
                    .collect();

                let mut dir = 0;
                for window in numbers.windows(2) {
                    match window[1] - window[0] {
                        x if x > 0 && x <= 3 && dir >= 0 => dir = 1,
                        x if x < 0 && x >= -3 && dir <= 0 => dir = -1,
                        _ => {
                            println!("{} is unsafe at {:?}", line, window);
                            continue 'outer;
                        }
                    };
                }
                println!("{} is safe", line);
                safe += 1;
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading!"),
        }
    }

    // Part 1
    println!("Safe: {}", safe);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}
