use heapless::binary_heap::{BinaryHeap, Min};
use heapless::Vec;
use std::env;
use std::fs;

const MAP_SIZE: usize = 71;
const BLOCKS_TO_DROP: usize = 1024;
type Map = [[bool; MAP_SIZE]; MAP_SIZE];
type Coord = (i16, i16);

fn main() {
    let inp = get_input_contents();

    let mut drops: Vec<Coord, 4096> = Vec::new();

    for line in inp.lines() {
        let (xstr, ystr) = line.split_once(',').unwrap();
        drops
            .push((
                i16::from_str_radix(xstr, 10).unwrap(),
                i16::from_str_radix(ystr, 10).unwrap(),
            ))
            .unwrap();
    }

    let start = (0, 0);
    let end = ((MAP_SIZE - 1) as i16, (MAP_SIZE - 1) as i16);

    // Part 1:
    let mut map = [[true; MAP_SIZE]; MAP_SIZE];
    for (x, y) in drops.iter().take(BLOCKS_TO_DROP) {
        map[*y as usize][*x as usize] = false;
    }
    dump_map(&map);
    let p1 = dijkstra(&map, start, end);
    println!("Part 1: {:?}", p1);

    // Part 2:
    map = [[true; MAP_SIZE]; MAP_SIZE];
    for (x, y) in drops.iter() {
        map[*y as usize][*x as usize] = false;
        match dijkstra(&map, start, end) {
            Some(cost) => {
                // println!("Still reachable in {cost}")
            }
            None => {
                dump_map(&map);
                println!("P2: First blocking byte: ({x},{y})");
                break;
            }
        }
    }
}

fn get_input_contents() -> String {
    let args: Vec<String, 10> = env::args().collect();
    let filename = &args[1];
    fs::read_to_string(filename).expect("Failed to read file")
}

fn dump_map(map: &Map) {
    for row in map {
        for x in row {
            match x {
                true => print!("."),
                false => print!("#"),
            };
        }
        print!("\n");
    }
    println!("\n");
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Node {
    pos: Coord,
    cost: u64,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.cost.cmp(&other.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn dijkstra(map: &Map, start: Coord, end: Coord) -> Option<u64> {
    let mut q: BinaryHeap<Node, Min, 1024> = BinaryHeap::new();
    let mut processed = [[false; MAP_SIZE]; MAP_SIZE];
    let mut distance = [[u64::MAX; MAP_SIZE]; MAP_SIZE];

    distance[start.1 as usize][start.0 as usize] = 0;
    q.push(Node {
        pos: start,
        cost: 0,
    })
    .unwrap();

    let mut min_end_cost = u64::MAX;

    while let Some(node) = q.pop() {
        // Skip if we've found better paths
        if node.cost > min_end_cost {
            continue;
        }

        if processed[node.pos.1 as usize][node.pos.0 as usize] {
            continue;
        }
        processed[node.pos.1 as usize][node.pos.0 as usize] = true;

        // Update min_end_cost if we've reached the end
        if node.pos == end {
            min_end_cost = min_end_cost.min(node.cost);
        }

        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let nx = node.pos.0 + dx;
            let ny = node.pos.1 + dy;

            if nx < 0 || ny < 0 || nx >= MAP_SIZE as i16 || ny >= MAP_SIZE as i16 {
                continue;
            }
            if !map[ny as usize][nx as usize] {
                continue;
            }

            let new_cost = node.cost + 1;

            let curr_dist = &mut distance[ny as usize][nx as usize];
            if new_cost < *curr_dist {
                *curr_dist = new_cost;
                let new_node = Node {
                    pos: (nx, ny),
                    cost: new_cost,
                };
                q.push(new_node).unwrap();
            }
        }
    }

    if min_end_cost < u64::MAX {
        Some(min_end_cost)
    } else {
        None
    }
}
