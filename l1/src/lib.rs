use std::{ffi::{c_int, c_void}, io::Write};

pub mod slot;
pub use slot::SlotNumber;

pub mod burst;
pub use burst::*;

pub mod dsp;
use dsp::L1Dsp;

#[repr(C)]
pub struct L1Callbacks {
    /// C function to process received burst(s).
    /// Called once per slot.
    pub rx_cb: extern "C" fn(
        arg: *mut c_void,
        slot: SlotNumber,
        burst: *const RxBurst,
    ),
    /// Argument passed to rx_cb.
    pub rx_cb_arg: *mut c_void,
    /// C function to produce a transmit burst.
    /// Called once per slot.
    pub tx_cb: extern "C" fn(
        arg: *mut c_void,
        slot: SlotNumber,
        burst: *mut TxBurst,
    ),
    /// Argument passed to tx_cb.
    pub tx_cb_arg: *mut c_void,
}

pub struct L1 {
    dsp: L1Dsp,
    timenow: i64,
}

impl L1 {
    fn new() -> Self {
        Self {
            dsp: L1Dsp::new(),
            timenow: 0,
        }
    }

    fn process(&mut self,
        callbacks: &L1Callbacks,
    ) -> std::io::Result<()> {
        // TODO: maybe move I/O to a separate struct
        // and make it more configurable.
        // Now just test by writing to stdout.
        use num::Complex;
        const BUFSIZE: usize = 7200;
        let mut buf: [Complex<f32>; BUFSIZE] = [ num::zero(); BUFSIZE ];

        self.dsp.process(self.timenow, &mut buf, callbacks);

        let mut stdout = std::io::stdout();
        // Let's be a bit lazy and use transmute to write the buffer to file.
        // Yes, the file format ends up depending on machine endianness etc,
        // so it's unsafe.
        // This is for initial testing purposes only.
        stdout.write_all(&unsafe { std::mem::transmute::<[Complex<f32>; BUFSIZE], [u8; BUFSIZE*8]>(buf) })?;

        // Simulate a 1.8 MHz sample rate by incrementing timestamp
        self.timenow += (BUFSIZE as f64 * 1e9 / 1.8e6) as i64;

        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn l1_init(
) -> *mut L1 {
    Box::into_raw(Box::<L1>::new(L1::new()))
}

/// C wrapper for L1::process.
/// Returns 0 on success, negative number on failure.
#[no_mangle]
pub extern "C" fn l1_process(
    l1: *mut L1,
    callbacks: L1Callbacks,
) -> c_int {
    let l1_ = unsafe { l1.as_mut().expect("l1 shall not be NULL") };
    match l1_.process(&callbacks) {
        Ok(()) => 0,
        _ => -1
    }
}
