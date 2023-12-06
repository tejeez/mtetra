
/// Number of a timeslot
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct SlotNumber {
    /// Timeslot number (TN) (1-4)
    pub timeslot: u8,
    /// TDMA Frame number (FN) (1-18)
    pub frame: u8,
    /// TDMA Multiframe Number (MN) (1-60)
    pub multiframe: u8,
}

impl SlotNumber {
    pub fn new(timeslot: u8, frame: u8, multiframe: u8) -> Self {
        assert!(timeslot >= 1 && timeslot <= 4);
        assert!(frame >= 1 && frame <= 18);
        assert!(multiframe >= 1 && multiframe <= 60);
        Self {
            timeslot: timeslot,
            frame: frame,
            multiframe: multiframe,
        }
    }

    /// Convert to an integer representing the number of slots
    /// from beginning of a hyperframe.
    pub fn to_int(self) -> i32 {
        ((self.timeslot-1)   as i32) +
        ((self.frame-1)      as i32) * 4 +
        ((self.multiframe-1) as i32) * (4 * 18)
    }

    pub fn from_int(i: i32) -> Self {
        Self {
            timeslot:   (i           .rem_euclid(4)  + 1) as u8,
            frame:      ((i / 4)     .rem_euclid(18) + 1) as u8,
            multiframe: ((i / (4*18)).rem_euclid(60) + 1) as u8,
        }
    }

    pub fn plus(self, slots: i32) -> Self {
        Self::from_int(self.to_int() + slots)
    }

    pub fn minus(self, slots: i32) -> Self {
        self.plus(-slots)
    }
}

/// Convert slot number to an integer representing
/// the number of slots from beginning of a hyperframe.
#[no_mangle]
pub extern "C" fn slot_to_int(slot: SlotNumber) -> i32 {
    slot.to_int()
}

/// Convert an integer representing the number of slots
/// from beginning of a hyperframe to a slot number.
#[no_mangle]
pub extern "C" fn slot_from_int(i: i32) -> SlotNumber {
    SlotNumber::from_int(i)
}

#[no_mangle]
pub extern "C" fn slot_plus(slot: SlotNumber, slots: i32) -> SlotNumber {
    slot.plus(slots)
}

#[no_mangle]
pub extern "C" fn slot_minus(slot: SlotNumber, slots: i32) -> SlotNumber {
    slot.minus(slots)
}
