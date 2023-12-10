//! Signal processing

use num::Complex;

use static_fir::FirFilter;

use crate::L1Callbacks;

mod modem;
use modem::Modulator;

mod cic;

type RxDdc = cic::CicDdc<4>;
type TxDuc = cic::CicDuc<4>;

/// Combined pulse shaping and CIC compensation filter
/// for a rate of 4 samples per symbol.
/// Coefficients from design_channel_filter.py
impl_fir!(ChannelFilter, f32, 25, [
    -0.00798239,
    -0.00565370,
    0.00620650,
    0.01909224,
    0.01867999,
    -0.00306238,
    -0.03680449,
    -0.05611604,
    -0.03220604,
    0.04504324,
    0.15462617,
    0.25133416,
    0.28986898,
    0.25133416,
    0.15462617,
    0.04504324,
    -0.03220604,
    -0.05611604,
    -0.03680449,
    -0.00306238,
    0.01867999,
    0.01909224,
    0.00620650,
    -0.00565370,
    -0.00798239
]);

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
}

struct TxCarrier {
    duc: TxDuc,
    // static_fir does not seem to support a complex filter
    // real coefficients, so use two separate filters for now.
    filter_re: FirFilter::<ChannelFilter>,
    filter_im: FirFilter::<ChannelFilter>,
    modulator: Modulator,
}

impl TxCarrier {
    pub fn new(
        common: &DspCommon,
        carrier_freq: f64,
    ) -> Self {
        Self {
            duc: TxDuc::new(common.sine_table.clone(), (carrier_freq / common.channel_spacing).round() as isize),
            filter_re: FirFilter::<ChannelFilter>::new(),
            filter_im: FirFilter::<ChannelFilter>::new(),
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
        modulated.re = self.filter_re.feed(modulated.re);
        modulated.im = self.filter_im.feed(modulated.im);
        // TODO: proper scaling of CIC input
        modulated *= 1000.0;
        self.duc.process(
            cic::IntegratorType {
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
