use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rainbow::rainbow::Rainbow;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn analyze_bandwidth(mime_type: &str, size: usize) -> f64 {
    let rainbow = Rainbow::new();
    let test_start = std::time::Instant::now();
    let stats = rainbow
        .analyze_bandwidth_range(&[size], Some(mime_type.to_string()))
        .unwrap();

    if let Some(stat) = stats.first() {
        let _overhead_ratio =
            (stat.total_packet_size + stat.expected_return_size) as f64 / stat.original_size as f64;
        let elapsed_ms = test_start.elapsed().as_secs_f64() * 1000.0;
        elapsed_ms
    } else {
        0.0
    }
}

fn parallel_analyze_bandwidth(mime_types: &[&str], sizes: &[usize]) -> HashMap<String, f64> {
    let rainbow = Arc::new(Rainbow::new());
    let chunk_size = 4; // 每个批次处理的大小

    // 为每个 MIME 类型创建计数器
    let mut mime_stats: HashMap<String, (Arc<AtomicUsize>, Arc<AtomicUsize>)> = HashMap::new();
    for mime_type in mime_types {
        mime_stats.insert(
            mime_type.to_string(),
            (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))), // (time, count)
        );
    }
    let mime_stats = Arc::new(mime_stats);

    // 对 test_sizes 进行分块并行处理
    let _: Vec<_> = sizes
        .par_chunks(chunk_size)
        .flat_map(|size_chunk| {
            let rainbow = Arc::clone(&rainbow);
            let mime_stats = Arc::clone(&mime_stats);

            // 对 MIME 类型也进行并行处理
            mime_types
                .par_iter()
                .filter_map(move |&mime_type| {
                    let test_start = std::time::Instant::now();
                    let stats = rainbow
                        .analyze_bandwidth_range(size_chunk, Some(mime_type.to_string()))
                        .unwrap();

                    if !stats.is_empty() {
                        let elapsed_ns = test_start.elapsed().as_nanos() as usize;
                        // 更新 MIME 类型统计
                        if let Some((time_counter, count_counter)) = mime_stats.get(mime_type) {
                            time_counter.fetch_add(elapsed_ns, Ordering::Relaxed);
                            count_counter.fetch_add(stats.len(), Ordering::Relaxed);
                        }
                    }
                    Some(())
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // 计算平均时间
    mime_stats
        .iter()
        .map(|(mime_type, (time_counter, count_counter))| {
            let total_ns = time_counter.load(Ordering::Relaxed);
            let count = count_counter.load(Ordering::Relaxed);
            let avg_time = if count > 0 {
                total_ns as f64 / count as f64 / 1_000_000.0 // 转换为毫秒
            } else {
                0.0
            };
            (mime_type.clone(), avg_time)
        })
        .collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    // 获取支持的 MIME 类型
    let rainbow = Rainbow::new();
    let mime_types: Vec<_> = rainbow.encoders.get_all_mime_types();
    let test_sizes = [100, 1000, 10000, 100 * 1024];

    // 单个 MIME 类型和大小的基准测试
    let mut group = c.benchmark_group("Single MIME Type");
    for size in test_sizes.iter() {
        for mime_type in &mime_types {
            group.bench_with_input(
                BenchmarkId::new((*mime_type).to_string(), size),
                &(mime_type, size),
                |b, &(mime_type, size)| {
                    b.iter(|| analyze_bandwidth(mime_type, *size));
                },
            );
        }
    }
    group.finish();

    // 并行处理所有 MIME 类型和大小的基准测试
    let mut group = c.benchmark_group("Parallel All MIME Types");
    for size in test_sizes.iter() {
        group.bench_with_input(
            BenchmarkId::new("all_types".to_string(), size),
            size,
            |b, &size| {
                b.iter(|| parallel_analyze_bandwidth(&mime_types, &[size]));
            },
        );
    }
    group.finish();

    // 完整的并行基准测试（所有大小和类型）
    let mut group = c.benchmark_group("Full Parallel Analysis");
    group.bench_function("all_sizes_and_types", |b| {
        b.iter(|| parallel_analyze_bandwidth(&mime_types, &test_sizes));
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
