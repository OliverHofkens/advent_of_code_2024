use esp_idf_svc::sys::vTaskDelay;
use std::io::{self, BufRead};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let mut buffer = String::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    loop {
        match handle.read_line(&mut buffer) {
            Ok(_) => log::info!("{}", buffer),
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::Interrupted => {
                    //log::info!("Error: {e}\r\n");
                    unsafe { vTaskDelay(10) };
                    continue;
                }
                _ => {
                    log::info!("Error: {e}\r\n");
                    continue;
                }
            },
        }
    }
}
