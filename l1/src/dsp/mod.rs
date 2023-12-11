//! Signal processing

use num::Complex;

use crate::L1Callbacks;

mod modem;
use modem::Modulator;

pub mod cic;
mod fir;

type RxDdc = cic::CicDdc<4>;
type TxDuc = cic::CicDuc<4>;

/// Combined pulse shaping and CIC compensation filter
/// for a rate of 4 samples per symbol.
/// Coefficients from design_channel_filter.py
const CHANNEL_FILTER_TAPS: [f32; 16] = [
    0.27991672,
    0.20776464,
    0.09827440,
    0.00030770,
   -0.05085948,
   -0.05025657,
   -0.01979160,
    0.01037267,
    0.02136001,
    0.01339594,
   -0.00062394,
   -0.00812343,
   -0.00578368,
    0.00089343,
    0.00460255,
    0.00273298
];

/// Common data used for all RX and TX carriers
struct DspCommon {
    // SDR I/Q sample rate (Hz)
    radio_fs: f64,
    // Channel spacing for carriers (Hz)
    channel_spacing: f64,
    // CIC decimation and interpolation factor
    cic_factor: usize,
    // Sine table for DDC/DUC
    sine_table: cic::SineTableType,
    // Channel filter taps
    filter_taps: fir::SymmetricRealTaps,
}

struct TxCarrier {
    duc: TxDuc,
    filter: fir::FirCf32Sym,
    modulator: Modulator,
}

impl TxCarrier {
    pub fn new(
        common: &DspCommon,
        carrier_freq: f64,
    ) -> Self {
        Self {
            duc: TxDuc::new(common.sine_table.clone(), (carrier_freq / common.channel_spacing).round() as isize),
            filter: fir::FirCf32Sym::new(common.filter_taps.clone()),
            modulator: Modulator::new(),
        }
    }

    /// Produce one CIC processing block of samples and add them to buffer.
    /// Buffer length shall be equal to CIC interpolation ratio.
    pub fn process(
        &mut self,
        time: i64,
        buf: &mut [cic::BufferType],
        callbacks: &L1Callbacks,
    ) {
        let mut modulated = self.modulator.sample(time, callbacks);
        modulated = self.filter.sample(modulated);
        // TODO: proper scaling of CIC input
        modulated *= 1000.0;
        self.duc.process(
            cic::BufferType {
                re: modulated.re as i64,
                im: modulated.im as i64, },
            buf);
    }
}

pub struct L1Dsp {
    common: DspCommon,
    tx_carriers: Vec<TxCarrier>,
}

impl L1Dsp {
    pub fn new() -> Self {
        let radio_fs = 1.8e6;
        let channel_spacing: f64 = 12500.0;
        let common = DspCommon {
            radio_fs: radio_fs,
            channel_spacing: channel_spacing,
            cic_factor: (radio_fs / modem::FS).round() as usize,
            sine_table: cic::make_sinetable_freq(radio_fs, channel_spacing),
            filter_taps: fir::convert_symmetric_real_taps(&CHANNEL_FILTER_TAPS),
        };

        Self {
            tx_carriers: vec![TxCarrier::new(&common, 25000.0)],
            common: common,
        }
    }

    pub fn process(
        &mut self,
        time: i64,
        //buf: &mut [Complex<f32>],
        buf: &mut [cic::BufferType], // TODO: change back to f32 and add type conversion
        callbacks: &L1Callbacks,
    ) {
        let mut timenow = time;

        for cicbuf in buf.chunks_exact_mut(self.common.cic_factor) {
            for v in cicbuf.iter_mut() { *v = num::zero(); }
            for carrier in self.tx_carriers.iter_mut() {
                carrier.process(timenow, cicbuf, callbacks);
            }
            // Simulate a 4*18 kHz sample rate by incrementing timestamp.
            // FIXME: This is not exact as it has been rounded to integer nanoseconds.
            timenow += 13889;
        }
    }
}
