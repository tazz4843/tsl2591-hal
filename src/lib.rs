#![no_std]
use embedded_hal::blocking::{delay::DelayMs, i2c::{Write, WriteRead}};
use bitfield::bitfield;
use core::num::Wrapping;

pub struct Driver<I2C> {
    i2c: I2C,
    integration_time: IntegrationTimes,
    gain: Gain

}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::I2cError(error)
    }
}

impl<I2C, I2cError> Driver<I2C>
where
    I2C: WriteRead<Error = I2cError> + Write<Error = I2cError>
{
    pub fn new(i2c: I2C) -> Result<Driver<I2C>, Error<I2cError>> {
        let mut driver = Driver {
            i2c,
            integration_time: IntegrationTimes::_200MS,
            gain: Gain::LOW
        };
        let id = driver.get_id()?;
        if id != chip::ID {
            return Err(Error::IdMismatch(id));
        }
        Ok(driver)
    }

    pub fn new_define_integration(i2c: I2C, integration_time: IntegrationTimes, gain: Gain) -> Result<Driver<I2C>, Error<I2cError>> {
        let mut driver = Driver {
            i2c,
            integration_time,
            gain

        };
        let id = driver.get_id()?;
        if id != chip::ID {
            return Err(Error::IdMismatch(id));
        }
        Ok(driver)
    }

    fn get_id(&mut self) -> Result<u8, Error<I2cError>> {
        let mut buffer = [0u8; 1];
        self.i2c.write_read(chip::I2C, &[chip::COMMAND_BIT | chip::ID_ADDR], &mut buffer)?;
        Ok(buffer[0])
    }

    pub fn set_gain(&mut self, gain: Option<Gain>) -> Result<(), Error<I2cError>> {
        if let Some(gain) = gain {
            self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_CONTROL, self.integration_time as u8 | gain as u8])?;
        } else {
            self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_CONTROL, self.integration_time as u8 | self.gain as u8])?;
        }
        Ok(())
    }

    pub fn set_timing(&mut self, integration_time: Option<Gain>) -> Result<(), Error<I2cError>> {
        if let Some(integration_time) = integration_time {
            self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_CONTROL, integration_time as u8| self.gain as u8])?;
        } else {
            self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_CONTROL, self.integration_time as u8 | self.gain as u8])?;
        }
        Ok(())
    }

    pub fn disable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_ENABLE, chip::ENABLE_POWEROFF])?;
        Ok(())
    }

    pub fn enable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_ENABLE,
            chip::ENABLE_POWERON | chip::ENABLE_AEN | chip::ENABLE_AIEN | chip::ENABLE_NPIEN])?;
        Ok(())
    }

    pub fn get_enable(&mut self) -> Result<Enable, Error<I2cError>> {
        let mut status = [0u8; 1];
        self.i2c.write_read(chip::I2C, &[chip::COMMAND_BIT | chip::REGISTER_ENABLE], &mut status)?;
        Ok(Enable(status[0]))
    }

    pub fn get_status(&mut self) -> Result<Status, Error<I2cError>> {
        let mut status = [0u8; 1];
        self.i2c.write_read(chip::I2C, &[chip::COMMAND_BIT | chip::REGISTER_STATUS], &mut status)?;
        Ok(Status(status[0]))
    }


    pub fn get_channel_data<D: DelayMs<u8>>(&mut self, delay: &mut D) -> Result<(u16, u16), Error<I2cError>> {
        self.enable()?;
        delay.delay_ms(240);
        let mut buffer_1 = [0u8; 2];
        let mut buffer_2 = [0u8; 2];
        self.i2c.write_read(chip::I2C, &[chip::COMMAND_BIT | chip::REGISTER_CHAN0_LOW], &mut buffer_1)?;
        self.i2c.write_read(chip::I2C, &[chip::COMMAND_BIT | chip::REGISTER_CHAN1_LOW], &mut buffer_2)?;
        let channel_0 = ((buffer_1[0] as u16) << 8) | buffer_1[1] as u16;
        let channel_1 = ((buffer_2[0] as u16) << 8) | buffer_2[1] as u16;
        self.disable()?;
        Ok((channel_0, channel_1))
    }

    pub fn get_luminosity<D: DelayMs<u8>>(&mut self, mode: Mode, delay: &mut D) -> Result<u16, Error<I2cError>> {
        let (channel_0, channel_1) = self.get_channel_data(delay)?;
        let full_luminosity: u32 = ((channel_1 as u32) << 16 ) | channel_0 as u32;

        match mode {
            Mode::FullSpectrum=> {
                Ok(( full_luminosity & 0xFFFF ) as u16)
            }
            Mode::Infrared => {
                Ok((full_luminosity >> 16) as u16)
            }
            Mode::Visible => {
                let x = Wrapping(full_luminosity & 0xFFFF);
                let y = Wrapping(full_luminosity >> 16);
                Ok((x - y).0 as u16)
            }
        }
    }

    pub fn calculate_lux(&mut self, ch_0: u16, ch_1: u16) -> Result<f32, Error<I2cError>> {
        let a_time = match self.integration_time {
            IntegrationTimes::_100MS => {
                100.
            }
            IntegrationTimes::_200MS => {
                200.
            }
            IntegrationTimes::_300MS => {
                300.
            }
            IntegrationTimes::_400MS => {
                400.
            }
            IntegrationTimes::_500MS => {
                500.
            }
            IntegrationTimes::_600MS => {
                600.
            }
        };

        let a_gain =  match self.gain {
            Gain::LOW => {
                1.0
            }
            Gain::MED => {
                25.
            }
            Gain::HIGH => {
                428.
            }
            Gain::MAX => {
                9876.
            }
        };

        if (ch_0 == 0xFFFF) | (ch_1 == 0xFFFF) {
            // Signal an overflow
            return Err(Error::SignalOverflow());
        }

        let cpl = (a_time * a_gain) / 408.0;

        let lux = ((ch_0 as f32 - ch_1 as f32)) * (1.0 - (ch_1 as f32  / ch_0 as f32)) / cpl;

        Ok(lux)
    }
}

pub enum Mode {
    Infrared,
    Visible,
    FullSpectrum,
}

#[derive(Clone, Copy)]
pub enum IntegrationTimes {
    _100MS = 0x00, // 100 
    _200MS = 0x01, // 200 millis
    _300MS = 0x02, // 300 millis
    _400MS = 0x03, // 400 millis
    _500MS = 0x04, // 500 millis
    _600MS = 0x05 // 600 millis
}

#[derive(Clone, Copy)]
pub enum Gain {
  LOW = 0x00,  // low gain (1x)
  MED = 0x10,  // medium gain (25x)
  HIGH = 0x20, // medium gain (428x)
  MAX= 0x30  // max gain (9876x)
}


mod chip {
    pub const I2C: u8 = 0x29;
    pub const ID: u8 = 0x50;
    pub const ID_ADDR: u8 = 0x12;
    pub const COMMAND_BIT: u8 = 0xA0;
    pub const ENABLE_POWERON: u8 = 0x01;
    pub const ENABLE_AEN: u8 = 0x02;
    pub const ENABLE_AIEN: u8 =  0x10;
    pub const ENABLE_POWEROFF: u8 = 0x00;
    pub const ENABLE_NPIEN: u8 = 0x80;
    pub const REGISTER_CHAN0_LOW: u8 =  0x14;
    pub const REGISTER_CHAN1_LOW: u8 = 0x16;
    pub const REGISTER_ENABLE: u8 = 0x00;
    pub const REGISTER_CONTROL: u8 = 0x01;
    pub const REGISTER_STATUS: u8 = 0x13;
}

#[derive(Clone, Copy, Debug)]
pub enum Error<I2cError> {
    I2cError(I2cError),
    IdMismatch(u8),
    SignalOverflow()
}

bitfield! {
    pub struct Enable(u8);
    impl Debug;
    pub NPIEN, _: 6;
    pub SAI, _: 5;
    pub RES, _: 4,3;
    pub AEN, _: 1;
    pub PON, _: 0;
}

bitfield! {
    pub struct Status(u8);
    impl Debug;
    pub NPINTR,_: 6,4;
    pub AINT, _: 4;
    pub RES, _: 3,3;
    pub AVALID, _: 0;
}
