
use esp_hal::{
    dma_rx_stream_buffer,
    i2s::master::I2sRx,
    Blocking,
};

// SPH0645: 16 kHz, 2 x 32 Bit pro Frame
const SAMPLE_RATE: usize = 16_000;
const CHUNK_SIZE: usize = 1024;
const BYTES_PER_FRAME: usize = 8;

// Ein 18-Bit-Sample ist im 32-Bit-Slot
// linksbündig angeordnet.
fn sample_to_float(raw: i32) -> f32 {
    let sample = raw >> 14;
    sample as f32 / 131_072.0
}
pub fn run<F>(i2s_rx: I2sRx<'_, Blocking>, mut on_rms: F,) -> !
where F: FnMut(f32, f32),{

    // 16 KiB DMA-Ringpuffer
    let rx_buffer =
        dma_rx_stream_buffer!(16_384, 1024);

    let mut transfer = i2s_rx
        .read(rx_buffer)
        .ok()
        .expect("I2S DMA konnte nicht starten");

    let mut data = [0u8; CHUNK_SIZE];

    let mut sum_left = 0.0f32;
    let mut sum_right = 0.0f32;
    let mut sample_count = 0usize;

    esp_println::println!("Mikrofon gestartet");

    loop {
        if transfer.available_bytes() < CHUNK_SIZE {
            core::hint::spin_loop();
            continue;
        }

        let bytes_read = transfer.pop(&mut data);

        if bytes_read != CHUNK_SIZE {
            panic!("Unvollstaendiger Audio-Block");
        }

        // 8 Byte = 32 Bit links + 32 Bit rechts
        for frame in data.chunks_exact(BYTES_PER_FRAME) {
            let left_raw = i32::from_le_bytes([
                frame[0], frame[1], frame[2], frame[3]
            ]);

            let right_raw = i32::from_le_bytes([
                frame[4], frame[5], frame[6], frame[7]
            ]);

            let left = sample_to_float(left_raw);
            let right = sample_to_float(right_raw);

            sum_left += left * left;
            sum_right += right * right;
            sample_count += 1;
        }
    if sample_count >= SAMPLE_RATE {
        let n = sample_count as f32;

        let rms_left = libm::sqrtf(sum_left / n);
        let rms_right = libm::sqrtf(sum_right / n);

        esp_println::println!(
            "RMS L: {:.6} | RMS R: {:.6}",
            rms_left,
            rms_right
        );

        // Messwerte an das Hauptprogramm senden
        on_rms(rms_left, rms_right);

        sum_left = 0.0;
        sum_right = 0.0;
        sample_count = 0;
    }
        // Einmal pro Sekunde RMS ausgeben

    }
}
