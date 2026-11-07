#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    i2s::master::{Channels, DataFormat, I2s, TdmConfig},
    main,
    time::Rate,
    usb::usb_serial_jtag::UsbSerialJtag,
};
use rust_s3::{
    display::OledDisplay,
    logging::recorder::UsbPcmLogger,
    sensors::microphone,
};

esp_bootloader_esp_idf::esp_app_desc!();

const RECORD_SECONDS: u32 = 20;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // OLED is initialized once. Avoid refreshing it during high-rate recording.
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO8)
    .with_scl(peripherals.GPIO9);
    let mut display = OledDisplay::new(i2c);
    display.show_recorder_status("USB LOGGER READY");

    // One microphone: SEL -> GND, left I2S slot only.
    let config = TdmConfig::new_tdm_philips()
        .with_sample_rate(Rate::from_hz(microphone::SAMPLE_RATE))
        .with_data_format(DataFormat::Data32Channel32)
        .with_channels(Channels::STEREO);

    let i2s = I2s::new(peripherals.I2S0, peripherals.DMA_CH0, config).unwrap();
    let i2s_rx = i2s
        .i2s_rx
        .with_bclk(peripherals.GPIO5)
        .with_ws(peripherals.GPIO6)
        .with_din(peripherals.GPIO7)
        .build();

    let usb = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let mut logger = UsbPcmLogger::new(usb);

    // Start only when Python has opened /dev/ttyACM* and sent 'S'.
    logger.wait_for_start();
    display.show_recorder_status("RECORDING 32K");

    let summary = microphone::capture(i2s_rx, RECORD_SECONDS, |mono_pcm, _rms, near_full| {
        logger.send_block(mono_pcm, near_full);
    });

    // No more raw binary packets will be transmitted.
    if summary.high_water_events > 0 {
        display.show_recorder_status("CHECK DMA BUFFER");
    } else {
        display.show_recorder_status("RECORD DONE");
    }

    // Press RESET to arm the next 20-second capture.
    loop {
        core::hint::spin_loop();
    }
}
