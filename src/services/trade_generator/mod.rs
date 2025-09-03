use crate::models::{Side, Trade};
use rand::Rng;
use rand::prelude::IndexedRandom;

pub struct TradeGen {
    #[allow(dead_code)]
    pairs: Vec<String>,
}

impl TradeGen {
    #[allow(dead_code)]
    pub fn new(pairs: Vec<String>) -> Self {
        Self { pairs }
    }

    #[allow(dead_code)]
    pub fn generate_batch(&self, count: usize) -> Vec<Trade> {
        let mut trades = Vec::with_capacity(count);
        for _ in 0..count {
            let pair = self
                .pairs
                .choose(&mut rand::rng())
                .expect("No pair found")
                .clone();
            let price = rand::rng().random_range(1000.0..60000.0);
            let amount = rand::rng().random_range(0.01..5.0);
            let side = if rand::random() {
                Side::Buy
            } else {
                Side::Sell
            };
            let timestamp = chrono::Utc::now().timestamp_millis() as u64;

            trades.push(Trade {
                timestamp,
                pair,
                price,
                amount,
                side,
            });
        }
        trades
    }
}
