#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{self};
use nom::combinator::{iterator, map, value};
use nom::multi::fold_many1;
use nom::sequence::{delimited, preceded, separated_pair};
use nom::IResult;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<4096>::new();
    let mut eof: bool = false;

    let mut machine = Machine::new();

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = reader.line();

                let parser = alt((instr_do, instr_dont, instr_mul, garbage));

                for instr in &mut iterator(line, parser) {
                    machine.exec(instr);
                }
            }
            Ok(false) => eof = true,
            Err(e) => println!("Error reading! {}", e),
        }
    }

    println!("Sum: {}", machine.sum);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

#[derive(Copy, Clone)]
enum Instr {
    Noop,
    Mul(i32, i32),
    Do,
    Dont,
}

fn instr_mul(inp: &[u8]) -> IResult<&[u8], Instr> {
    let args = separated_pair(complete::i32, tag(","), complete::i32);
    let call = delimited(tag("("), args, tag(")"));
    let mul = preceded(tag("mul"), call);
    map(mul, |(a, b)| Instr::Mul(a, b))(inp)
}

fn instr_do(inp: &[u8]) -> IResult<&[u8], Instr> {
    value(Instr::Do, tag("do()"))(inp)
}

fn instr_dont(inp: &[u8]) -> IResult<&[u8], Instr> {
    value(Instr::Dont, tag("don't()"))(inp)
}

fn garbage(inp: &[u8]) -> IResult<&[u8], Instr> {
    let (remaining, _discarded) = take(1usize)(inp)?;
    Ok((remaining, Instr::Noop))
}

#[derive(Copy, Clone)]
struct Machine {
    mul_enabled: bool,
    sum: i32,
}

impl Machine {
    fn new() -> Self {
        Machine {
            mul_enabled: true,
            sum: 0,
        }
    }

    fn exec(&mut self, instr: Instr) {
        match instr {
            Instr::Mul(a, b) if self.mul_enabled => self.sum += a * b,
            Instr::Do => self.mul_enabled = true,
            Instr::Dont => self.mul_enabled = false,
            _ => (),
        }
    }
}
