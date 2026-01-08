use tauri::{AppHandle, Emitter, Manager};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;
use crate::models::{KlineMessage, KlineWithIndicator};
use crate::state::AppState;
use crate::indicators::{calculate_sma, calculate_bollinger_bands, calculate_rsi, calculate_macd, calculate_stoch};

/// WebSocket接続を管理し、データを受信してフロントエンドに送信するタスク
pub async fn start_websocket_listener(app_handle: AppHandle) {
    let url = Url::parse("wss://stream.binance.com:9443/ws/btcusdt@kline_1m").unwrap();
    
    // 接続が切れた場合に再接続するための無限ループ
    loop {
        println!("Connecting to Binance WebSocket...");
        
        match connect_async(url.as_str()).await {
            Ok((mut ws_stream, _)) => {
                println!("Connected to Binance WebSocket");
                
                // メッセージ受信ループ
                while let Some(msg) = ws_stream.next().await {
                    // エラーメッセージはスキップ
                    let Ok(msg) = msg else { continue };
                    
                    // テキストメッセージ以外はスキップ
                    if !msg.is_text() { continue; }
                    
                    // テキストの取り出し
                    let Ok(text) = msg.to_text() else { continue };

                    // JSONパース
                    match serde_json::from_str::<KlineMessage>(text) {
                        Ok(parsed) => {
                            let new_kline = parsed.kline;
                            let state = app_handle.state::<AppState>();
                            
                            // 共有データをロックして更新
                            let (sma, upper, lower, rsi, macd, signal, hist, stoch_k, stoch_d) = {
                                let mut klines = state.klines.lock().unwrap();
                                
                                // データが空なら何もしない (fetch_candlesがまだ)
                                if klines.is_empty() {
                                    (None, None, None, None, None, None, None, None, None)
                                } else {
                                    // 最新データのタイムスタンプを確認
                                    let last_idx = klines.len() - 1;
                                    if klines[last_idx].time == new_kline.time {
                                        // 同じ足の更新: 上書き
                                        klines[last_idx] = new_kline.clone();
                                    } else if klines[last_idx].time < new_kline.time {
                                        // 新しい足: 追加
                                        klines.push(new_kline.clone());
                                        // 履歴が長すぎたら古いものを削除 (メモリ節約)
                                        if klines.len() > 2000 {
                                            klines.remove(0);
                                        }
                                    }

                                    // 現在の設定を取得
                                    let config = state.config.lock().unwrap();
                                    let period = config.period;
                                    let multiplier = config.multiplier;

                                    // 指標を再計算
                                    // Rustの速度なら毎回全計算しても十分高速だが、最適化の余地はある
                                    let sma_values = calculate_sma(&klines, period);
                                    let (u_bands, l_bands) = calculate_bollinger_bands(&klines, period, multiplier);
                                    let rsi_values = calculate_rsi(&klines, 14);
                                    let (macd_vals, signal_vals, hist_vals) = calculate_macd(&klines, 12, 26, 9);
                                    let (stoch_k_vals, stoch_d_vals) = calculate_stoch(&klines, 14, 3, 3);
                                    
                                    // 最新の計算結果を取得
                                    let i = klines.len() - 1;
                                    (
                                        sma_values[i], 
                                        u_bands[i], l_bands[i], 
                                        rsi_values[i], 
                                        macd_vals[i], signal_vals[i], hist_vals[i],
                                        stoch_k_vals[i], stoch_d_vals[i]
                                    )
                                }
                            };

                            // フロントエンドに送信するデータを作成
                            let update_data = KlineWithIndicator {
                                kline: new_kline,
                                sma,
                                upper_band: upper,
                                lower_band: lower,
                                rsi,
                                macd,
                                macd_signal: signal,
                                macd_hist: hist,
                                stoch_k,
                                stoch_d,
                            };

                            // イベント発行
                            let _ = app_handle.emit("kline-update", update_data);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect: {}. Retrying in 5 seconds...", e);
            }
        }
        
        // 再接続前の待機
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}