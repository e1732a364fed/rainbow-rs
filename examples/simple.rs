/*!
 * This example demonstrates how to use the Rainbow library to encode and decode data using different steganography methods.
 *
 * It will encode and decode the data using all the supported steganography methods, and save the results to the `examples/data` directory.
 */

use rainbow::rainbow::Rainbow;
use rainbow::{DecodeResult, EncodeOptions, EncodeResult, NetworkSteganographyProcessor};
use std::fs;
use tracing::info;

fn process(
    is_client: bool,
    rainbow: &Rainbow,
    data: &[u8],
    encoder_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("trying is client: {}", is_client);

    // 编码数据
    let EncodeResult {
        encoded_packets: packets,
        expected_return_packet_lengths: lengths,
    } = rainbow.encode_write(
        data,
        is_client,
        EncodeOptions {
            encoder: Some(encoder_name.to_string()),
            ..Default::default()
        },
    )?;

    info!("\nGenerated {} packets", packets.len());

    // info!("packets: {:?}", String::from_utf8_lossy(&packets[0]));

    let role = if is_client { "client" } else { "server" };

    let mime_type = rainbow
        .registry
        .encoders
        .get(encoder_name)
        .unwrap()
        .get_mime_type();

    // 创建输出目录
    fs::create_dir_all(format!("examples/data/{}_output/{}", role, encoder_name))?;

    // 保存并解码每个数据包
    for (i, (packet, length)) in packets.iter().zip(lengths.iter()).enumerate() {
        let file_path = format!(
            "examples/data/{}_output/{}/packet_{}.http",
            role, encoder_name, i
        );
        fs::write(&file_path, packet)?;

        let body_pos = rainbow::utils::find_crlf_crlf(packet).unwrap() + 4;
        let body = &packet[body_pos..];

        let file_path = format!(
            "examples/data/{}_output/{}/packet_{}.{}",
            role,
            encoder_name,
            i,
            rainbow::utils::mime_to_extension(mime_type)
        );
        fs::write(&file_path, body)?;

        info!("Writing packet {} to {}, length: {}", i, file_path, length);

        let DecodeResult {
            data: decoded,
            expected_return_length,
            is_read_end,
        } = rainbow.decrypt_single_read(packet.clone(), i, is_client)?;

        info!(
            "Decoded packet {}: length = {}, expected length = {}, is last packet = {}",
            i,
            decoded.len(),
            expected_return_length,
            is_read_end
        );
        info!("Decoded content: {}\n", String::from_utf8_lossy(&decoded));
    }
    Ok(())
}

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();

    let rainbow = Rainbow::default();

    let data = fs::read("res/test.txt")?;

    info!("mime_types: {:?}", rainbow.registry.get_all_mime_types());

    for name in rainbow.registry.encoders.keys() {
        info!("\nTesting {} steganography:", name);
        info!("Original data: {}", String::from_utf8_lossy(&data));

        process(true, &rainbow, &data, name)?;
        process(false, &rainbow, &data, name)?;
    }

    Ok(())
}
