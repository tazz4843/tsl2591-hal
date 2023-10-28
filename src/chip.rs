#![allow(dead_code)]

pub const I2C: u8 = 0x29;
pub const ID: u8 = 0x50;
pub const ID_ADDR: u8 = 0x12;
pub const COMMAND_BIT: u8 = 0xA0;
pub const ENABLE_POWERON: u8 = 0x01;
pub const ENABLE_AEN: u8 = 0x02;
pub const ENABLE_AIEN: u8 = 0x10;
pub const ENABLE_POWEROFF: u8 = 0x00;
pub const ENABLE_NPIEN: u8 = 0x80;
pub const CHAN0_LOW: u8 = 0x14;
pub const CHAN1_LOW: u8 = 0x16;
pub const ENABLE: u8 = 0x00;
pub const CONTROL: u8 = 0x01;
pub const STATUS: u8 = 0x13;
pub const TSL2591_THRESHOLD_AILTH: u8 = 0x05; // ALS low threshold upper byte
pub const TSL2591_THRESHOLD_AIHTL: u8 = 0x06; // ALS high threshold lower byte
pub const TSL2591_THRESHOLD_AIHTH: u8 = 0x07; // ALS high threshold upper byte
pub const TSL2591_THRESHOLD_NPAILTL: u8 = 0x08; // No Persist ALS low threshold lower byte
pub const TSL2591_THRESHOLD_NPAILTH: u8 = 0x09; // No Persist ALS low threshold higher byte
pub const TSL2591_THRESHOLD_NPAIHTL: u8 = 0x0A; // No Persist ALS high threshold lower byte
pub const TSL2591_THRESHOLD_NPAIHTH: u8 = 0x0B; // No Persist ALS high threshold higher byte
pub const TSL2591_PERSIST_FILTER: u8 = 0x0C; // Interrupt persistence filter
