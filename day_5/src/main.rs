#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::{Entry, FnvIndexMap, FnvIndexSet, Vec};

type RuleSet = FnvIndexMap<u8, FnvIndexSet<u8, 64>, 128>;
type Update = Vec<u8, 64>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<140>::new();
    let mut eof: bool = false;

    let mut reading_rules: bool = true;

    // PageNr => Pages that need to come AFTER it.
    let mut rules: RuleSet = FnvIndexMap::new();

    let mut sum: usize = 0;

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();

                if line == "" {
                    reading_rules = false;
                } else if reading_rules {
                    let mut parts = line.split('|').map(|s| u8::from_str_radix(s, 10).unwrap());
                    match rules.entry(parts.next().unwrap()) {
                        Entry::Vacant(v) => {
                            let mut set = FnvIndexSet::new();
                            set.insert(parts.next().unwrap()).unwrap();
                            v.insert(set).unwrap();
                        }
                        Entry::Occupied(mut v) => {
                            v.get_mut().insert(parts.next().unwrap()).unwrap();
                        }
                    }
                } else {
                    let update: Update = line
                        .split(',')
                        .map(|s| u8::from_str_radix(s, 10).unwrap())
                        .collect();
                    sum += check_update(&update, &rules) as usize;
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading! {}", e),
        }
    }

    println!("Sum: {}", sum);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

/// Parses `update` and returns the middle page if it's correctly sorted
/// according to `rules`.
fn check_update(update: &Update, rules: &RuleSet) -> u8 {
    let mut seen: FnvIndexSet<u8, 64> = FnvIndexSet::new();

    for page in update {
        let is_ok = match rules.get(&page) {
            Some(after) => seen.is_disjoint(after),
            None => true,
        };

        if !is_ok {
            return 0;
        }

        seen.insert(*page).unwrap();
    }

    let middle_idx = update.len() / 2;
    update[middle_idx]
}
