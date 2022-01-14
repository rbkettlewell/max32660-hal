use core::marker::PhantomData;
use void::Void;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};

pub mod typestate {
    pub struct NotConfigured;
    pub struct Input;
    pub struct Output;
}

use typestate::*;

pub struct Pin<MODE, const PORT: char, const INDEX: u8>{
    _marker: PhantomData<MODE>,
}

impl <const PORT: char, const INDEX: u8> OutputPin for Pin<Output, PORT, INDEX> {
    fn set_low(&mut self) -> Result<(), Self::Error> { unimplemented!()}
    fn set_high(&mut self) -> Result<(), Self::Error> { unimplemented!()}
    type Error = Void;
}