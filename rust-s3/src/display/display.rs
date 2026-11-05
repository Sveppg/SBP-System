
use core::fmt::Write;

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

use heapless::String;

use ssd1306::{
    mode::BufferedGraphicsMode,
    prelude::*,
    I2CDisplayInterface,
    Ssd1306,
};

type OledDriver<'d> = Ssd1306<
    I2CInterface<I2c<'d, Blocking>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub struct OledDisplay<'d> {
    inner: OledDriver<'d>,
}

impl<'d> OledDisplay<'d> {
    pub fn new(i2c: I2c<'d, Blocking>) -> Self {
        let interface =
            I2CDisplayInterface::new_custom_address(
                i2c, 0x3C
            );

        let mut inner = Ssd1306::new(
            interface,
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();

        inner.init().unwrap();

        Self { inner }
    }

    fn draw_line(&mut self, text: &str, y: i32) {
        let style = MonoTextStyle::new(
            &FONT_6X10,
            BinaryColor::On,
        );

        Text::with_baseline(
            text,
            Point::new(0, y),
            style,
            Baseline::Top,
        )
        .draw(&mut self.inner)
        .unwrap();
    }

    pub fn show_startup(&mut self) {
        self.inner.clear_buffer();

        self.draw_line("Predictive Maint.", 0);
        self.draw_line("ESP32-S3 / Rust", 20);
        self.draw_line("MIC: STARTING", 40);

        self.inner.flush().unwrap();
    }

    pub fn show_rms(
        &mut self,
        rms_left: f32,
        rms_right: f32,
    ) {
        let mut left: String<32> = String::new();
        let mut right: String<32> = String::new();

        write!(&mut left, "RMS L: {:.5}", rms_left)
            .unwrap();

        write!(&mut right, "RMS R: {:.5}", rms_right)
            .unwrap();

        self.inner.clear_buffer();

        self.draw_line("Predictive Maint.", 0);
        self.draw_line("MIC: ACTIVE", 13);
        self.draw_line(left.as_str(), 27);
        self.draw_line(right.as_str(), 40);
        self.draw_line("SAMPLE: 16 kHz", 53);

        // I2C-Fehler nicht zur gesamten
        // Firmware-Panic eskalieren lassen.
        if self.inner.flush().is_err() {
            esp_println::println!(
                "OLED: Update fehlgeschlagen"
            );
        }
    }
}
