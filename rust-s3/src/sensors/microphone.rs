//! SPH0645: streaming capture of LEFT I2S slot, 32 kHz, 32-bit slots.
//! Tested against the documented esp-hal 1.2.x API, NOT yet hardware tested.

use esp_hal::{
    dma_rx_stream_buffer,
    i2s::master::I2sRx,
    Blocking,
};

pub const SAMPLE_RATE: u32 = 32_000;
pub const SAMPLES_PER_BLOCK: usize = 512;

const BYTES_PER_STEREO_FRAME: usize = 8;
const DMA_BYTES: usize = 32_768;
const RX_BLOCK_BYTES: usize = SAMPLES_PER_BLOCK * BYTES_PER_STEREO_FRAME;
const MONO_BLOCK_BYTES: usize = SAMPLES_PER_BLOCK * 4;

#[derive(Clone, Copy)]
pub struct CaptureSummary {
    pub blocks: u32,
    pub last_rms: f32,
    pub high_water_events: u32,
}

/// Runs the configured mono microphone for `seconds` and calls back once per
/// 512 samples, with raw little-endian signed 32-bit samples and the AC RMS.

/// IMPORTANT: `near_full` is only a warning that the DMA ring is filling up;
/// it does NOT prove whether DMA samples have been dropped.

pub fn capture<F>(i2s_rx: I2sRx<'_, Blocking>, seconds: u32, mut on_block: F) -> CaptureSummary
where
    F: FnMut(&[u8], f32, bool),
{
    let rx_buffer = dma_rx_stream_buffer!(DMA_BYTES, 1024);
    let mut transfer = i2s_rx
        .read(rx_buffer)
        .ok()
        .expect("Could not start I2S DMA RX");

    let mut stereo = [0u8; RX_BLOCK_BYTES];
    let mut mono = [0u8; MONO_BLOCK_BYTES];


    // 32_000 * 20 is exactly divisible by 512, unlike 1 second alone.
    let total_samples = (SAMPLE_RATE as u64) * (seconds as u64);
    assert_eq!(total_samples % SAMPLES_PER_BLOCK as u64, 0);
    let target_blocks = (total_samples / SAMPLES_PER_BLOCK as u64) as u32;

    let mut high_water_events = 0;
    let mut last_rms = 0.0;

    for _ in 0..target_blocks {
        let available = loop {
            let available = transfer.available_bytes();
            if available >= RX_BLOCK_BYTES {
                break available;
            }
            core::hint::spin_loop();
        };

        let near_full = available >= DMA_BYTES - 2 * RX_BLOCK_BYTES;
        if near_full {
            high_water_events += 1;
        }

        let got = transfer.pop(&mut stereo);
        assert_eq!(got, RX_BLOCK_BYTES, "I2S DMA short read");

        let mut sum = 0.0f32;
        let mut sum_sq = 0.0f32;

        for (frame, output) in stereo
            .chunks_exact(BYTES_PER_STEREO_FRAME)
            .zip(mono.chunks_exact_mut(4))
        {
            // SEL=GND: microphone drives the left slot; the right slot is ignored.
            output.copy_from_slice(&frame[0..4]);
            let raw = i32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]);
            let x = raw as f32 / 2_147_483_648.0f32;
            sum += x;
            sum_sq += x * x;
        }

        let n = SAMPLES_PER_BLOCK as f32;
        let mean = sum / n;
        let variance = (sum_sq / n - mean * mean).max(0.0);
        last_rms = libm::sqrtf(variance);
        on_block(&mono, last_rms, near_full);
    }

    // Dropping the DMA transfer stops the recording.
    CaptureSummary {
        blocks: target_blocks,
        last_rms,
        high_water_events,
    }
}
