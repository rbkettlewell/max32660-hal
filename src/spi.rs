use crate::gpio::p0::Parts;
use crate::gpio::{AltFn, AltMode, Floating, Input, Level, Output, Pin, PushPull, AF1, AF2, Level};
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
        let sclk_electrical = sclk_mode.into_push_pull_output(Level::Low);
        let miso_electrical = miso_mode.into_floating_input();
        let mosi_electrical = mosi_mode.into_push_pull_output(Level::Low);
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
    pub fn configure(&mut self, mode: Mode, ss_pol: Level, sck_div: u8){
        // Selects between master and slave mode. Only master mode support currently.
        self.block().ctrl0.write(|w| w.master().en());
        // Sets slave select as output.
        self.block().ctrl0.write(|w| w.ss_io().output());
        // SS deasserts at the end of the transaction.
        self.block().ctrl0.write(|w| w.ss_ctrl().deassert());
        // SS Polarity typically active low.
        unsafe {
            if ss_pol == Level::High {
                self.block().ctrl2.write(|w| w.ss_pol().bits(0x01));
            }
            else{
                self.block().ctrl2.write(|w| w.ss_pol().bits(0x00));
            }
            // Number of bits per character
            self.block().ctrl2.write(|w| w.numbits().bits(8u8));
        }
        // Standard SCK polarity for MODE 0/1
        if mode.polarity == Polarity::IdleHigh {
            self.block().ctrl2.write(|w|w.cpol().normal());
        }
        // Inverted SCK polarity for MODE 2/3
        else{
            self.block().ctrl2.write(|w|w.cpol().inverted());
        }
        // SCK polarity for MODE 0/2
        if mode.phase == Phase::CaptureOnFirstTransition{
            self.block().ctrl2.write(|w| w.cpha().rising_edge());
        }
        // SCK polarity for MODE 1/3
        else{
            self.block().ctrl2.write(|w|w.cpha().rising_edge());
        }
        // Set SPI0 Peripheral Clock Scale
        unsafe{
            self.block().clk_cfg.write(|w| w.scale().bits(sck_div))
        }
    }

    /// Enables the SPI0 Port
    pub fn enable(&mut self){
        // Sets slave select as output.
        self.block().ctrl0.write(|w| w.en().en());
    }

    /// Disables the SPI0 Port
    pub fn disable(&mut self){
        // Sets slave select as output.
        self.block().ctrl0.write(|w| w.en().dis());
    }
}

impl FullDuplex<u8> for SpiPort<AF1, Spi0, 6, 4, 5, 7> {
    type Error = Void;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        unimplemented!();
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        unimplemented!();
    }

    

}