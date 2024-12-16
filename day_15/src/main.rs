#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::{print, println};
use heapless::Vec;

type Map = Vec<Vec<u8, 100>, 50>;
type Coord = (i8, i8);

const WIDE_MODE: bool = true;

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
                        if WIDE_MODE {
                            let mut row = Vec::new();
                            for c in line {
                                match c {
                                    b'#' => row.extend_from_slice(b"##").unwrap(),
                                    b'O' => row.extend_from_slice(b"[]").unwrap(),
                                    b'.' => row.extend_from_slice(b"..").unwrap(),
                                    b'@' => row.extend_from_slice(b"@.").unwrap(),
                                    x => row.push(*x).unwrap(),
                                };
                            }
                            map.push(row).unwrap();
                        } else {
                            map.push(Vec::from_slice(line).unwrap()).unwrap();
                        }
                    }
                } else {
                    for dir in line {
                        if let Some(new_pos) = maybe_move(&mut map, &pos, b'@', *dir) {
                            // dump_map(&map);
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

fn dump_map(map: &Map) {
    for row in map {
        for c in row {
            print!("{}", *c as char);
        }
        print!("\n")
    }
}

fn maybe_move(map: &mut Map, pos: &Coord, marker: u8, dir: u8) -> Option<Coord> {
    let (dx, dy) = match dir {
        b'^' => (0, -1),
        b'>' => (1, 0),
        b'v' => (0, 1),
        b'<' => (-1, 0),
        _ => unreachable!(),
    };

    if can_move(map, pos, marker, dx, dy) {
        Some(do_move(map, pos, marker, dx, dy, false))
    } else {
        None
    }
}

fn can_move(map: &Map, pos: &Coord, marker: u8, dx: i8, dy: i8) -> bool {
    let nx = pos.0 + dx;
    let ny = pos.1 + dy;

    let target = map[ny as usize][nx as usize];

    match target {
        b'#' => false,
        b'.' => true,
        b'O' => can_move(map, &(nx, ny), marker, dx, dy),
        b'[' | b']' if dy == 0 => can_move(map, &(nx, ny), marker, dx, dy),
        b'[' => {
            can_move(map, &(nx, ny), b'[', dx, dy) && can_move(map, &(nx + 1, ny), b']', dx, dy)
        }
        b']' => {
            can_move(map, &(nx - 1, ny), b'[', dx, dy) && can_move(map, &(nx, ny), b']', dx, dy)
        }
        _ => unreachable!(),
    }
}

fn do_move(map: &mut Map, pos: &Coord, marker: u8, dx: i8, dy: i8, is_child: bool) -> Coord {
    let nx = pos.0 + dx;
    let ny = pos.1 + dy;

    // println!(
    //     "Moving {} from {:?} to ({}, {})",
    //     marker as char, pos, nx, ny
    // );

    let target = map[ny as usize][nx as usize];

    // Move target first:
    if target == b'O' || target == b'[' || target == b']' {
        do_move(map, &(nx, ny), target, dx, dy, false);
    }

    // Move wide boxes together when moving up or down
    if !is_child && dy != 0 {
        if marker == b'[' {
            do_move(map, &(pos.0 + 1, pos.1), b']', dx, dy, true);
        } else if marker == b']' {
            do_move(map, &(pos.0 - 1, pos.1), b'[', dx, dy, true);
        }
    }

    // Move ourself
    map[ny as usize][nx as usize] = marker;
    map[pos.1 as usize][pos.0 as usize] = b'.';
    (nx, ny)
}

fn gps_sum(map: &Map) -> u64 {
    let mut sum = 0;
    for (y, row) in map.iter().enumerate() {
        sum += row
            .iter()
            .enumerate()
            .filter_map(|(x, c)| {
                if *c == b'O' || *c == b'[' {
                    Some(x)
                } else {
                    None
                }
            })
            .map(|x| 100 * y + x)
            .sum::<usize>() as u64;
    }
    sum
}
