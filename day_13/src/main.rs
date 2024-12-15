#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use core::u64;

use aoc_common::{io, solver};
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;

const P2_OFFSET: u64 = 10_000_000_000_000;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<140>::new();
    let mut eof: bool = false;

    let mut read_idx: u8 = 0;
    let mut current_machine = ClawMachine::default();
    let mut tokens_p1: u64 = 0;
    let mut tokens_p2: u64 = 0;

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();

                match read_idx % 4 {
                    0 => {
                        let (a_x, a_y) = get_coords(line, b'+');
                        current_machine = ClawMachine {
                            a_x,
                            a_y,
                            ..Default::default()
                        }
                    }
                    1 => {
                        let (b_x, b_y) = get_coords(line, b'+');
                        current_machine.b_x = b_x;
                        current_machine.b_y = b_y;
                    }
                    2 => {
                        let (p_x, p_y) = get_coords(line, b'=');
                        current_machine.p_x = p_x;
                        current_machine.p_y = p_y;
                        tokens_p1 += solve(&current_machine, 100).unwrap_or(0);

                        current_machine.p_x += P2_OFFSET;
                        current_machine.p_y += P2_OFFSET;
                        tokens_p2 += solve(&current_machine, i64::MAX).unwrap_or(0);
                    }
                    3 => (),
                    _ => unreachable!(),
                }

                read_idx += 1;
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    println!("Tokens P1: {tokens_p1}");
    println!("Tokens P2: {tokens_p2}");

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

#[derive(Default, Debug)]
struct ClawMachine {
    a_x: u64,
    a_y: u64,
    b_x: u64,
    b_y: u64,
    p_x: u64,
    p_y: u64,
}

fn get_coords(line: &[u8], prefix: u8) -> (u64, u64) {
    let mut parts = line
        .split(|b| b.is_ascii_whitespace() || *b == b',' || *b == prefix)
        .filter_map(|b| {
            core::str::from_utf8(b)
                .ok()
                .and_then(|s| u64::from_str_radix(s, 10).ok())
        });

    (parts.next().unwrap(), parts.next().unwrap())
}

fn solve(machine: &ClawMachine, limit: i64) -> Option<u64> {
    let (na, nb) = solver::solve_2x2_system(
        machine.a_x.try_into().unwrap(),
        machine.b_x.try_into().unwrap(),
        machine.p_x.try_into().unwrap(),
        machine.a_y.try_into().unwrap(),
        machine.b_y.try_into().unwrap(),
        machine.p_y.try_into().unwrap(),
    )?;

    if na < 0 || nb < 0 || na > limit || nb > limit {
        None
    } else {
        Some((na * 3 + nb).try_into().unwrap())
    }
}
