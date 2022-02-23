use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};
use nb;
use max32660_pac::SPI17Y as SPI0;
use max32660_pac::SPIMSS as SPI1;
use crate::gpio::p0::Parts;
use crate::gpio::{Pin, Input, Floating, Output, PushPull};

pub enum SpiPort{
    Spi0,
    Spi1,
}

// pub struct Pins<AF>{
//     //sclk: Pin<AF, Output<PushPull>>,
//     miso: Option<Pin<AF, Input<Floating>>>,
//     mosi: Option<Pin<AF, Output<PushPull>>>
// }

// pub fn configure_spi_pins(port: SpiPort, parts: Parts) -> Pins<AF1>{
//     match port {
//         SpiPort::Spi0 => {
//             let mut miso = parts.p0_04.degrade();
//             let miso1 = miso.into_floating_input();
//             let miso2 = miso1.set_mode_af1();
//             let pins = Pins{miso: Some(miso2), mosi: None };
//             pins
//         },
//         _ => unimplemented!()
//     }
// }
