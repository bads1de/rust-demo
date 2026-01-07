use tauri::Emitter;
use serde::{Deserialize, Serialize};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KlineData {
    #[serde(rename = "t")]
    time: i64,
    #[serde(rename = "o")]
    open: String,
    #[serde(rename = "h")]
    high: String,
    #[serde(rename = "l")]
    low: String,
    #[serde(rename = "c")]
    close: String,
    #[serde(rename = "v")]
    volume: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct KlineMessage {
    #[serde(rename = "k")]
    kline: KlineData,
}

#[tauri::command]
async fn fetch_candles() -> Result<Vec<KlineData>, String> {
    let client = reqwest::Client::new();
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

    if !res.status().is_success() {
        return Err(format!("API Error: {}", res.status()));
    }

    let rows: Vec<Vec<serde_json::Value>> = res.json().await.map_err(|e| e.to_string())?;
    let mut klines = Vec::new();

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
            
            tauri::async_runtime::spawn(async move {
                let url = Url::parse("wss://stream.binance.com:9443/ws/btcusdt@kline_1m").unwrap();
                loop {
                    match connect_async(url.as_str()).await {
                        Ok((mut ws_stream, _)) => {
                            println!("Connected to Binance WebSocket");
                            while let Some(msg) = ws_stream.next().await {
                                if let Ok(msg) = msg {
                                    if msg.is_text() {
                                        if let Ok(text) = msg.to_text() {
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