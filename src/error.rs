#[derive(Debug)]
pub enum CC1101Error<E> {
    GenericProblem,
    SpiError(E),
}

impl<E> From<E> for CC1101Error<E> {
    fn from(value: E) -> Self {
        Self::SpiError(value)
    }
}
