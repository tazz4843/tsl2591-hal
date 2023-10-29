use crate::{Gain, IntegrationTime};

const INTEGER_CONVERSION_FACTOR: i64 = 1_000_000_000;
const OVERFLOW_100MS: u16 = 36863;
const OVERFLOW_OTHER: u16 = 65535;

pub fn check_overflow(integration_time: IntegrationTime, ch_0: u16, ch_1: u16) -> bool {
    let overflow_value = if let IntegrationTime::_100MS = integration_time {
        OVERFLOW_100MS
    } else {
        OVERFLOW_OTHER
    };

    (ch_0 >= overflow_value) | (ch_1 >= overflow_value)
}

/// Calculate lux from raw channel data.
///
/// Exists because there's a few different ways to do this,
/// and you may want to be able to swap them out.
pub trait LuxConverter {
    fn calculate_nano_lux(
        integration_time: IntegrationTime,
        gain: Gain,
        ch_0: u16,
        ch_1: u16,
    ) -> Option<i64>;

    fn calculate_lux(
        integration_time: IntegrationTime,
        gain: Gain,
        ch_0: u16,
        ch_1: u16,
    ) -> Option<f32> {
        Self::calculate_nano_lux(integration_time, gain, ch_0, ch_1)
            .map(|lux| lux as f32 / 1_000_000_000.0)
    }
}

/// Lux conversions taken from the Adafruit Python library
pub struct AdafruitPythonLuxConverter;
impl AdafruitPythonLuxConverter {
    const LUX_DF: f32 = 408.0;
    const LUX_COEFB: f32 = 1.64;
    const LUX_COEFC: f32 = 0.59;
    const LUX_COEFD: f32 = 0.86;

    const LUX_DF_INT: i64 = 408_000_000_000;
    const LUX_COEFB_INT: i64 = 1_640_000_000;
    const LUX_COEFC_INT: i64 = 590_000_000;
    const LUX_COEFD_INT: i64 = 860_000_000;
}
impl LuxConverter for AdafruitPythonLuxConverter {
    fn calculate_nano_lux(
        integration_time: IntegrationTime,
        gain: Gain,
        ch_0: u16,
        ch_1: u16,
    ) -> Option<i64> {
        if check_overflow(integration_time, ch_0, ch_1) {
            // Signal an overflow
            return None;
        }

        let a_time =
            integration_time.get_integration_time_millis() as i64 * INTEGER_CONVERSION_FACTOR;
        let a_gain = gain.get_multiplier() as i64 * INTEGER_CONVERSION_FACTOR;

        let ch_0 = ch_0 as i64 * INTEGER_CONVERSION_FACTOR;
        let ch_1 = ch_1 as i64 * INTEGER_CONVERSION_FACTOR;

        let cpl = (a_time * a_gain) / Self::LUX_DF_INT;
        if cpl == 0 {
            return None; // avoid divide by zero
        }
        let lux1 = (ch_0 - (Self::LUX_COEFB_INT * ch_1)) / cpl;
        let lux2 = ((Self::LUX_COEFC_INT * ch_0) - (Self::LUX_COEFD_INT * ch_1)) / cpl;

        Some(i64::max(lux1, lux2))
    }
    fn calculate_lux(
        integration_time: IntegrationTime,
        gain: Gain,
        ch_0: u16,
        ch_1: u16,
    ) -> Option<f32> {
        if check_overflow(integration_time, ch_0, ch_1) {
            // Signal an overflow
            return None;
        }

        let a_time = integration_time.get_integration_time_millis() as f32;
        let a_gain = gain.get_multiplier() as f32;

        let cpl = (a_time * a_gain) / Self::LUX_DF;
        let lux1 = (ch_0 as f32 - (Self::LUX_COEFB * ch_1 as f32)) / cpl;
        let lux2 = ((Self::LUX_COEFC * ch_0 as f32) - (Self::LUX_COEFD * ch_1 as f32)) / cpl;

        Some(f32::max(lux1, lux2))
    }
}

/// Based on https://www.yoctopuce.com/EN/article/yocto-i2c-and-tsl2591
pub struct YoctoLuxConverter;
impl LuxConverter for YoctoLuxConverter {
    fn calculate_nano_lux(
        integration_time: IntegrationTime,
        gain: Gain,
        ch_0: u16,
        ch_1: u16,
    ) -> Option<i64> {
        if check_overflow(integration_time, ch_0, ch_1) {
            // Signal an overflow
            return None;
        }

        let nano_lux = match gain {
            Gain::Low => ch_0 as i64 * 1_000_000,
            Gain::Med => ch_0 as i64 * 25_000_000,
            Gain::High => ch_0 as i64 * 428_000_000,
            Gain::Max => ch_0 as i64 * 9_876_000_000,
        };

        if (gain != Gain::Max && nano_lux < 50)
            || (gain != Gain::Low && nano_lux > 37_000)
            || (nano_lux == 0)
        {
            return None;
        }

        Some(nano_lux)
    }
}
