use heapless::binary_heap::{BinaryHeap, Min};
use heapless::{Entry, FnvIndexMap, Vec};
use std::env;
use std::fs;

const MAP_SIZE: usize = 141;
type Map = Vec<Vec<bool, MAP_SIZE>, MAP_SIZE>;
type Pos = (i16, i16);
type CheatMap = FnvIndexMap<u64, Vec<Pos, 1024>, 1024>;

fn main() {
    let mut map = Map::new();
    let mut start: Pos = (0, 0);
    let mut end: Pos = (0, 0);

    let inp = get_input_contents();

    for (y, line) in inp.lines().enumerate() {
        let mut row = Vec::new();
        for (x, b) in line.bytes().enumerate() {
            match b {
                b'#' => row.push(false),
                b'.' => row.push(true),
                b'S' => {
                    start = (x as i16, y as i16);
                    row.push(true)
                }
                b'E' => {
                    end = (x as i16, y as i16);
                    row.push(true)
                }
                x => panic!("Unexpected byte on map: {x}"),
            }
            .unwrap();
        }
        map.push(row).unwrap();
    }

    println!("Start @ {:?}, End @ {:?}", start, end);

    let base_distance = dijkstra(&map, start, end).unwrap();
    println!("Base distance: {}", base_distance);

    let walls = list_possible_cheat_positions(&map);
    let mut cheats = CheatMap::new();

    for (x, y) in walls {
        // Remove the wall:
        map[y as usize][x as usize] = true;

        let new_distance = dijkstra(&map, start, end).unwrap();
        let saving = base_distance - new_distance;
        if saving > 0 {
            match cheats.entry(saving) {
                Entry::Vacant(v) => {
                    let mut cheat_positions = Vec::new();
                    cheat_positions.push((x as i16, y as i16)).unwrap();
                    v.insert(cheat_positions).unwrap();
                }
                Entry::Occupied(mut v) => {
                    v.get_mut().push((x as i16, y as i16)).unwrap();
                }
            }
        }

        // Place back the wall:
        map[y as usize][x as usize] = false;
    }

    println!("Cheats: {:?}", cheats);

    let p1: usize = cheats
        .iter()
        .filter(|(k, _v)| **k >= 100)
        .map(|(_k, v)| v.len())
        .sum();
    println!("Total cheats saving 100: {}", p1);
}

fn get_input_contents() -> std::string::String {
    let args: Vec<std::string::String, 10> = env::args().collect();
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
    pos: Pos,
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

fn dijkstra(map: &Map, start: Pos, end: Pos) -> Option<u64> {
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

fn list_possible_cheat_positions(map: &Map) -> Vec<Pos, 16384> {
    let mut res = Vec::new();

    for (y, row) in map.iter().enumerate() {
        // Don't check map borders
        if y == 0 || y == MAP_SIZE - 1 {
            continue;
        }

        for (x, is_road) in row.iter().enumerate() {
            // Don't check map borders or roads
            if x == 0 || x == MAP_SIZE - 1 || *is_road {
                continue;
            }

            res.push((x as i16, y as i16)).unwrap();
        }
    }

    res
}
