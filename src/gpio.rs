use core::marker::PhantomData;
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
    // New should be made private once the macro rules are used.
    pub fn new(pin:u8) -> Self {
        Self {pin, _mode: PhantomData}
    }

    #[inline]
    fn pin(&self) -> u8 {
        self.pin
    }

    fn block(&self) -> &gpio0::RegisterBlock{
        let ptr = unsafe { &*P0::ptr() };
        ptr
    }

    pub fn into_floating_input(self) -> Pin<Input<Floating>>{
        
        unsafe{ 
            // Turn output off
            self.block().out_en_clr.write(|w| w.bits(0x01 << self.pin())); 
            // Clear pulls for pin, not clear if this is necessary to use modify, #TODO test
            self.block().pad_cfg1.modify(|r, w| w.bits(r.bits() & !(0x01 << self.pin())));
        }
        
        Pin {
            pin: self.pin,
            _mode: PhantomData,
        }
    }

    pub fn into_pullup_input(self) -> Pin<Input<PullUp>> {
        let pin = self.into_floating_input();
        unsafe {
            // Is the modify necessary? PU is '1'
            pin.block().ps.modify(|r, w| w.bits(r.bits() | (0x01 << pin.pin())));
            // Enables the pullup
            pin.block().pad_cfg1.modify(|r, w| w.bits(r.bits() | (0x01 << pin.pin())));
        }
        Pin {
            pin: pin.pin,
            _mode: PhantomData,
        }
    }
    pub fn into_pulldown_input(self) -> Pin<Input<PullDown>> {
        let pin = self.into_floating_input();
        unsafe {
            // Is the modify necessary? PU is '0'
            pin.block().ps.modify(|r, w| w.bits(r.bits() & !(0x01 << pin.pin())));
            // Enables the pullup
            pin.block().pad_cfg1.modify(|r, w| w.bits(r.bits() | (0x01 << pin.pin())));
        }
        Pin {
            pin: pin.pin,
            _mode: PhantomData,
        }
    }

    pub fn into_push_pull_output(self, initial_output: Level) -> Pin<Output<PushPull>> {
        let mut pin = Pin {
            pin: self.pin,
            _mode: PhantomData,
        };

        unsafe { 
            self.block().out_en_set.write(|w| w.bits(0x01 << self.pin()));
        }

        match initial_output {
            Level::Low  => pin.set_low().unwrap(),
            Level::High => pin.set_high().unwrap(),
        }
        pin
    }
}

impl <MODE> InputPin for Pin <Input<MODE>> {
    type Error = Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.block().in_.read().bits() & (1 << self.pin()) == 0)
    }
}

impl <MODE> OutputPin for Pin <Output<MODE>> {
    type Error = Void;
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.block().out_clr.write(|w| w.bits(1u32 << self.pin()));
        }
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.block().out_set.write(|w| w.bits(1u32 << self.pin()))
        }
        Ok(())
    }
}

impl <MODE> StatefulOutputPin for Pin <Output<MODE>> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
        // TODO - I wish I could do something like `.pins$i()`...
        Ok(self.block().out.read().bits() & (1 << self.pin()) == 0)
    }
}