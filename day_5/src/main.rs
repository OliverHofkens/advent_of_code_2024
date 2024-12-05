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

    let mut ok_sum: usize = 0;
    let mut reordered_sum: usize = 0;

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
                    let mut update: Update = line
                        .split(',')
                        .map(|s| u8::from_str_radix(s, 10).unwrap())
                        .collect();

                    match unsorted_at(&update, &rules) {
                        None => ok_sum += update_val(&update),
                        Some(page) => {
                            reorder_update(&mut update, &rules, page);
                            reordered_sum += update_val(&update);
                        }
                    }
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading! {}", e),
        }
    }

    println!("Part 1: {}", ok_sum);
    println!("Part 2: {}", reordered_sum);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

/// Returns the page which is not sorted correctly, or None if everything is sorted correctly.
fn unsorted_at(update: &Update, rules: &RuleSet) -> Option<u8> {
    let mut seen: FnvIndexSet<u8, 64> = FnvIndexSet::new();

    for page in update {
        let is_ok = match rules.get(&page) {
            Some(after) => seen.is_disjoint(after),
            None => true,
        };

        if !is_ok {
            return Some(*page);
        }

        seen.insert(*page).unwrap();
    }

    return None;
}

fn update_val(update: &Update) -> usize {
    let middle_idx = update.len() / 2;
    update[middle_idx] as usize
}

fn reorder_update(update: &mut Update, rules: &RuleSet, mut wrong_page: u8) {
    loop {
        let before = rules.get(&wrong_page).unwrap();
        let wrong_idx = update.iter().position(|p| *p == wrong_page).unwrap();

        // Get earliest index to move before
        let earliest = before
            .iter()
            .filter_map(|pg| update.iter().position(|p| p == pg))
            .min()
            .unwrap();

        // Move the wrong page to before the earliest it should be.
        update.remove(wrong_idx);
        update.insert(earliest, wrong_page);

        // Check again, if we good, we good
        match unsorted_at(&update, &rules) {
            Some(x) => wrong_page = x,
            None => break,
        };
    }
}
