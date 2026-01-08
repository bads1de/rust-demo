use std::sync::Mutex;
use crate::models::KlineData;

/// アプリケーションの設定
pub struct Config {
    pub period: usize,
    pub multiplier: f64,
}

/// アプリケーション全体で共有する状態
pub struct AppState {
    /// 過去のローソク足データを保持する
    pub klines: Mutex<Vec<KlineData>>,
    /// 現在の計算設定を保持する
    pub config: Mutex<Config>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            klines: Mutex::new(Vec::new()),
            config: Mutex::new(Config {
                period: 20,
                multiplier: 2.0,
            }),
        }
    }
}
