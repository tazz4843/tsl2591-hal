#[derive(Clone, Copy, Debug)]
pub enum Error<I> {
    I2c(I),
    IdMismatch(u8),
    SignalOverflow,
    InfraredOverflow,
}

impl<I> From<I> for Error<I> {
    fn from(error: I) -> Self {
        Error::I2c(error)
    }
}
