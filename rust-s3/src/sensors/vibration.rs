
use esp_hal::{
    analog::adc::{
        Adc,
        AdcConfig,
        AdcPin,
        Attenuation,
    },
    peripherals::{ADC1, GPIO4},
    Blocking,
};

const SAMPLES: usize = 128;

pub struct VibrationReading {
    pub mean_adc: f32,
    pub rms_ac: f32,
    pub peak_to_peak: u16,
}

pub struct VibrationSensor<'d> {
    adc: Adc<'d, ADC1<'d>, Blocking>,
    pin: AdcPin<GPIO4<'d>, ADC1<'d>>,
}

impl<'d> VibrationSensor<'d> {
    pub fn new(
        adc_peripheral: ADC1<'d>,
        gpio: GPIO4<'d>,
    ) -> Self {
        // ADC1 auf GPIO4 konfigurieren
        let mut config = AdcConfig::new();

        let pin = config.enable_pin(
            gpio,
            Attenuation::_11dB,
        );

        let adc = Adc::new(adc_peripheral, config);

        Self { adc, pin }
    }

    pub fn measure_burst(&mut self) -> VibrationReading {
        let mut mean = 0.0f32;
        let mut m2 = 0.0f32;

        let mut min = u16::MAX;
        let mut max = u16::MIN;

        for i in 0..SAMPLES {
            let raw = self.adc.read_blocking(
                &mut self.pin
            );

            min = min.min(raw);
            max = max.max(raw);

            // Laufende Mittelwert- und
            // Varianzberechnung (Welford)
            let x = raw as f32;
            let n = (i + 1) as f32;

            let delta = x - mean;
            mean += delta / n;

            let delta2 = x - mean;
            m2 += delta * delta2;
        }

        let variance =
            (m2 / SAMPLES as f32).max(0.0);

        VibrationReading {
            mean_adc: mean,
            rms_ac: libm::sqrtf(variance),
            peak_to_peak: max - min,
        }
    }
}
