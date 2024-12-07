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
type Path = FnvIndexSet<Coord, 8192>;

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
    let mut path: Path = FnvIndexSet::new();

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

    let _ = simulate(&map, &guard_pos, &guard_dir, true, &mut path);
    println!("Part 1: {}", path.len());

    let p2 = find_looping_blockades(&mut map, &path, &guard_pos, &guard_dir);
    println!("Part 2: {p2}");

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

#[derive(Clone, Copy, Debug)]
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

    fn toggle(&mut self, pos: &Coord) {
        let x = pos.0 as usize;
        let y = pos.1 as usize;
        self.0[y][x] = !self.0[y][x];
    }
}

fn simulate<const N: usize>(
    map: &Map,
    start_pos: &Coord,
    start_dir: &Dir,
    do_trace: bool,
    tracepath: &mut FnvIndexSet<Coord, N>,
) -> bool {
    let mut guard_pos = start_pos.clone();
    let mut guard_dir = start_dir.clone();
    let mut path: FnvIndexSet<(isize, isize, u8), 256> = FnvIndexSet::new();
    let map_size = map.size();
    let mut is_loop = false;

    loop {
        if do_trace {
            tracepath.insert(guard_pos).unwrap();
        }

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
            let is_new = path
                .insert((guard_pos.0, guard_pos.1, guard_dir.to_u8()))
                .unwrap();
            if !is_new {
                is_loop = true;
                break;
            }
            guard_dir = guard_dir.rotate();
        }
    }

    is_loop
}

fn find_looping_blockades(
    map: &mut Map,
    orig_path: &Path,
    start_pos: &Coord,
    start_dir: &Dir,
) -> usize {
    let blockade_options = orig_path.iter().filter(|c| *c != start_pos);
    let mut hits: usize = 0;

    let mut empty_tracepath: FnvIndexSet<Coord, 2> = FnvIndexSet::new();

    for pos in blockade_options {
        map.toggle(pos);
        let is_loop = simulate(&map, start_pos, start_dir, false, &mut empty_tracepath);
        if is_loop {
            hits += 1;
        }
        map.toggle(pos);
    }

    hits
}
