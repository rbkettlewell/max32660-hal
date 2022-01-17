use core::marker::PhantomData;
use std::io::Write;
use void::Void;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};

/// Disconnected pin in input mode (type state, reset value).
pub struct Disconnected;

/// Input mode (type state)
pub struct Input<MODE>{
    _mode: PhantomData<MODE>,
}

/// Floating input (type state).
pub struct Floating;
/// Pulled down input (type state).
pub struct PullDown;
/// Pulled up input (type state).
pub struct PullUp;

/// Output mode (type state).
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state).
pub struct PushPull;
/// Open drain output (type state).
pub struct OpenDrain;

/// Represents a digital input or output level.
#[derive(Debug, Eq, PartialEq)]
pub enum Level {
    Low,
    High,
}

// ===============================================================
// Implement Generic Pins for this port, which allows you to use
// other peripherals without having to be completely rust-generic
// across all of the possible pins
// ===============================================================
/// Generic $PX pin
pub struct Pin<MODE> {
    pin: u8,
    _mode: PhantomData<MODE>,
}

use crate::pac::{gpio0, GPIO0 as P0};

// `<MODE>` Must precede the type to remain generic.
impl<MODE> Pin<MODE> {
    fn new(pin:u8) -> Self {
        Self {pin, _mode: PhantomData}
    }

    #[inline]
    fn pin(&self) -> u8 {
        self.pin
    }

    fn block(&self) -> &gpio0::RegisterBlock{
        &unsafe { *P0::ptr() }
    }

    pub fn into_floating_input(self) -> Pin<Input<Floating>>{
        // Turn output off
        unsafe{ self.block().out_en_clr.write(|w| w.bits(0x01 << self.pin())); }
        // Clear pulls for pin, not clear if this is necessary to use modify, #TODO test
        self.block().pad_cfg1.modify(|r, w| r.bits() & !(0x01 << self.pin()));
        Pin {
            pin: self.pin,
            _mode: PhantomData,
        }
    }

    pub fn into_pullup_input(self) -> Pin<Input<PullUp>> {
        self.into_floating_input();
        // Is the modify necessary? PU is '1'
        self.block().ps.modify(|r, w| r.bits() | (0x01 << self.pin()));
        // Enables the pullup
        self.block().pad_cfg1.modify(|r, w| r.bits() | (0x01 << self.pin()));
        
    }
}

impl <const PORT: char, const INDEX: u8> OutputPin for Pin<Output, PORT, INDEX> {
    fn set_low(&mut self) -> Result<(), Self::Error> { unimplemented!()}
    fn set_high(&mut self) -> Result<(), Self::Error> { unimplemented!()}
    type Error = Void;
}