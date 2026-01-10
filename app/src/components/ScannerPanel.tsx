import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Scan,
  Filter,
  TrendingUp,
  TrendingDown,
  ChevronRight,
  Loader2,
} from "lucide-react";

// 型定義
interface FilterCondition {
  RsiAbove?: number;
  RsiBelow?: number;
  StochKAbove?: number;
  StochKBelow?: number;
  StochKAboveD?: null;
  StochKBelowD?: null;
  CloseAboveSma?: null;
  CloseBelowSma?: null;
  CloseAboveUpperBand?: null;
  CloseBelowLowerBand?: null;
  MacdCrossUp?: null;
  MacdCrossDown?: null;
  MacdAboveSignal?: null;
  MacdBelowSignal?: null;
}

interface ScannerPreset {
  id: string;
  name: string;
  description: string;
  category: string;
  conditions: FilterCondition[];
}

interface ScanResult {
  exchange: string;
  symbol: string;
  price: number;
  rsi: number | null;
  stoch_k: number | null;
  stoch_d: number | null;
  macd: number | null;
  macd_signal: number | null;
  sma: number | null;
  upper_band: number | null;
  lower_band: number | null;
}

// カテゴリ情報
const categoryInfo: Record<
  string,
  { label: string; icon: React.ReactNode; color: string }
> = {
  overbought: {
    label: "過熱",
    icon: <TrendingUp size={14} />,
    color: "text-red-400 bg-red-400/10 border-red-400/20",
  },
  oversold: {
    label: "売られすぎ",
    icon: <TrendingDown size={14} />,
    color: "text-emerald-400 bg-emerald-400/10 border-emerald-400/20",
  },
  trend_up: {
    label: "上昇トレンド",
    icon: <TrendingUp size={14} />,
    color: "text-blue-400 bg-blue-400/10 border-blue-400/20",
  },
  trend_down: {
    label: "下降トレンド",
    icon: <TrendingDown size={14} />,
    color: "text-orange-400 bg-orange-400/10 border-orange-400/20",
  },
  multi: {
    label: "複合",
    icon: <Filter size={14} />,
    color: "text-purple-400 bg-purple-400/10 border-purple-400/20",
  },
};

interface ScannerPanelProps {
  exchange: string;
  onSymbolSelect: (symbol: string) => void;
}

export function ScannerPanel({ exchange, onSymbolSelect }: ScannerPanelProps) {
  const [presets, setPresets] = useState<ScannerPreset[]>([]);
  const [selectedPreset, setSelectedPreset] = useState<string | null>(null);
  const [interval, setInterval] = useState("1h");
  const [results, setResults] = useState<ScanResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // プリセット読み込み
  useEffect(() => {
    invoke<ScannerPreset[]>("get_scanner_presets")
      .then(setPresets)
      .catch((e) => setError(String(e)));
  }, []);

  // スキャン実行
  const runScan = async () => {
    if (!selectedPreset) return;

    setLoading(true);
    setError(null);
    setResults([]);

    try {
      const scanResults = await invoke<ScanResult[]>("run_scanner", {
        presetId: selectedPreset,
        exchangeName: exchange,
        interval,
      });
      setResults(scanResults);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  // カテゴリでグループ化
  const groupedPresets = presets.reduce((acc, preset) => {
    if (!acc[preset.category]) acc[preset.category] = [];
    acc[preset.category].push(preset);
    return acc;
  }, {} as Record<string, ScannerPreset[]>);

  return (
    <div className="flex flex-col h-full bg-[#111620] border-r border-white/5">
      {/* Header */}
      <div className="p-4 border-b border-white/5">
        <div className="flex items-center gap-2 mb-3">
          <Scan className="text-indigo-500" size={18} />
          <h2 className="text-sm font-bold text-white">SCANNER</h2>
        </div>

        {/* 時間足選択 */}
        <div className="flex gap-1 mb-3">
          {["1h", "4h", "1d"].map((tf) => (
            <button
              key={tf}
              onClick={() => setInterval(tf)}
              className={`px-3 py-1 text-[10px] font-bold rounded transition-all ${
                interval === tf
                  ? "bg-indigo-500 text-white"
                  : "bg-white/5 text-gray-400 hover:bg-white/10"
              }`}
            >
              {tf.toUpperCase()}
            </button>
          ))}
        </div>

        {/* スキャンボタン */}
        <button
          onClick={runScan}
          disabled={!selectedPreset || loading}
          className={`w-full py-2 rounded font-bold text-xs flex items-center justify-center gap-2 transition-all ${
            selectedPreset && !loading
              ? "bg-indigo-500 text-white hover:bg-indigo-600"
              : "bg-white/5 text-gray-500 cursor-not-allowed"
          }`}
        >
          {loading ? (
            <>
              <Loader2 className="animate-spin" size={14} />
              スキャン中...
            </>
          ) : (
            <>
              <Scan size={14} />
              スキャン実行
            </>
          )}
        </button>
      </div>

      {/* プリセット一覧 */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        {Object.entries(groupedPresets).map(([category, categoryPresets]) => {
          const info = categoryInfo[category] || {
            label: category,
            icon: <Filter size={14} />,
            color: "text-gray-400",
          };
          return (
            <div key={category} className="border-b border-white/5">
              <div
                className={`px-3 py-2 text-[10px] font-bold uppercase flex items-center gap-1.5 ${
                  info.color.split(" ")[0]
                }`}
              >
                {info.icon}
                {info.label}
              </div>
              {categoryPresets.map((preset) => (
                <button
                  key={preset.id}
                  onClick={() => setSelectedPreset(preset.id)}
                  className={`w-full px-3 py-2 text-left transition-all hover:bg-white/5 ${
                    selectedPreset === preset.id
                      ? "bg-indigo-500/10 border-l-2 border-l-indigo-500"
                      : ""
                  }`}
                >
                  <div className="text-xs font-bold text-gray-200">
                    {preset.name}
                  </div>
                  <div className="text-[10px] text-gray-500">
                    {preset.description}
                  </div>
                </button>
              ))}
            </div>
          );
        })}
      </div>

      {/* 結果表示 */}
      {(results.length > 0 || error) && (
        <div className="border-t border-white/5 max-h-75 overflow-y-auto custom-scrollbar">
          <div className="px-3 py-2 text-[10px] font-bold text-gray-500 uppercase bg-[#151b26] flex items-center justify-between">
            <span>結果</span>
            <span
              className={
                results.length > 0 ? "text-emerald-400" : "text-red-400"
              }
            >
              {results.length} 件
            </span>
          </div>

          {error && (
            <div className="p-3 text-xs text-red-400 bg-red-400/10">
              {error}
            </div>
          )}

          {results.map((result) => (
            <button
              key={`${result.exchange}-${result.symbol}`}
              onClick={() => onSymbolSelect(result.symbol)}
              className="w-full p-3 text-left border-b border-white/5 hover:bg-white/5 transition-all group"
            >
              <div className="flex items-center justify-between">
                <div className="font-bold text-gray-200 text-xs">
                  {result.symbol}
                </div>
                <ChevronRight
                  size={14}
                  className="text-gray-500 group-hover:text-indigo-500 transition-colors"
                />
              </div>
              <div className="flex gap-3 mt-1 text-[10px]">
                <span className="text-gray-500">
                  ${result.price.toFixed(result.price < 1 ? 6 : 2)}
                </span>
                {result.rsi && (
                  <span
                    className={
                      result.rsi > 70
                        ? "text-red-400"
                        : result.rsi < 30
                        ? "text-emerald-400"
                        : "text-gray-400"
                    }
                  >
                    RSI: {result.rsi.toFixed(1)}
                  </span>
                )}
                {result.stoch_k && (
                  <span className="text-gray-400">
                    K: {result.stoch_k.toFixed(1)}
                  </span>
                )}
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
