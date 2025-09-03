use crate::errors::{AppError, AppResult};
use crate::models::{Analytics, Side, Trade};
use crate::traits::TradeProcessor;
use dashmap::DashMap;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct MyTradeProcessor {
    analytics_data: Arc<DashMap<String, Analytics>>,
}

#[async_trait::async_trait]
impl TradeProcessor for MyTradeProcessor {
    async fn process_trades(&self, trades: Vec<Trade>) -> AppResult<()> {
        let mut by_pair: HashMap<String, Vec<Trade>> = HashMap::new();
        for t in trades {
            by_pair.entry(t.pair.clone()).or_default().push(t);
        }

        for ts in by_pair.values_mut() {
            ts.sort_by_key(|t| t.timestamp);
        }

        let mut tasks: FuturesUnordered<tokio::task::JoinHandle<Result<_, AppError>>> =
            FuturesUnordered::new();

        for (pair, ts) in by_pair {
            let map = self.analytics_data.clone();

            tasks.push(tokio::spawn(async move {
                let mut entry = map.entry(pair.clone()).or_default();

                for trade in ts {
                    entry.pair = trade.pair;
                    entry.last_price = trade.price;
                    entry.total_volume += trade.amount;
                    match trade.side {
                        Side::Buy => entry.buy_volume += trade.amount,
                        Side::Sell => entry.sell_volume += trade.amount,
                    }
                    if trade.price > entry.high || entry.high == 0.0 {
                        entry.high = trade.price;
                    }

                    if trade.price < entry.low || entry.low == 0.0 {
                        entry.low = trade.price;
                    }
                    entry.trade_count += 1;

                    log::debug!("Processed trade for pair {pair:?}");
                }
                Ok(())
            }));
        }

        while let Some(res) = tasks.next().await {
            res??; // propagate JoinError/AppError
        }

        Ok(())
    }

    async fn get_analytics(&self, pair: &str) -> AppResult<Option<Analytics>> {
        let data = self.analytics_data.clone();
        log::debug!("Getting analytics for pair: {pair:?}");
        Ok(data.get(pair).map(|entry| entry.value().clone()))
    }

    async fn get_all_analytics(&self) -> AppResult<HashMap<String, Analytics>> {
        let data = self.analytics_data.clone();
        Ok(data
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect())
    }
}
