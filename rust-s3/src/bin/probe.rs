
#![no_std]
#![no_main]

use esp_backtrace as _;

use esp_hal::{
    main,
    usb::usb_serial_jtag::UsbSerialJtag,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(
        esp_hal::Config::default()
    );

    let mut usb = UsbSerialJtag::new(
        peripherals.USB_DEVICE
    );

    loop {
        match usb.read_byte() {
            Ok(b'S') => {
                usb.write(b"SBP-USB-OK\n").unwrap();
                usb.flush_tx().unwrap();
            }

            Ok(_) => {}

            Err(_) => {
                core::hint::spin_loop();
            }
        }
    }
}
//cargo run --bin usb_probe
