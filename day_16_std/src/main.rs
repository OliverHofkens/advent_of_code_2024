use heapless::binary_heap::{BinaryHeap, Min};
use heapless::{FnvIndexSet, Vec};
use std::env;
use std::fs;

const MAP_SIZE: usize = 141;
type Map = Vec<Vec<char, MAP_SIZE>, MAP_SIZE>;
type Coord = (i16, i16);
type Distances = [[[u64; 4]; MAP_SIZE]; MAP_SIZE];
type PosSet = FnvIndexSet<Coord, 1024>;

fn main() {
    let inp = get_input_contents();

    let mut map = Map::new();
    for line in inp.lines() {
        let row: Vec<char, MAP_SIZE> = line.chars().collect();
        map.push(row).unwrap();
    }

    let start = find_on_map(&map, 'S').unwrap();
    let end = find_on_map(&map, 'E').unwrap();

    let (cost, optimal_tiles) = dijkstra(&map, start, end);
    println!("Cost: {cost}");
    println!("Optimal tiles: {optimal_tiles}");
}

fn get_input_contents() -> String {
    let args: Vec<String, 10> = env::args().collect();
    let filename = &args[1];
    fs::read_to_string(filename).expect("Failed to read file")
}

fn find_on_map(map: &Map, needle: char) -> Option<Coord> {
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
        self.cost.cmp(&other.cost)
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
    fn idx(&self) -> usize {
        match self {
            Self::N => 0,
            Self::E => 1,
            Self::S => 2,
            Self::W => 3,
        }
    }

    fn dxdy(&self) -> Coord {
        match self {
            Self::N => (0, -1),
            Self::E => (1, 0),
            Self::S => (0, 1),
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

fn dijkstra(map: &Map, start: Coord, end: Coord) -> (u64, u64) {
    let mut q: BinaryHeap<Node, Min, 512> = BinaryHeap::new();
    let mut processed = [[[false; 4]; MAP_SIZE]; MAP_SIZE];
    let mut distance: Distances = [[[u64::MAX; 4]; MAP_SIZE]; MAP_SIZE];
    let mut min_cost = u64::MAX;
    let mut final_dir: Dir = Dir::E;

    distance[start.1 as usize][start.0 as usize][Dir::E.idx()] = 0;
    q.push(Node {
        pos: start,
        dir: Dir::E,
        cost: 0,
    })
    .unwrap();

    while let Some(node) = q.pop() {
        // If we reached the end, update the min cost so we don't have to spend time on longer
        // paths anymore.
        if node.cost > min_cost {
            continue;
        }
        if node.pos == end {
            min_cost = node.cost;
            final_dir = node.dir;
            continue;
        }

        if processed[node.pos.1 as usize][node.pos.0 as usize][node.dir.idx()] {
            continue;
        }
        processed[node.pos.1 as usize][node.pos.0 as usize][node.dir.idx()] = true;

        // Find positions reachable from here
        for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
            let (dx, dy) = dir.dxdy();
            let nx = node.pos.0 + dx;
            let ny = node.pos.1 + dy;

            if nx < 0 || ny < 0 || nx >= MAP_SIZE as i16 || ny >= MAP_SIZE as i16 {
                continue;
            }
            if map[ny as usize][nx as usize] == '#' {
                continue;
            }

            let new_cost = node.cost + 1 + (1000 * node.dir.diff(&dir));

            if new_cost <= distance[ny as usize][nx as usize][dir.idx()] {
                distance[ny as usize][nx as usize][dir.idx()] = new_cost;

                let new_node = Node {
                    pos: (nx, ny),
                    dir,
                    cost: new_cost,
                };
                q.push(new_node).unwrap();
            }
        }
    }

    // Part 2
    let mut paths: PosSet = FnvIndexSet::new();
    let final_node = Node {
        pos: end,
        dir: final_dir,
        cost: min_cost,
    }

    walk_paths(final_node, &distance, &mut paths);

    (min_cost, paths.len() as u64)
}

fn walk_paths(node: &Node, distances: &Distances, seen: PosSet) {
    seen.insert(&pos).unwrap();

    let min_dist = u64::MAX;

    // Find neighbors
    for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
        let (dx, dy) = dir.dxdy();
        let nx = node.pos.0 + dx;
        let ny = node.pos.1 + dy;

        let dist = distances[ny as usize][nx as usize].iter().min();
    }
}
