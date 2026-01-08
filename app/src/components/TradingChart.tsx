import { useEffect, useRef, useState, useCallback } from "react";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type CandlestickData,
  type LineData,
  type Time,
  CandlestickSeries,
  LineSeries,
} from "lightweight-charts";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Settings2 } from "lucide-react";

// Rust側の KlineData 構造体に対応
interface KlineData {
  t: number;
  o: number;
  h: number;
  l: number;
  c: number;
  v: number;
}

// Rust側の KlineWithIndicator 構造体に対応
interface KlineWithIndicator extends KlineData {
  sma: number | null;
  upper_band: number | null;
  lower_band: number | null;
}

export const TradingChart = () => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candleSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const smaSeriesRef = useRef<ISeriesApi<"Line"> | null>(null);
  const upperBandSeriesRef = useRef<ISeriesApi<"Line"> | null>(null);
  const lowerBandSeriesRef = useRef<ISeriesApi<"Line"> | null>(null);

  // 設定状態
  const [period, setPeriod] = useState(20);
  const [multiplier, setMultiplier] = useState(2.0);

  // データ取得関数
  const fetchData = useCallback(async (p: number, m: number) => {
    if (!candleSeriesRef.current || !smaSeriesRef.current || !upperBandSeriesRef.current || !lowerBandSeriesRef.current) return;

    try {
      const data = await invoke<KlineWithIndicator[]>("fetch_candles", { period: p, multiplier: m });
      
      const candleData: CandlestickData<Time>[] = [];
      const smaData: LineData<Time>[] = [];
      const upperData: LineData<Time>[] = [];
      const lowerData: LineData<Time>[] = [];

      data.forEach((d) => {
        const time = (d.t / 1000) as Time;
        candleData.push({ time, open: d.o, high: d.h, low: d.l, close: d.c });
        if (d.sma !== null) smaData.push({ time, value: d.sma });
        if (d.upper_band !== null) upperData.push({ time, value: d.upper_band });
        if (d.lower_band !== null) lowerData.push({ time, value: d.lower_band });
      });

      candleSeriesRef.current.setData(candleData);
      smaSeriesRef.current.setData(smaData);
      upperBandSeriesRef.current.setData(upperData);
      lowerBandSeriesRef.current.setData(lowerData);

      // シリーズ名を更新
      smaSeriesRef.current.applyOptions({ title: `SMA ${p}` });
      upperBandSeriesRef.current.applyOptions({ title: `BB Upper (${m}σ)` });
      lowerBandSeriesRef.current.applyOptions({ title: `BB Lower (${m}σ)` });

    } catch (e) {
      console.error("Failed to fetch historical data", e);
    }
  }, []);

  useEffect(() => {
    if (!chartContainerRef.current) return;

    // Initialize Chart
    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { color: "#0f172a" },
        textColor: "#94a3b8",
      },
      grid: {
        vertLines: { color: "#1e293b" },
        horzLines: { color: "#1e293b" },
      },
      width: chartContainerRef.current.clientWidth,
      height: 500,
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
      },
    });

    // 各シリーズの初期化
    candleSeriesRef.current = chart.addSeries(CandlestickSeries, {
      upColor: "#10b981",
      downColor: "#ef4444",
      borderVisible: false,
      wickUpColor: "#10b981",
      wickDownColor: "#ef4444",
    });

    smaSeriesRef.current = chart.addSeries(LineSeries, {
      color: "#3b82f6",
      lineWidth: 2,
      crosshairMarkerVisible: false,
    });

    upperBandSeriesRef.current = chart.addSeries(LineSeries, {
      color: "#a855f7",
      lineWidth: 1,
      crosshairMarkerVisible: false,
    });

    lowerBandSeriesRef.current = chart.addSeries(LineSeries, {
      color: "#a855f7",
      lineWidth: 1,
      crosshairMarkerVisible: false,
    });

    chartRef.current = chart;

    // 初期データロード
    fetchData(period, multiplier);

    // Handle Resize
    const handleResize = () => {
      if (chartContainerRef.current) {
        chart.applyOptions({ width: chartContainerRef.current.clientWidth });
      }
    };
    window.addEventListener("resize", handleResize);

    // Tauri Event Listener
    let unlistenFunction: (() => void) | undefined;

    listen<KlineWithIndicator>("kline-update", (event) => {
      const data = event.payload;
      const time = (data.t / 1000) as Time;

      candleSeriesRef.current?.update({
        time,
        open: data.o,
        high: data.h,
        low: data.l,
        close: data.c,
      });

      if (data.sma !== null) smaSeriesRef.current?.update({ time, value: data.sma });
      if (data.upper_band !== null) upperBandSeriesRef.current?.update({ time, value: data.upper_band });
      if (data.lower_band !== null) lowerBandSeriesRef.current?.update({ time, value: data.lower_band });
    })
      .then((unlisten) => {
        unlistenFunction = unlisten;
      })
      .catch((e) => {
        console.error("Failed to listen to tauri event", e);
      });

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
      if (unlistenFunction) unlistenFunction();
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // 設定変更時にデータを再取得
  const applySettings = () => {
    fetchData(period, multiplier);
  };

  return (
    <div className="w-full bg-slate-900/50 backdrop-blur-md rounded-2xl border border-slate-800 overflow-hidden shadow-2xl p-4">
      <div className="flex flex-col md:flex-row md:items-center justify-between mb-6 gap-4">
        <div>
          <h2 className="text-slate-200 font-bold flex items-center gap-2 text-lg">
            <span className="w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse"></span>
            BTC / USDT 1m
          </h2>
          <p className="text-xs text-slate-500 mt-1 font-mono uppercase tracking-wider">
            Live from Binance • Real-time Rust Compute
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-3 bg-slate-800/40 p-2 rounded-xl border border-slate-700/50">
          <div className="flex items-center gap-2 px-2 text-slate-400">
            <Settings2 size={16} />
            <span className="text-xs font-semibold">INDICATORS</span>
          </div>
          
          <div className="h-4 w-px bg-slate-700"></div>

          <div className="flex items-center gap-2">
            <label className="text-[10px] font-bold text-slate-500 uppercase">Period</label>
            <input
              type="number"
              value={period}
              onChange={(e) => setPeriod(parseInt(e.target.value) || 20)}
              className="w-14 bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs text-slate-200 focus:outline-none focus:border-blue-500 transition-colors"
            />
          </div>

          <div className="flex items-center gap-2">
            <label className="text-[10px] font-bold text-slate-500 uppercase">σ Mult</label>
            <input
              type="number"
              step="0.1"
              value={multiplier}
              onChange={(e) => setMultiplier(parseFloat(e.target.value) || 2.0)}
              className="w-14 bg-slate-900 border border-slate-700 rounded px-2 py-1 text-xs text-slate-200 focus:outline-none focus:border-blue-500 transition-colors"
            />
          </div>

          <button
            onClick={applySettings}
            className="bg-blue-600 hover:bg-blue-500 text-white text-[10px] font-bold py-1 px-3 rounded uppercase tracking-wider transition-colors active:scale-95"
          >
            Apply
          </button>
        </div>
      </div>

      <div ref={chartContainerRef} className="w-full" />
      
      <div className="mt-4 flex gap-6 text-[10px] font-mono text-slate-500 overflow-x-auto pb-2">
        <div className="flex items-center gap-1.5 whitespace-nowrap">
          <div className="w-2 h-2 rounded-sm bg-blue-500"></div>
          <span>SMA ({period})</span>
        </div>
        <div className="flex items-center gap-1.5 whitespace-nowrap">
          <div className="w-2 h-2 rounded-sm bg-purple-500"></div>
          <span>BB ({multiplier}σ)</span>
        </div>
        <div className="ml-auto">
          DATA SOURCE: BINANCE WS API (btcusdt@kline_1m)
        </div>
      </div>
    </div>
  );
};

  return (
    <div className="w-full bg-slate-900/50 backdrop-blur-md rounded-2xl border border-slate-800 overflow-hidden shadow-2xl p-4">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-slate-200 font-bold flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          BTC / USDT 1m (SMA 20, BB 2.0)
        </h2>
        <div className="flex gap-4">
          <span className="text-xs text-slate-400 font-mono">
            LIVE / BINANCE
          </span>
        </div>
      </div>
      <div ref={chartContainerRef} className="w-full" />
    </div>
  );
};