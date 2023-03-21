#![no_std]
pub use max32660_pac as pac;

pub mod clocks;
pub mod delay;
pub mod gpio;
pub mod spi;
pub mod i2c;
