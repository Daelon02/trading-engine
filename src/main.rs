use crate::errors::AppResult;
use crate::services::trade_generator::TradeGen;
use crate::services::trade_processor::MyTradeProcessor;
use crate::traits::TradeProcessor;
use crate::utils::init_logging;

mod errors;
mod models;
mod services;
mod traits;
mod utils;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> AppResult<()> {
    init_logging()?;

    let trade_generator = TradeGen::new(vec!["BTC-USD".into(), "ETH-USD".into()]);
    let processor = MyTradeProcessor::default();

    let trades = trade_generator.generate_batch(100_000);

    processor.process_trades(trades).await?;

    let analytics = processor.get_all_analytics().await?;
    for (pair, data) in analytics {
        log::info!("Analytics for {pair}: {data:?}");
    }

    log::info!("Shutdown complete.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::{Side, Trade};
    use crate::services::trade_processor::MyTradeProcessor;
    use crate::traits::TradeProcessor;
    use crate::utils::init_logging;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_ordering_guarantee() {
        init_logging().expect("Failed to initialize logging");
        let processor = MyTradeProcessor::default();

        let trades = vec![
            Trade {
                timestamp: 100,
                pair: "BTC-USD".into(),
                price: 50000.0,
                amount: 1.0,
                side: Side::Buy,
            },
            Trade {
                timestamp: 300,
                pair: "BTC-USD".into(),
                price: 51000.0,
                amount: 1.0,
                side: Side::Buy,
            },
            Trade {
                timestamp: 200,
                pair: "BTC-USD".into(),
                price: 50500.0,
                amount: 1.0,
                side: Side::Sell,
            },
        ];

        processor
            .process_trades(trades)
            .await
            .expect("Failed to process trades");

        let analytics = processor
            .get_analytics("BTC-USD")
            .await
            .expect("Cannot get result for analytics by BTC-USD pair")
            .expect("Cannot get analytics result");

        assert_eq!(analytics.last_price, 51000.0);
        assert_eq!(analytics.trade_count, 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_saving_analytics_len() {
        let trade_gen = crate::services::trade_generator::TradeGen::new(vec![
            "BTC-USD".into(),
            "ETH-USD".into(),
            "XRP-USD".into(),
            "LTC-USD".into(),
            "BCH-USD".into(),
            "ADA-USD".into(),
            "DOT-USD".into(),
            "LINK-USD".into(),
            "BNB-USD".into(),
            "SOL-USD".into(),
        ]);
        let processor = MyTradeProcessor::default();
        let trades = trade_gen.generate_batch(1000);

        processor
            .process_trades(trades)
            .await
            .expect("Failed to process trades");

        let analytics = processor
            .get_all_analytics()
            .await
            .expect("Cannot get all analytics")
            .len();

        assert_eq!(analytics, 10);
    }
}
