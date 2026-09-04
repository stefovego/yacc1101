#[derive(Debug)]
pub enum CC1101Error<SpiE, PinE> {
    GenericProblem,
    /// The GDO pin never reached the expected level within the requested timeout.
    Timeout,
    Spi(SpiE),
    Gpio(PinE),
}

impl<SpiE, PinE> From<SpiE> for CC1101Error<SpiE, PinE> {
    fn from(value: SpiE) -> Self {
        Self::Spi(value)
    }
}
