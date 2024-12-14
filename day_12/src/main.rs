#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::{FnvIndexSet, Vec};

const MK_SEEN: u8 = b'.';

type Map = Vec<Vec<u8, 140>, 140>;
type Coord = (u8, u8);
type SeenCache = FnvIndexSet<Coord, 1024>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<140>::new();
    let mut eof: bool = false;

    let mut map = Map::new();

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();
                map.push(Vec::from_slice(line).unwrap()).unwrap();
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    // Part 1
    // let costs = calculate_costs(&mut map, Costing::PERIMETER);
    // println!("Cost: {}", costs);

    // Part 2
    let costs = calculate_costs(&mut map, &Costing::SIDES);
    println!("Cost: {}", costs);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

enum Costing {
    PERIMETER,
    SIDES,
}

fn calculate_costs(map: &mut Map, costing: &Costing) -> u64 {
    let mut sum: u64 = 0;
    while let Some((area, perim)) = boom_area(map, costing) {
        sum += area * perim;
    }
    sum
}

fn boom_area(map: &mut Map, costing: &Costing) -> Option<(u64, u64)> {
    // Find a starting point
    let mut area_start: Option<Coord> = None;
    for (y, row) in map.iter().enumerate() {
        if let Some(x_start) = row.iter().position(|c| *c != MK_SEEN) {
            area_start = Some((x_start as u8, y as u8));
            break;
        }
    }

    match area_start {
        None => None,
        Some((x, y)) => {
            let mut seen = SeenCache::new();
            let marker = map[y as usize][x as usize];

            let (area, perim) = scan(map, &(x, y), marker, costing, &mut seen);

            for (x, y) in seen.iter() {
                map[*y as usize][*x as usize] = MK_SEEN;
            }
            println!("{} @ ({x}, {y}): {area}A * {perim}C", marker as char);

            Some((area, perim))
        }
    }
}

fn scan(map: &Map, pos: &Coord, marker: u8, costing: &Costing, seen: &mut SeenCache) -> (u64, u64) {
    seen.insert(*pos).unwrap();

    let xsize = map[0].len() as u8;
    let ysize = map.len() as u8;

    let mut area = 1;
    let mut perim = 0;
    let mut sides: u64 = 0;

    let mut is_diff = [false; 9];
    let mut i = 0;

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                i += 1;
                continue;
            }

            let nx: Option<u8> = (pos.0 as i16 + dx).try_into().ok();
            let ny: Option<u8> = (pos.1 as i16 + dy).try_into().ok();

            match (nx, ny) {
                (None, _) | (_, None) => is_diff[i] = true,
                (Some(nx), Some(ny)) if nx >= xsize || ny >= ysize => is_diff[i] = true,
                (Some(nx), Some(ny)) if map[ny as usize][nx as usize] != marker => {
                    is_diff[i] = true
                }
                (Some(nx), Some(ny)) if seen.contains(&(nx, ny)) => (),
                (Some(nx), Some(ny)) if dx == 0 || dy == 0 => {
                    // Orthogonals
                    let (new_area, new_perim) = scan(map, &(nx, ny), marker, costing, seen);
                    area += new_area;
                    match costing {
                        Costing::PERIMETER => perim += new_perim,
                        Costing::SIDES => sides += new_perim,
                    }
                }
                _ => {
                    // Diagonal of same marker, don't do anything.
                }
            }

            i += 1;
        }
    }
    match costing {
        Costing::PERIMETER => {
            perim += is_diff
                .iter()
                .enumerate()
                .filter_map(|(i, x)| match i % 2 {
                    1 => Some(*x as u8),
                    _ => None,
                })
                .sum::<u8>() as u64;
            (area, perim)
        }
        Costing::SIDES => {
            // Paraphrased from Reddit because fuck this
            let [NW, N, NE, W, _, E, SW, S, SE] = is_diff;

            let corners = [
                !N && !W && NW,
                !N && !E && NE,
                !S && !W && SW,
                !S && !E && SE,
                N && W,
                N && E,
                S && W,
                S && E,
            ]
            .iter()
            .map(|b| *b as u8)
            .sum::<u8>() as u64;

            // println!(
            //     "[{}] @ {:?}: {corners}C, ({:?})",
            //     marker as char, pos, is_diff
            // );
            (area, sides + corners)
        }
    }
}
