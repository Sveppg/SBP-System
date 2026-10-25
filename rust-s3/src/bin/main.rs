
#![no_std]
#![no_main]

use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::Rate,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use ssd1306::{
    prelude::*,
    I2CDisplayInterface,
    Ssd1306,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    esp_hal::system::software_reset()
}

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(
        esp_hal::Config::default()
    );

    // I2C mit 100 kHz init
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default()
            .with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9);

    // SSD1306 at0x3C if its not working try 0x3D 
    let interface =
        I2CDisplayInterface::new_custom_address(
            i2c, 0x3C
        );

    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    let style = MonoTextStyle::new(
        &FONT_6X10,
        BinaryColor::On,
    );

    Text::with_baseline(
        "Predictive Maint:",
        Point::new(0, 0),
        style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    Text::with_baseline(
        "ESP32-S3 / Rust:",
        Point::new(0, 20),
        style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    Text::with_baseline(
        "Status: READY->",
        Point::new(0, 40),
        style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    display.flush().unwrap();

    loop {
        core::hint::spin_loop();
    }
}
