//! Delays.
// #FIXME HFCLK frequency assumption may be incorrect?
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use crate::clocks::HFCLK_FREQ;

/// System timer (SysTick) as a delay provider.
pub struct Delay {
    syst: SYST,
}

impl Delay {
    /// Configures the system timer (SysTick) as a delay provider.
    pub fn new(mut syst: SYST) -> Self {
        syst.set_clock_source(SystClkSource::Core);

        Delay { syst }
    }

    /// Releases the system timer (SysTick) resource.
    pub fn free(self) -> SYST {
        self.syst
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        for _ in 0..ms {
            self.delay_us(1_000u32);
        }
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        // The SysTick Reload Value register supports values between 1 and 0x00FFFFFF.
        const MAX_RVR: u32 = 0x00FF_FFFF;

        let mut total_rvr = us * (HFCLK_FREQ / 1_000_000);

        while total_rvr != 0 {
            let current_rvr = if total_rvr <= MAX_RVR {
                total_rvr
            } else {
                MAX_RVR
            };

            self.syst.set_reload(current_rvr);
            self.syst.clear_current();
            self.syst.enable_counter();

            // Update the tracking variable while we are waiting...
            total_rvr -= current_rvr;

            while !self.syst.has_wrapped() {}

            self.syst.disable_counter();
        }
    }
}

macro_rules! delay_xs {
    ($DXS: ident, $dxs: ident, $xs: ident, $T: ty) => {
        impl $DXS<$T> for Delay {
            fn $dxs(&mut self, $xs: $T) {
                self.$dxs($xs as u32);
            }
        }
    };
}

delay_xs!(DelayMs, delay_ms, ms, u16);
delay_xs!(DelayMs, delay_ms, ms, u8);
delay_xs!(DelayUs, delay_us, us, u16);
delay_xs!(DelayUs, delay_us, us, u8);
