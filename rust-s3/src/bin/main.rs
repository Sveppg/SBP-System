
#![no_std]
#![no_main]

use esp_backtrace as _;

use rust_s3::{
    display::OledDisplay,
    sensors::microphone,
};

use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    i2s::master::{
        Channels,
        DataFormat,
        I2s,
        TdmConfig,
    },
    main,
    time::Rate,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals =
        esp_hal::init(esp_hal::Config::default());

    // ===================================
    // OLED initialisieren
    // ===================================

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default()
            .with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9);

    let mut display = OledDisplay::new(i2c);

    display.show_startup();

    // ===================================
    // I2S Mikrofon initialisieren
    // ===================================

    let config = TdmConfig::new_tdm_msb()
        .with_sample_rate(Rate::from_hz(16_000))
        .with_data_format(DataFormat::Data32Channel32)
        .with_channels(Channels::STEREO);

    let i2s = I2s::new(
        peripherals.I2S0,
        peripherals.DMA_CH0,
        config,
    )
    .unwrap();

    let i2s_rx = i2s
        .i2s_rx
        .with_bclk(peripherals.GPIO5)
        .with_ws(peripherals.GPIO6)
        .with_din(peripherals.GPIO7)
        .build();

    // ===================================
    // Mikrofon starten + Display updaten
    // ===================================

    microphone::run(i2s_rx, |left, right| {
        display.show_rms(left, right);
    });
}
