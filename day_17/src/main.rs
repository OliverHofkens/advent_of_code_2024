#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use aoc_common::io;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::{print, println};
use heapless::Vec;

type Program = Vec<u8, 16>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut reader = io::LineReader::<64>::new();
    let mut eof: bool = false;

    let mut read_idx: usize = 0;
    let mut reg = [0u64; 3];
    let mut program = Program::new();

    while !eof {
        delay.delay(1.millis());
        reader.clear();

        match reader.read_until_newline(&mut usb_serial) {
            Ok(true) => {
                let line = core::str::from_utf8(reader.line()).unwrap();

                if read_idx < 3 {
                    let val = line
                        .split(":")
                        .skip(1)
                        .map(|s| u64::from_str_radix(s.trim(), 10).unwrap())
                        .next()
                        .unwrap();
                    reg[read_idx] = val;
                } else if read_idx == 4 {
                    let prog_str = line.split(":").skip(1).next().unwrap();
                    program = prog_str
                        .split(",")
                        .map(|s| u8::from_str_radix(s.trim(), 10).unwrap())
                        .collect();
                }

                read_idx += 1;
            }
            Ok(false) => eof = true,
            Err(e) => println!("Err reading! {}", e),
        }
    }

    let mut vm = VM::new(reg[0], reg[1], reg[2]);
    vm.run_program(&program);

    print!("Part 1: ");
    for o in &vm.out {
        print!("{o},");
    }
    print!("\n");

    println!("Part 2:");

    // Observation 1: The amount of digits in our output depends on the size of A.
    // A divides by 8 each loop, so to get x digits we need at least 8^x as input.
    // Observation 2: The output behaves like an octal number. The first digit changes every loop, the second
    // digit changes every 8 loops, the third digit every 64 loops, etc.
    let mut digit_idx = program.len() as u32 - 1;
    let mut bases = [0u64; 16];

    let res = reverse_engineer(&program, &mut bases, digit_idx).unwrap();
    println!("Res: {res}");

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn reverse_engineer(program: &Program, bases: &mut [u64; 16], digit_idx: u32) -> Option<u64> {
    for digit in 0..8 {
        let mut a = 0;
        bases[digit_idx as usize] = digit;
        for (pow, base) in bases.iter().enumerate() {
            a += base * 8u64.pow(pow as u32);
        }

        let mut vm = VM::new(a, 0, 0);
        vm.run_program(&program);

        // println!("A {a}: {:?}", &vm.out);
        if &vm.out == program {
            return Some(a);
        }

        if vm.out.len() >= digit_idx as usize
            && vm.out[digit_idx as usize] == program[digit_idx as usize]
        {
            // Found a digit! Check if we can make it from here by checking earlier digits
            // recursively:
            println!("Locked in digit {}", digit_idx);
            println!("A = {a} = {:?}", &vm.out);

            if let Some(a) = reverse_engineer(program, bases, digit_idx - 1) {
                return Some(a);
            }
        }
    }
    None
}

struct VM {
    instr_ptr: u8,
    reg_a: u64,
    reg_b: u64,
    reg_c: u64,
    out: Program,
}

impl VM {
    fn new(reg_a: u64, reg_b: u64, reg_c: u64) -> Self {
        VM {
            instr_ptr: 0,
            reg_a,
            reg_b,
            reg_c,
            out: Vec::new(),
        }
    }

    fn run_program(&mut self, program: &Program) {
        while self.instr_ptr < program.len() as u8 {
            let opcode = program[self.instr_ptr as usize];
            let operand = program[(self.instr_ptr + 1) as usize];
            self.op(opcode, operand);
        }
    }

    fn op(&mut self, opcode: u8, operand: u8) {
        match opcode {
            0 => self.adv(operand),
            1 => self.bxl(operand),
            2 => self.bst(operand),
            3 => self.jnz(operand),
            4 => self.bxc(operand),
            5 => self.out(operand),
            6 => self.bdv(operand),
            7 => self.cdv(operand),
            x => panic!("INVALID OP {x}"),
        };
    }

    fn combo(&self, operand: u8) -> u64 {
        match operand {
            l @ (0 | 1 | 2 | 3) => l as u64,
            4 => self.reg_a,
            5 => self.reg_b,
            6 => self.reg_c,
            x => panic!("INVALID COMBO {x}"),
        }
    }

    fn adv(&mut self, operand: u8) {
        let denom = 2u64.pow(self.combo(operand) as u32);
        let res = self.reg_a / denom;
        self.reg_a = res;
        self.instr_ptr += 2;
    }

    fn bxl(&mut self, operand: u8) {
        let other = operand as u64;
        let res = self.reg_b ^ other;
        self.reg_b = res;
        self.instr_ptr += 2;
    }

    fn bst(&mut self, operand: u8) {
        let res = self.combo(operand) % 8;
        self.reg_b = res;
        self.instr_ptr += 2;
    }

    fn jnz(&mut self, operand: u8) {
        if self.reg_a != 0 {
            self.instr_ptr = operand;
        } else {
            self.instr_ptr += 2;
        }
    }

    fn bxc(&mut self, _operand: u8) {
        let res = self.reg_b ^ self.reg_c;
        self.reg_b = res;
        self.instr_ptr += 2;
    }

    fn out(&mut self, operand: u8) {
        let res = self.combo(operand) % 8;
        self.out.push(res as u8).unwrap();
        self.instr_ptr += 2;
    }

    fn bdv(&mut self, operand: u8) {
        let denom = 2u64.pow(self.combo(operand) as u32);
        let res = self.reg_a / denom;
        self.reg_b = res;
        self.instr_ptr += 2;
    }

    fn cdv(&mut self, operand: u8) {
        let denom = 2u64.pow(self.combo(operand) as u32);
        let res = self.reg_a / denom;
        self.reg_c = res;
        self.instr_ptr += 2;
    }
}
