use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize::SmallInput};
use kvs::{KvsEngine, KvStore, SledKvsEngine};

use rand::seq::IteratorRandom;
use tempfile::TempDir;

const NUM_DATA: usize = 100;
const MAX_LEN: usize = 100000;

fn criterion_benchmark_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_write");

    let mut rng = rand::thread_rng();
    let range = (1..MAX_LEN).choose_multiple(&mut rng, NUM_DATA).to_vec();

    group.bench_function("sled", |b| {
        b.iter_batched(
            || {
                let temp = TempDir::new().expect("unable to create temp directory.");
                let engine = SledKvsEngine::open(temp.path()).expect("unable to create a new storage.");
                engine
            },
            |sled| {
                for i in &range {
                    sled.set(format!("{}", i), format!("{}", i)).expect("unable to set value");
                }
            }
        ,SmallInput,)
    });

    group.bench_function("kvs", |b| {
        b.iter_batched(
            || {
                let temp = TempDir::new().expect("unable to create temp directory.");
                let engine = KvStore::open(temp.path()).expect("unable to create a new storage.");
                engine
            },
            |sled| {
                for i in &range {
                    sled.set(format!("{}", i), format!("{}", i)).expect("unable to set value");
                }
            }
        ,SmallInput,)
    });

    group.finish()

}

criterion_group!(benches, criterion_benchmark_write);
criterion_main!(benches);