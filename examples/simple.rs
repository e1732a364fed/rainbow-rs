use rainbow::rainbow::Rainbow;
use rainbow::{DecodeResult, EncodeOptions, EncodeResult, NetworkSteganographyProcessor};
use std::fs;

fn do_job(
    is_client: bool,
    rainbow: &Rainbow,
    data: &[u8],
    encoder: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("trying is client: {}", is_client);

    let encoder = rainbow.registry.encoders.get(encoder).unwrap();

    // 编码数据
    let EncodeResult {
        encoded_packets: packets,
        expected_return_packet_lengths: lengths,
    } = rainbow.encode_write(
        data,
        is_client,
        EncodeOptions {
            mime_type: Some(encoder.get_mime_type().to_string()),
            ..Default::default()
        },
    )?;

    println!("\nGenerated {} packets", packets.len());

    // println!("packets: {:?}", String::from_utf8_lossy(&packets[0]));

    let name = if is_client { "client" } else { "server" };

    // 创建输出目录
    fs::create_dir_all(format!(
        "examples/data/{}_output/{}",
        name,
        encoder.get_mime_type().split('/').last().unwrap()
    ))?;

    // 保存并解码每个数据包
    for (i, (packet, length)) in packets.iter().zip(lengths.iter()).enumerate() {
        let file_path = format!(
            "examples/data/{}_output/{}/packet_{}.http",
            name,
            encoder.get_mime_type().split('/').last().unwrap(),
            i
        );
        fs::write(&file_path, packet)?;

        let body_pos = rainbow::utils::find_crlf_crlf(packet).unwrap() + 4;
        let body = &packet[body_pos..];

        let file_path = format!(
            "examples/data/{}_output/{}/packet_{}.{}",
            name,
            encoder.get_mime_type().split('/').last().unwrap(),
            i,
            rainbow::utils::mime_to_extension(encoder.get_mime_type())
        );
        fs::write(&file_path, body)?;

        println!("Writing packet {} to {}, length: {}", i, file_path, length);

        // 解码数据包
        let DecodeResult {
            data: decoded,
            expected_return_length,
            is_read_end,
        } = rainbow.decrypt_single_read(packet.clone(), i, is_client)?;

        println!(
            "Decoded packet {}: length = {}, expected length = {}, is last packet = {}",
            i,
            decoded.len(),
            expected_return_length,
            is_read_end
        );
        println!("Decoded content: {}\n", String::from_utf8_lossy(&decoded));
    }
    Ok(())
}

fn test_stego(
    rainbow: &Rainbow,
    data: &[u8],
    encoder: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting {} steganography:", encoder);
    println!("Original data: {}", String::from_utf8_lossy(data));

    do_job(true, rainbow, data, encoder)?;
    do_job(false, rainbow, data, encoder)?;
    Ok(())
}

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    init();

    // 创建 Rainbow 实例
    let rainbow = Rainbow::new();

    // 读取测试文件
    let data = fs::read("res/test.txt")?;

    // 测试所有支持的 MIME 类型
    // let mime_types = rainbow.encoders.get_all_mime_types();

    println!("mime_types: {:?}", rainbow.registry.get_all_mime_types());

    for name in rainbow.registry.encoders.keys() {
        test_stego(&rainbow, &data, name)?;
    }

    Ok(())
}
