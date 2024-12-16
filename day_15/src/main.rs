#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::Vec;

type Map = Vec<Vec<u8, 50>, 50>;
type Coord = (i8, i8);

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<1000>::new();
    let mut eof: bool = false;

    let mut map = Map::new();
    let mut reading_map: bool = true;
    let mut pos: Coord = (0, 0);

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();

                if reading_map {
                    if line.is_empty() {
                        reading_map = false;
                        pos = find_bot(&map);
                    } else {
                        map.push(Vec::from_slice(line).unwrap()).unwrap();
                    }
                } else {
                    for dir in line {
                        if let Some(new_pos) = do_move(&mut map, &pos, b'@', *dir) {
                            pos = new_pos;
                        }
                    }
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    let sum = gps_sum(&map);
    println!("Sum: {}", sum);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn find_bot(map: &Map) -> Coord {
    for (y, row) in map.iter().enumerate() {
        if let Some(x) = row.iter().position(|b| *b == b'@') {
            return (x as i8, y as i8);
        }
    }
    (0, 0)
}

fn do_move(map: &mut Map, pos: &Coord, marker: u8, dir: u8) -> Option<Coord> {
    let (dx, dy) = match dir {
        b'^' => (0, -1),
        b'>' => (1, 0),
        b'v' => (0, 1),
        b'<' => (-1, 0),
        _ => unreachable!(),
    };

    let nx = pos.0 + dx;
    let ny = pos.1 + dy;

    let target = map[ny as usize][nx as usize];

    match target {
        b'#' => None,
        b'.' => {
            // println!(
            //     "Moving {} from {:?} to ({}, {})",
            //     marker as char, pos, nx, ny
            // );
            map[ny as usize][nx as usize] = marker;
            map[pos.1 as usize][pos.0 as usize] = b'.';
            Some((nx, ny))
        }
        b'O' => {
            // Try to move the box, and only move if the box moved.
            match do_move(map, &(nx, ny), b'O', dir) {
                Some(_) => {
                    // println!(
                    //     "Moving {} from {:?} to ({}, {})",
                    //     marker as char, pos, nx, ny
                    // );
                    // Box moved, so we move:
                    map[ny as usize][nx as usize] = marker;
                    map[pos.1 as usize][pos.0 as usize] = b'.';
                    Some((nx, ny))
                }
                None => None,
            }
        }
        x => unreachable!(),
    }
}

fn gps_sum(map: &Map) -> u64 {
    let mut sum = 0;
    for (y, row) in map.iter().enumerate() {
        sum += row
            .iter()
            .enumerate()
            .filter_map(|(x, c)| if *c == b'O' { Some(x) } else { None })
            .map(|x| 100 * y + x)
            .sum::<usize>() as u64;
    }
    sum
}
