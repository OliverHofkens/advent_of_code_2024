#![no_std]
#![no_main]
use embedded_io::Read;
#[deny(clippy::mem_forget)] // core::mem::forget is dangerous on ESP32
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{delay::Delay, prelude::*};

const INPUT_BUF_SIZE_B: usize = 22 * 1000;

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    esp_println::logger::init_logger_from_env();
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut input_buf: [u8; INPUT_BUF_SIZE_B] = [0; INPUT_BUF_SIZE_B];

    loop {
        delay.delay(500.millis());
        let read = usb_serial.read(&mut input_buf).expect("read error!");
        defmt::println!("Read {} bytes", read);
    }
}
