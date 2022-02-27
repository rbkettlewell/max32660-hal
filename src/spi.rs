use crate::gpio::p0::Parts;
use crate::gpio::{AltFn, AltMode, Floating, Input, Level, Output, Pin, PushPull, AF1, AF2};
use core::marker::PhantomData;
use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};
// use max32660_pac::SPI17Y as SPI0;
// use max32660_pac::SPIMSS as SPI1;
use nb;

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
