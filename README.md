# Rainbow

[![Crate](https://img.shields.io/crates/v/rainbow.svg)](https://crates.io/crates/rainbow)
[![Documentation](https://docs.rs/rainbow/badge.svg)](https://docs.rs/rainbow)
[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](http://creativecommons.org/publicdomain/zero/1.0/)

Rainbow is a versatile HTTP steganography library that enables data hiding within various HTTP content types. It provides a robust framework for encoding and decoding hidden data within HTTP requests and responses, making it suitable for a wide range of applications requiring covert communication channels.

## Features

- Multiple steganography techniques:
  - HTML (Comments, Nested Divs)
  - CSS (Grid/Flex, Animations, Paint Worklet)
  - JSON (Metadata)
  - XML/RSS
  - SVG Paths
  - Audio (WAV)
  - Font properties
  - And more...
- Support for both client and server-side encoding
- Automatic MIME type selection
- Randomized encoding patterns
- Built-in error handling
- Async/await support
- Cross-platform compatibility

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rainbow = "0.1.0"
```

## Quick Start

```rust
use rainbow::rainbow::Rainbow;
use rainbow::NetworkSteganographyProcessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Rainbow
    let rainbow = Rainbow::new();
    
    // Encode data
    let data = b"Hello, Steganography!";
    let encode_result = rainbow.encode_write(
        data,
        true,  // is_client
        None,  // mime_type (auto-select)
    ).await?;
    
    // Decode data
    let decode_result = rainbow.decrypt_single_read(
        encode_result.encoded_packets[0].clone(),
        0,    // packet_index
        true, // is_client
    ).await?;
    
    assert_eq!(decode_result.data, data);
    Ok(())
}
```

## Command Line Usage

Rainbow also provides a command-line interface for quick encoding and decoding:

```bash
# Encode a file
cargo run -- encode --input examples/data/test.txt --output my_output_folder

# Decode a packet
cargo run -- decode --input my_output_folder/packet_0.http --output decoded.txt
```

## Advanced Usage

### Custom MIME Types

You can specify a MIME type for encoding:

```rust
let encode_result = rainbow.encode_write(
    data,
    true,
    Some("text/html".to_string()),
).await?;
```

### Multiple Packets

Rainbow supports splitting large data into multiple packets:

```rust
let encode_result = rainbow.encode_write(large_data, true, None).await?;
for (i, packet) in encode_result.encoded_packets.iter().enumerate() {
    // Process each packet...
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is dedicated to the public domain under the CC0 1.0 Universal license. You can copy, modify, distribute and perform the work, even for commercial purposes, all without asking permission.

For more information, see [Creative Commons CC0 1.0 Universal](http://creativecommons.org/publicdomain/zero/1.0/).

## Security Considerations

Rainbow is designed for educational and research purposes. While it implements various steganography techniques, it should not be considered a secure method for sensitive data transmission. Always use proper encryption for sensitive data.

## Acknowledgments

- This project is inspired by various steganography techniques and research in the field of information hiding.
- Special thanks to the Rust community for providing excellent tools and libraries that made this project possible.
