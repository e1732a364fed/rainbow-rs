use rainbow::rainbow::Rainbow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rayon::prelude::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::fmt::format::FmtSpan;

    // 创建文件日志记录器
    let file_appender = RollingFileAppender::new(Rotation::NEVER, "logs", "bandwidth_analysis.log");

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(file_appender)
        .with_ansi(false)
        .with_thread_ids(false)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .compact()
        .try_init();

    assert!(subscriber.is_ok(), "Failed to initialize logging");

    let rainbow = Arc::new(Rainbow::new());
    let mime_types: Vec<_> = rainbow.encoders.get_all_mime_types();
    let test_sizes = vec![100, 500, 1000, 2000, 5000, 10000, 100 * 1024, 1024 * 1024];

    // 为每个 MIME 类型创建计数器
    let mut mime_stats = HashMap::new();
    for mime_type in &mime_types {
        mime_stats.insert(
            mime_type.to_string(),
            (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))), // (time, count)
        );
    }
    let mime_stats = Arc::new(mime_stats);

    // 使用原子计数器跟踪总体进度
    let total_tests = test_sizes.len() * mime_types.len();
    let completed_tests = Arc::new(AtomicUsize::new(0));
    let completed_sizes = Arc::new(AtomicUsize::new(0));
    let max_overhead = Arc::new(AtomicUsize::new(0));
    let min_overhead = Arc::new(AtomicUsize::new(usize::MAX));

    // 对 test_sizes 进行并行处理，直接收集结果
    let start_time = std::time::Instant::now();
    let all_results: Vec<_> = test_sizes
        .par_iter()
        .flat_map(|&test_size| {
            let rainbow = Arc::clone(&rainbow);
            let completed_tests = Arc::clone(&completed_tests);
            let completed_sizes = Arc::clone(&completed_sizes);
            let max_overhead = Arc::clone(&max_overhead);
            let min_overhead = Arc::clone(&min_overhead);
            let mime_stats = Arc::clone(&mime_stats);

            // 对 MIME 类型也进行并行处理
            let size_results: Vec<_> = mime_types
                .par_iter()
                .filter_map(|mime_type| {
                    let test_start = std::time::Instant::now();
                    let stats = rainbow
                        .analyze_bandwidth_range(&[test_size], Some((*mime_type).to_string()))
                        .unwrap();

                    stats.first().map(|stat| {
                        let overhead_ratio = (stat.total_packet_size + stat.expected_return_size)
                            as f64
                            / stat.original_size as f64;

                        // 更新统计信息
                        completed_tests.fetch_add(1, Ordering::Relaxed);
                        let elapsed_ns = test_start.elapsed().as_nanos() as usize;

                        // 更新 MIME 类型统计
                        if let Some((time_counter, count_counter)) =
                            mime_stats.get(&(*mime_type).to_string())
                        {
                            time_counter.fetch_add(elapsed_ns, Ordering::Relaxed);
                            count_counter.fetch_add(1, Ordering::Relaxed);
                        }

                        // 更新最大最小开销比
                        let overhead_int = (overhead_ratio * 1000.0) as usize;
                        max_overhead.fetch_max(overhead_int, Ordering::Relaxed);
                        min_overhead.fetch_min(overhead_int, Ordering::Relaxed);

                        (
                            (*mime_type).to_string(),
                            test_size,
                            overhead_ratio,
                            stat.packet_count,
                        )
                    })
                })
                .collect();

            completed_sizes.fetch_add(1, Ordering::Relaxed);
            size_results
        })
        .collect();

    let total_time = start_time.elapsed();

    // 按大小分组计算平均值
    let mut size_averages = HashMap::new();
    for (_, size, ratio, _) in &all_results {
        let entry = size_averages.entry(*size).or_insert((0.0, 0));
        entry.0 += ratio;
        entry.1 += 1;
    }

    // 准备结果输出
    let mut output = Vec::new();

    output.push("\n=== Bandwidth Analysis Test Results ===\n".to_string());

    output.push(format!(
        "Configuration:\n- MIME types: {}\n- Test sizes: {}\n- Total tests: {}\n- Threads: {}\n",
        mime_types.len(),
        test_sizes.len(),
        total_tests,
        rayon::current_num_threads()
    ));

    output.push(format!(
        "\nPerformance Metrics:\n- Total time: {:.2?}\n- Tests per second: {:.2}/s\n",
        total_time,
        total_tests as f64 / total_time.as_secs_f64()
    ));

    // 添加每个 MIME 类型的性能统计
    output.push("\nMIME Type Performance:".to_string());

    // 收集并排序 MIME 类型性能数据
    let mut mime_perf: Vec<_> = mime_stats
        .iter()
        .map(|(mime_type, (time_counter, count_counter))| {
            let total_ns = time_counter.load(Ordering::Relaxed);
            let count = count_counter.load(Ordering::Relaxed);
            let avg_time = if count > 0 {
                total_ns as f64 / count as f64 / 1_000_000.0 // 转换为毫秒
            } else {
                0.0
            };
            (mime_type.as_str(), avg_time)
        })
        .collect();

    // 按平均时间排序
    mime_perf.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (mime_type, avg_time) in mime_perf {
        output.push(format!(
            "- {:<20} Average time: {:.2} ms",
            mime_type, avg_time
        ));
    }

    output.push("\nOverhead Ratios by Size:".to_string());
    let mut size_summaries: Vec<_> = size_averages
        .iter()
        .map(|(size, (total, count))| (*size, total / *count as f64))
        .collect();
    size_summaries.sort_by_key(|(size, _)| *size);

    for (size, avg_ratio) in size_summaries {
        output.push(format!(
            "- Size: {:>8} bytes, Average overhead ratio: {:.2}x",
            size, avg_ratio
        ));
    }

    let average_overhead_ratio = all_results
        .iter()
        .map(|(_mime, _size, ratio, _packets)| ratio)
        .sum::<f64>()
        / all_results.len() as f64;

    output.push(format!(
        "\nOverall Statistics:\n- Min overhead ratio: {:.2}x\n- Max overhead ratio: {:.2}x\n- Average overhead ratio: {:.2}x\n",
        min_overhead.load(Ordering::Relaxed) as f64 / 1000.0,
        max_overhead.load(Ordering::Relaxed) as f64 / 1000.0,
        average_overhead_ratio
    ));

    // 一次性输出所有结果
    println!("{}", output.join("\n"));

    Ok(())
}
