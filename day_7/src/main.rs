#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::{io, num};
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::Vec;

type Terms = Vec<u64, 16>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<140>::new();
    let mut eof: bool = false;

    let mut sum: u64 = 0;
    let mut terms: Terms = Vec::new();

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();

                let (test_val_str, rest) = line.split_once(": ").unwrap();
                let test_val = u64::from_str_radix(test_val_str, 10).unwrap();
                terms = rest
                    .split(' ')
                    .map(|s| u64::from_str_radix(s, 10).unwrap())
                    .collect();

                if is_solvable(test_val, terms[0], &terms[1..]) {
                    println!("{test_val} is solvable");
                    sum += test_val;
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }

        terms.clear();
    }

    println!("P1: {sum}");

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn is_solvable(test_val: u64, acc: u64, terms: &[u64]) -> bool {
    let next = terms[0];
    let rest = &terms[1..];
    let leaf = rest.is_empty();

    match acc.checked_add(next) {
        Some(x) if leaf && x == test_val => return true,
        Some(new_acc) if !leaf && new_acc <= test_val && is_solvable(test_val, new_acc, &rest) => {
            return true
        }
        _ => (),
    }

    match acc.checked_mul(next) {
        Some(x) if leaf && x == test_val => return true,
        Some(new_acc) if !leaf && new_acc <= test_val && is_solvable(test_val, new_acc, &rest) => {
            return true
        }
        _ => (),
    }

    match num::concat(acc, next) {
        Some(x) if leaf && x == test_val => true,
        Some(new_acc) if !leaf && new_acc <= test_val => is_solvable(test_val, new_acc, &rest),
        _ => false,
    }
}
