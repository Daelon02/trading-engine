#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: u64,
    pub pair: String,
    pub price: f64,
    pub amount: f64,
    pub side: Side,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Default)]
pub struct Analytics {
    pub pair: String,
    pub last_price: f64,
    pub total_volume: f64,
    pub buy_volume: f64,
    pub sell_volume: f64,
    pub high: f64,
    pub low: f64,
    pub trade_count: u64,
}
