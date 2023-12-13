use std::ffi::{c_int, c_void};

use num::Complex;

pub mod slot;
pub use slot::SlotNumber;

pub mod burst;
pub use burst::*;

pub mod dsp;
use dsp::L1Dsp;

pub mod io;

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
    radio:   io::soapy::SoapyIo,
    dsp:     L1Dsp,
}

impl L1 {
    fn new() -> Option<Self> {
        Some(Self {
            radio: io::soapy::SoapyIo::new(&io::soapy::SoapyIoConfig {
                fs: 1.8e6,
                rx_freq: 434e6,
                tx_freq: 434e6,
                rx_chan: 0,
                tx_chan: 0,
                rx_ant:  "LNAL",
                tx_ant:  "BAND1",
                rx_gain: 50.0,
                tx_gain: 50.0,
                dev_args: &[("driver", "lime")],
                rx_args: &[],
                tx_args: &[],
            })?,
            dsp: L1Dsp::new(),
        })
    }

    fn process(&mut self,
        callbacks: &L1Callbacks,
    ) -> Option<()> {
        self.radio.process(|time, buf: &mut [Complex<f32>]| {
            self.dsp.process(time, buf, callbacks)
        })
    }
}

#[no_mangle]
pub extern "C" fn l1_init(
) -> *mut L1 {
    match L1::new() {
        Some(l1) => Box::into_raw(Box::<L1>::new(l1)),
        None => core::ptr::null_mut()
    }
}

/// Free L1 instance.
/// This should always be called before programs exits
/// to make SDR device is properly shut down and closed.
#[no_mangle]
pub extern "C" fn l1_free(
    l1: *mut L1,
) {
    if !l1.is_null() {
        drop(unsafe { Box::from_raw(l1) })
    }
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
        Some(()) => 0,
        _ => -1
    }
}
