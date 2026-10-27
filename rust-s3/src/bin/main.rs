
#![no_std]
#![no_main]

use esp_backtrace as _;
use rust_s3::display::show_startup;

use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::Rate,
};
//APD for bootloader
esp_bootloader_esp_idf::esp_app_desc!();
#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(
        esp_hal::Config::default()
    );

    let mut i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default()
            .with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9);

    show_startup(&mut i2c);

    loop {
        core::hint::spin_loop();
    }
}