use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sui_mvr::prelude::*;
use std::time::Duration;
use tokio::runtime::Runtime;

fn create_test_resolver() -> MvrResolver {
    let overrides = MvrOverrides::new()
        .with_package("@bench/pkg1".to_string(), "0x111".to_string())
        .with_package("@bench/pkg2".to_string(), "0x222".to_string())
        .with_package("@bench/pkg3".to_string(), "0x333".to_string())
        .with_package("@bench/pkg4".to_string(), "0x444".to_string())
        .with_package("@bench/pkg5".to_string(), "0x555".to_string())
        .with_type("@bench/pkg1::Type1".to_string(), "0x111::module::Type1".to_string())
        .with_type("@bench/pkg2::Type2".to_string(), "0x222::module::Type2".to_string())
        .with_type("@bench/pkg3::Type3".to_string(), "0x333::module::Type3".to_string());

    MvrResolver::testnet().with_overrides(overrides)
}

fn bench_single_package_resolution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let resolver = create_test_resolver();

    c.bench_function("single_package_resolution", |b| {
        b.to_async(&rt).iter(|| async {
            let result = resolver
                .resolve_package(black_box("@bench/pkg1"))
                .await
                .unwrap();
            black_box(result);
        });
    });
}

fn bench_batch_package_resolution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let resolver = create_test_resolver();

    let mut group = c.benchmark_group("batch_package_resolution");
    
    for size in [1, 2, 4, 8, 16].iter() {
        let packages: Vec<&str> = (0..*size)
            .map(|i| match i % 5 {
                0 => "@bench/pkg1",
                1 => "@bench/pkg2", 
                2 => "@bench/pkg3",
                3 => "@bench/pkg4",
                _ => "@bench/pkg5",
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("packages", size), &packages, |b, packages| {
            b.to_async(&rt).iter(|| async {
                let result = resolver
                    .resolve_packages(black_box(packages))
                    .await
                    .unwrap();
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_type_resolution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let resolver = create_test_resolver();

    c.bench_function("single_type_resolution", |b| {
        b.to_async(&rt).iter(|| async {
            let result = resolver
                .resolve_type(black_box("@bench/pkg1::Type1"))
                .await
                .unwrap();
            black_box(result);
        });
    });
}

fn bench_cache_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let resolver = create_test_resolver();

    // Warm up cache
    rt.block_on(async {
        let _ = resolver.resolve_package("@bench/pkg1").await;
        let _ = resolver.resolve_package("@bench/pkg2").await;
    });

    let mut group = c.benchmark_group("cache_performance");
    
    group.bench_function("cache_hit", |b| {
        b.to_async(&rt).iter(|| async {
            // This should hit cache
            let result = resolver
                .resolve_package(black_box("@bench/pkg1"))
                .await
                .unwrap();
            black_box(result);
        });
    });

    group.bench_function("cache_miss", |b| {
        b.to_async(&rt).iter(|| async {
            // Create fresh resolver for each iteration to ensure cache miss
            let fresh_resolver = create_test_resolver();
            let result = fresh_resolver
                .resolve_package(black_box("@bench/pkg1"))
                .await
                .unwrap();
            black_box(result);
        });
    });

    group.finish();
}

fn bench_individual_vs_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let packages = vec!["@bench/pkg1", "@bench/pkg2", "@bench/pkg3", "@bench/pkg4"];

    let mut group = c.benchmark_group("individual_vs_batch");

    group.bench_function("individual_resolution", |b| {
        b.to_async(&rt).iter(|| async {
            let resolver = create_test_resolver();
            let mut results = Vec::new();
            
            for &pkg in black_box(&packages) {
                let result = resolver.resolve_package(pkg).await.unwrap();
                results.push(result);
            }
            
            black_box(results);
        });
    });

    group.bench_function("batch_resolution", |b| {
        b.to_async(&rt).iter(|| async {
            let resolver = create_test_resolver();
            let result = resolver
                .resolve_packages(black_box(&packages))
                .await
                .unwrap();
            black_box(result);
        });
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let resolver = create_test_resolver();

    c.bench_function("invalid_package_name", |b| {
        b.to_async(&rt).iter(|| async {
            let result = resolver
                .resolve_package(black_box("invalid-name"))
                .await;
            black_box(result);
        });
    });
}

fn bench_concurrent_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let resolver = create_test_resolver();

    c.bench_function("concurrent_resolution", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks = vec![
                resolver.resolve_package("@bench/pkg1"),
                resolver.resolve_package("@bench/pkg2"),
                resolver.resolve_package("@bench/pkg3"),
                resolver.resolve_package("@bench/pkg4"),
            ];
            
            let results = futures::future::join_all(black_box(tasks)).await;
            black_box(results);
        });
    });
}

fn bench_configuration_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration");

    group.bench_function("create_mainnet_resolver", |b| {
        b.iter(|| {
            let resolver = MvrResolver::mainnet();
            black_box(resolver);
        });
    });

    group.bench_function("create_testnet_resolver", |b| {
        b.iter(|| {
            let resolver = MvrResolver::testnet();
            black_box(resolver);
        });
    });

    group.bench_function("create_custom_config", |b| {
        b.iter(|| {
            let config = MvrConfig::mainnet()
                .with_cache_ttl(Duration::from_secs(1800))
                .with_timeout(Duration::from_secs(30));
            let resolver = MvrResolver::new(black_box(config));
            black_box(resolver);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_package_resolution,
    bench_batch_package_resolution,
    bench_type_resolution,
    bench_cache_performance,
    bench_individual_vs_batch,
    bench_error_handling,
    bench_concurrent_access,
    bench_configuration_overhead
);
criterion_main!(benches);