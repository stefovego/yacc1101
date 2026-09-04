#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    SLEEP = 0x00,
    #[default]
    IDLE = 0x01,
    XOFF = 0x02,
    VCOON_MC = 0x03,
    REGON_MC = 0x04,
    MANCAL = 0x05,
    VCOON = 0x06,
    REGON = 0x07,
    STARTCAL = 0x08,
    BWBOOST = 0x09,
    FS_LOCK = 0x0A,
    IFADCON = 0x0B,
    ENDCAL = 0x0C,
    RX = 0x0D,
    RX_END = 0x0E,
    RX_RST = 0x0F,
    TXRX_SWITCH = 0x10,
    RXFIFO_OVERFLOW = 0x11,
    FSTXON = 0x12,
    TX = 0x13,
    TX_END = 0x14,
    RXTX_SWITCH = 0x15,
    TXFIFO_UNDERFLOW = 0x16,
    /// MARCSTATE reported a value outside the documented 0x00-0x16 range.
    Unknown,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            0x00 => State::SLEEP,
            0x01 => State::IDLE,
            0x02 => State::XOFF,
            0x03 => State::VCOON_MC,
            0x04 => State::REGON_MC,
            0x05 => State::MANCAL,
            0x06 => State::VCOON,
            0x07 => State::REGON,
            0x08 => State::STARTCAL,
            0x09 => State::BWBOOST,
            0x0A => State::FS_LOCK,
            0x0B => State::IFADCON,
            0x0C => State::ENDCAL,
            0x0D => State::RX,
            0x0E => State::RX_END,
            0x0F => State::RX_RST,
            0x10 => State::TXRX_SWITCH,
            0x11 => State::RXFIFO_OVERFLOW,
            0x12 => State::FSTXON,
            0x13 => State::TX,
            0x14 => State::TX_END,
            0x15 => State::RXTX_SWITCH,
            0x16 => State::TXFIFO_UNDERFLOW,
            _ => State::Unknown,
        }
    }
}
