use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tokio::runtime::Runtime;

use trading_engine::{
    services::{trade_generator::TradeGen, trade_processor::MyTradeProcessor},
    traits::TradeProcessor,
};

fn make_pairs(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("PAIR-{i}")).collect()
}

fn bench_trade_throughput(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let scenarios = [
        (2, 10_000),
        (10usize, 50_000),
        (50, 100_000),
        (100, 200_000),
        (500, 500_000),
        (1000, 1_000_000),
    ];

    let mut group = c.benchmark_group("trade_throughput");

    for (n_pairs, batch_size) in scenarios {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_function(
            BenchmarkId::from_parameter(format!("{n_pairs}_pairs_{batch_size}_trades")),
            |b| {
                b.to_async(&rt).iter_batched(
                    move || {
                        let pairs = make_pairs(n_pairs);
                        let generator = TradeGen::new(pairs);
                        let trades = generator.generate_batch(batch_size);

                        let proc = MyTradeProcessor::default();

                        (proc, trades)
                    },
                    |(proc, trades)| async move {
                        let trades = std::hint::black_box(trades);
                        proc.process_trades(trades)
                            .await
                            .expect("Failed to process trades");
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_trade_throughput);
criterion_main!(benches);
