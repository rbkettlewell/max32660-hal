//! Module to configure the GPIO pins as I/O and Alternate Functions (AF).
//! 
/// | Package | Number of GPIO | Pins |
/// |---------------------------------|
/// | 16 WLP  | GPIO0[9:0]     |  10  |
/// | 20 TQFN | GPIO0[13:0]    |  14  |
/// | 24 TQFN | GPIO0[13:0]    |  14  |

use core::{marker::PhantomData};
use void::Void;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use crate::pac::{gpio0 as gpio, GPIO0 as P0};

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


pub trait AltFn{}

impl AltFn for AF1{}
impl AltFn for AF2{}
impl AltFn for AF3{}

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

pub enum DriveStrength{
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
            DriveStrength::OneMilliamps => DriveStrengthSetting{ds1:false, ds:false},
            DriveStrength::TwoMilliamps => DriveStrengthSetting{ds1:false, ds:true},
            DriveStrength::FourMilliamps => DriveStrengthSetting{ds1:true, ds:false},
            DriveStrength::SixMilliamps => DriveStrengthSetting{ds1:true, ds:true},
        }
    }
}

// ===============================================================
// Implement Generic Pins for this port, which allows you to use
// other peripherals without having to be completely rust-generic
// across all of the possible pins
// ===============================================================
/// Generic $PX pin
pub struct Pin<AF, MODE> {
    pin: u8,
    _af: PhantomData<AF>,
    _io: PhantomData<MODE>,
}

// `<MODE>` Must precede the type to remain generic.
impl<AF, MODE> Pin<AF, MODE> {
    // New should be made private once the macro rules are used.
    fn new(pin:u8) -> Self {
        Self {
            pin,
            _af: PhantomData, 
            _io: PhantomData,
        }
    }

    #[inline]
    fn pin(&self) -> u8 {
        self.pin
    }

    fn block(&self) -> &gpio::RegisterBlock{
        let ptr = unsafe { &*P0::ptr() };
        ptr
    }

    pub fn into_floating_input(self) -> Pin<AF, Input<Floating>> {
        
        unsafe{ 
            // Turn output off
            self.block().out_en_clr.write(|w| w.bits(0x01 << self.pin())); 
            // Clear pulls for pin, not clear if this is necessary to use modify, #TODO test
            self.block().pad_cfg1.modify(|r, w| w.bits(r.bits() & !(0x01 << self.pin())));
        }
        
        Pin {
            pin: self.pin,
            _af: PhantomData,
            _io: PhantomData,
        }
    }

    pub fn into_pullup_input(self) -> Pin<AF, Input<PullUp>> { // TODO Figure out how to use traits better
        let pin = self.into_floating_input();
        unsafe {
            // Is the modify necessary? PU is '1'
            pin.block().ps.modify(|r, w| w.bits(r.bits() | (0x01 << pin.pin())));
            // Enables the pullup
            pin.block().pad_cfg1.modify(|r, w| w.bits(r.bits() | (0x01 << pin.pin())));
        }
        Pin {
            pin: pin.pin,
            _af: PhantomData,
            _io: PhantomData,
        }
    }
    pub fn into_pulldown_input(self) -> Pin<AF, Input<PullDown>> {
        let pin = self.into_floating_input();
        unsafe {
            // Is the modify necessary? PU is '0'
            pin.block().ps.modify(|r, w| w.bits(r.bits() & !(0x01 << pin.pin())));
            // Enables the pullup
            pin.block().pad_cfg1.modify(|r, w| w.bits(r.bits() | (0x01 << pin.pin())));
        }
        Pin {
            pin: pin.pin,
            _af: PhantomData,
            _io: PhantomData,
        }
    }

    pub fn into_push_pull_output(self, initial_output: Level) -> Pin<AF, Output<PushPull>> {
        let mut pin = Pin {
            pin: self.pin,
            _af: PhantomData,
            _io: PhantomData,
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

impl <MODE> Pin<Gpio, MODE> {
    fn into_af(&self) -> Pin<AF3, MODE> {
        unimplemented!()
    }
}

impl <AF, MODE> InputPin for Pin <AF,Input<MODE>> {
    type Error = Void;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|v| !v)
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.block().in_.read().bits() & (1 << self.pin()) == 0)
    }
}

impl <AF, MODE> OutputPin for Pin <AF, Output<MODE>> {
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

impl <AF, MODE> StatefulOutputPin for Pin<AF, Output<MODE>> {
    /// Is the output pin set as high?
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.is_set_low().map(|v| !v)
    }

    /// Is the output pin set as low?
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.block().out.read().bits() & (1 << self.pin()) == 0)
    }
}

impl <AF, MODE> Pin<AF, Output<MODE>> {
    pub fn set_drive_strength(&self, drive_strength: DriveStrength) -> () {
        let ds_settings = drive_strength.get_setting();
        let ds_val = ds_settings.ds as u32;
        let ds1_val = ds_settings.ds1 as u32;
        unsafe{
            self.block().ds.modify(|r, w| w.bits(r.bits() | (ds_val << self.pin())));
            self.block().ds1.modify(|r, w| w.bits(r.bits() | (ds1_val << self.pin())));
        } 
    }
}

macro_rules! gpio {
    (
        $PX: ident, $px: ident, [
            $($PXi: ident: ($pxi: ident, $i: expr, $AF: ty, $IO: ty),)+
        ]
    ) => {
        // GPIO
        pub mod $px {
            
            use super::{
                Pin,

                Floating,
                Disconnected,
                DriveStrength,
                Input,
                Level,
                //OpenDrain,
                Output,
                PullDown,
                PullUp,
                PushPull,
                Gpio,
                PhantomData,
                $PX
            };
            use embedded_hal::digital::v2::{OutputPin, StatefulOutputPin, InputPin};
            // FIXME for multiport future MAX326xx support
            use crate::pac::gpio0 as gpio;
            use void::Void;

            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi<$AF, $IO>,
                )+
            }

            impl Parts {
                pub fn new(_gpio: $PX) -> Self {
                    Self {
                        $(
                            $pxi: $PXi {
                                _af: PhantomData,
                                _io: PhantomData,
                            },
                        )+
                    }
                }
            }
            // ===============================================================
            // Implement each of the typed pins usable through the nrf-hal
            // defined interface
            // ===============================================================
            $(
                pub struct $PXi<AF, IO> {
                    _af: PhantomData <AF>,
                    _io: PhantomData <IO>,
                }

                impl<AF, IO> $PXi<AF, IO> {

                    fn block(&self) -> &gpio::RegisterBlock{
                        let ptr = unsafe { &*$PX::ptr() };
                        ptr
                    }

                    /// Convert the pin to be a floating input
                    pub fn into_floating_input(self) -> $PXi <AF, Input<Floating>> {
                        unsafe { 
                            // Turn output off
                            self.block().out_en_clr.write(|w| w.bits(0x01 << $i)); 
                            // Clear pulls for pin, not clear if this is necessary to use modify, #TODO test
                            self.block().pad_cfg1.modify(|r, w| w.bits(r.bits() & !(0x01 << $i)));
                        };

                        $PXi {
                            _af: PhantomData,
                            _io: PhantomData,
                        }
                    }

                    pub fn into_pullup_input(self) -> $PXi <AF, Input<PullUp>> {
                        let pin = self.into_floating_input();
                        unsafe {
                            // Is the modify necessary? PU is '1'
                            pin.block().ps.modify(|r, w| w.bits(r.bits() | (0x01 << $i)));
                            // Enables the pullup
                            pin.block().pad_cfg1.modify(|r, w| w.bits(r.bits() | (0x01 << $i)));
                        }

                        $PXi {
                            _af: PhantomData,
                            _io: PhantomData,
                        }
                    }

                    pub fn into_pulldown_input(self) -> $PXi<AF, Input<PullDown>> {
                        let pin = self.into_floating_input();
                        unsafe {
                            // Is the modify necessary? PU is '0'
                            pin.block().ps.modify(|r, w| w.bits(r.bits() & !(0x01 << $i)));
                            // Enables the pullup
                            pin.block().pad_cfg1.modify(|r, w| w.bits(r.bits() | (0x01 << $i)));
                        }

                        $PXi {
                            _af: PhantomData,
                            _io: PhantomData,
                        }
                    }

                    /// Convert the pin to bepin a push-pull output with normal drive
                    pub fn into_push_pull_output(self, initial_output: Level)
                        -> $PXi<AF, Output<PushPull>>
                    {
                        let mut pin = $PXi {
                            _af: PhantomData,
                            _io: PhantomData,
                        };

                        match initial_output {
                            Level::Low  => pin.set_low().unwrap(),
                            Level::High => pin.set_high().unwrap(),
                        }

                        unsafe { 
                            self.block().out_en_set.write(|w| w.bits(0x01 << $i));
                        }

                        pin
                    }

                    /// Disconnects the pin.
                    /// 
                    /// Determine how to actually disconnect/turn off pins. #FIXME
                    pub fn into_disconnected(self) -> $PXi<AF, Disconnected> {
                        self.into_floating_input();
                        let pin = $PXi::<AF,Disconnected>{_af: PhantomData,_io: PhantomData,};

                        pin
                    }

                    /// Degrade to a generic pin struct, which can be used with peripherals
                    pub fn degrade(self) -> Pin<AF, IO> {
                        Pin::new($i)
                    }
                }

                impl<AF, IO> InputPin for $PXi<AF, Input<IO>> {
                    type Error = Void;

                    fn is_high(&self) -> Result<bool, Self::Error> {
                        self.is_low().map(|v| !v)
                    }

                    fn is_low(&self) -> Result<bool, Self::Error> {
                        Ok(self.block().in_.read().bits() & (1 << $i) == 0)
                    }
                }

                impl<AF, IO> From<$PXi<AF, IO>> for Pin<AF, IO> {
                    fn from(value: $PXi<AF, IO>) -> Self {
                        value.degrade()
                    }
                }

                impl<AF, IO> OutputPin for $PXi<AF, Output<IO>> {
                    type Error = Void;

                    /// Set the output as high
                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        unsafe {
                            self.block().out_set.write(|w| w.bits(1u32 << $i))
                        }
                        Ok(())
                    }

                    /// Set the output as low
                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        // NOTE(unsafe) atomic write to a stateless register - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        unsafe {
                            self.block().out_clr.write(|w| w.bits(1u32 << $i));
                        }
                        Ok(())
                    }
                }

                impl <AF, IO> $PXi <AF, Output<IO>> {
                    pub fn set_drive_strength(&self, drive_strength: DriveStrength) -> () {
                        let ds_settings = drive_strength.get_setting();
                        let ds_val = ds_settings.ds as u32;
                        let ds1_val = ds_settings.ds1 as u32;
                        unsafe{
                            self.block().ds.modify(|r, w| w.bits(r.bits() | (ds_val << $i)));
                            self.block().ds1.modify(|r, w| w.bits(r.bits() | (ds1_val << $i)));
                        } 
                    }
                }

                impl<AF, IO> StatefulOutputPin for $PXi<AF, Output<IO>> {
                    /// Is the output pin set as high?
                    fn is_set_high(&self) -> Result<bool, Self::Error> {
                        self.is_set_low().map(|v| !v)
                    }

                    /// Is the output pin set as low?
                    fn is_set_low(&self) -> Result<bool, Self::Error> {
                        // NOTE(unsafe) atomic read with no side effects - TODO(AJM) verify?
                        // TODO - I wish I could do something like `.pins$i()`...
                        Ok(self.block().out.read().bits() & (1 << $i) == 0)
                    }
                }
            )+      
        }  
    };
}


// #FIXME should generate the correct number of pins based on which package is being used.
gpio!(P0, p0, [
    P0_00: (p0_00,  0, Gpio, Disconnected),
    P0_01: (p0_01,  1, Gpio, Disconnected),
    P0_02: (p0_02,  2, Gpio, Disconnected),
    P0_03: (p0_03,  3, Gpio, Disconnected),
    P0_04: (p0_04,  4, Gpio, Disconnected),
    P0_05: (p0_05,  5, Gpio, Disconnected),
    P0_06: (p0_06,  6, Gpio, Disconnected),
    P0_07: (p0_07,  7, Gpio, Disconnected),
    P0_08: (p0_08,  8, Gpio, Disconnected),
    P0_09: (p0_09,  9, Gpio, Disconnected),
    P0_10: (p0_10, 10, Gpio, Disconnected),
    P0_11: (p0_11, 11, Gpio, Disconnected),
    P0_12: (p0_12, 12, Gpio, Disconnected),
    P0_13: (p0_13, 13, Gpio, Disconnected),
]);