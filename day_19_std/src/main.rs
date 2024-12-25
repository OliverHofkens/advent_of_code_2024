use core::str::FromStr;
use heapless::{FnvIndexMap, String, Vec};
use std::env;
use std::fs;

type Towel = String<8>;
type Towels = Vec<Towel, 500>;

const CACHE_KEY_SIZE: usize = 64;
type CacheKey = String<CACHE_KEY_SIZE>;
type Cache = FnvIndexMap<CacheKey, u64, 32768>;

fn main() {
    let mut read_idx = 0usize;
    let mut towels = Towels::new();
    let mut cache = Cache::new();
    let mut designs_made = 0u64;
    let mut arrangements = 0u64;

    let inp = get_input_contents();

    for line in inp.lines() {
        if read_idx == 0 {
            towels = line
                .split(", ")
                .map(|s| Towel::from_str(s).unwrap())
                .collect();
        } else if read_idx > 1 {
            let ways = try_make_design(line, &towels, &mut cache);
            if ways > 0 {
                designs_made += 1;
                arrangements += ways;
            }
        }
        read_idx += 1;
    }

    println!("Designs made: {}", designs_made);
    println!("Arrangements: {}", arrangements);
}

fn get_input_contents() -> std::string::String {
    let args: Vec<std::string::String, 10> = env::args().collect();
    let filename = &args[1];
    fs::read_to_string(filename).expect("Failed to read file")
}

fn try_make_design(design: &str, towels: &Towels, cache: &mut Cache) -> u64 {
    if design.len() == 0 {
        return 1;
    }

    if design.len() <= CACHE_KEY_SIZE {
        if let Some(res) = cache.get(&CacheKey::from_str(design).unwrap()) {
            return *res;
        }
    }

    let mut res = 0u64;

    for towel in towels {
        if design.starts_with(towel.as_str()) {
            let rest = &design[towel.len()..];
            res += try_make_design(rest, towels, cache);
        }
    }

    if design.len() <= CACHE_KEY_SIZE {
        cache
            .insert(CacheKey::from_str(design).unwrap(), res)
            .unwrap();
    }

    res
}
