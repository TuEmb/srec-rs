# srec-rs

A Rust library for parsing Motorola S-Record (SREC) files and extracting memory layout and data regions.

## Features

- Parse SREC files (S0, S1, S2, S3, S5, S7, S8, S9 records)
- Validate checksums
- Extract all data bytes from S1/S2/S3 records
- Build memory layout regions, merging adjacent/overlapping regions up to a configurable maximum size
- Simple API for accessing header, memory layout, and data

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
srec-rs = { path = "." }
```

### Example

```rust
use std::fs::File;
use srec_rs::SRecord;

fn main() {
    let file = File::open("test_data/test.srec").expect("Failed to open file");
    let srec = SRecord::<0x8000>::from_srec(file);

    // Print header if present
    if let Some(header) = srec.get_header() {
        println!("Header: {}", header);
    }

    // Print memory layout
    for (addr, size) in srec.get_data_memory_layout() {
        println!("Region: {:?}, size: {:#X}", addr, size);
    }

    // Access raw data
    let data = srec.get_data();
    println!("Total data length: {}", srec.get_data_length());
}
```

## Testing

To run tests and see debug output:

```sh
cargo test -- --nocapture
```

## License

This project is licensed under the MIT License. See [LICENSE-MIT](LICENSE-MIT) for details.

---

**Author:** TuEmb  
**Year:** 2025