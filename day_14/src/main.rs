#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::{print, println};
use heapless::Vec;

type Bots = Vec<Bot, 500>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<32>::new();
    let mut eof: bool = false;

    let mut bots = Bots::new();

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();

                let nums: Vec<i16, 4> = line
                    .split(&[' ', ',', '='])
                    .filter_map(|s| i16::from_str_radix(s, 10).ok())
                    .collect();

                bots.push(Bot {
                    x: nums[0],
                    y: nums[1],
                    vx: nums[2],
                    vy: nums[3],
                })
                .unwrap();
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    const WIDTH: i16 = 101;
    const HEIGHT: i16 = 103;

    // Part 1
    // for _ in 0..100 {
    //     step(&mut bots, WIDTH, HEIGHT);
    // }
    // println!("Bots: {:?}", bots);
    //
    // let safety = safety_factor(&bots, WIDTH, HEIGHT);
    // println!("Safety: {safety}");

    // Part 2
    // The more uniform (noisy) the distribution, the higher the safety score.
    // So only print the image if it's a lower score than we've seen so far.
    let mut lowest_seen: u64 = u64::MAX;
    let mut pic = [[' '; WIDTH as usize]; HEIGHT as usize];
    for i in 0..10_000 {
        println!("Iter {i}");
        step(&mut bots, WIDTH, HEIGHT);

        let safety = safety_factor(&bots, WIDTH, HEIGHT);

        if safety < lowest_seen {
            lowest_seen = safety;
            pic = [[' '; WIDTH as usize]; HEIGHT as usize];

            for bot in &bots {
                pic[bot.y as usize][bot.x as usize] = '█'
            }

            println!("{i} SEC");
            for line in pic {
                for c in line {
                    print!("{c}");
                }
                print!("\n");
            }
            println!("---");

            delay.delay(1.millis());
        }
    }

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

#[derive(Debug)]
struct Bot {
    x: i16,
    y: i16,
    vx: i16,
    vy: i16,
}

fn step(bots: &mut Bots, width: i16, height: i16) {
    for bot in bots {
        bot.x += bot.vx;
        bot.y += bot.vy;

        if bot.x < 0 {
            bot.x += width;
        } else if bot.x >= width {
            bot.x -= width;
        }

        if bot.y < 0 {
            bot.y += height;
        } else if bot.y >= height {
            bot.y -= height;
        }
    }
}

fn safety_factor(bots: &Bots, width: i16, height: i16) -> u64 {
    let mut quad_counts = [0u64; 4];

    let q1x = width / 2;
    let q2x = (width + 1) / 2;

    let q1y = height / 2;
    let q2y = (height + 1) / 2;

    // println!("Q1 [0 -> {q1x}, 0 -> {q1y}]");
    // println!("Q2 [{q2x} -> ., 0 -> {q1y}]");
    // println!("Q3 [0 -> {q1x}, {q2y} -> .]");
    // println!("Q4 [{q2x} -> ., {q2y} -> .]");

    for bot in bots {
        if bot.x < q1x && bot.y < q1y {
            quad_counts[0] += 1;
        } else if bot.x >= q2x && bot.y < q1y {
            quad_counts[1] += 1;
        } else if bot.x < q1x && bot.y >= q2y {
            quad_counts[2] += 1;
        } else if bot.x >= q2x && bot.y >= q2y {
            quad_counts[3] += 1;
        }
    }

    // println!("Quads: {:?}", quad_counts);

    quad_counts.iter().product()
}
