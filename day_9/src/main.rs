#![deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
#![no_std]
#![no_main]
use embedded_io::Read;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::{print, println};
use heapless::Deque;

#[derive(Debug, Clone)]
struct Block {
    id: i16,
    size: u8,
}

type Disk = Deque<Block, 20_000>;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut byte_buf = [0u8; 1];

    let mut idx: usize = 0;
    let mut disk: Disk = Disk::new();

    loop {
        match usb_serial.read(&mut byte_buf) {
            Ok(0) => break,
            Ok(_) => {
                let byte = byte_buf[0];
                if byte == b'\x04' || byte == b'\n' {
                    break;
                }

                let size = (byte as char).to_digit(10).unwrap() as u8;
                match idx % 2 {
                    0 => disk.push_back(Block {
                        id: (idx / 2) as i16,
                        size,
                    }),
                    _ => disk.push_back(Block { id: -1, size }),
                }
                .unwrap();
                idx += 1;
            }
            Err(e) => println!("Err {}", e),
        }
    }

    // Part 1:
    // let checksum = compact(&mut disk);
    // println!("Checksum: {}", checksum);

    // Part 2:
    let checksum = defrag(&mut disk);
    println!("Checksum: {}", checksum);

    println!("<EOT>");
    loop {
        delay.delay(100.millis());
    }
}

fn compact(disk: &mut Disk) -> u64 {
    let mut sum: u64 = 0;
    let mut idx: usize = 0;

    while let Some(mut blk) = disk.pop_front() {
        if blk.id >= 0 {
            // File, so checksum
            for pos in idx..idx + blk.size as usize {
                sum += pos as u64 * blk.id as u64;
                print!("{}", blk.id);
            }
            idx += blk.size as usize;
        } else {
            // Free space, so fill up from back.
            loop {
                match disk.pop_back() {
                    None => break,
                    Some(b) if b.id < 0 => continue,
                    Some(mut file) => {
                        if file.size < blk.size {
                            blk.size -= file.size;
                            disk.push_front(blk).unwrap();
                            disk.push_front(file).unwrap();
                        } else if file.size == blk.size {
                            disk.push_front(file).unwrap();
                        } else {
                            disk.push_front(Block {
                                id: file.id,
                                size: blk.size,
                            })
                            .unwrap();
                            file.size -= blk.size;
                            disk.push_back(file).unwrap();
                        }
                        break;
                    }
                }
            }
        }
    }
    print!("\n");
    sum
}

fn defrag(disk: &mut Disk) -> u64 {
    let mut sum: u64 = 0;
    let mut idx: usize = 0;

    while let Some(mut blk) = disk.pop_front() {
        if blk.id >= 0 {
            // File, so checksum
            for pos in idx..idx + blk.size as usize {
                sum += pos as u64 * blk.id as u64;
            }
            idx += blk.size as usize;
        } else {
            // Free space, so fill up from back, with last file that fits.
            let mut new_front: Option<Block> = None;

            match disk
                .iter_mut()
                .rev()
                .find(|b| b.id >= 0 && b.size > 0 && b.size <= blk.size)
            {
                Some(file) => {
                    blk.size -= file.size;
                    new_front = Some(file.clone());
                    // Turn the file into free space
                    file.id = -1;
                }
                None => {
                    // No file fits, so skip indexes for calculation
                    idx += blk.size as usize;
                    continue;
                }
            }

            // If we have free space remaining, re-add it:
            if blk.size > 0 {
                disk.push_front(blk);
            }

            if let Some(file) = new_front {
                disk.push_front(file).unwrap();
            }
        }
        // dump_disk(&disk);
    }
    sum
}

fn dump_disk(disk: &Disk) {
    for blk in disk.iter() {
        match blk.id {
            x if x < 0 => {
                for _ in 0..blk.size {
                    print!(".");
                }
            }
            x => {
                for _ in 0..blk.size {
                    print!("{x}");
                }
            }
        }
    }
    print!("\n")
}
