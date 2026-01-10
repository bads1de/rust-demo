//! # Rust-Chart Library
//! 
//! このライブラリは、暗号通貨のリアルタイムチャートアプリケーションのバックエンドロジックを提供します。
//! 以下の主要なモジュールで構成されています。
//!
//! - `api`: フロントエンドからのリクエストを処理するAPIロジック。
//! - `indicators`: テクニカル指標（SMA, Bollinger Bands, RSI, MACD, Stochastic）の計算ロジック。
//! - `models`: アプリケーションで使用するデータ構造（KlineDataなど）の定義。
//! - `websocket`: Binance WebSocket APIとの通信およびデータ配信ロジック。
//! - `state`: アプリケーション全体で共有される状態（メモリ内データベース）の管理。

/// HTTPリクエスト処理およびコマンドの実装
pub mod api;

/// テクニカル指標の計算ロジック（taクレートのラッパーを含む）
pub mod indicators;

/// データ構造体（DTO: Data Transfer Objects）の定義
pub mod models;

/// WebSocket通信とリアルタイム更新の管理
pub mod websocket;

/// スレッドセーフな状態管理（AppState）
pub mod state;

/// 取引所ごとの実装を抽象化するモジュール
pub mod exchanges;

/// スキャナー機能（プリセットフィルタによる銘柄スクリーニング）
pub mod scanner;