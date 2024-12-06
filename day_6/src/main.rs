#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::{FnvIndexSet, Vec};

type Coord = (isize, isize);

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<140>::new();
    let mut eof: bool = false;

    let mut map: Map = Map::new();
    let mut read_pos: Coord = (0, 0);
    let mut guard_pos: Coord = (0, 0);
    let mut guard_dir: Dir = Dir::N;

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();

                let mut map_row = Vec::new();
                read_pos.0 = 0;

                for c in line.iter() {
                    match c {
                        b'#' => map_row.push(true).unwrap(),
                        b'.' => map_row.push(false).unwrap(),
                        g @ (b'<' | b'^' | b'>' | b'v') => {
                            map_row.push(false).unwrap();
                            guard_pos = read_pos;
                            guard_dir = Dir::from_u8(&g);
                        }
                        x => panic!("Unsupported tile {x}"),
                    }
                    read_pos.0 += 1;
                }
                map.0.push(map_row).unwrap();
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading! {}", e),
        }
        read_pos.1 += 1;
    }

    let p1 = simulate(&map, &mut guard_pos, guard_dir);
    println!("Part 1: {p1}");

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

#[derive(Debug)]
enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    fn from_u8(c: &u8) -> Self {
        match c {
            b'^' => Dir::N,
            b'>' => Dir::E,
            b'v' => Dir::S,
            b'<' => Dir::W,
            x => panic!("Unsupported dir {x}"),
        }
    }

    fn rotate(&self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::E => Dir::S,
            Dir::S => Dir::W,
            Dir::W => Dir::N,
        }
    }
}

struct Map(Vec<Vec<bool, 130>, 130>);

impl Map {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn is_obstacle(&self, pos: &Coord) -> bool {
        if let Some(row) = self.0.get(pos.1 as usize) {
            return match row.get(pos.0 as usize) {
                Some(c) => *c,
                None => false,
            };
        }
        false
    }

    fn size(&self) -> (isize, isize) {
        (self.0[0].len() as isize, self.0.len() as isize)
    }
}

fn simulate(map: &Map, guard_pos: &mut Coord, mut guard_dir: Dir) -> usize {
    let mut seen: FnvIndexSet<Coord, 16384> = FnvIndexSet::new();
    let map_size = map.size();

    loop {
        seen.insert(*guard_pos).unwrap();
        let (dx, dy) = match guard_dir {
            Dir::N => (0, -1),
            Dir::E => (1, 0),
            Dir::S => (0, 1),
            Dir::W => (-1, 0),
        };

        guard_pos.0 += dx;
        guard_pos.1 += dy;

        if guard_pos.0 < 0
            || guard_pos.0 >= map_size.0
            || guard_pos.1 < 0
            || guard_pos.1 >= map_size.1
        {
            break;
        }

        let next_pos: Coord = (guard_pos.0 + dx, guard_pos.1 + dy);
        if map.is_obstacle(&next_pos) {
            guard_dir = guard_dir.rotate();
            println!(
                "Guard blocked at {:?}, turned to {:?}",
                guard_pos, guard_dir
            );
        }
    }

    seen.len()
}
