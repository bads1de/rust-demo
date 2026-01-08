use rust_chart_lib::api::fetch_candles as lib_fetch_candles;
use rust_chart_lib::models::KlineWithIndicator;
use rust_chart_lib::websocket::start_websocket_listener;
use rust_chart_lib::state::AppState;
use tauri::State;

/// Tauriのコマンドとして登録するためのラッパー
/// ライブラリ側のロジックを呼び出します。
#[tauri::command]
async fn fetch_candles(
    state: State<'_, AppState>,
    period: usize,
    multiplier: f64,
) -> Result<Vec<KlineWithIndicator>, String> {
    lib_fetch_candles(state, period, multiplier).await
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new()) // アプリケーションの状態を初期化して管理
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // 非同期タスクとして WebSocket 接続を開始
            tauri::async_runtime::spawn(start_websocket_listener(app_handle));
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![fetch_candles])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}