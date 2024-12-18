#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::{print, println};
use heapless::binary_heap::{BinaryHeap, Min};
use heapless::Vec;

const MAP_SIZE: usize = 141;
type Map = Vec<Vec<u8, MAP_SIZE>, MAP_SIZE>;
type Coord = (i16, i16);

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<MAP_SIZE>::new();
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

    let start = find_on_map(&map, b'S').unwrap();
    let end = find_on_map(&map, b'E').unwrap();
    println!("Start at {:?}, End at {:?}", start, end);

    let cost = dijkstra(&map, start, end);
    println!("Cost: {cost}");

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn find_on_map(map: &Map, needle: u8) -> Option<Coord> {
    for (y, row) in map.iter().enumerate() {
        if let Some(x) = row.iter().position(|c| *c == needle) {
            return Some((x as i16, y as i16));
        }
    }
    None
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Node {
    pos: Coord,
    dir: Dir,
    cost: u64,
}

// Implementation for BinaryHeap ordering
impl Ord for Node {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Reverse ordering for min-heap instead of max-heap
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    fn dxdy(self) -> Coord {
        match self {
            Self::N => (0, 1),
            Self::E => (1, 0),
            Self::S => (0, -1),
            Self::W => (-1, 0),
        }
    }

    fn diff(&self, other: &Self) -> u64 {
        match (self, other) {
            _ if self == other => 0,
            (Dir::N, Dir::S) | (Dir::S, Dir::N) | (Dir::E, Dir::W) | (Dir::W, Dir::E) => 2,
            _ => 1,
        }
    }
}

fn dijkstra(map: &Map, start: Coord, end: Coord) -> u64 {
    let mut q: BinaryHeap<Node, Min, 32> = BinaryHeap::new();
    let mut processed = [[false; MAP_SIZE]; MAP_SIZE];
    let mut distance = [[u64::MAX; MAP_SIZE]; MAP_SIZE];

    distance[start.1 as usize][start.0 as usize] = 0;
    q.push(Node {
        pos: start,
        dir: Dir::E,
        cost: 0,
    })
    .unwrap();

    while let Some(node) = q.pop() {
        if node.pos == end {
            // for line in distance {
            //     for cost in line {
            //         print!("{cost},");
            //     }
            //     print!("\n");
            // }
            return node.cost;
        }

        if processed[node.pos.1 as usize][node.pos.0 as usize] {
            continue;
        }
        processed[node.pos.1 as usize][node.pos.0 as usize] = true;

        // Find positions reachable from here
        for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
            let (dx, dy) = dir.dxdy();
            let nx = node.pos.0 + dx;
            let ny = node.pos.1 + dy;

            if nx < 0 || ny < 0 || nx >= MAP_SIZE as i16 || ny >= MAP_SIZE as i16 {
                continue;
            }
            if map[ny as usize][nx as usize] == b'#' {
                continue;
            }

            let new_cost = node.cost + 1 + (1000 * node.dir.diff(&dir));

            if new_cost < distance[ny as usize][nx as usize] {
                distance[ny as usize][nx as usize] = new_cost;
                q.push(Node {
                    pos: (nx, ny),
                    dir,
                    cost: new_cost,
                })
                .unwrap();
            }
        }
    }

    0
}
