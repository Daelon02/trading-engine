use criterion::{
    criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput, black_box,
};
use tokio::runtime::Runtime;

use trading_engine::{
    errors::AppResult,
    models::{Trade},
    services::trade_generator::TradeGenerator,
    traits::TradeProcessor,
};
use trading_engine::my_trade_processor::MyTradeProcessor;

fn make_pairs(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("PAIR-{i}")).collect()
}

fn gen_trades(mut gen: TradeGenerator, count: usize) -> (TradeGenerator, Vec<Trade>) {
    let batch = gen.generate_batch(count);
    (gen, batch)
}

fn bench_trade_throughput(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let scenarios = [
        (10usize, 50_000usize),
        (50, 100_000),
        (100, 200_000),
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
                        let gen = TradeGenerator::new(pairs.clone());
                        let (gen, trades) = gen_trades(gen, batch_size);

                        let proc = MyTradeProcessor::new(gen);

                        (proc, trades)
                    },
                    |(proc, trades)| async move {
                        let trades = black_box(trades);
                        proc.process_trades(trades).await.unwrap();
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