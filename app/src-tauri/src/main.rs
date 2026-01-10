// 必要なモジュールやクレートをインポートします。
// `rust_chart_lib` は、私たちが作成したライブラリクレートの名前です。
use rust_chart_lib::api::{
    fetch_candles as lib_fetch_candles,
    get_scanner_presets as lib_get_scanner_presets,
    run_scanner as lib_run_scanner,
};
use rust_chart_lib::models::KlineWithIndicator;
use rust_chart_lib::scanner::{ScannerPreset, ScanResult};
use rust_chart_lib::websocket::start_websocket_listener;
use rust_chart_lib::state::AppState;
use tauri::State;

/// Tauriのコマンドとして登録するためのラッパー関数です。
///
/// フロントエンド（React）から `invoke("fetch_candles", ...)` で呼び出されます。
///
/// # Arguments
/// * `state`: Tauriによって自動的に注入される、アプリケーション共有の状態（`AppState`）。
/// * `symbol`: 取得対象の通貨ペア（例: "BTCUSDT"）。
/// * `period`: テクニカル指標（SMA, BB等）の計算期間。
/// * `multiplier`: ボリンジャーバンドの標準偏差倍率。
///
/// # Returns
/// * `Result<Vec<KlineWithIndicator>, String>`:
///     - 成功時: テクニカル指標が付与されたローソク足データのベクタ。
///     - 失敗時: エラーメッセージ（文字列）。
///
/// # Note
/// この関数自体はロジックを持たず、ライブラリ側の `lib_fetch_candles` に処理を委譲しています。
/// これにより、ロジック（ライブラリ）とアプリケーション（バイナリ）の責務を分離しています。
#[tauri::command]
async fn fetch_candles(
    state: State<'_, AppState>,
    exchange_name: String,
    symbol: String,
    period: usize,
    multiplier: f64,
) -> Result<Vec<KlineWithIndicator>, String> {
    lib_fetch_candles(state, exchange_name, symbol, period, multiplier).await
}

/// スキャナープリセット一覧を取得します。
#[tauri::command]
fn get_scanner_presets() -> Vec<ScannerPreset> {
    lib_get_scanner_presets()
}

/// 指定されたプリセットで全銘柄をスキャンします。
#[tauri::command]
async fn run_scanner(
    preset_id: String,
    exchange_name: String,
    interval: String,
) -> Result<Vec<ScanResult>, String> {
    lib_run_scanner(preset_id, exchange_name, interval).await
}

/// アプリケーションのエントリーポイント
fn main() {
    // Tauriアプリケーションのビルドと実行を開始します
    tauri::Builder::default()
        // アプリケーション全体で共有する状態(`AppState`)を初期化して管理下に置きます。
        // これにより、どのコマンドからも `state` 引数でアクセスできるようになります。
        .manage(AppState::new())
        
        // アプリケーション起動時のセットアップ処理
        .setup(|app| {
            // アプリケーションハンドルをクローンして、非同期タスクに渡せるようにします。
            let app_handle = app.handle().clone();
            
            // WebSocketリスナーをバックグラウンドタスク（別スレッド）として起動します。
            // `tauri::async_runtime::spawn` はTokioの軽量スレッドを使用するため、
            // UIスレッドをブロックすることなくネットワーク待機が可能です。
            tauri::async_runtime::spawn(start_websocket_listener(app_handle, "binance".to_string()));
            
            Ok(())
        })
        
        // フロントエンドから呼び出し可能なコマンドを登録します。
        .invoke_handler(tauri::generate_handler![
            fetch_candles,
            get_scanner_presets,
            run_scanner
        ])
        
        // アプリケーションを実行します。
        // ここでコンテキスト（設定ファイルなど）を生成します。
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}