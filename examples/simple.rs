use rainbow::rainbow::Rainbow;
use rainbow::{DecodeResult, EncodeResult, NetworkSteganographyProcessor};
use std::fs;

fn do_job(
    is_client: bool,
    rainbow: &Rainbow,
    data: &[u8],
    mime_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("trying is client: {}", is_client);
    // 编码数据
    let EncodeResult {
        encoded_packets: packets,
        expected_return_packet_lengths: lengths,
    } = rainbow.encode_write(data, is_client, Some(mime_type.to_string()))?;

    println!("\nGenerated {} packets", packets.len());

    // println!("packets: {:?}", String::from_utf8_lossy(&packets[0]));

    let name = if is_client { "client" } else { "server" };

    // 创建输出目录
    fs::create_dir_all(format!(
        "examples/data/{}_output/{}",
        name,
        mime_type.split('/').last().unwrap()
    ))?;

    // 保存并解码每个数据包
    for (i, (packet, length)) in packets.iter().zip(lengths.iter()).enumerate() {
        let file_path = format!(
            "examples/data/{}_output/{}/packet_{}.http",
            name,
            mime_type.split('/').last().unwrap(),
            i
        );
        fs::write(&file_path, packet)?;
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
    mime_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting {} steganography:", mime_type);
    println!("Original data: {}", String::from_utf8_lossy(data));

    do_job(true, rainbow, data, mime_type)?;
    do_job(false, rainbow, data, mime_type)?;
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
    let mime_types = rainbow.encoders.get_all_mime_types();

    println!("mime_types: {:?}", mime_types);

    for mime_type in mime_types {
        test_stego(&rainbow, &data, mime_type)?;
    }

    Ok(())
}
