use num;
use num::Complex;
use crate::slot::SlotNumber;
use crate::burst::TxBurst;
use crate::L1Callbacks;

/// Symbol rate
pub const SYMBOLRATE: f64 = 18000.0;

/// Samples per symbol
pub const SPS: usize = 4;

/// Sample rate used by modulator and demodulator
pub const FS: f64 = SYMBOLRATE * (SPS as f64);


/// Length of a hyperframe in nanoseconds.
const HYPERFRAME_NS: i64 = 1000_000 * 255*4*18*60 / 18;

fn ns_to_symbols(ns: i64) -> i32 {
    (ns * 9 / 500000) as i32
}

#[allow(dead_code)]
fn symbols_to_ns(symbols: i32) -> i64 {
    (symbols as i64) * 500000 / 9
}

pub struct Modulator {
    /// Timestamp at the beginning of a hyperframe
    /// is used as a reference point.
    htime: i64,

    prev_hsym: i32,

    mapper: DqpskMapper,

    /// Slot number of the current burst
    burst_slot: SlotNumber,
    /// Current burst being transmitted
    burst: TxBurst,
}

impl Modulator {
    pub fn new() -> Self {
        Self {
            htime: 0,
            prev_hsym: 255,
            burst_slot: SlotNumber::new(4, 18, 60),
            burst: TxBurst::None,
            mapper: DqpskMapper::new(),
        }
    }

    /// Produce a sample of transmit signal before matched filtering.
    pub fn sample(
        &mut self,
        time: i64,
        callbacks: &L1Callbacks,
    ) -> Complex<f32> {
        let mut output: Complex<f32> = num::zero();
        // Current symbol number within a hyperframe
        let hsym = ns_to_symbols((time - self.htime).rem_euclid(HYPERFRAME_NS));
        // Is it time for a new symbol?
        if hsym != self.prev_hsym {
            // Split into a slot number and a symbol number within a slot.
            let symnum = hsym.rem_euclid(255);
            let slot = SlotNumber::from_int(hsym.div_euclid(255));
            // Did a new slot just begin?
            if slot != self.burst_slot {
                self.burst_slot = slot;
                // Ask for a new burst to transmit.
                (callbacks.tx_cb)(callbacks.tx_cb_arg, slot, &mut self.burst);
            }
            output = match self.burst {
                TxBurst::None => num::zero(),
                TxBurst::Dl(bits) =>
                    self.mapper.symbol(
                        bits[symnum as usize * 2]     != 0,
                        bits[symnum as usize * 2 + 1] != 0),
                _ => todo!(),
            };
        }

        self.prev_hsym = hsym;
        output
    }
}


struct DqpskMapper {
    pub phase: i8,
}

impl DqpskMapper {
    pub fn new() -> Self {
        Self { phase: 0 }
    }

    #[allow(dead_code)]
    pub fn reset_phase(&mut self) {
        self.phase = 0;
    }

    pub fn symbol(&mut self, bit0: bool, bit1: bool) -> Complex<f32> {
        self.phase = (self.phase + match (bit0, bit1) {
            (true,  true)  => -3,
            (true,  false) => -1,
            (false, false) =>  1,
            (false, true)  =>  3,
        }) & 7;
        // Look-up table to map phase (in multiples of pi/4)
        // to constellation points. Generated in Python with:
        // import numpy as np
        // print(",\n".join("Complex{ re: %9.6f, im: %9.6f }" % (v.real, v.imag) for v in np.exp(1j*np.linspace(0, np.pi*2, 8, endpoint=False))))
        const CONSTELLATION: [Complex<f32>; 8] = [
            Complex{ re:  1.000000, im:  0.000000 },
            Complex{ re:  0.707107, im:  0.707107 },
            Complex{ re:  0.000000, im:  1.000000 },
            Complex{ re: -0.707107, im:  0.707107 },
            Complex{ re: -1.000000, im:  0.000000 },
            Complex{ re: -0.707107, im: -0.707107 },
            Complex{ re: -0.000000, im: -1.000000 },
            Complex{ re:  0.707107, im: -0.707107 }
        ];
        CONSTELLATION[self.phase as usize]
    }
}
