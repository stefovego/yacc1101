#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Regs {
    IOCFG2 = 0x00,   //GDO2 Output Pin Configuration
    IOCFG1 = 0x01,   //GDO1 Output Pin Configuration
    IOCFG0 = 0x02,   //self.GDO0 Output Pin Configuration
    FIFOTHR = 0x03,  //RX FIFO and TX FIFO Thresholds
    SYNC1 = 0x04,    //Sync Word, High Byte
    SYNC0 = 0x05,    //Sync Word, Low Byte
    PKTLEN = 0x06,   //Packet Length
    PKTCTRL1 = 0x07, //Packet Automation Control
    PKTCTRL0 = 0x08, //Packet Automation Control
    ADDR = 0x09,     //Device Address
    CHANNR = 0x0A,   //Channel Number
    FSCTRL1 = 0x0B,  //Frequency Synthesizer Control
    FSCTRL0 = 0x0C,  //Frequency Synthesizer Control
    FREQ2 = 0x0D,    //Frequency Control Word, High Byte
    FREQ1 = 0x0E,    //Frequency Control Word, Middle Byte
    FREQ0 = 0x0F,    //Frequency Control Word, Low Byte
    MDMCFG4 = 0x10,  //Modem Configuration
    MDMCFG3 = 0x11,  //Modem Configuration
    MDMCFG2 = 0x12,  //Modem Configuration
    MDMCFG1 = 0x13,  //Modem Configuration
    MDMCFG0 = 0x14,  //Modem Configuration
    DEVIATN = 0x15,  //Modem Deviation Setting
    MCSM2 = 0x16,    //Main Radio Control State Machine Configuration
    MCSM1 = 0x17,    //Main Radio Control State Machine Configuration
    MCSM0 = 0x18,    //Main Radio Control State Machine Configuration
    FOCCFG = 0x19,   //Frequency Offset Compensation Configuration
    BSCFG = 0x1A,    //Bit Synchronization Configuration
    AGCCTRL2 = 0x1B, //AGC Control
    AGCCTRL1 = 0x1C, //AGC Control
    AGCCTRL0 = 0x1D, //AGC Control
    WOREVT1 = 0x1E,  //High Byte Event0 Timeout
    WOREVT0 = 0x1F,  //Low Byte Event0 Timeout
    WORCTRL = 0x20,  //Wake On Radio Control
    FREND1 = 0x21,   //Front End RX Configuration
    FREND0 = 0x22,   //Front End TX Configuration
    FSCAL3 = 0x23,   //Frequency Synthesizer Calibration
    FSCAL2 = 0x24,   //Frequency Synthesizer Calibration
    FSCAL1 = 0x25,   //Frequency Synthesizer Calibration
    FSCAL0 = 0x26,   //Frequency Synthesizer Calibration
    RCCTRL1 = 0x27,  //RC Oscillator Configuration
    RCCTRL0 = 0x28,  //RC Oscillator Configuration

    // Configuration Register Details - Registers that Loose Programming in SLEEP State
    FSTEST = 0x29,  //Frequency Synthesizer Calibration Control
    PTEST = 0x2A,   //Production Test
    AGCTEST = 0x2B, //AGC Test
    TEST2 = 0x2C,   //Various Test Settings
    TEST1 = 0x2D,   //Various Test Settings
    TEST0 = 0x2E,   //Various Test Settings

    //  Command Strobe Registers
    SRES = 0x30,    //Reset chip
    SFSTXON = 0x31, //Enable and calibrate frequency synthesizer (if MCSM0.FS_AUTOCAL=1).

    SXOFF = 0x32, //Turn off crystal oscillator.
    SCAL = 0x33,  //Calibrate frequency synthesizer and turn it off.

    SRX = 0x34, //Enable RX. Perform calibration first if coming from IDLE and MCSM0.FS_AUTOCAL=1.
    STX = 0x35, //In IDLE state: Enable TX. Perform calibration first

    SIDLE = 0x36, //Exit RX / TX, turn off frequency synthesizer and exit Wake-On-Radio mode if applicable.
    SWOR = 0x38,  //Start automatic RX polling sequence (Wake-on-Radio)

    SPWD = 0x39,    //Enter power down mode when CSn goes high.
    SFRX = 0x3A,    //Flush the RX FIFO buffer. Only issue SFRX in IDLE or RXFIFO_OVERFLOW states.
    SFTX = 0x3B,    //Flush the TX FIFO buffer. Only issue SFTX in IDLE or TXFIFO_UNDERFLOW states.
    SWORRST = 0x3C, //Reset real time clock to Event1 value.
    SNOP = 0x3D,    //No operation. May be used to get access to the chip status byte.

    PATABLE = 0x3E,  //PATABLE
    RXTXFIFO = 0x3F, //TXFIFO

    // Status Register Details
    PARTNUM = 0xF0,        //Chip ID
    VERSION = 0xF1,        //Chip ID
    FREQEST = 0xF2,        //Frequency Offset Estimate from Demodulator
    LQI = 0xF3,            //Demodulator Estimate for Link Quality
    RSSI = 0xF4,           //Received Signal Strength Indication
    MARCSTATE = 0xF5,      //Main Radio Control State Machine State
    WORTIME1 = 0xF6,       //High Byte of WOR Time
    WORTIME0 = 0xF7,       //Low Byte of WOR Time
    PKTSTATUS = 0xF8,      //Current GDOx Status and Packet Status
    VCO_VC_DAC = 0xF9,     //Current Setting from PLL Calibration Module
    TXBYTES = 0xFA,        //Underflow and Number of Bytes
    RXBYTES = 0xFB,        //Overflow and Number of Bytes
    RCCTRL1_STATUS = 0xFC, //Last RC Oscillator Calibration Result
    RCCTRL0_STATUS = 0xFD, //Last RC Oscillator Calibration Result
}
