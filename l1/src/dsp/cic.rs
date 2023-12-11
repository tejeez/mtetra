use std::rc::Rc;

use num::Complex;

/// Data type used for integrators
pub type IntegratorType = Complex<i64>;
/// Data type of real and imaginary parts used for sine table
pub type SineTypeReal = i16;
/// Data type used for elements of sine table
pub type SineType = Complex<SineTypeReal>;
/// Data type used for the whole sine table
pub type SineTableType = Rc<[SineType]>;
/// Data type used for DDC input and DUC output buffers
pub type BufferType = Complex<i64>;

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

/// Multiply an integrator value with sine table value,
/// shifting right after multiplication for better precision.
#[inline]
fn mul_int_sine_a(v: IntegratorType, s: SineType) -> IntegratorType {
    let m = v * IntegratorType { re: s.re as i64, im: s.im as i64 };
    IntegratorType { re: m.re >> 16, im: m.im >> 16 }
}

/// Multiply an integrator value with sine table value,
/// shifting right before multiplication to avoid overflow.
#[inline]
fn mul_int_sine_b(v: IntegratorType, s: SineType) -> IntegratorType {
    IntegratorType { re: v.re >> 16,  im: v.im >> 16 } *
    IntegratorType { re: s.re as i64, im: s.im as i64 }
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
            integrator: [num::zero(); N],
            comb:       [num::zero(); N],
            sinetable:  sinetable,
        }
    }

    /// Process a block of samples, returning one output sample.
    /// Length of the input slice shall be equal to decimation ratio.
    pub fn process(
        &mut self,
        input: &[BufferType]
    ) -> IntegratorType {
        // Last integrator and first comb are combined into a sum
        let mut output: IntegratorType = num::zero();
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
            self.integrator[N-1] += mul_int_sine_a(*in_, self.sinetable[self.phase]);

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
        return output;
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
            integrator: [num::zero(); N],
            comb:       [num::zero(); N],
            sinetable:  sinetable,
        }
    }

    /// Process one input sample, adding output samples to a slice.
    /// Length of the output slice shall be equal to interpolation ratio.
    pub fn process(
        &mut self,
        input: IntegratorType,
        output: &mut [BufferType]
    ) {
        let mut sample = input;

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
}
