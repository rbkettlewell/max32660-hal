//! Module to configure the GPIO pins as I/O and Alternate Functions (AF).
//!
use crate::pac::{gpio0 as gpio, GPIO0 as P0};
/// | Package | Number of GPIO | Pins |
/// |---------------------------------|
/// | 16 WLP  | GPIO0[9:0]     |  10  |
/// | 20 TQFN | GPIO0[13:0]    |  14  |
/// | 24 TQFN | GPIO0[13:0]    |  14  |
use core::marker::PhantomData;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use void::Void;

/// Disconnected pin in input mode (type state, reset value).
pub struct Disconnected;

/// Alternate Function Gpio mode (type state)
pub struct Gpio;

/// Alternate Function 1 (type state)
pub struct AF1;

/// Alternate Function 2
pub struct AF2;

/// Alternate Function 3
pub struct AF3;

/// Trait to enable bounds on Pin Alternate Mode types
pub trait AltMode {}

impl AltMode for Gpio {}
impl AltMode for AF1 {}
impl AltMode for AF2 {}
impl AltMode for AF3 {}

pub trait AltFn {
    fn set_mode(&mut self);
}
impl<IO, const IDX: u8> AltFn for Pin<Gpio, IO, IDX> {
    fn set_mode(&mut self) {
        unsafe {
            self.block().en_set.write(|w| w.bits(self.mask()));
            self.block().en1_clr.write(|w| w.bits(self.mask()));
            self.block().en2_clr.write(|w| w.bits(self.mask()));
        }
    }
}
impl<IO, const IDX: u8> AltFn for Pin<AF1, IO, IDX> {
    fn set_mode(&mut self) {
        unsafe {
            self.block().en_clr.write(|w| w.bits(self.mask()));
            self.block().en1_clr.write(|w| w.bits(self.mask()));
            self.block().en2_clr.write(|w| w.bits(self.mask()));
        }
    }
}
impl<IO, const IDX: u8> AltFn for Pin<AF2, IO, IDX> {
    fn set_mode(&mut self) {
        unsafe {
            self.block().en_clr.write(|w| w.bits(self.mask()));
            self.block().en1_set.write(|w| w.bits(self.mask()));
            self.block().en2_clr.write(|w| w.bits(self.mask()));
        }
    }
}
impl<IO, const IDX: u8> AltFn for Pin<AF3, IO, IDX> {
    fn set_mode(&mut self) {
        unsafe {
            self.block().en_set.write(|w| w.bits(self.mask()));
            self.block().en1_set.write(|w| w.bits(self.mask()));
            self.block().en2_clr.write(|w| w.bits(self.mask()));
        }
    }
}

/// Input mode (type state)
pub struct Input<MODE> {
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

pub enum DriveStrength {
    OneMilliamps,
    TwoMilliamps,
    FourMilliamps,
    SixMilliamps,
}

struct DriveStrengthSetting {
    ds1: bool,
    ds: bool,
}

impl DriveStrength {
    fn get_setting(self) -> DriveStrengthSetting {
        match self {
            DriveStrength::OneMilliamps => DriveStrengthSetting {
                ds1: false,
                ds: false,
            },
            DriveStrength::TwoMilliamps => DriveStrengthSetting {
                ds1: false,
                ds: true,
            },
            DriveStrength::FourMilliamps => DriveStrengthSetting {
                ds1: true,
                ds: false,
            },
            DriveStrength::SixMilliamps => DriveStrengthSetting {
                ds1: true,
                ds: true,
            },
        }
    }
}

// ===============================================================
// Implement Generic Pins for this port, which allows you to use
// other peripherals without having to be completely rust-generic
// across all of the possible pins
// ===============================================================
/// Generic $PX pin
pub struct Pin<AF: AltMode, IO, const IDX: u8> {
    _af: PhantomData<AF>,
    _io: PhantomData<IO>,
}

// `<MODE>` Must precede the type to remain generic.
impl<AF: AltMode, IO, const IDX: u8> Pin<AF, IO, IDX> {
    pub fn new() -> Self {
        Self {
            _af: PhantomData,
            _io: PhantomData,
        }
    }

    #[inline]
    fn pin(&self) -> u8 {
        IDX
    }

    fn mask(&self) -> u32 {
        0x01 << (IDX as u32)
    }

    fn block(&self) -> &gpio::RegisterBlock {
        let ptr = unsafe { &*P0::ptr() };
        ptr
    }

    pub fn into_mode<M: AltMode>(self) -> Pin<M, IO, IDX>
    where
        Pin<M, IO, IDX>: AltFn,
    {
        let mut pin = Pin::new();
        pin.set_mode();
        pin
    }

    pub fn into_floating_input(self) -> Pin<AF, Input<Floating>, IDX> {
        unsafe {
            // Turn output off
            self.block().out_en_clr.write(|w| w.bits(self.mask()));
            // Select GPIO Mode

            // Clear pulls for pin, not clear if this is necessary to use modify, #TODO test
            self.block()
                .pad_cfg1
                .modify(|r, w| w.bits(r.bits() & !self.mask()));
        }
        Pin::new()
    }

    pub fn into_pullup_input(self) -> Pin<AF, Input<PullUp>, IDX> {
        let pin = self.into_floating_input();
        unsafe {
            //  PU is '1'
            pin.block().ps.modify(|r, w| w.bits(r.bits() | pin.mask()));
            // Enables the pullup
            pin.block()
                .pad_cfg1
                .modify(|r, w| w.bits(r.bits() | pin.mask()));
        }
        Pin::new()
    }
    pub fn into_pulldown_input(self) -> Pin<AF, Input<PullDown>, IDX> {
        let pin = self.into_floating_input();
        unsafe {
            // PU is '0'
            pin.block().ps.modify(|r, w| w.bits(r.bits() & !pin.mask()));
            // Enables the pullup
            pin.block()
                .pad_cfg1
                .modify(|r, w| w.bits(r.bits() | pin.mask()));
        }
        Pin::new()
    }

    pub fn into_push_pull_output(self, initial_output: Level) -> Pin<AF, Output<PushPull>, IDX> {
        let mut pin = Pin::new();
        unsafe {
            self.block()
                .out_en_set
                .write(|w| w.bits(0x01 << self.pin()));
        }

        match initial_output {
            Level::Low => pin.set_low().unwrap(),
            Level::High => pin.set_high().unwrap(),
        }
        pin
    }
}

impl<AF: AltMode, MODE, const IDX: u8> InputPin for Pin<AF, Input<MODE>, IDX> {
    type Error = Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.block().in_.read().bits() & (1 << self.pin()) == 0)
    }
}

impl<AF: AltMode, MODE, const IDX: u8> OutputPin for Pin<AF, Output<MODE>, IDX> {
    type Error = Void;
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.block().out_clr.write(|w| w.bits(1u32 << self.pin()));
        }
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe { self.block().out_set.write(|w| w.bits(1u32 << self.pin())) }
        Ok(())
    }
}

impl<AF: AltMode, MODE, const IDX: u8> StatefulOutputPin for Pin<AF, Output<MODE>, IDX> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.block().out.read().bits() & (1 << self.pin()) == 0)
    }
}

impl<AF: AltMode, MODE, const IDX: u8> Pin<AF, Output<MODE>, IDX> {
    pub fn set_drive_strength(&mut self, drive_strength: DriveStrength) -> () {
        let ds_settings = drive_strength.get_setting();
        let ds_val = ds_settings.ds as u32;
        let ds1_val = ds_settings.ds1 as u32;
        unsafe {
            self.block()
                .ds
                .modify(|r, w| w.bits(r.bits() | (ds_val << self.pin())));
            self.block()
                .ds1
                .modify(|r, w| w.bits(r.bits() | (ds1_val << self.pin())));
        }
    }
}

macro_rules! gpio {
    (
        $PX: ident, $px: ident, [
            $(($pxi: ident, $i: expr),)+
        ]
    ) => {
        // GPIO
        pub mod $px {

            use super::{
                Pin,
                Gpio,
                Disconnected,
                $PX
            };

            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: Pin<Gpio, Disconnected, $i>,
                )+
            }

            impl Parts {
                // TODO is this _gpio input adding clarity, or clutter
                pub fn new(_gpio: $PX) -> Self {
                    Self {
                        $(
                            $pxi: Pin::new(),
                        )+
                    }
                }
            }
        }
    };
}

#[cfg(any(feature = "pkg-wlp"))]
gpio!(
    P0,
    p0,
    [
        (p0_00, 0),
        (p0_01, 1),
        (p0_02, 2),
        (p0_03, 3),
        (p0_04, 4),
        (p0_05, 5),
        (p0_06, 6),
        (p0_07, 7),
        (p0_08, 8),
        (p0_09, 9),
    ]
);

#[cfg(any(feature = "pkg-tqfn"))]
gpio!(
    P0,
    p0,
    [
        (p0_00, 0),
        (p0_01, 1),
        (p0_02, 2),
        (p0_03, 3),
        (p0_04, 4),
        (p0_05, 5),
        (p0_06, 6),
        (p0_07, 7),
        (p0_08, 8),
        (p0_09, 9),
        (p0_10, 10),
        (p0_11, 11),
        (p0_12, 12),
        (p0_13, 13),
    ]
);
