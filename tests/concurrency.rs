#[cfg(test)]
mod concurrency_tests {
    use rand::rng;
    use rand::seq::SliceRandom;
    use std::sync::Arc;
    use tokio::join;
    use trading_engine::models::{Side, Trade};
    use trading_engine::services::trade_processor::MyTradeProcessor;
    use trading_engine::traits::TradeProcessor;

    fn t(ts: u64, pair: &str, price: f64, amount: f64, side: Side) -> Trade {
        Trade {
            timestamp: ts,
            pair: pair.to_string(),
            price,
            amount,
            side,
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ordering_within_single_batch_is_preserved() {
        let processor = MyTradeProcessor::default();

        let mut trades = vec![
            t(300, "BTC-USD", 51000.0, 1.0, Side::Buy),
            t(100, "BTC-USD", 50000.0, 1.0, Side::Buy),
            t(200, "BTC-USD", 50500.0, 1.0, Side::Sell),
        ];
        trades.shuffle(&mut rng());

        processor
            .process_trades(trades)
            .await
            .expect("process trades");

        let a = processor
            .get_analytics("BTC-USD")
            .await
            .expect("Cannot get result")
            .expect("Cannot get value");
        assert_eq!(a.trade_count, 3);
        assert_eq!(a.last_price, 51000.0);
        assert!((a.high - 51000.0).abs() < 1e-9);
        assert!((a.low - 50000.0).abs() < 1e-9);
        assert!((a.total_volume - 3.0).abs() < 1e-9);
        assert!((a.buy_volume - 2.0).abs() < 1e-9);
        assert!((a.sell_volume - 1.0).abs() < 1e-9);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn concurrent_batches_same_pair_ordering_across_batches() {
        let processor = MyTradeProcessor::default();

        let mut batch_a: Vec<Trade> = (1..=1000)
            .map(|ts| t(ts, "BTC-USD", 50_000.0, 1.0, Side::Buy))
            .collect();

        let mut batch_b: Vec<Trade> = (500..=1500)
            .map(|ts| t(ts, "BTC-USD", 60_000.0, 1.0, Side::Sell))
            .collect();

        batch_a.shuffle(&mut rng());
        batch_b.shuffle(&mut rng());

        let (r1, r2) = join!(
            processor.process_trades(batch_a),
            processor.process_trades(batch_b)
        );
        r1.expect("Cannot process batch A");
        r2.expect("Cannot process batch B");

        let a = processor
            .get_analytics("BTC-USD")
            .await
            .expect("Cannot get result")
            .expect("Cannot get value");

        assert_eq!(a.trade_count, 2001);
        assert!(
            (a.last_price - (60_000.0)).abs() < 1e-9,
            "last_price must come from max timestamp across all batches"
        );
        assert!((a.low - 50_000.0).abs() < 1e-9);
        assert!((a.high - 60_000.0).abs() < 1e-9);

        assert!((a.buy_volume - 1000.0).abs() < 1e-9);
        assert!((a.sell_volume - 1001.0).abs() < 1e-9);
        assert!((a.total_volume - 2001.0).abs() < 1e-9);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn concurrent_batches_different_pairs_are_independent() {
        let processor = MyTradeProcessor::default();

        let mut btc: Vec<Trade> = (1..=50_000)
            .map(|ts| t(ts, "BTC-USD", 40_000.0 + (ts % 10) as f64, 1.0, Side::Buy))
            .collect();
        btc.shuffle(&mut rng());

        let mut eth: Vec<Trade> = (1..=50_000)
            .map(|ts| t(ts, "ETH-USD", 2_000.0 + (ts % 7) as f64, 1.0, Side::Sell))
            .collect();
        eth.shuffle(&mut rng());

        let (r_btc, r_eth) = join!(processor.process_trades(btc), processor.process_trades(eth));
        r_btc.expect("Cannot process BTC trades");
        r_eth.expect("Cannot process ETH trades");

        let a_btc = processor
            .get_analytics("BTC-USD")
            .await
            .expect("Cannot get result")
            .expect("Cannot get result");
        let a_eth = processor
            .get_analytics("ETH-USD")
            .await
            .expect("Cannot get result")
            .expect("Cannot get result");

        assert_eq!(a_btc.trade_count, 50_000);
        assert_eq!(a_eth.trade_count, 50_000);
        assert!((a_btc.total_volume - 50_000.0).abs() < 1e-9);
        assert!((a_eth.total_volume - 50_000.0).abs() < 1e-9);
        assert!(a_btc.high >= a_btc.low);
        assert!(a_eth.high >= a_eth.low);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn many_tasks_many_pairs_accumulate_correctly() {
        let n_pairs = 20usize;
        let pairs: Vec<String> = (0..n_pairs).map(|i| format!("PAIR-{i}")).collect();

        let processor = Arc::new(MyTradeProcessor::default());

        let tasks = 40usize;
        let per_task = 10_000usize;
        let total = tasks * per_task;

        let mut handles = Vec::new();
        for seed in 0..tasks {
            let processor = processor.clone();
            let pairs = pairs.clone();

            let trades: Vec<Trade> = (0..per_task)
                .map(|i| {
                    let pair = &pairs[(i + seed) % pairs.len()];
                    let ts = (seed * per_task + i) as u64 + 1;
                    let price = 10_000.0 + ((i % 97) as f64);
                    let amount = 1.0;
                    let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
                    t(ts, pair, price, amount, side)
                })
                .collect();

            handles.push(tokio::spawn(async move {
                processor.process_trades(trades).await
            }));
        }

        for h in handles {
            h.await.expect("Cannot get result").expect("I/O error");
        }

        let all = processor
            .get_all_analytics()
            .await
            .expect("Cannot get analytics");
        let total_count: u64 = all.values().map(|a| a.trade_count).sum();
        let total_volume: f64 = all.values().map(|a| a.total_volume).sum();
        let total_buy: f64 = all.values().map(|a| a.buy_volume).sum();
        let total_sell: f64 = all.values().map(|a| a.sell_volume).sum();

        assert_eq!(total_count as usize, total);
        assert!((total_volume - total as f64).abs() < 1e-9);
        assert!((total_buy + total_sell - total as f64).abs() < 1e-9);
    }
}
