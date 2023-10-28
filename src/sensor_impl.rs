use crate::{
    chip,
    error::Error,
    types::{Enable, Gain, IntegrationTime, Mode, Status},
};
use core::marker::PhantomData;
#[cfg(feature = "blocking")]
use embedded_hal::{
    delay::DelayUs,
    i2c::{I2c, SevenBitAddress},
};
#[cfg(feature = "async")]
use embedded_hal_async::{
    delay::DelayUs,
    i2c::{I2c, SevenBitAddress},
};

pub struct Tsl2591<I, D> {
    i2c: I,
    integration_time: IntegrationTime,
    gain: Gain,
    delay: PhantomData<D>,
}

#[cfg(feature = "blocking")]
impl<I2C, I2cError, Delay> Tsl2591<I2C, Delay>
where
    I2C: I2c<SevenBitAddress, Error = I2cError>,
    Delay: DelayUs,
{
    pub fn new(i2c: I2C) -> Result<Tsl2591<I2C, Delay>, Error<I2cError>> {
        let mut driver = Tsl2591 {
            i2c,
            integration_time: IntegrationTime::_200MS,
            gain: Gain::Low,
            delay: PhantomData,
        };
        let id = driver.get_id()?;
        if id != chip::ID {
            return Err(Error::IdMismatch(id));
        }
        Ok(driver)
    }

    pub fn new_define_integration(
        i2c: I2C,
        integration_time: IntegrationTime,
        gain: Gain,
    ) -> Result<Tsl2591<I2C, Delay>, Error<I2cError>> {
        let mut driver = Tsl2591 {
            i2c,
            integration_time,
            gain,
            delay: PhantomData,
        };
        let id = driver.get_id()?;
        if id != chip::ID {
            return Err(Error::IdMismatch(id));
        }
        Ok(driver)
    }

    fn get_id(&mut self) -> Result<u8, Error<I2cError>> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(chip::I2C, &[chip::COMMAND_BIT | chip::ID_ADDR], &mut buffer)?;
        Ok(buffer[0])
    }

    pub fn set_gain(&mut self, gain: Option<Gain>) -> Result<(), Error<I2cError>> {
        if let Some(gain) = gain {
            self.i2c.write(
                0x29,
                &[
                    chip::COMMAND_BIT | chip::CONTROL,
                    self.integration_time as u8 | gain as u8,
                ],
            )?;
        } else {
            self.i2c.write(
                0x29,
                &[
                    chip::COMMAND_BIT | chip::CONTROL,
                    self.integration_time as u8 | self.gain as u8,
                ],
            )?;
        }
        Ok(())
    }

    pub fn set_timing(
        &mut self,
        integration_time: Option<IntegrationTime>,
    ) -> Result<(), Error<I2cError>> {
        if let Some(integration_time) = integration_time {
            self.i2c.write(
                0x29,
                &[
                    chip::COMMAND_BIT | chip::CONTROL,
                    integration_time as u8 | self.gain as u8,
                ],
            )?;
        } else {
            self.i2c.write(
                0x29,
                &[
                    chip::COMMAND_BIT | chip::CONTROL,
                    self.integration_time as u8 | self.gain as u8,
                ],
            )?;
        }
        Ok(())
    }

    pub fn disable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(
            0x29,
            &[chip::COMMAND_BIT | chip::ENABLE, chip::ENABLE_POWEROFF],
        )?;
        Ok(())
    }

    pub fn enable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c.write(
            0x29,
            &[
                chip::COMMAND_BIT | chip::ENABLE,
                chip::ENABLE_POWERON | chip::ENABLE_AEN | chip::ENABLE_AIEN | chip::ENABLE_NPIEN,
            ],
        )?;
        Ok(())
    }

    pub fn get_enable(&mut self) -> Result<Enable, Error<I2cError>> {
        let mut status = [0u8; 1];
        self.i2c
            .write_read(chip::I2C, &[chip::COMMAND_BIT | chip::ENABLE], &mut status)?;
        Ok(Enable(status[0]))
    }

    pub fn get_status(&mut self) -> Result<Status, Error<I2cError>> {
        let mut status = [0u8; 1];
        self.i2c
            .write_read(chip::I2C, &[chip::COMMAND_BIT | chip::STATUS], &mut status)?;
        Ok(Status(status[0]))
    }

    pub fn get_channel_data(&mut self, delay: &mut Delay) -> Result<(u16, u16), Error<I2cError>> {
        delay.delay_ms(120);
        let mut buffer_1 = [0u8; 2];
        let mut buffer_2 = [0u8; 2];
        self.i2c.write_read(
            chip::I2C,
            &[chip::COMMAND_BIT | chip::CHAN0_LOW],
            &mut buffer_1,
        )?;
        self.i2c.write_read(
            chip::I2C,
            &[chip::COMMAND_BIT | chip::CHAN1_LOW],
            &mut buffer_2,
        )?;
        let channel_0 = ((buffer_1[0] as u16) << 8) | buffer_1[1] as u16;
        let channel_1 = ((buffer_2[0] as u16) << 8) | buffer_2[1] as u16;
        Ok((channel_0, channel_1))
    }

    pub fn get_luminosity(
        &mut self,
        mode: Mode,
        delay: &mut Delay,
    ) -> Result<u16, Error<I2cError>> {
        let (channel_0, channel_1) = self.get_channel_data(delay)?;
        let full_luminosity: u32 = ((channel_1 as u32) << 16) | channel_0 as u32;

        match mode {
            Mode::FullSpectrum => Ok((full_luminosity & 0xFFFF) as u16),
            Mode::Infrared => Ok((full_luminosity >> 16) as u16),
            Mode::Visible => {
                let infrared_and_visible = full_luminosity & 0xFFFF;
                let infrared = full_luminosity >> 16;
                if infrared > infrared_and_visible {
                    Err(Error::InfraredOverflow)
                } else {
                    Ok((infrared_and_visible - infrared) as u16)
                }
            }
        }
    }

    pub fn calculate_lux(&mut self, ch_0: u16, ch_1: u16) -> Result<f32, Error<I2cError>> {
        if (ch_0 == 0xFFFF) | (ch_1 == 0xFFFF) {
            // Signal an overflow
            return Err(Error::SignalOverflow);
        }

        let a_time = self.integration_time.get_integration_time_millis() as f32;
        let a_gain = self.gain.get_multiplier() as f32;
        let cpl = (a_time * a_gain) / 408.0;
        let lux = (ch_0 as f32 - ch_1 as f32) * (1.0 - (ch_1 as f32 / ch_0 as f32)) / cpl;

        Ok(lux)
    }
}

#[cfg(feature = "async")]
impl<I2C, I2cError, Delay> Tsl2591<I2C, Delay>
where
    I2C: I2c<SevenBitAddress, Error = I2cError>,
    Delay: DelayUs,
{
    pub async fn new(i2c: I2C) -> Result<Tsl2591<I2C, Delay>, Error<I2cError>> {
        let mut driver = Tsl2591 {
            i2c,
            integration_time: IntegrationTime::_200MS,
            gain: Gain::Low,
            delay: PhantomData,
        };
        let id = driver.get_id().await?;
        if id != chip::ID {
            return Err(Error::IdMismatch(id));
        }
        Ok(driver)
    }

    pub async fn new_define_integration(
        i2c: I2C,
        integration_time: IntegrationTime,
        gain: Gain,
    ) -> Result<Tsl2591<I2C, Delay>, Error<I2cError>> {
        let mut driver = Tsl2591 {
            i2c,
            integration_time,
            gain,
            delay: PhantomData,
        };
        let id = driver.get_id().await?;
        if id != chip::ID {
            return Err(Error::IdMismatch(id));
        }
        Ok(driver)
    }

    async fn get_id(&mut self) -> Result<u8, Error<I2cError>> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(chip::I2C, &[chip::COMMAND_BIT | chip::ID_ADDR], &mut buffer)
            .await?;
        Ok(buffer[0])
    }

    pub async fn set_gain(&mut self, gain: Option<Gain>) -> Result<(), Error<I2cError>> {
        if let Some(gain) = gain {
            self.i2c
                .write(
                    0x29,
                    &[
                        chip::COMMAND_BIT | chip::CONTROL,
                        self.integration_time as u8 | gain as u8,
                    ],
                )
                .await?;
        } else {
            self.i2c
                .write(
                    0x29,
                    &[
                        chip::COMMAND_BIT | chip::CONTROL,
                        self.integration_time as u8 | self.gain as u8,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn set_timing(
        &mut self,
        integration_time: Option<IntegrationTime>,
    ) -> Result<(), Error<I2cError>> {
        if let Some(integration_time) = integration_time {
            self.i2c
                .write(
                    0x29,
                    &[
                        chip::COMMAND_BIT | chip::CONTROL,
                        integration_time as u8 | self.gain as u8,
                    ],
                )
                .await?;
        } else {
            self.i2c
                .write(
                    0x29,
                    &[
                        chip::COMMAND_BIT | chip::CONTROL,
                        self.integration_time as u8 | self.gain as u8,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn disable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c
            .write(
                0x29,
                &[chip::COMMAND_BIT | chip::ENABLE, chip::ENABLE_POWEROFF],
            )
            .await?;
        Ok(())
    }

    pub async fn enable(&mut self) -> Result<(), Error<I2cError>> {
        self.i2c
            .write(
                0x29,
                &[
                    chip::COMMAND_BIT | chip::ENABLE,
                    chip::ENABLE_POWERON
                        | chip::ENABLE_AEN
                        | chip::ENABLE_AIEN
                        | chip::ENABLE_NPIEN,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn get_enable(&mut self) -> Result<Enable, Error<I2cError>> {
        let mut status = [0u8; 1];
        self.i2c
            .write_read(chip::I2C, &[chip::COMMAND_BIT | chip::ENABLE], &mut status)
            .await?;
        Ok(Enable(status[0]))
    }

    pub async fn get_status(&mut self) -> Result<Status, Error<I2cError>> {
        let mut status = [0u8; 1];
        self.i2c
            .write_read(chip::I2C, &[chip::COMMAND_BIT | chip::STATUS], &mut status)
            .await?;
        Ok(Status(status[0]))
    }

    pub async fn get_channel_data(
        &mut self,
        delay: &mut Delay,
    ) -> Result<(u16, u16), Error<I2cError>> {
        delay.delay_ms(120).await;
        let mut buffer_1 = [0u8; 2];
        let mut buffer_2 = [0u8; 2];
        self.i2c
            .write_read(
                chip::I2C,
                &[chip::COMMAND_BIT | chip::CHAN0_LOW],
                &mut buffer_1,
            )
            .await?;
        self.i2c
            .write_read(
                chip::I2C,
                &[chip::COMMAND_BIT | chip::CHAN1_LOW],
                &mut buffer_2,
            )
            .await?;
        let channel_0 = ((buffer_1[0] as u16) << 8) | buffer_1[1] as u16;
        let channel_1 = ((buffer_2[0] as u16) << 8) | buffer_2[1] as u16;
        Ok((channel_0, channel_1))
    }

    pub async fn get_luminosity(
        &mut self,
        mode: Mode,
        delay: &mut Delay,
    ) -> Result<u16, Error<I2cError>> {
        let (channel_0, channel_1) = self.get_channel_data(delay).await?;
        let full_luminosity: u32 = ((channel_1 as u32) << 16) | channel_0 as u32;

        match mode {
            Mode::FullSpectrum => Ok((full_luminosity & 0xFFFF) as u16),
            Mode::Infrared => Ok((full_luminosity >> 16) as u16),
            Mode::Visible => {
                let infrared_and_visible = full_luminosity & 0xFFFF;
                let infrared = full_luminosity >> 16;
                if infrared > infrared_and_visible {
                    Err(Error::InfraredOverflow)
                } else {
                    Ok((infrared_and_visible - infrared) as u16)
                }
            }
        }
    }

    pub fn calculate_lux(&mut self, ch_0: u16, ch_1: u16) -> Result<f32, Error<I2cError>> {
        if (ch_0 == 0xFFFF) | (ch_1 == 0xFFFF) {
            // Signal an overflow
            return Err(Error::SignalOverflow);
        }

        let a_time = self.integration_time.get_integration_time_millis() as f32;
        let a_gain = self.gain.get_multiplier() as f32;
        let cpl = (a_time * a_gain) / 408.0;
        let lux = (ch_0 as f32 - ch_1 as f32) * (1.0 - (ch_1 as f32 / ch_0 as f32)) / cpl;

        Ok(lux)
    }
}
