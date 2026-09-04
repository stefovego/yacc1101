use embedded_hal;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use embedded_hal::spi::Operation;

use crate::Regs;
use crate::State;
use crate::error::CC1101Error;

const WRITE_SINGLE_BYTE: u8 = 0x00;
const WRITE_BURST: u8 = 0x40;
const READ_SINGLE_BYTE: u8 = 0x80;
const READ_BURST: u8 = 0xC0;
const PA_TABLE: [u8; 8] = [0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

pub struct CC1101<SPI, GDOPin, DelayT> {
    spi: SPI,
    gdo: GDOPin,
    delay: DelayT,
}

impl<SPI, GDOPin, DelayT, SpiE, PinE> CC1101<SPI, GDOPin, DelayT>
where
    SPI: embedded_hal::spi::SpiDevice<Error = SpiE>,
    GDOPin: InputPin<Error = PinE>,
    DelayT: DelayNs,
{
    pub fn new(spi: SPI, gdo: GDOPin, delay: DelayT) -> Self {
        Self { spi, gdo, delay }
    }

    pub fn read_register(&mut self, register: Regs) -> Result<u8, CC1101Error<SpiE, PinE>> {
        let mut read_buffer = [0_u8];
        self.spi.transaction(&mut [
            Operation::Write(&[READ_SINGLE_BYTE | register as u8]),
            Operation::Read(&mut read_buffer),
        ])?;
        Ok(read_buffer[0])
    }

    /// Current main radio control state machine state (MARCSTATE, bits 4:0).
    pub fn get_status(&mut self) -> Result<u8, CC1101Error<SpiE, PinE>> {
        Ok(self.read_register(Regs::MARCSTATE)? & 0x1F)
    }

    pub fn get_state(&mut self) -> Result<State, CC1101Error<SpiE, PinE>> {
        Ok(State::from(self.get_status()?))
    }

    /// Reads the PARTNUM and VERSION status registers, useful to confirm the chip is present.
    pub fn chip_id(&mut self) -> Result<(u8, u8), CC1101Error<SpiE, PinE>> {
        let part = self.read_register(Regs::PARTNUM)?;
        let version = self.read_register(Regs::VERSION)?;
        Ok((part, version))
    }

    /// Received Signal Strength Indicator, converted to dBm.
    pub fn get_rssi(&mut self) -> Result<i16, CC1101Error<SpiE, PinE>> {
        let raw = self.read_register(Regs::RSSI)? as i16;
        let rssi = if raw >= 128 {
            (raw - 256) / 2 - 74
        } else {
            raw / 2 - 74
        };
        Ok(rssi)
    }

    /// Link Quality Indicator for the last received packet (0-127, lower is better).
    pub fn get_lqi(&mut self) -> Result<u8, CC1101Error<SpiE, PinE>> {
        Ok(self.read_register(Regs::LQI)? & 0x7F)
    }

    pub fn init(&mut self, frequency: u32, offset: u32) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.reset()?;
        self.configure()?;
        self.set_frequency(frequency, offset)?;
        self.write_single_byte(Regs::SYNC1, 0x66)?;
        self.write_single_byte(Regs::SYNC0, 0x6A)?;
        Ok(())
    }

    /// Sets the carrier frequency in Hz, trimmed by `offset` Hz to compensate for crystal error.
    pub fn set_frequency(
        &mut self,
        frequency: u32,
        offset: u32,
    ) -> Result<(), CC1101Error<SpiE, PinE>> {
        let frequency = ((frequency + offset) as u64 * 2_u64.pow(16)) / 26_000_000;
        let byte2: u8 = (frequency as u32 >> 16) as u8 & 0xff;
        let byte1: u8 = (frequency as u16 >> 8) as u8 & 0xff;
        let byte0: u8 = frequency as u8 & 0xff;
        self.write_single_byte(Regs::FREQ2, byte2)?;
        self.write_single_byte(Regs::FREQ1, byte1)?;
        self.write_single_byte(Regs::FREQ0, byte0)?;
        self.strobe(Regs::SFTX)?; // flush TX FIFO
        self.strobe(Regs::SFRX)?; // flush RX FIFO
        Ok(())
    }

    fn strobe(&mut self, address: Regs) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.spi.write(&[address as u8])?;
        Ok(())
    }

    fn write_single_byte(
        &mut self,
        register: Regs,
        byte_data: u8,
    ) -> Result<(), CC1101Error<SpiE, PinE>> {
        let databuffer = [WRITE_SINGLE_BYTE | register as u8, byte_data];
        self.spi.write(&databuffer)?;
        Ok(())
    }

    fn write_burst(
        &mut self,
        register: Regs,
        data: &[u8],
    ) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.spi.transaction(&mut [
            Operation::Write(&[WRITE_BURST | register as u8]),
            Operation::Write(data),
        ])?;
        Ok(())
    }

    fn read_burst(
        &mut self,
        register: Regs,
        buffer: &mut [u8],
    ) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.spi.transaction(&mut [
            Operation::Write(&[READ_BURST | register as u8]),
            Operation::Read(buffer),
        ])?;
        Ok(())
    }

    fn write_register_field(
        &mut self,
        register: Regs,
        value: u8,
        msb: u8,
        lsb: u8,
    ) -> Result<(), CC1101Error<SpiE, PinE>> {
        let current = self.read_register(register)?;
        let width = msb - lsb + 1;
        let mask: u8 = (((1u16 << width) - 1) as u8) << lsb;
        let new_value = (current & !mask) | ((value << lsb) & mask);
        self.write_single_byte(register, new_value)?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.delay.delay_us(50);
        self.strobe(Regs::SRES)?;
        self.delay.delay_us(50);
        Ok(())
    }

    /// Loads a known-good default register set. Shared by `setup_tx` and `setup_rx` since the
    /// GDO0 configuration below (asserts on sync word, de-asserts at end of packet) works for
    /// both directions.
    fn configure(&mut self) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.write_single_byte(Regs::IOCFG2, 0x29)?;
        self.write_single_byte(Regs::IOCFG1, 0x2E)?;
        self.write_single_byte(Regs::IOCFG0, 0x06)?;
        self.write_single_byte(Regs::FIFOTHR, 0x47)?;
        self.write_single_byte(Regs::PKTCTRL1, 0x00)?;
        self.write_single_byte(Regs::PKTCTRL0, 0x00)?;
        self.write_single_byte(Regs::ADDR, 0x00)?;
        self.write_single_byte(Regs::CHANNR, 0x00)?;
        self.write_single_byte(Regs::FSCTRL1, 0x06)?;
        self.write_single_byte(Regs::FSCTRL0, 0x00)?;
        self.write_single_byte(Regs::MDMCFG4, 0xE9)?;
        self.write_single_byte(Regs::MDMCFG3, 0x42)?;
        self.write_single_byte(Regs::MDMCFG2, 0x30)?; // 32 would be 16/16 sync word bits
        self.write_single_byte(Regs::MDMCFG1, 0x22)?;
        self.write_single_byte(Regs::MDMCFG0, 0xF8)?;
        self.write_single_byte(Regs::DEVIATN, 0x15)?;
        self.write_single_byte(Regs::MCSM2, 0x07)?;
        self.write_single_byte(Regs::MCSM1, 0x20)?;
        self.write_single_byte(Regs::MCSM0, 0x18)?;
        self.write_single_byte(Regs::FOCCFG, 0x14)?;
        self.write_single_byte(Regs::BSCFG, 0x6C)?;
        self.write_single_byte(Regs::AGCCTRL2, 0x03)?;
        self.write_single_byte(Regs::AGCCTRL1, 0x00)?;
        self.write_single_byte(Regs::AGCCTRL0, 0x92)?;
        self.write_single_byte(Regs::WOREVT1, 0x87)?;
        self.write_single_byte(Regs::WOREVT0, 0x6B)?;
        self.write_single_byte(Regs::WORCTRL, 0xFB)?;
        self.write_single_byte(Regs::FREND1, 0x56)?;
        self.write_single_byte(Regs::FREND0, 0x11)?;
        self.write_single_byte(Regs::FSCAL3, 0xE9)?;
        self.write_single_byte(Regs::FSCAL2, 0x2A)?;
        self.write_single_byte(Regs::FSCAL1, 0x00)?;
        self.write_single_byte(Regs::FSCAL0, 0x1F)?;
        self.write_single_byte(Regs::RCCTRL1, 0x41)?;
        self.write_single_byte(Regs::RCCTRL0, 0x00)?;
        self.write_single_byte(Regs::FSTEST, 0x59)?;
        self.write_single_byte(Regs::PTEST, 0x7F)?;
        self.write_single_byte(Regs::AGCTEST, 0x3F)?;
        self.write_single_byte(Regs::TEST2, 0x81)?;
        self.write_single_byte(Regs::TEST1, 0x35)?;
        self.write_single_byte(Regs::TEST0, 0x0B)?;
        self.write_burst(Regs::PATABLE, &PA_TABLE)?;
        Ok(())
    }

    pub fn setup_tx(&mut self) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.configure()
    }

    pub fn setup_rx(&mut self) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.configure()
    }

    // set baudrate in kbaud
    pub fn set_data_rate(&mut self, data_rate: u32) -> Result<(), CC1101Error<SpiE, PinE>> {
        let xosc: u64 = 26_000;
        let data_rate = data_rate as u64;
        let mut e: u32 = ((data_rate * (1 << 20)) / xosc).ilog2();
        let numerator = data_rate * (1 << (28 - e));
        let mut m = ((numerator + xosc / 2) / xosc).saturating_sub(256);

        if m == 256 {
            m = 0;
            e += 1;
        }

        self.write_register_field(Regs::MDMCFG4, e as u8, 3, 0)?;
        self.write_register_field(Regs::MDMCFG3, m as u8, 7, 0)?;
        Ok(())
    }

    /// Queues `data` in the TX FIFO and starts transmission. Does not block until the
    /// transmission finishes; use `wait_transmit_done` or poll `remaining_bytes` for that.
    pub fn transmit(&mut self, data: &[u8]) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.strobe(Regs::SIDLE)?;
        self.delay.delay_us(800);
        self.strobe(Regs::SFTX)?;
        self.write_single_byte(Regs::PKTLEN, data.len() as u8)?;
        self.write_burst(Regs::RXTXFIFO, data)?;
        self.strobe(Regs::STX)?;
        Ok(())
    }

    /// Transmits `data` and blocks until GDO0 reports the packet has gone out, or `timeout_ms`
    /// elapses.
    pub fn transmit_blocking(
        &mut self,
        data: &[u8],
        timeout_ms: u32,
    ) -> Result<(), CC1101Error<SpiE, PinE>> {
        self.transmit(data)?;
        self.wait_gdo_high(timeout_ms)?;
        self.wait_gdo_low(timeout_ms)?;
        Ok(())
    }

    pub fn remaining_bytes(&mut self) -> Result<u8, CC1101Error<SpiE, PinE>> {
        Ok(self.read_register(Regs::TXBYTES)? & 0x7F)
    }

    /// Number of bytes currently held in the RX FIFO.
    pub fn rx_bytes_available(&mut self) -> Result<u8, CC1101Error<SpiE, PinE>> {
        Ok(self.read_register(Regs::RXBYTES)? & 0x7F)
    }

    /// Puts the radio in RX mode expecting a packet up to `buffer.len()` bytes long, waits (via
    /// GDO0) for sync word and end-of-packet, then reads the received bytes into `buffer`.
    /// Returns the number of bytes actually written into `buffer`. Aborts and returns
    /// `CC1101Error::Timeout` if no packet is received within `timeout_ms`.
    pub fn receive(
        &mut self,
        buffer: &mut [u8],
        timeout_ms: u32,
    ) -> Result<usize, CC1101Error<SpiE, PinE>> {
        self.strobe(Regs::SIDLE)?;
        self.delay.delay_us(800);
        self.strobe(Regs::SFRX)?;
        self.write_single_byte(Regs::PKTLEN, buffer.len() as u8)?;
        self.strobe(Regs::SRX)?;

        if let Err(e) = self.wait_gdo_high(timeout_ms) {
            self.strobe(Regs::SIDLE)?;
            return Err(e);
        }
        if let Err(e) = self.wait_gdo_low(timeout_ms) {
            self.strobe(Regs::SIDLE)?;
            return Err(e);
        }

        let available = self.rx_bytes_available()? as usize;
        let count = available.min(buffer.len());
        self.read_burst(Regs::RXTXFIFO, &mut buffer[..count])?;
        Ok(count)
    }

    fn wait_gdo_high(&mut self, timeout_ms: u32) -> Result<(), CC1101Error<SpiE, PinE>> {
        for _ in 0..timeout_ms {
            if self.gdo.is_high().map_err(CC1101Error::Gpio)? {
                return Ok(());
            }
            self.delay.delay_ms(1);
        }
        Err(CC1101Error::Timeout)
    }

    fn wait_gdo_low(&mut self, timeout_ms: u32) -> Result<(), CC1101Error<SpiE, PinE>> {
        for _ in 0..timeout_ms {
            if self.gdo.is_low().map_err(CC1101Error::Gpio)? {
                return Ok(());
            }
            self.delay.delay_ms(1);
        }
        Err(CC1101Error::Timeout)
    }
}
