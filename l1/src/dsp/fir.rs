use std::rc::Rc;
use num::Complex;
use wide::f32x4;

pub type SymmetricRealTaps = Rc<[f32x4]>;

/// Convert symmetric filter taps to a format used by FirCf32Sym.
/// halftaps is the second half of impulse response, i.e.
/// starting from the centermost tap.
pub fn convert_symmetric_real_taps(halftaps: &[f32]) -> SymmetricRealTaps {
    halftaps.chunks(4).map(|v| {
        // Pad with zeros if not a multiple of vector size
        let mut t: [f32; 4] = [0.0; 4];
        t[0..v.len()].copy_from_slice(v);
        f32x4::from(v)
    }).collect()
}


/// FIR filter for complex signal with symmetric real taps.
pub struct FirCf32Sym {
    i:           usize,
    /// Real part of first half of history.
    /// Data is repeated twice for "fake circular buffering".
    history_re:  Vec<f32>,
    /// Imaginary part.
    history_im:  Vec<f32>,
    /// Real part of second half of history.
    /// The signal is reversed here to make it easier
    /// to implement a symmetric filter.
    reversed_re: Vec<f32>,
    /// Imaginary part.
    reversed_im: Vec<f32>,
    taps:        SymmetricRealTaps,
}

impl FirCf32Sym {
    pub fn new(taps: SymmetricRealTaps) -> Self {
        let len = taps.len() * 4 * 2;
        Self {
            i:           0,
            history_re:  vec![num::zero(); len],
            history_im:  vec![num::zero(); len],
            reversed_re: vec![num::zero(); len],
            reversed_im: vec![num::zero(); len],
            taps:        taps,
        }
    }

    pub fn sample(&mut self, in_: Complex<f32>) -> Complex<f32> {
        let taps: &[f32x4] = &self.taps;
        let len = taps.len() * 4;
        // Index to history buffer
        let i = self.i;
        // Index to reversed history buffer
        let ir = len - 1 - i;

        // Move older samples to reversed history buffer
        self.reversed_re[ir]       = self.history_re[i];
        self.reversed_re[ir + len] = self.history_re[i];
        self.reversed_im[ir]       = self.history_im[i];
        self.reversed_im[ir + len] = self.history_im[i];
        // Put new sample in first history buffer
        self.history_re [i]        = in_.re;
        self.history_re [i + len]  = in_.re;
        self.history_im [i]        = in_.im;
        self.history_im [i + len]  = in_.im;

        let mut sum_re: f32x4 = f32x4::ZERO;
        let mut sum_im: f32x4 = f32x4::ZERO;
        for ((((t, h_re), h_im), r_re), r_im) in
            taps.iter()
            .zip(self.history_re [i+1 .. i+1+len].chunks_exact(4))
            .zip(self.history_im [i+1 .. i+1+len].chunks_exact(4))
            .zip(self.reversed_re[ir ..  ir +len].chunks_exact(4))
            .zip(self.reversed_im[ir ..  ir +len].chunks_exact(4))
        {
            sum_re += (f32x4::from(h_re) + f32x4::from(r_re)) * t;
            sum_im += (f32x4::from(h_im) + f32x4::from(r_im)) * t;
        }

        // Increment index
        self.i = if self.i < len-1 { self.i + 1 } else { 0 };

        Complex::<f32> { re: sum_re.reduce_add(), im: sum_im.reduce_add() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fir_cf32_sym() {
        const TAPS: [f32; 8] = [ 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0 ];
        let mut fir = FirCf32Sym::new(convert_symmetric_real_taps(&TAPS));

        let mut out = Vec::<Complex<f32>>::new();

        // Test feeding it some impulses with different values.
        // Add different numbers of zero samples in between to see that
        // buffer indexing works correctly in every case.
        let impulses_in = [
            Complex::<f32>{ re: 1.0, im: 0.0 },
            Complex::<f32>{ re: 0.0, im: 1.0 },
            Complex::<f32>{ re: 0.1, im: 0.2 },
            Complex::<f32>{ re:-0.3, im:-0.4 },
        ];
        let nzeros: [usize; 4] = [ 100, 101, 102, 123 ];
        for (in_, zeros) in impulses_in.iter().zip(nzeros) {
            out.clear();
            out.push(fir.sample(*in_));
            for _ in 0..zeros {
                out.push(fir.sample(num::zero()));
            }
            //eprintln!("{:?}", out);
            // The filter should first output values of taps reversed
            // and then not reversed, multiplied by the input value.
            // Check if the output is close enough to the expected value,
            // allowing for some rounding errors.
            fn check(value: Complex<f32>, expected: Complex<f32>) {
                //eprintln!("Output {}, should be {}", value, expected);
                assert!((expected.re - value.re).abs() < 1e-6);
                assert!((expected.im - value.im).abs() < 1e-6);
            }
            for i in 0..TAPS.len() {
                //eprintln!("Checking tap {}", i);
                // Reversed part of impulse response
                check(out[i], *in_ * TAPS[TAPS.len() - 1 - i]);
                // Non-reversed part
                check(out[TAPS.len() + i], in_ * TAPS[i]);
            }
            // Rest of output should be zeros
            //eprintln!("Checking output is zeros when it should be");
            for value in out[TAPS.len()*2 ..].iter() {
                check(*value, num::zero());
            }
        }
    }
}
