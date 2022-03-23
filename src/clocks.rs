use crate::pac::gcr::CLKCN;

/// High Frequency Clock Frequency (in Hz).
pub const HFCLK_FREQ: u32 = 96_000_000;
pub const PCLK_FREQ: u32 = HFCLK_FREQ / 2;
/// Low Frequency Clock Frequency (in Hz).
pub const LFCLK_FREQ: u32 = 32_768;

trait Oscillator {
    fn enable(&self, clkcn: &CLKCN) -> ();
    fn disable(&self, clkcn: &CLKCN) -> ();
}


struct X32k;
/// Nano Ring oscillator cannot be disabled.
struct NanoRing;
struct Hfio;

impl Oscillator for X32k {
    fn enable(&self, clkcn: &CLKCN) -> () {
        clkcn.write(|w| w.x32k_en().en());
    }
    fn disable(&self, clkcn: &CLKCN) -> () {
        clkcn.write(|w| w.x32k_en().dis());
    }
}

impl Oscillator for Hfio {
    fn enable(&self, clkcn: &CLKCN) -> () {
        clkcn.write(|w| w.hirc_en().en());
    }
    fn disable(&self, clkcn: &CLKCN) -> () {
        clkcn.write(|w| w.hirc_en().en());
    }
}

pub struct Clocks<'a>{
    x32k: X32k,
    nano: NanoRing,
    hfio: Hfio,
    clkcn: &'a CLKCN,
}

pub enum ClkSrc{
    Hfio,
    Nano,
    X32k,
}

pub enum Div {
    D1,
    D2,
    D4,
    D8,
    D16,
    D32,
    D64,
    D128
}

// Before setting OVR and thus changing the SysClk rate you need to select Nano or X32K for SysClk
impl<'a> Clocks<'a>{
    pub fn new(clkcn: &'a CLKCN) -> Self{
        Self{x32k: X32k, nano: NanoRing, hfio: Hfio, clkcn}
    }

    pub fn enable(&self, clk_src: ClkSrc){
        match clk_src {
            ClkSrc::Hfio => self.hfio.enable(&self.clkcn),
            ClkSrc::Nano => (),
            ClkSrc::X32k => self.x32k.enable(&self.clkcn)
        }
    }

    pub fn disable(&self, clk_src: ClkSrc){
        match clk_src {
            ClkSrc::Hfio => self.hfio.disable(&self.clkcn),
            ClkSrc::Nano => (),
            ClkSrc::X32k => self.x32k.disable(&self.clkcn)
        }
    }

    pub fn set_sys_osc_source(&self, clk_src: ClkSrc) -> () {
        match clk_src {
            ClkSrc::Hfio => self.clkcn.write(|w| w.clksel().hirc()),
            ClkSrc::Nano => self.clkcn.write(|w| w.clksel().nano_ring()),
            ClkSrc::X32k => self.clkcn.write(|w| w.clksel().hfx_in())
        }
        
        // Wait for clock selection to finish
        while self.clkcn.read().ckrdy().is_busy() {}
    }

    pub fn set_sys_osc_prescaler(&self, psc_div: Div) {
        match psc_div {
            Div::D1 => self.clkcn.write(|w| w.psc().div1()),
            Div::D2 => self.clkcn.write(|w| w.psc().div2()),
            Div::D4 => self.clkcn.write(|w| w.psc().div4()),
            Div::D8 => self.clkcn.write(|w| w.psc().div8()),
            Div::D16 => self.clkcn.write(|w| w.psc().div16()),
            Div::D32 => self.clkcn.write(|w| w.psc().div32()),
            Div::D64 => self.clkcn.write(|w| w.psc().div64()),
            Div::D128 => self.clkcn.write(|w| w.psc().div128()),
        }
    }
}
