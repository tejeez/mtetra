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
    // CIC DDC scaling factors
    ddc_scale: (f32, f32),
    // CIC DUC scaling factors
    duc_scale: (f32, f32),
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
        common: &DspCommon,
        time: i64,
        buf: &mut [cic::BufferType],
        callbacks: &L1Callbacks,
    ) {
        let mut modulated = self.modulator.sample(time, callbacks);
        modulated = self.filter.sample(modulated);
        self.duc.process(
            cic::cf32_to_sample(modulated, common.duc_scale.0),
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
        let cic_factor = (radio_fs / modem::FS).round() as usize;
        let common = DspCommon {
            radio_fs: radio_fs,
            channel_spacing: channel_spacing,
            cic_factor: cic_factor,
            ddc_scale: RxDdc::scaling(cic_factor, 2.0),
            duc_scale: TxDuc::scaling(cic_factor, 2.0),
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
        buf: &mut [Complex<f32>],
        callbacks: &L1Callbacks,
    ) {
        let mut timenow = time;

        // TODO: allocate this buffer only once and store it in self.common
        let mut cicbuf: Vec<cic::BufferType> = vec![num::zero(); self.common.cic_factor];

        for bufblock in buf.chunks_exact_mut(self.common.cic_factor) {
            for v in cicbuf.iter_mut() { *v = num::zero(); }
            for carrier in self.tx_carriers.iter_mut() {
                carrier.process(&self.common, timenow, &mut cicbuf[..], callbacks);
            }
            cic::buf_to_cf32(&cicbuf[..], bufblock, self.common.duc_scale.1);
            // Increment timestamp for a 4*18 kHz sample rate.
            // FIXME: This is not exact as it has been rounded to integer nanoseconds.
            timenow += 13889;
        }
    }
}
