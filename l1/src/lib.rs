use std::ffi::{c_int, c_void};

pub mod slot;
pub use slot::SlotNumber;

pub mod burst;
pub use burst::*;

pub struct L1 {
    // TODO
}

#[no_mangle]
pub extern "C" fn l1_init(
) -> *mut L1 {
    Box::into_raw(Box::<L1>::new(L1 { }))
}

#[no_mangle]
pub extern "C" fn l1_process(
    l1: *mut L1,
    rx_cb: extern "C" fn(
        arg: *mut c_void,
        slot: SlotNumber,
        burst: *const RxBurst,
    ),
    rx_cb_arg: *mut c_void,
    tx_cb: extern "C" fn(
        arg: *mut c_void,
        slot: SlotNumber,
        burst: *mut TxBurst,
    ),
    tx_cb_arg: *mut c_void,
) -> c_int {
    // TODO
    0
}

#[cfg(test)]
mod tests {
    use super::*;
}
