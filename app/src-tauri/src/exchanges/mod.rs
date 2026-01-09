//! # Exchanges Module
//! 
//! このモジュールは、異なる暗号通貨取引所のAPIを共通のインターフェースで扱うための
//! 抽象化レイヤーを提供します。新しい取引所を追加する場合は、`Exchange` トレイトを
//! 実装した構造体を作成してください。

use async_trait::async_trait;
use crate::models::KlineData;
use anyhow::Result;

/// 取引所ごとの固有ロジックを共通化するためのトレイト。
/// 
/// `async_trait` マクロを使用することで、トレイト内での非同期メソッド定義を可能にしています。
#[async_trait]
pub trait Exchange: Send + Sync {
    /// 取引所の識別子を返します (例: "binance", "bybit")。
    /// 主に内部的な分岐や、ログ出力に使用されます。
    fn id(&self) -> &str;

    /// 取引所から取引可能な全てのシンボル（USDTペアなど）のリストを取得します。
    /// 
    /// # Returns
    /// シンボル名のベクタ (例: `vec!["BTCUSDT", "ETHUSDT", ...]`)
    async fn get_symbols(&self) -> Result<Vec<String>>;

    /// 指定されたシンボルと時間足のローソク足データを取得します。
    /// 
    /// # Arguments
    /// * `symbol` - 通貨ペア名 (取引所形式)
    /// * `interval` - 時間足 (例: "1m", "1min", "1" など、取引所ごとの形式)
    /// 
    /// # Returns
    /// `KlineData` のベクタ。原則として、古いデータから新しいデータの順（昇順）で返されます。
    async fn fetch_candles(&self, symbol: &str, interval: &str) -> Result<Vec<KlineData>>;
}

pub mod binance;

pub mod bybit;

pub mod bitget;

pub mod okx;

pub mod kucoin;

pub mod kraken;
