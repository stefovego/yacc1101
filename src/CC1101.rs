use embedded_hal;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use embedded_hal::spi::Operation;

use crate::Regs;
use crate::error::CC1101Error;

const WRITE_SINGLE_BYTE: u8 = 0x00;
const WRITE_BURST: u8 = 0x40;
const READ_SINGLE_BYTE: u8 = 0x80;
const READ_BURST: u8 = 0xC0;
const PA_TABLE: [u8; 8] = [0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CC1101<SPI, GDOPin, DelayT> {
    spi: SPI,
    gdo: GDOPin,
    delay: DelayT,
}

impl<SPI, GDOPin, DelayT, E> CC1101<SPI, GDOPin, DelayT>
where
    SPI: embedded_hal::spi::SpiDevice<Error = E>,
    GDOPin: InputPin,
    DelayT: DelayNs,
{
    pub fn read_register(&mut self, register: Regs) -> Result<u8, CC1101Error<E>> {
        let mut read_buffer = [0_u8];
        self.spi.transaction(&mut [
            Operation::Write(&[READ_SINGLE_BYTE | register as u8]),
            Operation::Read(&mut read_buffer),
        ])?;
        Ok(read_buffer[0])
    }

    pub fn get_status(&mut self) -> Result<u8, CC1101Error<E>> {
        self.read_register(Regs::MARCSTATE)
    }

    pub fn new(spi: SPI, gdo: GDOPin, delay: DelayT) -> Self {
        Self { spi, gdo, delay }
    }

    pub fn init(&mut self, frequency: u32, offset: u32) -> Result<(), CC1101Error<E>> {
        self.reset()?;
        self.set_frequency(frequency, offset)?;
        self.write_single_byte(Regs::SYNC1, 0x66)?;
        self.write_single_byte(Regs::SYNC0, 0x6A)?;
        Ok(())
    }

    pub fn set_frequency(&mut self, frequency: u32, offset: u32) -> Result<(), CC1101Error<E>> {
        let frequency = (frequency as u64 * 2_u64.pow(16)) / 26_000_000;
        let byte2: u8 = (frequency as u32 >> 16) as u8 & 0xff;
        let byte1: u8 = (frequency as u16 >> 8) as u8 & 0xff;
        let byte0: u8 = frequency as u8 & 0xff;
        self.write_single_byte(Regs::FREQ2, byte2)?;
        self.write_single_byte(Regs::FREQ1, byte1)?;
        self.write_single_byte(Regs::FREQ0, byte0)?;
        self.write_burst(Regs::PATABLE, &PA_TABLE)?;
        self.strobe(Regs::SFTX)?; // flush TX FIFO
        self.strobe(Regs::SFRX)?; // flush RX FIFO
        Ok(())
    }

    fn strobe(&mut self, address: Regs) -> Result<(), CC1101Error<E>> {
        let databuffer = [address as u8, 0 as u8, 0 as u8];
        self.spi.transaction(&mut [Operation::Write(&databuffer)])?;
        Ok(())
    }

    fn write_single_byte(&mut self, register: Regs, byte_data: u8) -> Result<(), CC1101Error<E>> {
        let databuffer = [WRITE_SINGLE_BYTE | register as u8, byte_data];
        self.spi.write(&databuffer)?;
        Ok(())
    }

    fn write_burst(&mut self, register: Regs, data: &[u8]) -> Result<(), CC1101Error<E>> {
        let buffer = [&[WRITE_BURST | register as u8], &data[..]].concat();
        self.spi.write(&buffer)?;
        Ok(())
    }

    fn write_register_field(
        &mut self,
        register: Regs,
        mut value: u8,
        msb: u8,
        lsb: u8,
    ) -> Result<(), CC1101Error<E>> {
        let current = self.read_register(register)?;
        value <<= lsb;
        let mask: u8 =
            (((1 as u8).wrapping_shl((msb - lsb + 1) as u32)) - 1).wrapping_shl(lsb as u32);
        //let mask: u8 = ((1 << (msb - lsb + 1)) - 1) << lsb;
        let value = (current & !mask) | (value & mask);
        self.write_single_byte(register, value)?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), CC1101Error<E>> {
        self.delay.delay_ms(1);
        self.strobe(Regs::SRES)?;
        Ok(())
    }

    pub fn setup_tx(&mut self) -> Result<(), CC1101Error<E>> {
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

    // set baudrate in kbaud
    pub fn set_data_rate(&mut self, data_rate: u32) -> Result<(), CC1101Error<E>> {
        let xosc = 26_000;
        let mut e: u8 = (data_rate * (1 << 20) / xosc).ilog2() as u8;
        let mut m: u32 = ((data_rate as u64 * (1 << (28 - e)) as u64 / xosc as u64) as f32 - 256.0)
            .round() as u32;

        if m == 256 {
            m = 0;
            e += 1;
        }

        self.write_register_field(Regs::MDMCFG4, e, 3, 0)?;
        self.write_register_field(Regs::MDMCFG3, m as u8, 7, 0)?;
        Ok(())
    }

    pub fn transmit(&mut self, data: &[u8]) -> Result<(), CC1101Error<E>> {
        self.strobe(Regs::SIDLE)?;
        self.delay.delay_us(800);
        self.strobe(Regs::SCAL)?;
        self.delay.delay_us(800);
        self.write_single_byte(Regs::PKTLEN, data.len() as u8)?;
        self.strobe(Regs::SRX)?;
        self.write_burst(Regs::RXTXFIFO, data)?;
        self.delay.delay_us(1);
        self.strobe(Regs::STX)?;
        Ok(())
    }

    pub fn remaining_bytes(&mut self) -> Result<u8, CC1101Error<E>> {
        Ok(self.read_register(Regs::TXBYTES)? & 0x7F)
    }
}
