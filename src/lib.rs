#![no_std]
use chip::{ENABLE_POWEROFF, GAIN_LOW, INTEGRATIONTIME_100MS};
use embedded_hal::blocking::i2c::{ Write, WriteRead };

pub struct Driver<I2C> {
    i2c: I2C,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::I2cError(error)
    }
}

impl<I2C, I2cError> Driver<I2C>
where
    I2C: WriteRead<Error = I2cError>,
    I2C: Write<Error = I2cError>
{
    pub fn new(i2c: I2C) -> Result<Driver<I2C>, Error<I2cError>> {
        let mut driver = Driver {
            i2c,
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

    pub fn set_gain(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_CONTROL, INTEGRATIONTIME_100MS | GAIN_LOW])?;
        Ok(())
    }

    pub fn set_timing(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_CONTROL, INTEGRATIONTIME_100MS | GAIN_LOW])?;
        Ok(())
    }

    pub fn disable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_ENABLE, ENABLE_POWEROFF])?;
        Ok(())
    }

    pub fn enable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(0x29, &[chip::COMMAND_BIT | chip::REGISTER_ENABLE,
            chip::ENABLE_POWERON | chip::ENABLE_AEN | chip::ENABLE_AIEN | chip::ENABLE_NPIEN])?;
        Ok(())
    }

    pub fn get_channel_data(&mut self) -> Result<([u8; 2], [u8; 2]), Error<I2cError>> {
        let mut channel_1 = [0u8; 2];
        let mut channel_2= [0u8; 2];
        self.i2c.write_read(chip::I2C, &[chip::COMMAND_BIT | chip::REGISTER_CHAN0_LOW], &mut channel_1)?;
        self.i2c.write_read(chip::I2C, &[chip::COMMAND_BIT | chip::REGISTER_CHAN1_LOW], &mut channel_2)?;
        Ok((channel_1, channel_2))
    }
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
    pub const INTEGRATIONTIME_100MS: u8 = 0x00;
    pub const GAIN_LOW: u8 = 0x00;
}

#[derive(Clone, Copy, Debug)]
pub enum Error<I2cError> {
    I2cError(I2cError),
    IdMismatch(u8)
}

// TODO: Need to learn how to use this macro
// bitfield!{
//     pub struct Status(u8),
//     impl Debug;
//     pub calibrate, _: 7;
// }
