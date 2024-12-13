#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::{io, num};
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::FnvIndexMap;

type Cache = FnvIndexMap<(u64, u8), u64, 8192>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<64>::new();
    let mut eof: bool = false;

    let mut cache = Cache::new();

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();

                let count = line
                    .split(' ')
                    .map(|s| u64::from_str_radix(s, 10).unwrap())
                    .fold(0u64, |acc, num| {
                        acc.checked_add(simulate_blinks(num, 25, &mut cache))
                            .unwrap()
                    });

                println!("Count: {}", count);
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn simulate_blinks(stone: u64, blinks: u8, cache: &mut Cache) -> u64 {
    if blinks == 0 {
        return 1;
    }

    if let Some(res) = cache.get(&(stone, blinks)) {
        return *res;
    }

    let next_blinks = blinks - 1;
    let n_digits = num::count_digits(stone);

    let res = if stone == 0 {
        simulate_blinks(1, next_blinks, cache)
    } else if n_digits % 2 == 0 {
        let (l, r) = num::split(stone, n_digits / 2);
        simulate_blinks(l, next_blinks, cache) + simulate_blinks(r, next_blinks, cache)
    } else {
        simulate_blinks(stone.checked_mul(2024).unwrap(), next_blinks, cache)
    };

    if cache.len() < cache.capacity() {
        cache.insert((stone, blinks), res).unwrap();
    }
    res
}
