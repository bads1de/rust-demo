use tauri::Emitter;
use serde::{Deserialize, Serialize};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;

/// ローソク足データ (Kline/Candlestick data) を表す構造体
/// Binance API から取得される個々のデータポイントに対応します。
#[derive(Debug, Serialize, Deserialize, Clone)]
struct KlineData {
    /// タイムスタンプ (Unix time in milliseconds)
    #[serde(rename = "t")]
    time: i64,
    /// 始値 (Open price)
    #[serde(rename = "o")]
    open: String,
    /// 高値 (High price)
    #[serde(rename = "h")]
    high: String,
    /// 安値 (Low price)
    #[serde(rename = "l")]
    low: String,
    /// 終値 (Close price)
    #[serde(rename = "c")]
    close: String,
    /// 出来高 (Volume)
    #[serde(rename = "v")]
    volume: String,
}

/// WebSocket から受信するメッセージ構造体
/// リアルタイムの Kline データ更新を含みます。
#[derive(Debug, Serialize, Deserialize, Clone)]
struct KlineMessage {
    /// Kline データの実体
    #[serde(rename = "k")]
    kline: KlineData,
}

/// 過去のローソク足データを Binance API から取得する非同期コマンド
///
/// # Returns
/// - `Ok(Vec<KlineData>)`: 取得に成功した場合、ローソク足データのリストを返します。
/// - `Err(String)`: HTTPリクエストやパースに失敗した場合、エラーメッセージを返します。
#[tauri::command]
async fn fetch_candles() -> Result<Vec<KlineData>, String> {
    // HTTPクライアントの生成
    let client = reqwest::Client::new();
    
    // Binance API へリクエスト送信 (BTCUSDT, 1分足, 1000件)
    let res = client
        .get("https://api.binance.com/api/v3/klines")
        .query(&[
            ("symbol", "BTCUSDT"),
            ("interval", "1m"),
            ("limit", "1000"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // レスポンスステータスの確認
    if !res.status().is_success() {
        return Err(format!("API Error: {}", res.status()));
    }

    // JSON レスポンスをパース (配列の配列として返ってくるため)
    let rows: Vec<Vec<serde_json::Value>> = res.json().await.map_err(|e| e.to_string())?;
    let mut klines = Vec::new();

    // 各行を KlineData 構造体に変換
    for row in rows {
        if let (Some(time_val), Some(open_val), Some(high_val), Some(low_val), Some(close_val), Some(volume_val)) = (
            row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)
        ) {
             let time = time_val.as_i64().unwrap_or(0);
             let open = open_val.as_str().unwrap_or("0").to_string();
             let high = high_val.as_str().unwrap_or("0").to_string();
             let low = low_val.as_str().unwrap_or("0").to_string();
             let close = close_val.as_str().unwrap_or("0").to_string();
             let volume = volume_val.as_str().unwrap_or("0").to_string();

             klines.push(KlineData {
                 time, open, high, low, close, volume
             });
        }
    }
    Ok(klines)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // 非同期タスクとして WebSocket 接続を開始
            tauri::async_runtime::spawn(async move {
                let url = Url::parse("wss://stream.binance.com:9443/ws/btcusdt@kline_1m").unwrap();
                
                // 接続が切れた場合に再接続するための無限ループ
                loop {
                    match connect_async(url.as_str()).await {
                        Ok((mut ws_stream, _)) => {
                            println!("Connected to Binance WebSocket");
                            // メッセージ受信ループ
                            while let Some(msg) = ws_stream.next().await {
                                if let Ok(msg) = msg {
                                    if msg.is_text() {
                                        if let Ok(text) = msg.to_text() {
                                            // メッセージをパースしてフロントエンドにイベントを発行
                                            if let Ok(parsed) = serde_json::from_str::<KlineMessage>(text) {
                                                let _ = app_handle.emit("kline-update", parsed.kline);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to connect: {}. Retrying in 5 seconds...", e);
                            // 再接続前の待機
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![fetch_candles])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}