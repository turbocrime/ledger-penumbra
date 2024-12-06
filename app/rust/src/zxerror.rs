#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ZxErr {
    Unknown = 0b00000000,
    Ok = 0b00000011,
    NoData = 0b00000101,
    BufferTooSmall = 0b00000110,
    OutOfBounds = 0b00001001,
    EncodingFailed = 0b00001010,
    InvalidCryptoSettings = 0b00001100,
    LedgerApiError = 0b00001111,
}

impl From<ZxErr> for u32 {
    fn from(err: ZxErr) -> Self {
        err as u32
    }
}

impl TryFrom<u32> for ZxErr {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b00000000 => Ok(ZxErr::Unknown),
            0b00000011 => Ok(ZxErr::Ok),
            0b00000101 => Ok(ZxErr::NoData),
            0b00000110 => Ok(ZxErr::BufferTooSmall),
            0b00001001 => Ok(ZxErr::OutOfBounds),
            0b00001010 => Ok(ZxErr::EncodingFailed),
            0b00001100 => Ok(ZxErr::InvalidCryptoSettings),
            0b00001111 => Ok(ZxErr::LedgerApiError),
            _ => Err(()),
        }
    }
}

impl From<ZxErr> for u8 {
    fn from(err: ZxErr) -> Self {
        err as u8
    }
}

impl TryFrom<u8> for ZxErr {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00000000 => Ok(ZxErr::Unknown),
            0b00000011 => Ok(ZxErr::Ok),
            0b00000101 => Ok(ZxErr::NoData),
            0b00000110 => Ok(ZxErr::BufferTooSmall),
            0b00001001 => Ok(ZxErr::OutOfBounds),
            0b00001010 => Ok(ZxErr::EncodingFailed),
            0b00001100 => Ok(ZxErr::InvalidCryptoSettings),
            0b00001111 => Ok(ZxErr::LedgerApiError),
            _ => Err(()),
        }
    }
}
