use core::str::FromStr;
use heapless::{FnvIndexMap, String, Vec};
use std::env;
use std::fs;

type Towel = String<8>;
type Towels = Vec<Towel, 500>;

const CACHE_KEY_SIZE: usize = 64;
type CacheKey = String<CACHE_KEY_SIZE>;
type Cache = FnvIndexMap<CacheKey, bool, 16384>;

fn main() {
    let mut read_idx = 0usize;
    let mut towels = Towels::new();
    let mut cache = Cache::new();
    let mut designs_made = 0usize;

    let inp = get_input_contents();

    for line in inp.lines() {
        if read_idx == 0 {
            towels = line
                .split(", ")
                .map(|s| Towel::from_str(s).unwrap())
                .collect();
        } else if read_idx > 1 {
            if try_make_design(line, &towels, &mut cache) {
                println!("Made design {}", line);
                designs_made += 1;
            } else {
                println!("Can't make {}", line);
            }
        }
        read_idx += 1;
    }

    println!("Designs made: {}", designs_made);
}

fn get_input_contents() -> std::string::String {
    let args: Vec<std::string::String, 10> = env::args().collect();
    let filename = &args[1];
    fs::read_to_string(filename).expect("Failed to read file")
}

fn try_make_design(design: &str, towels: &Towels, cache: &mut Cache) -> bool {
    // println!("Trying to make {design}");
    if design.len() == 0 {
        return true;
    }

    if design.len() <= CACHE_KEY_SIZE {
        if let Some(res) = cache.get(&CacheKey::from_str(design).unwrap()) {
            return *res;
        }
    }

    let mut res = false;

    for towel in towels {
        if design.starts_with(towel.as_str()) {
            let rest = &design[towel.len()..];
            if try_make_design(rest, towels, cache) {
                res = true;
                break;
            }
        }
    }

    if design.len() <= CACHE_KEY_SIZE {
        cache
            .insert(CacheKey::from_str(design).unwrap(), res)
            .unwrap();
    }

    res
}
