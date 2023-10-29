//! A WIP rust implementation of the TSL2591 lux sensor
//! Most of what's here is a straight port of the [Adafruit C++
//! library](https://github.com/adafruit/Adafruit_TSL2591_Library)
//!
//! - [x] Basic reading and lux calculation
//! - [ ] Interrupt support
//!
//! # Example
//! ```
//! t.enable().unwrap();
//! t.set_timing(None).unwrap();
//! t.set_gain(None).unwrap();
//! loop {
//!     let (ch_0, ch_1) = t.get_channel_data(&mut delay).unwrap();
//!     let test = t.calculate_lux(ch_0, ch_1).unwrap();
//!                                                                 
//!     iprintln!(&mut cp.ITM.stim[0], "{}", test);
//!                                                                 
//! }
//! ```

#![no_std]

mod chip;
mod error;
mod lux_conversion;
mod sensor_impl;
mod types;

pub use error::Error;
pub use lux_conversion::{
    check_overflow, AdafruitPythonLuxConverter, LuxConverter, YoctoLuxConverter,
};
pub use sensor_impl::Tsl2591;
pub use types::{Enable, Gain, IntegrationTime, Mode, Status};

#[cfg(not(any(feature = "blocking", feature = "async")))]
compile_error!("You must enable exactly one of the following features: `blocking`, `async`");
#[cfg(all(feature = "blocking", feature = "async"))]
compile_error!("You must enable exactly one of the following features: `blocking`, `async`");
