use async_trait::async_trait;
use crate::models::{KlineData, KlineWithIndicator};
use anyhow::Result;

#[async_trait]
pub trait Exchange: Send + Sync {
    /// 取引所の識別子 (例: "binance", "bybit")
    fn id(&self) -> &str;

    /// 取引可能なシンボル一覧を取得する
    async fn get_symbols(&self) -> Result<Vec<String>>;

    /// 指定されたシンボル、期間のローソク足データを取得する
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>>;
}

pub mod binance;
pub mod bybit;
pub mod bitget;
pub mod okx;
