use heapless::binary_heap::{BinaryHeap, Min};
use heapless::Vec;
use std::env;
use std::fs;

const MAP_SIZE: usize = 141;
type Map = Vec<Vec<char, MAP_SIZE>, MAP_SIZE>;
type Coord = (i16, i16);

fn main() {
    let inp = get_input_contents();

    let mut map = Map::new();
    for line in inp.lines() {
        let row: Vec<char, MAP_SIZE> = line.chars().collect();
        map.push(row).unwrap();
    }

    let start = find_on_map(&map, 'S').unwrap();
    let end = find_on_map(&map, 'E').unwrap();

    let (cost, tile_count) = dijkstra(&map, start, end);
    println!("Cost: {cost}");
    println!("Tiles on shortest paths: {tile_count}");
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

    fn from_idx(idx: usize) -> Self {
        match idx {
            0 => Dir::N,
            1 => Dir::E,
            2 => Dir::S,
            3 => Dir::W,
            _ => panic!("Invalid direction index"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct PrevNode {
    pos: Coord,
    dir: Dir,
}

// Define a default PrevNode for initialization
impl Default for PrevNode {
    fn default() -> Self {
        Self {
            pos: (0, 0),
            dir: Dir::N,
        }
    }
}

fn dijkstra(map: &Map, start: Coord, end: Coord) -> (u64, usize) {
    let mut q: BinaryHeap<Node, Min, 512> = BinaryHeap::new();
    let mut processed = [[[false; 4]; MAP_SIZE]; MAP_SIZE];
    let mut distance = [[[u64::MAX; 4]; MAP_SIZE]; MAP_SIZE];
    let mut prev = [[[[PrevNode::default(); 8]; 4]; MAP_SIZE]; MAP_SIZE];

    distance[start.1 as usize][start.0 as usize][Dir::E.idx()] = 0;
    q.push(Node {
        pos: start,
        dir: Dir::E,
        cost: 0,
    })
    .unwrap();

    let mut min_end_cost = u64::MAX;

    while let Some(node) = q.pop() {
        // Skip if we've found better paths
        if node.cost > min_end_cost {
            continue;
        }

        if processed[node.pos.1 as usize][node.pos.0 as usize][node.dir.idx()] {
            continue;
        }
        processed[node.pos.1 as usize][node.pos.0 as usize][node.dir.idx()] = true;

        // Update min_end_cost if we've reached the end
        if node.pos == end {
            min_end_cost = min_end_cost.min(node.cost);
        }

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

            let curr_dist = &mut distance[ny as usize][nx as usize][dir.idx()];
            if new_cost <= *curr_dist {
                let prev_arr = &mut prev[ny as usize][nx as usize][dir.idx()];

                if new_cost < *curr_dist {
                    // Clear previous nodes if we found a better path
                    *curr_dist = new_cost;
                    prev_arr[0] = PrevNode {
                        pos: node.pos,
                        dir: node.dir,
                    };
                    for i in 1..prev_arr.len() {
                        prev_arr[i] = PrevNode::default();
                    }
                } else {
                    // Add to previous nodes if cost is equal
                    if let Some(empty_slot) = prev_arr.iter_mut().find(|p| p.pos == (0, 0)) {
                        *empty_slot = PrevNode {
                            pos: node.pos,
                            dir: node.dir,
                        };
                    }
                }

                let new_node = Node {
                    pos: (nx, ny),
                    dir,
                    cost: new_cost,
                };
                q.push(new_node).unwrap();
            }
        }
    }

    // Count tiles that appear in any shortest path
    let mut visited = [[false; MAP_SIZE]; MAP_SIZE];
    for dir_idx in 0..4 {
        if distance[end.1 as usize][end.0 as usize][dir_idx] == min_end_cost {
            mark_path_tiles(&prev, end, Dir::from_idx(dir_idx), &mut visited);
        }
    }

    let tile_count = visited
        .iter()
        .map(|row| row.iter().filter(|&&v| v).count())
        .sum();

    (min_end_cost, tile_count)
}

fn mark_path_tiles(
    prev: &[[[[PrevNode; 8]; 4]; MAP_SIZE]; MAP_SIZE],
    pos: Coord,
    dir: Dir,
    visited: &mut [[bool; MAP_SIZE]; MAP_SIZE],
) {
    visited[pos.1 as usize][pos.0 as usize] = true;

    let prev_nodes = &prev[pos.1 as usize][pos.0 as usize][dir.idx()];
    for prev_node in prev_nodes {
        if prev_node.pos != (0, 0) {
            // Check if this is a valid previous node
            mark_path_tiles(prev, prev_node.pos, prev_node.dir, visited);
        }
    }
}
