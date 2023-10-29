use crate::{Gain, IntegrationTime};

const LUX_DF: f32 = 408.0;
const LUX_COEFB: f32 = 1.64;
const LUX_COEFC: f32 = 0.59;
const LUX_COEFD: f32 = 0.86;

const INTEGER_CONVERSION_FACTOR: i64 = 1_000_000_000;

const LUX_DF_INT: i64 = 408_000_000_000;
const LUX_COEFB_INT: i64 = 1_640_000_000;
const LUX_COEFC_INT: i64 = 590_000_000;
const LUX_COEFD_INT: i64 = 860_000_000;

const OVERFLOW_100MS: u16 = 36863;
const OVERFLOW_OTHER: u16 = 65535;

/// Calculate lux from raw channel data.
///
/// Returns `None` on overflow.
pub fn calculate_lux(
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

    let cpl = (a_time * a_gain) / LUX_DF;
    let lux1 = (ch_0 as f32 - (LUX_COEFB * ch_1 as f32)) / cpl;
    let lux2 = ((LUX_COEFC * ch_0 as f32) - (LUX_COEFD * ch_1 as f32)) / cpl;

    Some(f32::max(lux1, lux2))
}

/// Same as above, but uses integer math instead of floats.
/// Floating point math loses precision at the extremes (188 μlux) , so this is useful.
///
/// Returns `None` on overflow.
pub fn calculate_nano_lux(
    integration_time: IntegrationTime,
    gain: Gain,
    ch_0: u16,
    ch_1: u16,
) -> Option<i64> {
    if check_overflow(integration_time, ch_0, ch_1) {
        // Signal an overflow
        return None;
    }

    let a_time = integration_time.get_integration_time_millis() as i64;
    let a_gain = gain.get_multiplier() as i64;

    let ch_0 = ch_0 as i64 * INTEGER_CONVERSION_FACTOR;
    let ch_1 = ch_1 as i64 * INTEGER_CONVERSION_FACTOR;

    let cpl = (a_time * a_gain) / LUX_DF_INT;
    if cpl == 0 {
        return None; // avoid divide by zero
    }
    let lux1 = (ch_0 - (LUX_COEFB_INT * ch_1)) / cpl;
    let lux2 = ((LUX_COEFC_INT * ch_0) - (LUX_COEFD_INT * ch_1)) / cpl;

    Some(i64::max(lux1, lux2))
}

fn check_overflow(integration_time: IntegrationTime, ch_0: u16, ch_1: u16) -> bool {
    let overflow_value = if let IntegrationTime::_100MS = integration_time {
        OVERFLOW_100MS
    } else {
        OVERFLOW_OTHER
    };

    (ch_0 >= overflow_value) | (ch_1 >= overflow_value)
}
