use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};
use nb;
use max32660_pac::SPI17Y as SPI0;
use max32660_pac::SPIMSS as SPI1;