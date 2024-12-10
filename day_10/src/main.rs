#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::{FnvIndexSet, Vec};

const MAPDIM: usize = 55;
type Map = Vec<Vec<u8, MAPDIM>, MAPDIM>;
type Coord = (isize, isize);
type Path = FnvIndexSet<Coord, 256>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<MAPDIM>::new();
    let mut eof: bool = false;

    let mut map: Map = Map::new();

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();
                let row = line
                    .iter()
                    .map(|b| (*b as char).to_digit(10).unwrap_or(13) as u8)
                    .collect();
                map.push(row).unwrap();
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    let mut size = (map[0].len() as isize, map.len() as isize);

    let mut p1: usize = 0;
    let mut p2: usize = 0;

    // Find all zeroes and start walking:
    for (y, row) in map.iter().enumerate() {
        for (x, height) in row.iter().enumerate() {
            if *height == 0 {
                let mut path = Path::new();
                p1 += walk(&map, &(x as isize, y as isize), &size, &mut path);
                p2 += rating(&map, &(x as isize, y as isize), &size);
            }
        }
    }

    println!("Part 1: {}", p1);
    println!("Part 2: {}", p2);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn walk(map: &Map, pos: &Coord, size: &Coord, seen: &mut Path) -> usize {
    seen.insert(*pos);

    let height = map[pos.1 as usize][pos.0 as usize];

    if height == 9 {
        return 1;
    }

    let mut sum = 0;

    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
        match (pos.0 + dx, pos.1 + dy) {
            (nx, ny) if (0..size.0).contains(&nx) && (0..size.1).contains(&ny) => {
                if map[ny as usize][nx as usize] == height + 1 && !seen.contains(&(nx, ny)) {
                    sum += walk(&map, &(nx, ny), size, seen);
                }
            }
            _ => continue,
        }
    }
    sum
}

fn rating(map: &Map, pos: &Coord, size: &Coord) -> usize {
    let height = map[pos.1 as usize][pos.0 as usize];

    if height == 9 {
        return 1;
    }

    let mut sum = 0;

    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
        match (pos.0 + dx, pos.1 + dy) {
            (nx, ny) if (0..size.0).contains(&nx) && (0..size.1).contains(&ny) => {
                if map[ny as usize][nx as usize] == height + 1 {
                    sum += rating(&map, &(nx, ny), size);
                }
            }
            _ => continue,
        }
    }
    sum
}
