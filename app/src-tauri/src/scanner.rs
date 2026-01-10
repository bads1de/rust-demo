use serde::{Deserialize, Serialize};

/// スキャナーのフィルタ条件を表す列挙型。
///
/// 各バリアントは、テクニカル指標に基づくフィルタ条件を定義します。
/// 複数の条件を組み合わせる場合はAND条件として評価されます。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterCondition {
    /// RSI が指定値を超える (例: RSI > 75 で買われすぎ)
    RsiAbove(f64),
    /// RSI が指定値を下回る (例: RSI < 25 で売られすぎ)
    RsiBelow(f64),
    /// ストキャスティクス %K が指定値を超える
    StochKAbove(f64),
    /// ストキャスティクス %K が指定値を下回る
    StochKBelow(f64),
    /// %K > %D (ゴールデンクロス方向)
    StochKAboveD,
    /// %K < %D (デッドクロス方向)
    StochKBelowD,
    /// 終値 > SMA (強気状態)
    CloseAboveSma,
    /// 終値 < SMA (弱気状態)
    CloseBelowSma,
    /// 終値 > ボリンジャーバンド上限 (過熱)
    CloseAboveUpperBand,
    /// 終値 < ボリンジャーバンド下限 (売られすぎ)
    CloseBelowLowerBand,
    /// MACD ゴールデンクロス (MACD > Signal かつ 前足で MACD < Signal)
    MacdCrossUp,
    /// MACD デッドクロス (MACD < Signal かつ 前足で MACD > Signal)
    MacdCrossDown,
    /// MACD > Signal (上昇トレンド中)
    MacdAboveSignal,
    /// MACD < Signal (下降トレンド中)
    MacdBelowSignal,
}

/// スキャナープリセットを表す構造体。
///
/// 名前、説明、およびフィルタ条件のリストを持ちます。
/// 条件リスト内のすべての条件がAND条件として評価されます。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerPreset {
    /// プリセットの識別子 (例: "rsi_overbought")
    pub id: String,
    /// 表示名 (例: "RSI過熱")
    pub name: String,
    /// プリセットの説明
    pub description: String,
    /// カテゴリ (例: "overbought", "oversold", "trend_up", "trend_down", "multi")
    pub category: String,
    /// フィルタ条件のリスト (AND条件)
    pub conditions: Vec<FilterCondition>,
}

/// スキャン結果を表す構造体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 取引所ID (例: "binance")
    pub exchange: String,
    /// シンボル (例: "BTCUSDT")
    pub symbol: String,
    /// 現在価格
    pub price: f64,
    /// RSI値
    pub rsi: Option<f64>,
    /// ストキャスティクス %K
    pub stoch_k: Option<f64>,
    /// ストキャスティクス %D
    pub stoch_d: Option<f64>,
    /// MACD
    pub macd: Option<f64>,
    /// MACDシグナル
    pub macd_signal: Option<f64>,
    /// SMA
    pub sma: Option<f64>,
    /// ボリンジャーバンド上限
    pub upper_band: Option<f64>,
    /// ボリンジャーバンド下限
    pub lower_band: Option<f64>,
    /// 前足のMACD（クロス判定用）
    pub prev_macd: Option<f64>,
    /// 前足のMACDシグナル（クロス判定用）
    pub prev_macd_signal: Option<f64>,
}

impl ScanResult {
    /// 単一のフィルタ条件を評価します。
    ///
    /// 必要なデータがNoneの場合、条件はfalseとして評価されます。
    pub fn evaluate_condition(&self, condition: &FilterCondition) -> bool {
        match condition {
            FilterCondition::RsiAbove(threshold) => {
                self.rsi.map(|v| v > *threshold).unwrap_or(false)
            }
            FilterCondition::RsiBelow(threshold) => {
                self.rsi.map(|v| v < *threshold).unwrap_or(false)
            }
            FilterCondition::StochKAbove(threshold) => {
                self.stoch_k.map(|v| v > *threshold).unwrap_or(false)
            }
            FilterCondition::StochKBelow(threshold) => {
                self.stoch_k.map(|v| v < *threshold).unwrap_or(false)
            }
            FilterCondition::StochKAboveD => {
                match (self.stoch_k, self.stoch_d) {
                    (Some(k), Some(d)) => k > d,
                    _ => false,
                }
            }
            FilterCondition::StochKBelowD => {
                match (self.stoch_k, self.stoch_d) {
                    (Some(k), Some(d)) => k < d,
                    _ => false,
                }
            }
            FilterCondition::CloseAboveSma => {
                match self.sma {
                    Some(sma) => self.price > sma,
                    _ => false,
                }
            }
            FilterCondition::CloseBelowSma => {
                match self.sma {
                    Some(sma) => self.price < sma,
                    _ => false,
                }
            }
            FilterCondition::CloseAboveUpperBand => {
                match self.upper_band {
                    Some(upper) => self.price > upper,
                    _ => false,
                }
            }
            FilterCondition::CloseBelowLowerBand => {
                match self.lower_band {
                    Some(lower) => self.price < lower,
                    _ => false,
                }
            }
            FilterCondition::MacdCrossUp => {
                // 現在: MACD > Signal, 前足: MACD < Signal
                match (self.macd, self.macd_signal, self.prev_macd, self.prev_macd_signal) {
                    (Some(m), Some(s), Some(pm), Some(ps)) => m > s && pm < ps,
                    _ => false,
                }
            }
            FilterCondition::MacdCrossDown => {
                // 現在: MACD < Signal, 前足: MACD > Signal
                match (self.macd, self.macd_signal, self.prev_macd, self.prev_macd_signal) {
                    (Some(m), Some(s), Some(pm), Some(ps)) => m < s && pm > ps,
                    _ => false,
                }
            }
            FilterCondition::MacdAboveSignal => {
                match (self.macd, self.macd_signal) {
                    (Some(m), Some(s)) => m > s,
                    _ => false,
                }
            }
            FilterCondition::MacdBelowSignal => {
                match (self.macd, self.macd_signal) {
                    (Some(m), Some(s)) => m < s,
                    _ => false,
                }
            }
        }
    }

    /// プリセットのすべての条件を評価します（AND条件）。
    pub fn matches_preset(&self, preset: &ScannerPreset) -> bool {
        preset.conditions.iter().all(|c| self.evaluate_condition(c))
    }
}

/// デフォルトのプリセット一覧を返します。
pub fn get_default_presets() -> Vec<ScannerPreset> {
    vec![
        // 🔥 過熱シグナル（売り候補）
        ScannerPreset {
            id: "rsi_overbought".to_string(),
            name: "RSI過熱".to_string(),
            description: "RSI(14) > 75 で買われすぎ状態".to_string(),
            category: "overbought".to_string(),
            conditions: vec![FilterCondition::RsiAbove(75.0)],
        },
        ScannerPreset {
            id: "stoch_overbought".to_string(),
            name: "ストキャス過熱".to_string(),
            description: "Stoch %K > 80 かつ %K < %D".to_string(),
            category: "overbought".to_string(),
            conditions: vec![
                FilterCondition::StochKAbove(80.0),
                FilterCondition::StochKBelowD,
            ],
        },
        ScannerPreset {
            id: "bb_upper_break".to_string(),
            name: "ボリバン上限突破".to_string(),
            description: "終値 > ボリンジャーバンド上限".to_string(),
            category: "overbought".to_string(),
            conditions: vec![FilterCondition::CloseAboveUpperBand],
        },

        // ❄️ 売られすぎシグナル（買い候補）
        ScannerPreset {
            id: "rsi_oversold".to_string(),
            name: "RSI売られすぎ".to_string(),
            description: "RSI(14) < 25 で売られすぎ状態".to_string(),
            category: "oversold".to_string(),
            conditions: vec![FilterCondition::RsiBelow(25.0)],
        },
        ScannerPreset {
            id: "stoch_oversold".to_string(),
            name: "ストキャス売られすぎ".to_string(),
            description: "Stoch %K < 20 かつ %K > %D".to_string(),
            category: "oversold".to_string(),
            conditions: vec![
                FilterCondition::StochKBelow(20.0),
                FilterCondition::StochKAboveD,
            ],
        },
        ScannerPreset {
            id: "bb_lower_break".to_string(),
            name: "ボリバン下限突破".to_string(),
            description: "終値 < ボリンジャーバンド下限".to_string(),
            category: "oversold".to_string(),
            conditions: vec![FilterCondition::CloseBelowLowerBand],
        },

        // 📈 上昇トレンド
        ScannerPreset {
            id: "macd_golden_cross".to_string(),
            name: "MACDゴールデンクロス".to_string(),
            description: "MACDがシグナルを上抜け".to_string(),
            category: "trend_up".to_string(),
            conditions: vec![FilterCondition::MacdCrossUp],
        },
        ScannerPreset {
            id: "price_above_sma".to_string(),
            name: "価格がSMA上".to_string(),
            description: "終値 > SMA(20)".to_string(),
            category: "trend_up".to_string(),
            conditions: vec![FilterCondition::CloseAboveSma],
        },
        ScannerPreset {
            id: "strong_uptrend".to_string(),
            name: "強い上昇トレンド".to_string(),
            description: "RSI > 55 かつ 終値 > SMA".to_string(),
            category: "trend_up".to_string(),
            conditions: vec![
                FilterCondition::RsiAbove(55.0),
                FilterCondition::CloseAboveSma,
            ],
        },

        // 📉 下降トレンド
        ScannerPreset {
            id: "macd_dead_cross".to_string(),
            name: "MACDデッドクロス".to_string(),
            description: "MACDがシグナルを下抜け".to_string(),
            category: "trend_down".to_string(),
            conditions: vec![FilterCondition::MacdCrossDown],
        },
        ScannerPreset {
            id: "price_below_sma".to_string(),
            name: "価格がSMA下".to_string(),
            description: "終値 < SMA(20)".to_string(),
            category: "trend_down".to_string(),
            conditions: vec![FilterCondition::CloseBelowSma],
        },
        ScannerPreset {
            id: "strong_downtrend".to_string(),
            name: "強い下降トレンド".to_string(),
            description: "RSI < 45 かつ 終値 < SMA".to_string(),
            category: "trend_down".to_string(),
            conditions: vec![
                FilterCondition::RsiBelow(45.0),
                FilterCondition::CloseBelowSma,
            ],
        },

        // 🎯 複合条件
        ScannerPreset {
            id: "multi_buy_signal".to_string(),
            name: "マルチ買いシグナル".to_string(),
            description: "RSI < 30 かつ Stoch %K < 20 かつ ボリバン下限突破".to_string(),
            category: "multi".to_string(),
            conditions: vec![
                FilterCondition::RsiBelow(30.0),
                FilterCondition::StochKBelow(20.0),
                FilterCondition::CloseBelowLowerBand,
            ],
        },
        ScannerPreset {
            id: "multi_sell_signal".to_string(),
            name: "マルチ売りシグナル".to_string(),
            description: "RSI > 70 かつ Stoch %K > 80 かつ ボリバン上限突破".to_string(),
            category: "multi".to_string(),
            conditions: vec![
                FilterCondition::RsiAbove(70.0),
                FilterCondition::StochKAbove(80.0),
                FilterCondition::CloseAboveUpperBand,
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_presets_count() {
        let presets = get_default_presets();
        assert_eq!(presets.len(), 14, "Should have 14 default presets");
    }

    #[test]
    fn test_preset_categories() {
        let presets = get_default_presets();
        let overbought_count = presets.iter().filter(|p| p.category == "overbought").count();
        let oversold_count = presets.iter().filter(|p| p.category == "oversold").count();
        let trend_up_count = presets.iter().filter(|p| p.category == "trend_up").count();
        let trend_down_count = presets.iter().filter(|p| p.category == "trend_down").count();
        let multi_count = presets.iter().filter(|p| p.category == "multi").count();
        
        assert_eq!(overbought_count, 3);
        assert_eq!(oversold_count, 3);
        assert_eq!(trend_up_count, 3);
        assert_eq!(trend_down_count, 3);
        assert_eq!(multi_count, 2);
    }

    #[test]
    fn test_filter_condition_serialization() {
        let condition = FilterCondition::RsiAbove(75.0);
        let json = serde_json::to_string(&condition).unwrap();
        assert!(json.contains("RsiAbove"));
        assert!(json.contains("75"));
    }
}
