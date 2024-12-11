use aoc_common::num;
use heapless::{FnvIndexMap, Vec};
use std::env;
use std::fs;

type Cache = FnvIndexMap<(u64, u8), u64, 8192>;

fn main() {
    let mut cache = Cache::new();

    let inp = get_input_contents();
    println!("{}", inp);

    let count = inp
        .trim()
        .split(' ')
        .filter_map(|s| u64::from_str_radix(s, 10).ok())
        .fold(0u64, |acc, num| {
            acc.checked_add(simulate_blinks(num, 75, &mut cache))
                .unwrap()
        });

    println!("Count: {}", count);
}

fn get_input_contents() -> String {
    let args: Vec<String, 10> = env::args().collect();
    let filename = &args[1];
    fs::read_to_string(filename).expect("Failed to read file")
}

fn simulate_blinks(stone: u64, blinks: u8, cache: &mut Cache) -> u64 {
    if blinks == 0 {
        return 1;
    }

    if let Some(res) = cache.get(&(stone, blinks)) {
        return *res;
    }

    let next_blinks = blinks - 1;
    let n_digits = num::count_digits(stone);

    let res = if stone == 0 {
        simulate_blinks(1, next_blinks, cache)
    } else if n_digits % 2 == 0 {
        let (l, r) = num::split(stone, n_digits / 2);
        simulate_blinks(l, next_blinks, cache) + simulate_blinks(r, next_blinks, cache)
    } else {
        simulate_blinks(stone.checked_mul(2024).unwrap(), next_blinks, cache)
    };

    if cache.len() < cache.capacity() {
        cache.insert((stone, blinks), res).unwrap();
    }
    res
}
