#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::{io, iter};
use core::cmp::max;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::{Entry, FnvIndexMap, FnvIndexSet, Vec};

type Coord = (isize, isize);
type Map = FnvIndexMap<u8, Vec<Coord, 16>, 64>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<140>::new();
    let mut eof: bool = false;

    let mut map: Map = Map::new();
    let mut size: Coord = (0, 0);
    let mut y: isize = 0;

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();

                for (x, byte) in line.iter().enumerate() {
                    if *byte != b'.' {
                        match map.entry(*byte) {
                            Entry::Vacant(ent) => {
                                let mut items = Vec::new();
                                items.push((x as isize, y)).unwrap();
                                ent.insert(items).unwrap();
                            }
                            Entry::Occupied(mut ent) => {
                                ent.get_mut().push((x as isize, y)).unwrap()
                            }
                        }
                    }

                    size.0 = max(size.0, x as isize + 1);
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }

        size.1 = y;
        y += 1;
    }

    println!("Unique Antinodes: {:?}", count_antinodes(&map, &size));
    println!(
        "Unique Harmonics: {:?}",
        count_antinodes_with_harmonics(&map, &size)
    );

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn count_antinodes(map: &Map, size: &Coord) -> usize {
    let mut antis: FnvIndexSet<Coord, 512> = FnvIndexSet::new();

    for (_freq, pts) in map.iter() {
        for (pt1, pt2) in iter::PairIterator::new(pts) {
            for (start, through) in [(pt1, pt2), (pt2, pt1)] {
                let (x, y) = project(start, through);
                if (0..size.0).contains(&x) && (0..size.1).contains(&y) {
                    antis.insert((x, y)).unwrap();
                }
            }
        }
    }

    antis.len()
}

fn count_antinodes_with_harmonics(map: &Map, size: &Coord) -> usize {
    let mut antis: FnvIndexSet<Coord, 2048> = FnvIndexSet::new();

    for (_freq, pts) in map.iter() {
        for (pt1, pt2) in iter::PairIterator::new(pts) {
            for (start, through) in [(pt1, pt2), (pt2, pt1)] {
                let mut proj_pt = *start;
                let mut through_pt = *through;
                while (0..size.0).contains(&proj_pt.0) && (0..size.1).contains(&proj_pt.1) {
                    antis.insert(proj_pt).unwrap();
                    let next_proj = through_pt;
                    through_pt = project(&proj_pt, &through_pt);
                    proj_pt = next_proj;
                }
            }
        }
    }

    antis.len()
}

fn project(pt: &Coord, through: &Coord) -> Coord {
    let dx = through.0 - pt.0;
    let dy = through.1 - pt.1;
    (through.0 + dx, through.1 + dy)
}
