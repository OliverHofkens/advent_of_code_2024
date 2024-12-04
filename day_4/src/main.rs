#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use heapless::{Deque, Vec};

// We only need 7 lines in memory at once to search for XMAS in all directions.
// The "middle" element is where we're currently looking for XMAS'es.
// `Option`s are used to signify the boundaries of the puzzle.
type BufView = Deque<Option<Vec<u8, 140>>, 7>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<140>::new();
    let mut eof: bool = false;

    // We only need 7 lines in memory at once to search for XMAS in all directions.
    // The "middle" element is where we're currently looking for XMAS'es.
    // `Option`s are used to signify the boundaries of the puzzle.
    let mut buf: BufView = Deque::new();

    // Prime the buffer so we always have a full view:
    for _ in 0..buf.capacity() {
        buf.push_back(None).unwrap();
    }

    let mut xmas_count: usize = 0;
    let mut x_mas_count: usize = 0;

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();

                // Shift one line into the buf
                buf.pop_front();
                buf.push_back(Some(Vec::from_slice(line).unwrap())).unwrap();

                // If we have text in our search line, we can start counting:
                if let Some(_) = buf.iter().nth(3).unwrap() {
                    let (p1_count, p2_count) = count_xmas_on_line(&buf);
                    xmas_count += p1_count;
                    x_mas_count += p2_count;
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading! {}", e),
        }

        // print_buf(&buf);
        // println!("---")
    }

    // When all lines are read, we still have to continue our push loop until
    // there's no more lines in focus:
    for _ in 0..3 {
        buf.pop_front();
        buf.push_back(None).unwrap();
        let (p1_count, p2_count) = count_xmas_on_line(&buf);
        xmas_count += p1_count;
        x_mas_count += p2_count;
        // print_buf(&buf);
        // println!("---")
    }

    println!("XMAS: {}", xmas_count);
    println!("X-MAS: {}", x_mas_count);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn print_buf(buf: &BufView) {
    for line in buf.iter() {
        let content = match line {
            None => "",
            Some(l) => core::str::from_utf8(&l).unwrap(),
        };
        println!("{content}");
    }
}

/// Counts the XMAS's and X-MAS's visible from the middle of the BufView.
fn count_xmas_on_line(buf: &BufView) -> (usize, usize) {
    let mut xmas_count: usize = 0;
    let mut x_mas_count: usize = 0;

    let middle = buf
        .iter()
        .nth(3)
        .unwrap()
        .as_ref()
        .expect("Middle of bufview is empty!");

    // for XMAS We always start our search on an X on our focused line:
    for (x, _) in middle.iter().enumerate().filter(|(_, c)| **c == 'X' as u8) {
        xmas_count += star_count_xmas(buf, x);
    }

    // for X-MAS We always start our search on an A on our focused line:
    for (x, _) in middle.iter().enumerate().filter(|(_, c)| **c == 'A' as u8) {
        x_mas_count += star_count_x_mas(buf, x);
    }

    (xmas_count, x_mas_count)
}

// Part 1
// During vertical and diagonal search, which letter do we expect in which y coordinate.
// During horizontal search, which letter do we expect in which x offset.
const EXP_CHAR: [u8; 7] = [
    'S' as u8, 'A' as u8, 'M' as u8, 'X' as u8, 'M' as u8, 'A' as u8, 'S' as u8,
];

/// Given the x coordinate of an 'X', count the amounts of "MAS" originating from there.
fn star_count_xmas(buf: &BufView, at_idx: usize) -> usize {
    // Directions we can still spot "XMAS" = 1.
    // 0 = diagonal top left, 8 = diagonal down right.
    let mut candidates: [usize; 8] = [1; 8];

    for (y_idx, line) in buf.iter().enumerate() {
        let dir_offset = if y_idx < 3 { 0 } else { 5 };

        if let None = line {
            candidates[0 + dir_offset] = 0;
            candidates[1 + dir_offset] = 0;
            candidates[2 + dir_offset] = 0;
            continue;
        }
        let text = line.as_ref().unwrap();

        if y_idx == 3 {
            // On the third line, we search left and right.
            for (x_offset, exp_char) in (-3..=3).zip(EXP_CHAR) {
                let x = at_idx as isize + x_offset;
                let dir: usize = if x_offset < 0 { 3 } else { 4 };

                if x < 0 || x as usize >= text.len() || text[x as usize] != exp_char {
                    candidates[dir] = 0;
                }
            }
        } else {
            // On other lines, we do the vertical search
            // X offset we expect to find relevant letters on this line.
            let x_offset: isize = (3isize - y_idx as isize).abs();
            let exp_char = EXP_CHAR[y_idx];

            for x_mul in [-1isize, 0, 1] {
                let dir = (x_mul + 1) as usize + dir_offset;
                let x = at_idx as isize + (x_offset * x_mul);

                if x < 0 || x as usize >= text.len() || text[x as usize] != exp_char {
                    candidates[dir] = 0;
                }
            }
        }
    }

    candidates.iter().sum()
}

// Part 2
/// Given the x coordinate of an 'A', count the amounts of "MAS" centered there.
fn star_count_x_mas(buf: &BufView, at_idx: usize) -> usize {
    0
}
