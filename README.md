# Trading Engine

A high-performance concurrent trade analytics engine written in **Rust**.  
This project demonstrates efficient **trade processing**, **parallel analytics updates**, and **throughput benchmarking**.

---

## 📌 Overview

The engine processes large volumes of trades in real time while ensuring:

- **Ordering guarantees per trading pair** (trades within the same pair are processed sequentially by timestamp).
- **Parallelism across pairs** (different pairs are processed concurrently).
- **Memory efficiency** (only analytics are stored, not the full trade history).
- **Scalability** (capable of handling tens of thousands of trades per second).

The project is designed as a technical assessment and reference implementation of **concurrent system design in Rust**.

---

## ⚙️ Features

- **Data Model**
    - `Trade` struct (timestamp, pair, price, amount, side).
    - `Analytics` struct (per-pair statistics: last price, volume, buy/sell breakdown, high/low, trade count).
    - `Side` enum (`Buy` / `Sell`).

- **Trade Processor**
    - `TradeProcessor` trait defining core methods:
        - `process_trades` – batch processing with per-pair ordering.
        - `get_analytics` – fetch analytics for a single pair.
        - `get_all_analytics` – fetch analytics for all pairs.
    - Implementation using `DashMap` for thread-safe, lock-minimized concurrency.

- **Mock Trade Generator**
    - Generates random trades across multiple pairs for testing and benchmarking.
    - Allows control of pair set and batch size.

- **Testing**
    - Unit tests for correctness of analytics.
    - Concurrency tests ensuring ordering guarantees.
    - Stress tests for multiple parallel tasks.

- **Benchmarking**
    - Criterion-based benchmarks for throughput measurement.
    - Example throughput tests with `cargo bench` and `examples/throughput.rs`.

---

## 📂 Project Structure

```
trading-engine/
├── Cargo.toml                # Project configuration
├── src/
│   ├── main.rs               # CLI entrypoint
│   ├── lib.rs                # Library exports
│   ├── models.rs             # Data models (Trade, Analytics, Side)
│   ├── errors.rs             # Error handling (AppError, AppResult)
│   ├── traits.rs             # TradeProcessor trait
│   ├── utils.rs              # Utility functions
│   └── services/
│       ├── trade_processor/  # MyTradeProcessor implementation
│       └── trade_generator/  # TradeGenerator implementation
├── tests/
│   ├── tests.rs              # Unit tests
│   └── concurrency.rs        # Parallelism & ordering tests
├── benches/
│   └── trade_throughput.rs   # Criterion benchmarks
└── examples/
    └── throughput.rs         # Quick runtime benchmark
```

---

## 🚀 Getting Started

### Prerequisites
- Rust (latest stable recommended)
- Cargo package manager

### Build
```bash
cargo build --release
```

### Run Example
```bash
cargo run --release --example throughput
```

### Run Tests
```bash
cargo test -- --nocapture
```

### Run Benchmarks
```bash
cargo bench
```

---

## 🧪 Example Usage

```rust
use trading_engine::models::{Trade, Side};
use trading_engine::services::trade_generator::TradeGenerator;
use trading_engine::services::trade_processor::MyTradeProcessor;
use trading_engine::traits::TradeProcessor;

#[tokio::main]
async fn main() {
    let pairs = vec!["BTC-USD".to_string(), "ETH-USD".to_string()];
    let generator = TradeGenerator::new(pairs.clone());

    let processor = MyTradeProcessor::new(generator);

    // Generate random trades
    let trades = processor.trade_generator().generate_batch(1000);

    // Process trades
    processor.process_trades(trades).await.unwrap();

    // Get analytics for BTC-USD
    let btc = processor.get_analytics("BTC-USD").await.unwrap().unwrap();
    println!("BTC-USD analytics: {:?}", btc);
}
```

---

## 📊 Benchmarks

### Criterion Benchmark
```bash
cargo bench
```
Outputs median throughput (`trades/sec`) for various batch sizes and pair counts.

### Quick Example Benchmark
```bash
cargo run --release --example throughput
```
Outputs wall-clock time and calculated throughput.

---

## 🔍 Concurrency Tests

The project includes dedicated concurrency tests (`tests/concurrency.rs`):

- **Ordering within batch** – verifies trades in one batch are processed by timestamp order.
- **Concurrent batches (same pair)** – detects violations if ordering is not preserved across multiple calls.
- **Concurrent batches (different pairs)** – ensures independent parallel updates.
- **Massive parallelism test** – validates correctness of aggregated analytics under heavy load.

---
