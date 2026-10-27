
use esp_hal::{
    i2c::master::I2c,
    Blocking,
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

pub fn show_startup(i2c: &mut I2c<'_, Blocking>) {
    let interface =
        I2CDisplayInterface::new_custom_address(
            i2c, 0x3C // try 0x3D if doesnt work for esp32-s3 
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
        "Predictive Maint.",
        Point::new(0, 0),
        style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    Text::with_baseline(
        "ESP32-S3 / Rust",
        Point::new(0, 20),
        style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    Text::with_baseline(
        "Status: DISPLAY ON",
        Point::new(0, 40),
        style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    display.flush().unwrap();
}
