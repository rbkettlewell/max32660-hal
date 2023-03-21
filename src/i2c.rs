//! I2C implementation of the embedded_hal i2c traits and configuration
//!
use crate::clocks::PCLK_FREQ;
use crate::gpio::{AltFn, AltMode, Pin, AF1, Input, Floating};
use core::marker::PhantomData;
use cortex_m::asm::nop;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::i2c;
use heapless::Vec;
use max32660_pac::i2c0::ctrl::SCL_PP_MODE_A;

use crate::pac::i2c0;
use crate::pac::I2C0 as I2C0;

use nb;
use void::Void;

/// I2C Pins
pub const I2C0_SCL: u8 = 8;
pub const I2C0_SDA: u8 = 9;
// pub const I2C1_SCL: u8 = 2;
// pub const I2C1_SDA: u8 = 3;

// FIFO and DMA constants
/// Assumes write word size is a u8 and only half the fifo is being used.
const TX_FIFO_LEVEL: u8 = 16;

#[derive(PartialEq)]
pub enum Command{
    Read,
    Write
}

pub struct I2C0Slave;
pub struct I2C0Master;

pub trait I2CShared {}

impl I2CShared for I2C0Slave {}
impl I2CShared for I2C0Master {}

pub struct Pins<
    AF: AltMode,
    const SCL_IDX: u8,
    const SDA_IDX: u8,
> {
    scl: Pin<AF, Input<Floating>, SCL_IDX>,
    sda: Pin<AF, Input<Floating>, SDA_IDX>
}

pub struct I2CPort<
    AF: AltMode,
    PORT,
    const SCL_IDX: u8,
    const SDA_IDX: u8,
> {
    pins: Pins<AF, SCL_IDX, SDA_IDX>,
    _port: PhantomData<PORT>,
}

impl<
        AF: AltMode,
        PORT,
        const SCL_IDX: u8,
        const SDA_IDX: u8,
    > I2CPort<AF, PORT, SCL_IDX, SDA_IDX>
{
    fn configure_pins<SA: AltMode, IA: AltMode, SM, IM>(
        scl: Pin<SA, SM, SCL_IDX>,
        sda: Pin<IA, IM, SDA_IDX>,
    ) -> Pins<AF, SCL_IDX, SDA_IDX>
    where
        Pin<AF, SM, SCL_IDX>: AltFn,
        Pin<AF, IM, SDA_IDX>: AltFn,
    {
        let scl_mode = scl.into_mode::<AF>();
        let sda_mode = sda.into_mode::<AF>();
        
        // Unused mode
        let scl_electrical = scl_mode.into_floating_input();
        let sda_electrical = sda_mode.into_floating_input();

        Pins {
            scl: scl_electrical,
            sda: sda_electrical
        }
    }

}

macro_rules! i2c_ports{
    (
        [$(($new: ident, $AF: ident, $PORT: ident, $a: expr, $b: expr),)+]
    ) => {
        $(
            impl I2CPort<$AF, $PORT, $a, $b>{
                pub fn $new<SA:AltMode, IA: AltMode, SM, IM>(
                    scl: Pin::<SA, SM, $a>,
                    sda: Pin::<IA, IM, $b>
                ) -> Self {
                    let pins = Self::configure_pins(scl, sda);
                    Self{
                        pins: pins,
                        _port: PhantomData,
                    }
                }
            }
        )+
    };
}

i2c_ports!([
    (new_i2c0_slave, AF1, I2C0Slave, I2C0_SCL, I2C0_SDA),
]);


pub type I2CPort0Slave = I2CPort<AF1, I2C0Slave, I2C0_SCL, I2C0_SDA>;


impl <AF: AltMode, P: I2CShared> I2CPort<AF, P, I2C0_SCL, I2C0_SDA> {
    fn block(&self) -> &i2c0::RegisterBlock {
        let ptr = unsafe { &*I2C0::ptr() };
        ptr
    }

    /// Enables the I2C Peripheral
    pub fn enable(&mut self) {
        // Enable FIFOs
        // self.block().dma.modify(|_, w| w.tx_fifo_en().en());
        // self.block().dma.modify(|_, w| w.rx_fifo_en().en());

        // Enable the I2C peripheral
        self.block().ctrl.modify(|_, w| w.i2c_en().en());
    }

    /// Disables the I2C Port
    pub fn disable(&mut self) {    
        // Disable FIFOs
        // self.block().dma.modify(|_, w| w.tx_fifo_en().dis());
        // self.block().dma.modify(|_, w| w.rx_fifo_en().dis());
        // Sets slave select as output.
        // Diable the I2C peripheral
        self.block().ctrl.modify(|_, w| w.i2c_en().dis());
    }

    /// Check the I2C transaction type Read or Write 
    pub fn get_command(&mut self) -> Command{
        let read_command = self.block().ctrl.read().read().is_read();
        if read_command {
            Command::Read
        }
        else{
            Command::Write
        }
    }

    /// Flush RX FIFO
    pub fn flush_tx_fifo(&mut self){
        self.block().tx_ctrl0.write(|w| w.tx_flush().flush());
    }

    // Clear TX FIFO Lock
    fn clear_tx_fifo_lock(&mut self){
        self.block().int_fl0.modify(|_, w| w.tx_lock_out().set_bit());
    }

    /// Check rx_num_elements
    pub fn num_elements_tx_fifo(&mut self) -> u8 {
        self.block().tx_ctrl1.read().tx_fifo().bits()
    }

    /// Clear tranfer done interrupt
    pub fn clear_done_intr(&mut self){
        self.block().int_fl0.modify(|_, w| w.done().set_bit());
    }

    /// Check transfer done flag
    pub fn check_done_flag(&mut self)-> bool{
        self.block().int_fl0.read().done().bit()
    }

    /// Flush RX FIFO
    pub fn flush_rx_fifo(&mut self){
        self.block().rx_ctrl0.write(|w| w.rx_flush().flush());
    }

    /// Check rx_num_elements
    pub fn num_elements_rx_fifo(&mut self) -> u8 {
        self.block().rx_ctrl1.read().rx_fifo().bits()
    }

    /// Enables the stop interrupt event.
    pub fn enable_stop_intr(&mut self) {
        self.block().int_en0.modify(|_, w| w.stop().en());
    }
    /// Check if the stop interrupt has fired
    pub fn check_stop_intr(&self) -> bool{
        self.block().int_fl0.read().stop().bit_is_set()
    }
    /// Check if the stop interrupt has fired
    pub fn clear_stop_intr(&self){
        self.block().int_fl0.modify(|_, w| w.stop().set_bit());
    }
    /// Disables the stop interrupt event.
    pub fn disable_stop_intr(&mut self) {
        self.block().int_en0.modify(|_, w| w.stop().dis());
    }

    /// Enables the transfer complete interrupt event.
    pub fn enable_done_intr(&mut self) {
        self.block().int_en0.modify(|_, w| w.done().en());
    }
    /// Disables the transfer complete interrupt event.
    pub fn disable_done_intr(&mut self) {
        self.block().int_en0.modify(|_, w| w.done().dis());
    }

    ///  Enables the slave mode address match interrupt event.
    pub fn enable_addr_match_intr(&mut self) {
        self.block().int_en0.modify(|_, w| w.addr_match().en());
    }

    ///  Disables the slave mode address match interrupt event.
    pub fn disable_addr_match_intr(&mut self) {
        self.block().int_en0.modify(|_, w| w.addr_match().dis());
    }

    /// Clear address match interrupt flag
    pub fn clear_addr_match_intr(&mut self) {
        // Set bit to clear
        self.block().int_fl0.modify(|_, w| w.addr_match().set_bit());
    }

    /// Check address match interrupt
    pub fn check_addr_match_intr(&self) -> bool{
        self.block().int_fl0.read().addr_match().bit()
    }

    /// Write data to fifo
    pub fn write(&mut self, data: &Vec<u8, 16>){
        self.clear_tx_fifo_lock();
        let len = data.len();
        for i in 0..len{
            self.block().fifo.write(|w| unsafe{w.data().bits(data[i])})
        }
    }
    /// Read data from fifo and return the amount read
    pub fn read(&mut self, data: &mut Vec<u8, 16>){
        let read_count = self.num_elements_rx_fifo() as usize;
        for _ in 0..read_count{
            data.push(self.block().fifo.read().data().bits()).unwrap();
        }
    }
}

impl I2CPort0Slave {

    /// Configures the I2C port as a slave interface
    pub fn configure(&mut self, addr: u16) {
        self.block().ctrl.modify(|_,w| w.mst().slave_mode());
        self.block().ctrl.modify(|_, w| w.gen_call_addr().dis());
        self.block().ctrl.modify(|_, w| w.scl_clk_strech_dis().en());

        // Enable for High-speed mode (Hs-mode) operation 3.4Mb/s. 
        // Disable for Standard (100Kb/s), Fast (400Kb/s) or Fast-Plus (1Mb/s).
        self.block().ctrl.modify(|_, w| w.hs_mode().dis());

        self.block().rx_ctrl0.modify(|_, w| w.dnr().respond());
        self.block().tx_ctrl0.modify(|_, w| w.tx_preload().clear_bit());

        // 1/fpclk * (clk_hi + 1) > Tsu_data(min)
        let clk_hi = 32u16; // Unknown Tsu_data
        self.block().clk_hi.modify(|_, w| unsafe{w.ckh().bits(clk_hi)});
        let clk_hi_hs = 32u8; // Unknown Tsu_data
        self.block().hs_clk.modify(|_, w| unsafe{w.hs_clk_hi().bits(clk_hi_hs)});

        self.block().slave_addr.modify(|_, w|unsafe{w.slave_addr().bits(addr)});
        if addr > 127u16 {
            self.block().slave_addr.modify(|_, w| w.ex_addr().set_bit());
        }
        else{
            self.block().slave_addr.modify(|_, w| w.ex_addr().clear_bit())
        }
        
    }

}