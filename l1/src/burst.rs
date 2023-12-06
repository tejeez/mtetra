//! Burst data types passed between L1 and L2.

#[repr(C)]
pub struct RxBurstInfo {
    timestamp: i64,
    /// Received signal strength (unit TBD)
    rssi: f32,
    /// Esimated carrier frequency offset in Hz
    cfo: f32,
}

#[repr(C)]
pub struct RxDlBurst {
    info: RxBurstInfo,
    bits: [u8; 510],
}

#[repr(C)]
pub struct RxUlNormalBurst {
    info: RxBurstInfo,
    bits: [u8; 462],
}

#[repr(C)]
pub struct RxUlControlBurst {
    info: RxBurstInfo,
    bits: [u8; 206],
}

#[repr(C)]
pub struct RxDmoBurst {
    info: RxBurstInfo,
    bits: [u8; 470],
}

#[repr(C)]
pub enum RxSubslotBurst {
    /// No burst detected in subslot.
    None,
    /// Control up-link burst in subslot.
    UlControl(RxUlControlBurst),
}

#[repr(C)]
pub enum RxBurst {
    /// No burst detected in slot.
    None,

    /// Normal continuous down-link burst
    /// with normal training sequence 1.
    DlNormal1(RxDlBurst),
    /// Normal continuous down-link burst
    /// with normal training sequence 2.
    DlNormal2(RxDlBurst),
    /// Synchronization continuous donk-link burst.
    DlSync(RxDlBurst),

    /// Normal up-link burst.
    UlNormal(RxUlNormalBurst),
    /// Up-link burst(s) in subslots.
    /// Each subslot may contain a control up-link burst
    /// or no burst.
    Subslots([RxSubslotBurst; 2]),

    /// Direct mode normal burst
    /// with normal training sequence 1.
    DmoNormal1(RxDmoBurst),
    /// Direct mode normal burst
    /// with normal training sequence 2.
    DmoNormal2(RxDmoBurst),
    /// Direct mode synchronization burst.
    DmoSync(RxDmoBurst),
}

#[repr(C)]
pub enum TxBurst {
    /// No burst to transmit.
    None,
    /// Continuous down-link burst.
    /// Modulator does not care whether it is a normal or synchronization
    /// burst (since they have the same number of symbols),
    /// so both use the same value.
    Dl([u8; 510]),
    /// Direct mode burst.
    /// Modulator does not care whether it is a normal or synchronization
    /// burst (since they have the same number of symbols),
    /// so both use the same value.
    Dmo([u8; 470]),
}
