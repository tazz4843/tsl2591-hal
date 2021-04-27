#![no_std]
#![no_main]

//! Example usage for ADC on STM32F303
use cortex_m::asm::delay;
use panic_halt as _;
use core::ops::Range;
pub use cortex_m::{asm::bkpt, iprint, iprintln};
use cortex_m_rt::entry;
use cortex_m_semihosting::{ hprintln, hprint };
use stm32f3xx_hal::pac::I2C1;

const VALID_ADDR_RANGE: Range<u8> = 0x08..0x78;

use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

const TSL2591_COMMAND_BIT: u8 = 0xA0;
const TSL2591_REGISTER_ENABLE: u8 = 0x00;
const TSL2591_ENABLE_POWERON: u8 = 0x01;
const TSL2591_ENABLE_AEN: u8 = 0x01;
const TSL2591_ENABLE_AIEN: u8 =  0x10;
const TSL2591_ENABLE_NPIEN: u8 = 0x80;
const TSL2591_REGISTER_CHAN0_LOW: u8 =  0x14;
const TSL2591_REGISTER_CHAN1_LOW: u8 = 0x16;
const TSL2591_ENABLE_POWEROFF: u8 = 0x00;
const TSL2591_REGISTER_CONTROL: u8 = 0x01;
const TSL2591_INTEGRATIONTIME_100MS: u8 = 0x0;
const TSL2591_GAIN_LOW: u8 = 0x0;

pub fn read_register(i2c: &mut I2c<I2C1, (stm32f3xx_hal::gpio::gpiob::PB6<stm32f3xx_hal::gpio::AF4>, stm32f3xx_hal::gpio::gpiob::PB7<stm32f3xx_hal::gpio::AF4>)>, addr: u8) -> [u8; 16] {
    let mut buffer = [0u8; 16];
    match i2c.write(0x29, &[TSL2591_REGISTER_ENABLE, TSL2591_ENABLE_POWERON | TSL2591_ENABLE_AEN]) {
        Ok(_) => {
            hprintln!("ENABLED!").unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };
    match i2c.write_read(0x29, &[addr], &mut buffer) {
        Ok(_) => {
            hprintln!("Read {:x} OK!", addr).unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };
    return buffer;
}

pub fn enable(i2c: &mut I2c<I2C1, (stm32f3xx_hal::gpio::gpiob::PB6<stm32f3xx_hal::gpio::AF4>, stm32f3xx_hal::gpio::gpiob::PB7<stm32f3xx_hal::gpio::AF4>)>) {
    match i2c.write(0x29, &[0x00, TSL2591_REGISTER_ENABLE,
         TSL2591_ENABLE_POWERON | TSL2591_ENABLE_AEN | TSL2591_ENABLE_AIEN |
             TSL2591_ENABLE_NPIEN]) {
        Ok(_) => {
            hprintln!("ENABLED!").unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };

    match i2c.write(0x29, &[TSL2591_REGISTER_CONTROL, TSL2591_INTEGRATIONTIME_100MS | TSL2591_GAIN_LOW]) {
        Ok(_) => {
            hprintln!("GAIN SET").unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };
}

pub fn disable(i2c: &mut I2c<I2C1, (stm32f3xx_hal::gpio::gpiob::PB6<stm32f3xx_hal::gpio::AF4>, stm32f3xx_hal::gpio::gpiob::PB7<stm32f3xx_hal::gpio::AF4>)>) {
    match i2c.write(0x29<<0, &[0x00, TSL2591_REGISTER_ENABLE, TSL2591_ENABLE_POWEROFF]) {
        Ok(_) => {
            hprintln!("ENABLED!").unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };

    match i2c.write(0x29<<0, &[TSL2591_COMMAND_BIT| TSL2591_REGISTER_CONTROL, TSL2591_INTEGRATIONTIME_100MS | TSL2591_GAIN_LOW]) {
        Ok(_) => {
            hprintln!("GAIN SET").unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };
}

#[entry]
/// Main Thread
fn main() -> ! {
    // Get peripherals, clocks and freeze them
    // let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut dp.FLASH.constrain().acr);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let mut i2c = I2c::new(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);
    let mut buffer = [0u8; 1];
    match i2c.write(0x29<<0, &[0x00]) {
        Ok(_) => {
            hprintln!("Ok!").unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };
    hprintln!("{:?}", buffer).unwrap();
    match i2c.write_read(0x29<<0, &[0x11], &mut buffer) {
        Ok(val) => {
            hprintln!("{:?}", val).unwrap();
        }
        Err(e) => {
            hprintln!("{:?}", e).unwrap();
        }
    };

    enable(&mut i2c);
    delay(10000000);
    disable(&mut i2c);

    let y = read_register(&mut i2c, TSL2591_REGISTER_CHAN0_LOW);
    let x = read_register(&mut i2c, TSL2591_REGISTER_CHAN1_LOW);

    hprintln!("x: {:?}", x).unwrap();
    hprintln!("y: {:?}", y).unwrap();
    // read_register("Control", &mut i2c, 0x01);
    // read_register("Device ID", &mut i2c, 0x12);
    // read_register("Status", &mut i2c, 0x13);
    // read_register("C0DATAL", &mut i2c, 0x14);
    // read_register("C0DATAH", &mut i2c, 0x15);
    // read_register("C1DATAL", &mut i2c, 0x16);
    // read_register("C1DATAH", &mut i2c, 0x17);


    // for addr in 0x00_u8..0x80 {
    //     // Write the empty array and check the slave response.
    //     if VALID_ADDR_RANGE.contains(&addr) && i2c.write(addr, &[]).is_ok() {
    //         hprint!("{:02x}", addr).unwrap();
    //     } else {
    //         hprint!("..").unwrap();
    //     }
    //     if addr % 0x10 == 0x0F {
    //         hprintln!().unwrap();
    //     } else {
    //         hprint!(" ").unwrap();
    //     }
    // }

    loop {
    }
}


