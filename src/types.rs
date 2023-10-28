#![allow(non_snake_case)]

use bitfield::bitfield;

pub enum Mode {
    Infrared,
    Visible,
    FullSpectrum,
}

#[derive(Clone, Copy)]
pub enum IntegrationTime {
    _100MS = 0x00, // 100
    _200MS = 0x01, // 200 millis
    _300MS = 0x02, // 300 millis
    _400MS = 0x03, // 400 millis
    _500MS = 0x04, // 500 millis
    _600MS = 0x05, // 600 millis
}

impl IntegrationTime {
    pub fn get_integration_time_millis(&self) -> u32 {
        match self {
            Self::_100MS => 100,
            Self::_200MS => 200,
            Self::_300MS => 300,
            Self::_400MS => 400,
            Self::_500MS => 500,
            Self::_600MS => 600,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Gain {
    Low = 0x00,  // low gain (1x)
    Med = 0x10,  // medium gain (25x)
    High = 0x20, // medium gain (428x)
    Max = 0x30,  // max gain (9876x)
}

impl Gain {
    pub fn get_multiplier(&self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Med => 25,
            Self::High => 428,
            Self::Max => 9876,
        }
    }
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
    #[allow(non_snake_case)]
    pub struct Status(u8);
    impl Debug;
    pub NPINTR,_: 6,4;
    pub AINT, _: 4;
    pub RES, _: 3,3;
    pub AVALID, _: 0;
}
