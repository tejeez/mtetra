//! Signal processing

use num::Complex;

use crate::{L1Callbacks, SlotNumber, TxBurst};

mod modem;
use modem::Modulator;

pub mod cic;
mod fir;

/// Modem sample duration in nanoseconds.
/// Modem runs at a sample rate of 4*18 kHz.
const MODEM_SAMPLE_NS: i64 = 13889;

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
    // DUC input scaling multiplied by other scaling factors
    // to combine all of them to the same processing step.
    duc_input_scaling_combined: f32,
    // Sine table for DDC/DUC
    sine_table: cic::SineTableType,
    // Channel filter taps
    filter_taps: fir::SymmetricRealTaps,
}

struct TxCarrier {
    id: i32,
    duc: TxDuc,
    filter: fir::FirCf32Sym,
    modulator: Modulator,
}

impl TxCarrier {
    pub fn new(
        common: &DspCommon,
        id: i32,
        carrier_freq: f64,
    ) -> Self {
        Self {
            id,
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
        let mut modulated = self.modulator.sample(time,
            &mut |slot: SlotNumber, slot_time: i64, burst: &mut TxBurst| {
                (callbacks.tx_burst)(callbacks.tx_burst_arg, self.id, slot, slot_time, burst)
            }
        );
        modulated = self.filter.sample(modulated);
        self.duc.process(
            cic::cf32_to_sample(modulated, common.duc_input_scaling_combined),
            buf);
    }
}

pub struct L1Dsp {
    common: DspCommon,
    tx_carriers: Vec<TxCarrier>,
}

impl L1Dsp {
    pub fn new(radio_fs: f64) -> Self {
        let channel_spacing: f64 = 12500.0;
        let cic_factor = (radio_fs / modem::FS).round() as usize;
        let common = DspCommon {
            radio_fs: radio_fs,
            channel_spacing: channel_spacing,
            cic_factor: cic_factor,
            ddc_scale: RxDdc::scaling(cic_factor, 2.0),
            // Output amplitude is designed to stay below 1.0, but CIC
            // compensation filter may result in somewhat higher input values,
            // so specify 2.0 as maximum input to have plenty of margin.
            duc_scale: TxDuc::scaling(cic_factor, 2.0),
            // Computed each time process() is run to also work correctly
            // in case we end up adding more carriers after initialization.
            duc_input_scaling_combined: 0.0,
            sine_table: cic::make_sinetable_freq(radio_fs, channel_spacing),
            filter_taps: fir::convert_symmetric_real_taps(&CHANNEL_FILTER_TAPS),
        };

        Self {
            tx_carriers: vec![TxCarrier::new(&common, 0, 25000.0), TxCarrier::new(&common, 1, 50000.0)],
            common: common,
        }
    }

    pub fn process(
        &mut self,
        buf: &mut [Complex<f32>],
        rx_time: i64,
        tx_time: i64,
        callbacks: &L1Callbacks,
    ) {
        let mut rx_time_now = rx_time;
        let mut tx_time_now = tx_time;

        // TODO: allocate this buffer only once and store it in self.common
        let mut cicbuf: Vec<cic::BufferType> = vec![num::zero(); self.common.cic_factor];

        // Adjusted to keep peak magnitude just below 1.0.
        // Reduce carrier amplitude with number of carriers
        // to keep worst case peaks after combining just below 1.0.
        let modulator_scaling = 0.68 * modem::SPS as f32 / self.tx_carriers.len() as f32;
        self.common.duc_input_scaling_combined = modulator_scaling * self.common.duc_scale.0;

        for bufblock in buf.chunks_exact_mut(self.common.cic_factor) {
            for v in cicbuf.iter_mut() { *v = num::zero(); }
            for carrier in self.tx_carriers.iter_mut() {
                carrier.process(&self.common, tx_time_now, &mut cicbuf[..], callbacks);
            }
            cic::buf_to_cf32(&cicbuf[..], bufblock, self.common.duc_scale.1);
            // Increment timestamps for a 4*18 kHz sample rate.
            // FIXME: This is not exact as it has been rounded to integer nanoseconds.
            rx_time_now += MODEM_SAMPLE_NS;
            tx_time_now += MODEM_SAMPLE_NS;
        }
    }
}
