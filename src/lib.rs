//! SREC file parsing and memory layout utilities.

use std::{fs::File, io::BufRead, io::BufReader};
mod record;
pub use record::{Address, Data, Record};

/// Errors which may occur during reading or parsing SREC files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Checksum did not match calculated checksum")]
    CheckSumError,
    #[error("Data length was not as expected")]
    DataLengthError,
    #[error("Character was unexpected")]
    UnexpectedCharacter,
    #[error("Can't open srec file")]
    SrecFileError,
}

/// Represents a parsed SREC file and its memory layout.
pub struct SRecord<const MAX: u32> {
    /// All parsed SREC records.
    record: Vec<record::Record>,
    /// Memory layout: (start address, region size) for each region.
    data_memory_layout: Vec<(Address, usize)>,
    /// Concatenated data bytes from all S1/S2/S3 records.
    data: Vec<u8>,
    /// Total data length in bytes.
    data_length: usize,
}

impl<const MAX: u32> SRecord<MAX> {
    /// Returns the header string from the S0 record, if present.
    pub fn get_header(&self) -> Option<&str> {
        for rec in &self.record {
            if let record::Record::S0(header) = rec {
                return Some(header);
            }
        }
        None
    }

    /// Returns the memory layout as a slice of (Address, size) tuples.
    pub fn get_data_memory_layout(&self) -> &[(Address, usize)] {
        &self.data_memory_layout
    }

    /// Returns a slice of all concatenated data bytes.
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the total data length in bytes.
    pub fn get_data_length(&self) -> usize {
        self.data_length
    }

    /// Parse an SREC file and build the memory layout and data.
    ///
    /// - Merges adjacent/overlapping regions up to MAX bytes per region.
    /// - Supports S1, S2, S3 records for data.
    pub fn from_srec(f: File) -> Result<Self, Error> {
        let reader = BufReader::new(f);
        let mut records = Vec::new();
        let mut regions = Vec::new();
        let mut data = Vec::new();

        // Parse each line and collect regions and data
        for line in reader.lines() {
            if let Ok(line) = line {
                let rec = record::Record::parse_from_str(&line)?;
                match &rec {
                    record::Record::S1(d) => {
                        regions.push((d.address as u32, d.data.len()));
                        data.extend(&d.data);
                    }
                    record::Record::S2(d) => {
                        regions.push((d.address, d.data.len()));
                        data.extend(&d.data);
                    }
                    record::Record::S3(d) => {
                        regions.push((d.address, d.data.len()));
                        data.extend(&d.data);
                    }
                    _ => {}
                }
                records.push(rec);
            } else {
                return Err(Error::SrecFileError);
            }
        }

        // Sort regions by address and merge adjacent/overlapping ones, splitting if MAX is exceeded
        regions.sort_by_key(|&(addr, _)| addr);
        let mut merged = Vec::new();
        for (addr, size) in regions {
            if let Some((last_addr, last_size)) = merged.last_mut() {
                let last_end = *last_addr + *last_size as u32;
                if addr <= last_end && (last_end - *last_addr) < MAX {
                    // Merge overlapping/adjacent, but do not exceed MAX
                    let new_end = (addr + size as u32).max(last_end);
                    let region_size = (new_end - *last_addr) as usize;
                    if region_size as u32 <= MAX {
                        *last_size = region_size;
                    } else {
                        // Split region if it would exceed MAX
                        let allowed = (MAX - (*last_size as u32)) as usize;
                        *last_size = MAX as usize;
                        merged.push((addr, size - allowed));
                    }
                } else {
                    merged.push((addr, size));
                }
            } else {
                merged.push((addr, size));
            }
        }

        // Split regions that are larger than MAX
        let mut final_regions = Vec::new();
        for (mut addr, mut size) in merged {
            while size as u32 > MAX {
                final_regions.push((Address::Address32(addr), MAX as usize));
                addr += MAX;
                size -= MAX as usize;
            }
            if size > 0 {
                final_regions.push((Address::Address32(addr), size));
            }
        }

        let data_length = data.len();

        Ok(Self {
            record: records,
            data_memory_layout: final_regions,
            data,
            data_length,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    /// Test loading an SREC file and checking the memory layout and data.
    #[test]
    fn test_from_srec_file() {
        let file = File::open("test_data/test.srec").expect("Failed to open test.srec");
        let srec = SRecord::<0x8000>::from_srec(file).unwrap();

        // Check that records were parsed
        assert!(!srec.record.is_empty(), "No records parsed");

        // Check that data_memory_layout is not empty and contains reasonable regions
        assert!(
            !srec.data_memory_layout.is_empty(),
            "No memory layout regions found"
        );

        // Check that data and data_length are consistent
        assert_eq!(srec.data.len(), srec.data_length);

        // Optionally, print for debug
        for (addr, size) in &srec.data_memory_layout {
            println!("Region: {:?}, size: {:#X}", addr, size);
        }
    }
}
