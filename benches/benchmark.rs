use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::prelude::*;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn reads() -> Result<u64, Box<dyn std::error::Error>> {
    let mut one_kb_string: String = "".to_string();
    for _i in 1..128{
        one_kb_string += "12345678";
    }
    let write_url = "http://144.24.175.153/v1/channel/benchmarkread/write/".to_string() + &one_kb_string;
    let _resp = reqwest::blocking::get(write_url)?;

    let mut num_reads = Vec::new();
    for i in 1..50{
        num_reads.push(i);
    }
    let total = num_reads.par_iter().map(|_| {
        let read_url = "http://144.24.175.153/v1/channel/benchmarkread/read";
        let mut len = 0;
        for _i in 1..10{
            let resp = reqwest::blocking::get(read_url).unwrap();
            len += resp.content_length().unwrap() as u64;
        }
        len
    }).sum();

    println!("{:#?}", total);
    Ok(total)
}

fn criterion_benchmark(c: &mut Criterion) {
    rayon::ThreadPoolBuilder::new().num_threads(20).build_global().unwrap();
    //c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("read", |b| b.iter(|| reads()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
