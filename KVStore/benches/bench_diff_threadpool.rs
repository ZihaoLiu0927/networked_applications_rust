use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use crossbeam_utils::sync::WaitGroup;
use kvs::{
    client::Client,
    server::Server,
    thread_pool::{RayonThreadPool, SharedQueueThreadPool},
    KvStore, SledKvsEngine, ThreadPool,
};
use log::LevelFilter;
extern crate env_logger;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
};

use tempfile::TempDir;

const NUM_THREADS: [u32; 6] = [1, 2, 4, 8, 16, 32];
const NUM_REQUEST: usize = 100;

fn criterion_benchmark_shared_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("shared_queue");

    env_logger::builder()
        .filter_level(LevelFilter::Error)
        .init();

    for num in NUM_THREADS.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(num), num, |b, n| {
            let temp = TempDir::new().expect("unable to create temp directory.");
            let engine = KvStore::open(temp.path()).expect("unable to create a new storage.");

            let pool = SharedQueueThreadPool::new(*n).expect("unable to create thread pool.");

            let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4004);

            let killed = Arc::new(AtomicBool::new(false));

            let mut server = Server::new(engine, addr, pool, Arc::clone(&killed))
                .expect("unale to create server.");

            let handle = thread::spawn(move || {
                server.run().expect("unable to run the server");
            });

            let keys: Vec<String> = (0..NUM_REQUEST)
                .map(|x| format!("randomeKey{}", x))
                .collect();

            let value = "randomValue:rustacean".to_owned();

            let client_pool = SharedQueueThreadPool::new(NUM_REQUEST as u32)
                .expect("unable to create client pool.");

            b.iter(|| {
                let wg = WaitGroup::new();
                for i in 0..NUM_REQUEST {
                    let key = keys[i].clone();
                    let value = value.clone();
                    let wg = wg.clone();
                    client_pool.spawn(move || {
                        match Client::new(addr) {
                            Ok(mut client) => {
                                if let Err(e) = client.set(key, value) {
                                    eprintln!("error in executing client request: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("error in create client object: {}", e);
                            }
                        }
                        drop(wg);
                    });
                }
                wg.wait();
            });

            killed.store(true, Ordering::SeqCst);

            // IMPORTANT: create a new client to unblock the server from listener.incoming() method.
            let _ = Client::new(addr);

            if let Err(e) = handle.join() {
                eprintln!("unable to exit server: {:?}", e);
            }
        });
    }
}

fn criterion_benchmark_rayon_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("rayon_pool");

    for num in NUM_THREADS.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(num), num, |b, n| {
            let temp = TempDir::new().expect("unable to create temp directory.");
            let engine = KvStore::open(temp.path()).expect("unable to create a new storage.");

            let pool = RayonThreadPool::new(*n).expect("unable to create thread pool.");

            let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4004);

            let killed = Arc::new(AtomicBool::new(false));

            let mut server = Server::new(engine, addr, pool, Arc::clone(&killed))
                .expect("unale to create server.");

            let handle = thread::spawn(move || {
                server.run().expect("unable to run the server");
            });

            let keys: Vec<String> = (0..NUM_REQUEST)
                .map(|x| format!("randomeKey{}", x))
                .collect();

            let value = "randomValue:rustacean".to_owned();

            let client_pool =
                RayonThreadPool::new(NUM_REQUEST as u32).expect("unable to create client pool.");

            b.iter(|| {
                let wg = WaitGroup::new();
                for i in 0..NUM_REQUEST {
                    let key = keys[i].clone();
                    let value = value.clone();
                    let wg = wg.clone();
                    client_pool.spawn(move || {
                        match Client::new(addr) {
                            Ok(mut client) => {
                                if let Err(e) = client.set(key, value) {
                                    eprintln!("error in executing client request: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("error in create client object: {}", e);
                            }
                        }
                        drop(wg);
                    });
                }
                wg.wait();
            });

            killed.store(true, Ordering::SeqCst);

            // IMPORTANT: create a new client to unblock the server from listener.incoming() method.
            let _ = Client::new(addr);

            if let Err(e) = handle.join() {
                eprintln!("unable to exit server: {:?}", e);
            }
        });
    }
}

criterion_group!(
    benches,
    criterion_benchmark_shared_queue,
    criterion_benchmark_rayon_pool
);
criterion_main!(benches);
