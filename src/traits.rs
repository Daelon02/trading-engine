use crate::errors::AppResult;
use crate::models::{Analytics, Trade};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
#[allow(dead_code)]
pub trait TradeProcessor: Send + Sync {
    async fn process_trades(&self, trades: Vec<Trade>) -> AppResult<()>;
    async fn get_analytics(&self, pair: &str) -> AppResult<Option<Analytics>>;
    async fn get_all_analytics(&self) -> AppResult<HashMap<String, Analytics>>;
}
