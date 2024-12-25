#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use core::str::FromStr;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::{FnvIndexMap, String, Vec};

type Towel = String<8>;
type Towels = Vec<Towel, 500>;

const CACHE_KEY_SIZE: usize = 52;
type CacheKey = String<CACHE_KEY_SIZE>;
type Cache = FnvIndexMap<CacheKey, bool, 4092>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<3000>::new();
    let mut eof: bool = false;

    let mut read_idx = 0usize;
    let mut towels = Towels::new();
    let mut cache = Cache::new();
    let mut designs_made = 0usize;

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();

                if read_idx == 0 {
                    towels = line
                        .split(", ")
                        .map(|s| Towel::from_str(s).unwrap())
                        .collect();
                    println!("{:?}", towels);
                } else if read_idx > 1 {
                    println!("Trying to make {}", line);
                    if try_make_design(line, &towels, &mut cache) {
                        println!("Made design {}", line);
                        designs_made += 1;
                    } else {
                        println!("Can't make {}", line);
                    }
                }
                read_idx += 1;
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    println!("Designs made: {}", designs_made);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn try_make_design(design: &str, towels: &Towels, cache: &mut Cache) -> bool {
    // println!("Trying to make {design}");
    if design.len() == 0 {
        return true;
    }

    if design.len() <= CACHE_KEY_SIZE {
        if let Some(res) = cache.get(&CacheKey::from_str(design).unwrap()) {
            // println!("Cache hit!");
            return *res;
        }
    }

    let mut res = false;

    for towel in towels {
        if design.starts_with(towel.as_str()) {
            let rest = &design[towel.len()..];
            if try_make_design(rest, towels, cache) {
                res = true;
                break;
            }
        }
    }

    if design.len() <= CACHE_KEY_SIZE {
        cache
            .insert(CacheKey::from_str(design).unwrap(), res)
            .unwrap();
    }

    res
}
