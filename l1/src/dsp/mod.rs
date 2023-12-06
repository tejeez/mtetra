//! Signal processing

use num::Complex;

mod modem;
use modem::Modulator;
use crate::L1Callbacks;

pub struct L1Dsp {
    modulator: Modulator,
}

impl L1Dsp {
    pub fn new() -> Self {
        Self {
            modulator: Modulator::new(),
        }
    }

    pub fn process(
        &mut self,
        time: i64,
        buf: &mut [Complex<f32>],
        callbacks: &L1Callbacks,
    ) {
        let mut timenow = time;
        for sample in buf.iter_mut() {
            *sample = self.modulator.sample(timenow, callbacks);
            // Simulate a 100 kHz sample rate by incrementing timestamp
            timenow += 10000;
        }
    }
}
