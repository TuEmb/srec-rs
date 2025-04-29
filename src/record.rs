use crate::Error;

pub type Address16 = u16;
pub type Address24 = u32;
pub type Address32 = u32;
pub type Count16 = u16;
pub type Count24 = u32;

/// Record data field
#[derive(Debug, Clone)]
pub struct Data<T> {
    /// Start address
    pub address: T,
    /// Data bytes
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Address {
    /// 16-bit address
    Address16(Address16),
    /// 24-bit address
    Address24(Address24),
    /// 32-bit address
    Address32(Address32),
}

impl Address {
    pub fn to_le_bytes(self) -> Vec<u8> {
        match self {
            Address::Address16(addr) => addr.to_le_bytes().to_vec(),
            Address::Address24(addr) => addr.to_le_bytes().to_vec(),
            Address::Address32(addr) => addr.to_le_bytes().to_vec(),
        }
    }

    pub fn to_be_bytes(self) -> Vec<u8> {
        match self {
            Address::Address16(addr) => addr.to_be_bytes().to_vec(),
            Address::Address24(addr) => addr.to_be_bytes().to_vec(),
            Address::Address32(addr) => addr.to_be_bytes().to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Record {
    /// Header
    S0(String),
    /// Data with 16-bit address
    S1(Data<Address16>),
    /// Data with 24-bit address
    S2(Data<Address24>),
    /// Data with 32-bit address
    S3(Data<Address32>),
    /// This record type is not defined by the official S-record standard.
    S4,
    /// 16-bit data record count
    S5(Count16),
    /// 24-bit data record count (Non-standard)
    S6(Count24),
    /// 32-bit start address
    S7(Address32),
    /// 24-bit start address
    S8(Address24),
    /// 16-bit start address
    S9(Address16),
}

impl Record {
    /// Parse an S-Record string into a Record enum
    pub fn parse_from_str(record: &str) -> Result<Self, Error> {
        let record = record.trim();
        if !record.starts_with('S') || record.len() < 2 {
            return Err(Error::DataLengthError);
        }

        let rec_type = &record[1..2];
        let bytes = (2..record.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&record[i..i + 2], 16).unwrap_or(0))
            .collect::<Vec<_>>();

        // Checksum calculation (last byte is checksum)
        let sum: u32 = bytes.iter().map(|&b| b as u32).sum();
        if (sum & 0xFF) != 0xFF {
            return Err(Error::CheckSumError);
        }

        match rec_type {
            "0" => {
                // S0: Header, address is 2 bytes, data is rest minus checksum
                let data = &bytes[2..bytes.len() - 1];
                let text = String::from_utf8_lossy(data).to_string();
                Ok(Record::S0(text))
            }
            "1" => {
                // S1: 16-bit address + data
                let address = ((bytes[1] as u16) << 8) | (bytes[2] as u16);
                let data = bytes[3..bytes.len() - 1].to_vec();
                Ok(Record::S1(Data { address, data }))
            }
            "2" => {
                // S2: 24-bit address + data
                let address =
                    ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32);
                let data = bytes[4..bytes.len() - 1].to_vec();
                Ok(Record::S2(Data { address, data }))
            }
            "3" => {
                // S3: 32-bit address + data
                let address = ((bytes[1] as u32) << 24)
                    | ((bytes[2] as u32) << 16)
                    | ((bytes[3] as u32) << 8)
                    | (bytes[4] as u32);
                let data = bytes[5..bytes.len() - 1].to_vec();
                Ok(Record::S3(Data { address, data }))
            }
            "4" => Ok(Record::S4),
            "5" => {
                // S5: 16-bit count
                let count = ((bytes[1] as u16) << 8) | (bytes[2] as u16);
                Ok(Record::S5(count))
            }
            "6" => {
                // S6: 24-bit count
                let count =
                    ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32);
                Ok(Record::S6(count))
            }
            "7" => {
                // S7: 32-bit start address
                let address = ((bytes[1] as u32) << 24)
                    | ((bytes[2] as u32) << 16)
                    | ((bytes[3] as u32) << 8)
                    | (bytes[4] as u32);
                Ok(Record::S7(address))
            }
            "8" => {
                // S8: 24-bit start address
                let address =
                    ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32);
                Ok(Record::S8(address))
            }
            "9" => {
                // S9: 16-bit start address
                let address = ((bytes[1] as u16) << 8) | (bytes[2] as u16);
                Ok(Record::S9(address))
            }
            _ => Err(Error::DataLengthError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_to_le_bytes() {
        let addr16 = Address::Address16(0x1234);
        assert_eq!(addr16.to_le_bytes(), 0x1234u16.to_le_bytes().to_vec());

        let addr24 = Address::Address24(0x123456);
        assert_eq!(addr24.to_le_bytes(), 0x123456u32.to_le_bytes().to_vec());

        let addr32 = Address::Address32(0x12345678);
        assert_eq!(addr32.to_le_bytes(), 0x12345678u32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_address_to_be_bytes() {
        let addr16 = Address::Address16(0x1234);
        assert_eq!(addr16.to_be_bytes(), 0x1234u16.to_be_bytes().to_vec());

        let addr24 = Address::Address24(0x123456);
        assert_eq!(addr24.to_be_bytes(), 0x123456u32.to_be_bytes().to_vec());

        let addr32 = Address::Address32(0x12345678);
        assert_eq!(addr32.to_be_bytes(), 0x12345678u32.to_be_bytes().to_vec());
    }

    #[test]
    fn test_parse_s1_record() {
        // Example S1 record from Wikipedia: S1137AF000A0A0D000000000000000000000000072
        let srec = "S1137AF000A0A0D000000000000000000000000072";
        let rec = Record::parse_from_str(srec).unwrap();

        match rec {
            Record::S1(data) => {
                assert_eq!(data.address, 0x7AF0);
                assert_eq!(
                    data.data,
                    vec![
                        0x00, 0xA0, 0xA0, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00
                    ]
                );
            }
            _ => panic!("Expected S1 record"),
        }
    }

    #[test]
    fn test_parse_s2_record() {
        // Example S2 record from Wikipedia: S214010480C04671B604207146014218D0EFF30983C5
        let srec = "S214010480C04671B604207146014218D0EFF30983C5";
        let rec = Record::parse_from_str(srec).unwrap();

        match rec {
            Record::S2(data) => {
                assert_eq!(data.address, 0x010480);
                assert_eq!(
                    data.data,
                    vec![
                        0xC0, 0x46, 0x71, 0xB6, 0x04, 0x20, 0x71, 0x46, 0x01, 0x42, 0x18, 0xD0,
                        0xEF, 0xF3, 0x09, 0x83
                    ]
                );
            }
            _ => panic!("Expected S1 record"),
        }
    }
}
