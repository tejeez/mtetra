use std::rc::Rc;

use num::Complex;
use wide::i64x2;

/// Data type used for integrators
pub type IntegratorType = i64x2;
/// Data type of real and imaginary parts used for sine table
pub type SineTypeReal = i16;
/// Data type used for elements of sine table
pub type SineType = Complex<SineTypeReal>;
/// Data type used for the whole sine table
pub type SineTableType = Rc<[SineType]>;
/// Data type used for DDC input and DUC output buffers
pub type BufferType = Complex<i64>;
/// Data type used for DDC output and DUC input samples
pub type SampleType = BufferType;

const SINE_SHIFT: usize = 16;

/// Make a sine table with a given length.
pub fn make_sinetable(length: usize) -> SineTableType {
    // Frequency in radians per bin
    let freq: f32 = std::f32::consts::PI * 2.0 / (length as f32);
    (0..length).map(|i| {
        let phase = i as f32 * freq;
        let scale = i16::MAX as f32;
        SineType {
            re: (phase.cos() * scale).round() as SineTypeReal,
            im: (phase.sin() * scale).round() as SineTypeReal,
        }
    }).collect()
}

/// Make a sine table for a given channel spacing and sample rate.
pub fn make_sinetable_freq(fs: f64, channel_spacing: f64) -> SineTableType {
    make_sinetable((fs / channel_spacing).round() as usize)
}

/// Convert buffer type to integrator type.
#[inline]
fn buf_to_int(v: BufferType) -> IntegratorType {
    IntegratorType::from([v.re, v.im])
}

/// Convert integrator type to buffer type.
#[inline]
fn int_to_buf(v: IntegratorType) -> BufferType {
    let a = v.to_array();
    BufferType { re: a[0], im: a[1] }
}

/// Multiply a buffer value with sine table value,
/// shifting right after multiplication for better precision.
#[inline]
fn mul_buf_sine_a(v: BufferType, s: SineType) -> IntegratorType {
    let m = v * BufferType { re: s.re as i64, im: s.im as i64 };
    IntegratorType::from([ m.re >> SINE_SHIFT, m.im >> SINE_SHIFT ])
}

/// Multiply an integrator value with sine table value,
/// shifting right before multiplication to avoid overflow.
#[inline]
fn mul_int_sine_b(v: IntegratorType, s: SineType) -> BufferType {
    let a = v.to_array();
    BufferType { re: a[0] >> SINE_SHIFT, im: a[1] >> SINE_SHIFT } *
    BufferType { re: s.re as i64, im: s.im as i64 }
}

/// Convert Complex<f32> to DUC input sample.
pub fn cf32_to_sample(sample: Complex<f32>, scaling: f32) -> SampleType {
    SampleType {
        re: (sample.re * scaling) as i64,
        im: (sample.im * scaling) as i64,
    }
}

/// Convert DDC output sample to Complex<f32>.
pub fn sample_to_cf32(sample: SampleType, scaling: f32) -> Complex<f32> {
    Complex::<f32> {
        re: sample.re as f32 * scaling,
        im: sample.im as f32 * scaling,
    }
}

/// Convert Complex<f32> slice to CIC buffer type.
pub fn cf32_to_buf(input: &[Complex<f32>], output: &mut [BufferType], scaling: f32) {
    input.iter().zip(output).for_each(|(in_, out)| {
        *out = BufferType {
            re: (in_.re * scaling) as i64,
            im: (in_.im * scaling) as i64,
        }
    });
}

/// Convert CIC buffer slice to Complex<f32>.
pub fn buf_to_cf32(input: &[BufferType], output: &mut [Complex<f32>], scaling: f32) {
    input.iter().zip(output).for_each(|(in_, out)| {
        *out = Complex::<f32> {
            re: in_.re as f32 * scaling,
            im: in_.im as f32 * scaling,
        }
    });
}

/// Digital down-converter using a CIC filter.
/// Number of integrator and comb stages is N+1.
/// Minimum supported is N=1, i.e. a 2-stage CIC.
pub struct CicDdc<const N: usize> {
    phase:      usize,
    freq:       usize,
    integrator: [IntegratorType; N],
    comb:       [IntegratorType; N],
    sinetable:  SineTableType,
}

impl<const N: usize> CicDdc<N> {
    pub fn new(
        sinetable: Rc<[SineType]>,
        freq:      isize,
    ) -> Self {
        Self {
            phase:      0,
            freq:       (-freq).rem_euclid(sinetable.len() as isize) as usize,
            integrator: [IntegratorType::ZERO; N],
            comb:       [IntegratorType::ZERO; N],
            sinetable:  sinetable,
        }
    }

    /// Process a block of samples, returning one output sample.
    /// Length of the input slice shall be equal to decimation ratio.
    pub fn process(
        &mut self,
        input: &[BufferType]
    ) -> SampleType {
        // Last integrator and first comb are combined into a sum
        let mut output: IntegratorType = IntegratorType::ZERO;
        for in_ in input {
            // Computations are ordered so that each integrator
            // takes a result from previous loop iteration,
            // making the code more "pipeline-friendly".
            // This adds a few samples of delay to the input signal
            // but it is not really a problem.
            output += self.integrator[0];
            for n in 0..N-1 {
                self.integrator[n] += self.integrator[n+1];
            }
            self.integrator[N-1] += mul_buf_sine_a(*in_, self.sinetable[self.phase]);

            self.phase += self.freq;
            if self.phase >= self.sinetable.len() {
                self.phase -= self.sinetable.len();
            }
        }
        // Comb filters
        for n in 0..N {
            let previous = output;
            output -= self.comb[n];
            self.comb[n] = previous;
        }
        int_to_buf(output)
    }

    /// Compute scaling factors for a given decimation ratio
    /// and maximum f32 input value.
    /// Returns a tuple (input_scaling, output_scaling).
    /// Input scaling factor should be passed to cf32_to_buf
    /// and output scaling to sample_to_cf32.
    pub fn scaling(ratio: usize, max_in: f32) -> (f32, f32) {
        // How much integrator cascade grows numbers
        let growth = (ratio as f32).powi((N + 1) as i32);
        // Maximum value that can be fed to CIC without overflow.
        let cic_in_max = (i64::MAX as f32) / growth;
        // Maximum value that can be multiplied by sine table.
        let sine_in_max = (i64::MAX >> SINE_SHIFT) as f32;
        // Input scaling for convert_cf32_buf
        let input_scaling = cic_in_max.min(sine_in_max) / max_in;
        // Sine table has an amplitude of 0.5 to make sure
        // complex multiplication does not grow numbers.
        // Compensate for that in output scaling.
        let output_scaling = 2.0 / (input_scaling * growth);
        (input_scaling, output_scaling)
    }
}


/// Digital up-converter using a CIC filter.
/// Number of integrator and comb stages is N+1.
/// Minimum supported is N=1, i.e. a 2-stage CIC.
pub struct CicDuc<const N: usize> {
    phase:      usize,
    freq:       usize,
    integrator: [IntegratorType; N],
    comb:       [IntegratorType; N],
    sinetable:  SineTableType,
}

impl<const N: usize> CicDuc<N> {
    pub fn new(
        sinetable: Rc<[SineType]>,
        freq:      isize,
    ) -> Self {
        Self {
            phase:      0,
            freq:       freq.rem_euclid(sinetable.len() as isize) as usize,
            integrator: [IntegratorType::ZERO; N],
            comb:       [IntegratorType::ZERO; N],
            sinetable:  sinetable,
        }
    }

    /// Process one input sample, adding output samples to a slice.
    /// Length of the output slice shall be equal to interpolation ratio.
    pub fn process(
        &mut self,
        input: SampleType,
        output: &mut [BufferType]
    ) {
        let mut sample = buf_to_int(input);

        // Comb filters
        for n in 0..N {
            let previous = sample;
            sample -= self.comb[n];
            self.comb[n] = previous;
        }

        // Last comb and first integrator are implemented
        // by repeating the input sample.
        for out in output.iter_mut() {
            *out += mul_int_sine_b(self.integrator[0], self.sinetable[self.phase]);
            self.phase += self.freq;
            if self.phase >= self.sinetable.len() {
                self.phase -= self.sinetable.len();
            }
            // Computations are ordered so that each integrator
            // takes a result from previous loop iteration,
            // making the code more "pipeline-friendly".
            // This adds a few samples of delay to the output signal
            // but it is not really a problem.
            for n in 0..N-1 {
                self.integrator[n] += self.integrator[n+1];
            }
            self.integrator[N-1] += sample;
        }
    }

    /// Compute scaling factors for a given interpolation ratio
    /// and maximum f32 input value.
    /// Returns a tuple (input_scaling, output_scaling).
    /// Input scaling factor should be passed to cf32_to_sample
    /// and output scaling to buf_to_cf32.
    pub fn scaling(ratio: usize, max_in: f32) -> (f32, f32) {
        // How much integrator cascade grows numbers
        let growth = (ratio as f32).powi(N as i32);
        // Maximum value that can be fed to CIC without overflow
        let cic_in_max = (i64::MAX as f32) / growth;

        let input_scaling = cic_in_max / max_in;

        // Sine table has an amplitude of 0.5 to make sure
        // complex multiplication does not grow numbers.
        // Compensate for that in output scaling.
        let output_scaling = 2.0 / (input_scaling * growth);
        (input_scaling, output_scaling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duc_scaling() {
        type DucType = CicDuc<4>;
        let sinetable = make_sinetable(100);

        for ratio in [1usize, 10, 100, 1000] {
            eprintln!("Testing interpolation ratio {}", ratio);
            let mut duc = DucType::new(sinetable.clone(), 1);
            let max_in = 1.0;
            let (scale_in, scale_out) = DucType::scaling(ratio, max_in);
            // Test with worst case value (both elements max_in)
            // to check it does not overflow.
            let v_in = Complex::<f32> { re: max_in, im: max_in };
            for i in 0..100 {
                let mut cicbuf: Vec<BufferType> = vec![num::zero(); ratio];
                let mut floatbuf: Vec<Complex<f32>> = vec![num::zero(); ratio];
                duc.process(cf32_to_sample(v_in, scale_in), &mut cicbuf[..]);
                buf_to_cf32(&cicbuf[..], &mut floatbuf[..], scale_out);
                // Skip test for first few samples where output has not settled yet
                if i > 10 {
                    // Output should be sine wave with the same amplitude
                    // as DUC input value. Check that it is close enough.
                    for o in &floatbuf[..] {
                        let gain = o.norm() / v_in.norm();
                        //eprintln!("{}", gain);
                        assert!(gain > 0.99);
                        assert!(gain < 1.01);
                    }
                }
            }
        }
    }

    #[test]
    fn test_ddc_scaling() {
        type DdcType = CicDdc<4>;
        let sinetable = make_sinetable(100);

        for ratio in [1usize, 10, 100, 1000] {
            eprintln!("Testing decimation ratio {}", ratio);
            // TODO: test with sine wave input.
            // For now set center frequency to 0 so it can be fed DC for testing.
            let mut ddc = DdcType::new(sinetable.clone(), 0);
            let max_in = 1.0;
            let (scale_in, scale_out) = DdcType::scaling(ratio, max_in);
            // Test with worst case value (both elements max_in)
            // to check it does not overflow.
            let v_in = Complex::<f32> { re: max_in, im: max_in };
            let floatbuf: Vec<Complex<f32>> = vec![v_in; ratio];
            let mut cicbuf: Vec<BufferType> = vec![num::zero(); ratio];
            cf32_to_buf(&floatbuf[..], &mut cicbuf[..], scale_in);
            for i in 0..100 {
                let ddcout = ddc.process(&cicbuf[..]);
                let o = sample_to_cf32(ddcout, scale_out);
                // Skip test for first few samples where output has not settled yet
                if i > 10 {
                    let gain = o.norm() / v_in.norm();
                    //eprintln!("{}", gain);
                    assert!(gain > 0.99);
                    assert!(gain < 1.01);
                }
            }
        }
    }
}
