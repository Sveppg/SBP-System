//! SBP1 USB stream: 20-byte header + 512 little-endian i32 samples.
//!   header = magic[4] + seq:u32 + sample_rate:u32 + count:u16
//!          + flags:u16 + fnv1a32(payload):u32
//! Bit 0 of flags: DMA ring was close to full (warning, not proof of loss).

use esp_hal::{usb::usb_serial_jtag::UsbSerialJtag, Blocking};
use crate::sensors::microphone::{SAMPLE_RATE, SAMPLES_PER_BLOCK};

pub struct UsbPcmLogger<'d> {
    usb: UsbSerialJtag<'d, Blocking>,
    sequence: u32,
}

impl<'d> UsbPcmLogger<'d> {
    pub fn new(usb: UsbSerialJtag<'d, Blocking>) -> Self {
        Self { usb, sequence: 0 }
    }

    /// Host must open the serial port and send ASCII 'S' before DMA starts.
    pub fn wait_for_start(&mut self) {
        loop {
            match self.usb.read_byte() {
                Ok(b'S') => {
                    // Startkommando empfangen
                    break;
                }

                Ok(_) => {
                    // Andere Zeichen ignorieren
                }

                Err(_) => {
                    // Noch keine Daten vorhanden
                    core::hint::spin_loop();
                }
            }
        }
    }

    pub fn send_block(&mut self, pcm32_le: &[u8], dma_near_full: bool) {
        assert_eq!(pcm32_le.len(), SAMPLES_PER_BLOCK * 4);

        let mut header = [0u8; 20];
        header[0..4].copy_from_slice(b"SBP1");
        header[4..8].copy_from_slice(&self.sequence.to_le_bytes());
        header[8..12].copy_from_slice(&SAMPLE_RATE.to_le_bytes());
        header[12..14].copy_from_slice(&(SAMPLES_PER_BLOCK as u16).to_le_bytes());
        let flags: u16 = if dma_near_full { 1 } else { 0 };
        header[14..16].copy_from_slice(&flags.to_le_bytes());
        header[16..20].copy_from_slice(&fnv1a32(pcm32_le).to_le_bytes());

        // Write without using esp_println on this port. These functions may block
        // until the PC reads, so DMA high-water flags must be monitored.
        self.usb.write(&header).unwrap();
        self.usb.write(pcm32_le).unwrap();
        self.sequence = self.sequence.wrapping_add(1);
    }
}

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811C9DC5;
    for &byte in bytes {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}
