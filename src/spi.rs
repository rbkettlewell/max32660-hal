use crate::gpio::p0::Parts;
use crate::gpio::{AltFn, AltMode, Floating, Input, Level, Output, Pin, PushPull, AF1, AF2};
use core::marker::PhantomData;
use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};

use crate::pac::SPI17Y as SPI0;
use crate::pac::spi17y;
use crate::pac::SPIMSS as SPI1;
use crate::pac::spimss;
use nb;
use void::Void;

pub struct Spi0;

pub struct Spi1;

pub struct Pins<
    AF: AltMode,
    const SCLK_IDX: u8,
    const MISO_IDX: u8,
    const MOSI_IDX: u8,
    const SS_IDX: u8,
> {
    sclk: Pin<AF, Output<PushPull>, SCLK_IDX>,
    miso: Pin<AF, Input<Floating>, MISO_IDX>,
    mosi: Pin<AF, Output<PushPull>, MOSI_IDX>,
    ss: Pin<AF, Output<PushPull>, SS_IDX>,
}

pub struct SpiPort<
    AF: AltMode,
    PORT,
    const SCLK_IDX: u8,
    const MISO_IDX: u8,
    const MOSI_IDX: u8,
    const SS_IDX: u8,
> {
    pins: Pins<AF, SCLK_IDX, MISO_IDX, MOSI_IDX, SS_IDX>,
    _af: PhantomData<AF>,
    _port: PhantomData<PORT>,
}

impl<AF: AltMode, PORT, const SCLK_IDX: u8, const MISO_IDX: u8, const MOSI_IDX: u8, const SS_IDX: u8,> 
    SpiPort<AF, PORT, SCLK_IDX, MISO_IDX, MOSI_IDX, SS_IDX>
{
    fn configure_pins<SA: AltMode, IA: AltMode, OA: AltMode, XA: AltMode, SM, IM, OM, XM>(
        sclk: Pin<SA, SM, SCLK_IDX>,
        miso: Pin<IA, IM, MISO_IDX>,
        mosi: Pin<OA, OM, MOSI_IDX>,
        ss: Pin<XA, XM, SS_IDX>,
    ) -> Pins<AF, SCLK_IDX, MISO_IDX, MOSI_IDX, SS_IDX>
    where
        Pin<AF, SM, SCLK_IDX>: AltFn,
        Pin<AF, IM, MISO_IDX>: AltFn,
        Pin<AF, OM, MOSI_IDX>: AltFn,
        Pin<AF, XM, SS_IDX>: AltFn,
    {
        let sclk_mode = sclk.into_mode::<AF>();
        let miso_mode = miso.into_mode::<AF>();
        let mosi_mode = mosi.into_mode::<AF>();
        let ss_mode = ss.into_mode::<AF>();
        let sclk_electrical = sclk_mode.into_push_pull_output(Level::High);
        let miso_electrical = miso_mode.into_floating_input();
        let mosi_electrical = mosi_mode.into_push_pull_output(Level::High);
        let ss_electrical = ss_mode.into_push_pull_output(Level::High);
        Pins {
            sclk: sclk_electrical,
            miso: miso_electrical,
            mosi: mosi_electrical,
            ss: ss_electrical,
        }
    }
}

macro_rules! spi_ports{
    (
        [$(($new: ident, $AF: ident, $PORT: ident, $a: expr, $b: expr, $c: expr, $d: expr),)+]
    ) => {
        $(
            impl SpiPort<$AF, $PORT, $a, $b, $c, $d>{
                pub fn $new<SA:AltMode, IA: AltMode, OA: AltMode, XA: AltMode, SM, IM, OM, XM>(
                    sclk: Pin::<SA, SM, $a>,
                    miso: Pin::<IA, IM, $b>,
                    mosi: Pin::<OA, OM, $c>,
                    ss: Pin::<XA, XM, $d>
                ) -> Self {
                    let pins = Self::configure_pins(sclk, miso, mosi, ss);
                    Self{
                        pins: pins,
                        _af: PhantomData,
                        _port: PhantomData,
                    }
                }
            }
        )+
    };
}

spi_ports!([
    (new_spi_0, AF1, Spi0, 6, 4, 5, 7),
    (new_spi_1, AF2, Spi1, 2, 0, 1, 3),
]);

impl SpiPort<AF1, Spi0, 6, 4, 5, 7>{
    fn block(&self) -> &spi17y::RegisterBlock {
        let ptr = unsafe { &*SPI0::ptr() };
        ptr
    }

    /// Configures the SPI0 Port polarity, mode
    pub fn configure(&mut self, mode: Mode, ss_active_pol: Level, sck_div: u8){
        // Selects between master and slave mode. Only master mode support currently.
        self.block().ctrl0.modify(|_, w| w.master().en());
        // Enables the slave select pin for the master
        self.block().ctrl0.modify(|r, w| unsafe{w.bits(r.bits() | 0x01 << 16)});
        // Sets slave select as output.
        self.block().ctrl0.modify(|_, w| w.ss_io().output()); // modify with _, w will modify just sub bits
        // SS deasserts at the end of the transaction.
        self.block().ctrl0.modify(|_, w| w.ss_ctrl().deassert()); // write will overwrite all other bits to reset
        // SS Polarity typically active low.
        unsafe {
            if ss_active_pol == Level::High {
                self.block().ctrl2.modify(|_, w| w.ss_pol().bits(1));
            }
            else{
                self.block().ctrl2.modify(|_, w| w.ss_pol().bits(0));
            }
            // Number of bits per character
            self.block().ctrl2.modify(|_, w| w.numbits().bits(8u8));
        }
        // Standard SCK polarity for MODE 0/1
        if mode.polarity == Polarity::IdleLow {
            self.block().ctrl2.modify(|_, w|w.cpol().normal());
        }
        // Inverted SCK polarity for MODE 2/3
        else{
            self.block().ctrl2.modify(|_, w|w.cpol().inverted());
        }
        // SCK polarity for MODE 0/2
        if mode.phase == Phase::CaptureOnFirstTransition{
            self.block().ctrl2.modify(|_, w| w.cpha().rising_edge());
        }
        // SCK polarity for MODE 1/3
        else{
            self.block().ctrl2.modify(|_, w|w.cpha().rising_edge());
        }
        // Set SPI0 Peripheral Clock Scale
        unsafe{
            self.block().clk_cfg.modify(|_, w| w.scale().bits(sck_div))
        }
    }

    /// Enables the SPI0 Port
    pub fn enable(&mut self){
        // Sets slave select as output.
        self.block().ctrl0.modify(|_, w| w.en().en());
    }

    /// Disables the SPI0 Port
    pub fn disable(&mut self){
        // Sets slave select as output.
        self.block().ctrl0.modify(|_, w| w.en().dis());
    }

    pub fn is_busy(&self) -> bool {
        self.block().stat.read().busy().is_active()
    }
}

impl FullDuplex<u8> for SpiPort<AF1, Spi0, 6, 4, 5, 7> {
    type Error = Void;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        Ok(self.block().data8()[0].read().bits())
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        while self.is_busy() {}
        unsafe{
            self.block().ctrl1.modify(|_, w| w.tx_num_char().bits(1));
            let x = 0x40046000 as *mut u32;
            core::ptr::write_volatile(0x40046000 as *mut u32, 0xAA);
            //self.block().data8()[0].write(|w| w.data().bits(word));
            self.block().ctrl0.modify(|_, w| w.start().set_bit());
        }
        Ok(())
    }
}

impl embedded_hal::blocking::spi::write::Default<u8> for SpiPort<AF1, Spi0, 6, 4, 5, 7> {}
impl embedded_hal::blocking::spi::transfer::Default<u8> for SpiPort<AF1, Spi0, 6, 4, 5, 7> {}
