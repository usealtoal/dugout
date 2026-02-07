use age::x25519;
use burrow::core::cipher::{decrypt, encrypt, Age, Cipher};
use burrow::core::config::Config;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::TempDir;

/// Generate a payload of given size.
fn generate_payload(size: usize) -> String {
    "x".repeat(size)
}

/// Generate age recipients.
fn generate_recipients(count: usize) -> Vec<x25519::Recipient> {
    (0..count)
        .map(|_| {
            let identity = x25519::Identity::generate();
            identity.to_public()
        })
        .collect()
}

/// Benchmark encrypt/decrypt roundtrip with varying payload sizes.
fn bench_encrypt_decrypt_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_decrypt_roundtrip");

    let sizes = [32, 256, 1024, 4096];
    let recipient_counts = [1, 3];

    for size in sizes {
        for &recipient_count in &recipient_counts {
            let payload = generate_payload(size);
            let recipients = generate_recipients(recipient_count);
            let identity = x25519::Identity::generate();
            let recipients_with_identity = {
                let mut r = recipients.clone();
                r.push(identity.to_public());
                r
            };

            group.throughput(Throughput::Bytes(size as u64));

            group.bench_with_input(
                BenchmarkId::new(
                    format!("{}_recipients", recipient_count),
                    format!("{}B", size),
                ),
                &payload,
                |b, payload| {
                    b.iter(|| {
                        let encrypted =
                            encrypt(black_box(payload), black_box(&recipients_with_identity))
                                .unwrap();
                        let decrypted =
                            decrypt(black_box(&encrypted), black_box(&identity)).unwrap();
                        black_box(decrypted);
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark encryption only (no decryption).
fn bench_encrypt_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_only");

    let sizes = [32, 256, 1024, 4096];
    let recipient_counts = [1, 3];

    for size in sizes {
        for &recipient_count in &recipient_counts {
            let payload = generate_payload(size);
            let recipients = generate_recipients(recipient_count);

            group.throughput(Throughput::Bytes(size as u64));

            group.bench_with_input(
                BenchmarkId::new(
                    format!("{}_recipients", recipient_count),
                    format!("{}B", size),
                ),
                &payload,
                |b, payload| {
                    b.iter(|| {
                        let encrypted =
                            encrypt(black_box(payload), black_box(&recipients)).unwrap();
                        black_box(encrypted);
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark decryption only (with pre-encrypted data).
fn bench_decrypt_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("decrypt_only");

    let sizes = [32, 256, 1024, 4096];
    let identity = x25519::Identity::generate();
    let recipients = vec![identity.to_public()];

    for size in sizes {
        let payload = generate_payload(size);
        let encrypted = encrypt(&payload, &recipients).unwrap();

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}B", size)),
            &encrypted,
            |b, encrypted| {
                b.iter(|| {
                    let decrypted = decrypt(black_box(encrypted), black_box(&identity)).unwrap();
                    black_box(decrypted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark config save/load operations.
fn bench_config_save_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_save_load");

    // Create temp directories for each benchmark iteration
    let secret_counts = [5, 20, 50];

    for &count in &secret_counts {
        group.bench_with_input(
            BenchmarkId::new("save_load", format!("{}_secrets", count)),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        // Setup: create temp dir and config
                        let temp_dir = TempDir::new().unwrap();
                        std::env::set_current_dir(temp_dir.path()).unwrap();

                        let mut config = Config::new();
                        config.recipients.insert(
                            "test".to_string(),
                            "age1test1234567890abcdefghijklmnopqrstuvwxyz1234567890abc".to_string(),
                        );

                        for i in 0..count {
                            config.secrets.insert(
                                format!("SECRET_KEY_{}", i),
                                format!("age-encrypted-value-{}", i),
                            );
                        }

                        (temp_dir, config)
                    },
                    |(temp_dir, config)| {
                        // Benchmark: save and load
                        config.save().unwrap();
                        let loaded = Config::load().unwrap();
                        black_box(loaded);
                        drop(temp_dir); // Cleanup
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark config save only.
fn bench_config_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_save");

    let secret_counts = [5, 20, 50];

    for &count in &secret_counts {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_secrets", count)),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        std::env::set_current_dir(temp_dir.path()).unwrap();

                        let mut config = Config::new();
                        config.recipients.insert(
                            "test".to_string(),
                            "age1test1234567890abcdefghijklmnopqrstuvwxyz1234567890abc".to_string(),
                        );

                        for i in 0..count {
                            config.secrets.insert(
                                format!("SECRET_KEY_{}", i),
                                format!("age-encrypted-value-{}", i),
                            );
                        }

                        (temp_dir, config)
                    },
                    |(temp_dir, config)| {
                        config.save().unwrap();
                        drop(temp_dir);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encrypt_decrypt_roundtrip,
    bench_encrypt_only,
    bench_decrypt_only,
    bench_config_save_load,
    bench_config_save
);
criterion_main!(benches);
