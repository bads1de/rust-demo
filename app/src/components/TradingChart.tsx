import { useEffect, useRef, useState, useCallback } from "react";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type CandlestickData,
  type LineData,
  type HistogramData,
  type Time,
  CandlestickSeries,
  LineSeries,
  HistogramSeries,
} from "lightweight-charts";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Settings2, Activity, BarChart2, TrendingUp } from "lucide-react";

// Types
interface KlineData {
  t: number; o: number; h: number; l: number; c: number; v: number;
}

interface KlineWithIndicator extends KlineData {
  sma: number | null;
  upper_band: number | null;
  lower_band: number | null;
  rsi: number | null;
  macd: number | null;
  macd_signal: number | null;
  macd_hist: number | null;
  stoch_k: number | null;
  stoch_d: number | null;
}

export const TradingChart = () => {
  // Container Refs
  const mainRef = useRef<HTMLDivElement>(null);
  const rsiRef = useRef<HTMLDivElement>(null);
  const macdRef = useRef<HTMLDivElement>(null);
  const stochRef = useRef<HTMLDivElement>(null);

  // Chart Refs
  const mainChart = useRef<IChartApi | null>(null);
  const rsiChart = useRef<IChartApi | null>(null);
  const macdChart = useRef<IChartApi | null>(null);
  const stochChart = useRef<IChartApi | null>(null);

  // Series Refs
  const series = useRef({
    candle: null as ISeriesApi<"Candlestick"> | null,
    sma: null as ISeriesApi<"Line"> | null,
    upperBB: null as ISeriesApi<"Line"> | null,
    lowerBB: null as ISeriesApi<"Line"> | null,
    rsi: null as ISeriesApi<"Line"> | null,
    macd: null as ISeriesApi<"Line"> | null,
    macdSignal: null as ISeriesApi<"Line"> | null,
    macdHist: null as ISeriesApi<"Histogram"> | null,
    stochK: null as ISeriesApi<"Line"> | null,
    stochD: null as ISeriesApi<"Line"> | null,
  });

  // State
  const [period, setPeriod] = useState(20);
  const [multiplier, setMultiplier] = useState(2.0);
  
  // Visibility States
  const [showMain, setShowMain] = useState({ sma: true, bb: true });
  const [showSub, setShowSub] = useState({ rsi: true, macd: true, stoch: true });

  // Update Main Series Visibility
  useEffect(() => {
    series.current.sma?.applyOptions({ visible: showMain.sma });
    series.current.upperBB?.applyOptions({ visible: showMain.bb });
    series.current.lowerBB?.applyOptions({ visible: showMain.bb });
  }, [showMain]);

  // Sync Function
  const syncCharts = useCallback(() => {
    const charts = [mainChart.current, rsiChart.current, macdChart.current, stochChart.current].filter(c => c !== null) as IChartApi[];
    
    charts.forEach(c1 => {
      c1.timeScale().subscribeVisibleLogicalRangeChange(range => {
        if (!range) return;
        charts.forEach(c2 => {
          if (c1 !== c2) c2.timeScale().setVisibleLogicalRange(range);
        });
      });
    });
  }, []);

  // Fetch & Update Data
  const fetchData = useCallback(async (p: number, m: number) => {
    try {
      const data = await invoke<KlineWithIndicator[]>("fetch_candles", { period: p, multiplier: m });
      
      const arrays = {
        candle: [] as CandlestickData<Time>[],
        sma: [] as LineData<Time>[],
        upper: [] as LineData<Time>[],
        lower: [] as LineData<Time>[],
        rsi: [] as LineData<Time>[],
        macd: [] as LineData<Time>[],
        signal: [] as LineData<Time>[],
        hist: [] as HistogramData<Time>[],
        k: [] as LineData<Time>[],
        d: [] as LineData<Time>[],
      };

      data.forEach(d => {
        const time = (d.t / 1000) as Time;
        arrays.candle.push({ time, open: d.o, high: d.h, low: d.l, close: d.c });
        if (d.sma) arrays.sma.push({ time, value: d.sma });
        if (d.upper_band) arrays.upper.push({ time, value: d.upper_band });
        if (d.lower_band) arrays.lower.push({ time, value: d.lower_band });
        if (d.rsi) arrays.rsi.push({ time, value: d.rsi });
        if (d.macd) arrays.macd.push({ time, value: d.macd });
        if (d.macd_signal) arrays.signal.push({ time, value: d.macd_signal });
        if (d.macd_hist) arrays.hist.push({ time, value: d.macd_hist, color: d.macd_hist >= 0 ? '#26a69a' : '#ef5350' });
        if (d.stoch_k) arrays.k.push({ time, value: d.stoch_k });
        if (d.stoch_d) arrays.d.push({ time, value: d.stoch_d });
      });

      series.current.candle?.setData(arrays.candle);
      series.current.sma?.setData(arrays.sma);
      series.current.upperBB?.setData(arrays.upper);
      series.current.lowerBB?.setData(arrays.lower);
      series.current.rsi?.setData(arrays.rsi);
      series.current.macd?.setData(arrays.macd);
      series.current.macdSignal?.setData(arrays.signal);
      series.current.macdHist?.setData(arrays.hist);
      series.current.stochK?.setData(arrays.k);
      series.current.stochD?.setData(arrays.d);

    } catch (e) {
      console.error("Fetch error:", e);
    }
  }, []);

  // Initialize Charts
  useEffect(() => {
    if (!mainRef.current) return;

    const commonOptions = {
      layout: { background: { color: "#0f172a" }, textColor: "#94a3b8" },
      grid: { vertLines: { color: "#1e293b" }, horzLines: { color: "#1e293b" } },
      timeScale: { timeVisible: true, secondsVisible: false },
    };

    // --- Main Chart ---
    mainChart.current = createChart(mainRef.current, {
      ...commonOptions,
      width: mainRef.current.clientWidth,
      height: 400,
    });
    series.current.candle = mainChart.current.addSeries(CandlestickSeries, { upColor: "#10b981", downColor: "#ef4444" });
    series.current.sma = mainChart.current.addSeries(LineSeries, { color: "#3b82f6", lineWidth: 2, title: "SMA" });
    series.current.upperBB = mainChart.current.addSeries(LineSeries, { color: "rgba(168, 85, 247, 0.6)", lineWidth: 1, title: "BB Upper" });
    series.current.lowerBB = mainChart.current.addSeries(LineSeries, { color: "rgba(168, 85, 247, 0.6)", lineWidth: 1, title: "BB Lower" });

    // --- RSI Chart ---
    if (rsiRef.current) {
      rsiChart.current = createChart(rsiRef.current, { ...commonOptions, width: rsiRef.current.clientWidth, height: 150 });
      series.current.rsi = rsiChart.current.addSeries(LineSeries, { color: "#f59e0b", lineWidth: 2, title: "RSI 14" });
      // 30/70 Lines
      const high = rsiChart.current.addSeries(LineSeries, { color: "rgba(255,255,255,0.2)", lineWidth: 1, lineStyle: 2 });
      const low = rsiChart.current.addSeries(LineSeries, { color: "rgba(255,255,255,0.2)", lineWidth: 1, lineStyle: 2 });
      // Draw static lines far into future
      const now = Math.floor(Date.now() / 1000) as Time;
      const data = [{ time: (now - 1000000) as Time, value: 70 }, { time: (now + 1000000) as Time, value: 70 }];
      const dataLow = [{ time: (now - 1000000) as Time, value: 30 }, { time: (now + 1000000) as Time, value: 30 }];
      high.setData(data); low.setData(dataLow);
    }

    // --- MACD Chart ---
    if (macdRef.current) {
      macdChart.current = createChart(macdRef.current, { ...commonOptions, width: macdRef.current.clientWidth, height: 150 });
      series.current.macdHist = macdChart.current.addSeries(HistogramSeries, { title: "Hist" });
      series.current.macd = macdChart.current.addSeries(LineSeries, { color: "#2962ff", lineWidth: 2, title: "MACD" });
      series.current.macdSignal = macdChart.current.addSeries(LineSeries, { color: "#ff6d00", lineWidth: 2, title: "Signal" });
    }

    // --- Stoch Chart ---
    if (stochRef.current) {
      stochChart.current = createChart(stochRef.current, { ...commonOptions, width: stochRef.current.clientWidth, height: 150 });
      series.current.stochK = stochChart.current.addSeries(LineSeries, { color: "#2962ff", lineWidth: 2, title: "%K" });
      series.current.stochD = stochChart.current.addSeries(LineSeries, { color: "#ff6d00", lineWidth: 2, title: "%D" });
      // 20/80 Lines
      const high = stochChart.current.addSeries(LineSeries, { color: "rgba(255,255,255,0.2)", lineWidth: 1, lineStyle: 2 });
      const low = stochChart.current.addSeries(LineSeries, { color: "rgba(255,255,255,0.2)", lineWidth: 1, lineStyle: 2 });
      const now = Math.floor(Date.now() / 1000) as Time;
      high.setData([{ time: (now - 1000000) as Time, value: 80 }, { time: (now + 1000000) as Time, value: 80 }]);
      low.setData([{ time: (now - 1000000) as Time, value: 20 }, { time: (now + 1000000) as Time, value: 20 }]);
    }

    // Sync all charts
    syncCharts();

    // Initial Load
    fetchData(period, multiplier);

    // Resize Handler
    const handleResize = () => {
      if (mainRef.current) mainChart.current?.applyOptions({ width: mainRef.current.clientWidth });
      if (rsiRef.current) rsiChart.current?.applyOptions({ width: rsiRef.current.clientWidth });
      if (macdRef.current) macdChart.current?.applyOptions({ width: macdRef.current.clientWidth });
      if (stochRef.current) stochChart.current?.applyOptions({ width: stochRef.current.clientWidth });
    };
    window.addEventListener("resize", handleResize);

    // Tauri Event Listener
    let unlisten: (() => void) | undefined;
    listen<KlineWithIndicator>("kline-update", (e) => {
      const d = e.payload;
      const t = (d.t / 1000) as Time;
      
      series.current.candle?.update({ time: t, open: d.o, high: d.h, low: d.l, close: d.c });
      if (d.sma) series.current.sma?.update({ time: t, value: d.sma });
      if (d.upper_band) series.current.upperBB?.update({ time: t, value: d.upper_band });
      if (d.lower_band) series.current.lowerBB?.update({ time: t, value: d.lower_band });
      if (d.rsi) series.current.rsi?.update({ time: t, value: d.rsi });
      if (d.macd) series.current.macd?.update({ time: t, value: d.macd });
      if (d.macd_signal) series.current.macdSignal?.update({ time: t, value: d.macd_signal });
      if (d.macd_hist) series.current.macdHist?.update({ time: t, value: d.macd_hist, color: d.macd_hist >= 0 ? '#26a69a' : '#ef5350' });
      if (d.stoch_k) series.current.stochK?.update({ time: t, value: d.stoch_k });
      if (d.stoch_d) series.current.stochD?.update({ time: t, value: d.stoch_d });
    }).then(u => unlisten = u);

    return () => {
      window.removeEventListener("resize", handleResize);
      mainChart.current?.remove();
      rsiChart.current?.remove();
      macdChart.current?.remove();
      stochChart.current?.remove();
      if (unlisten) unlisten();
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="w-full bg-slate-950 text-slate-300 font-sans p-4 min-h-screen">
      
      {/* Header & Controls */}
      <div className="flex flex-wrap items-center justify-between gap-4 mb-4 bg-slate-900/50 p-4 rounded-xl border border-slate-800">
        <div>
          <h1 className="text-xl font-bold text-white flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-emerald-500 animate-pulse"></span>
            BTC/USDT <span className="text-slate-500 text-sm font-normal">1m • Rust TA Engine</span>
          </h1>
        </div>

        <div className="flex flex-wrap items-center gap-4">
          
          {/* Main Overlays */}
          <div className="flex items-center gap-3 bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-700">
            <Settings2 size={14} className="text-blue-400" />
            <span className="text-xs font-bold text-slate-400">OVERLAYS</span>
            <div className="h-4 w-px bg-slate-600"></div>
            <label className="flex items-center gap-1.5 cursor-pointer hover:text-white transition-colors">
              <input type="checkbox" checked={showMain.sma} onChange={e => setShowMain(s => ({ ...s, sma: e.target.checked }))} className="rounded border-slate-600 bg-slate-700 text-blue-500 focus:ring-0" />
              <span className="text-xs font-mono">SMA</span>
            </label>
            <label className="flex items-center gap-1.5 cursor-pointer hover:text-white transition-colors">
              <input type="checkbox" checked={showMain.bb} onChange={e => setShowMain(s => ({ ...s, bb: e.target.checked }))} className="rounded border-slate-600 bg-slate-700 text-purple-500 focus:ring-0" />
              <span className="text-xs font-mono">BB</span>
            </label>
          </div>

          {/* Sub Charts */}
          <div className="flex items-center gap-3 bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-700">
            <Activity size={14} className="text-amber-400" />
            <span className="text-xs font-bold text-slate-400">SUB-CHARTS</span>
            <div className="h-4 w-px bg-slate-600"></div>
            <label className="flex items-center gap-1.5 cursor-pointer hover:text-white transition-colors">
              <input type="checkbox" checked={showSub.rsi} onChange={e => setShowSub(s => ({ ...s, rsi: e.target.checked }))} className="rounded border-slate-600 bg-slate-700 text-amber-500 focus:ring-0" />
              <span className="text-xs font-mono">RSI</span>
            </label>
            <label className="flex items-center gap-1.5 cursor-pointer hover:text-white transition-colors">
              <input type="checkbox" checked={showSub.macd} onChange={e => setShowSub(s => ({ ...s, macd: e.target.checked }))} className="rounded border-slate-600 bg-slate-700 text-cyan-500 focus:ring-0" />
              <span className="text-xs font-mono">MACD</span>
            </label>
            <label className="flex items-center gap-1.5 cursor-pointer hover:text-white transition-colors">
              <input type="checkbox" checked={showSub.stoch} onChange={e => setShowSub(s => ({ ...s, stoch: e.target.checked }))} className="rounded border-slate-600 bg-slate-700 text-pink-500 focus:ring-0" />
              <span className="text-xs font-mono">STOCH</span>
            </label>
          </div>

          {/* Parameters */}
          <div className="flex items-center gap-2">
            <input type="number" value={period} onChange={e => setPeriod(Number(e.target.value))} className="w-12 bg-slate-800 border border-slate-700 rounded px-2 py-1 text-xs text-white" />
            <input type="number" value={multiplier} onChange={e => setMultiplier(Number(e.target.value))} className="w-12 bg-slate-800 border border-slate-700 rounded px-2 py-1 text-xs text-white" />
            <button onClick={() => fetchData(period, multiplier)} className="bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold py-1 px-3 rounded">APPLY</button>
          </div>
        </div>
      </div>

      {/* Charts Grid */}
      <div className="flex flex-col gap-1">
        {/* Main Chart */}
        <div ref={mainRef} className="w-full rounded-lg overflow-hidden border border-slate-800 shadow-xl" />
        
        {/* Sub Charts (Conditional Rendering) */}
        <div className={`w-full rounded-lg overflow-hidden border border-slate-800 shadow-xl transition-all duration-300 ${showSub.rsi ? 'h-[150px] opacity-100' : 'h-0 opacity-0 border-none'}`}>
          <div ref={rsiRef} className="w-full h-full" />
        </div>
        
        <div className={`w-full rounded-lg overflow-hidden border border-slate-800 shadow-xl transition-all duration-300 ${showSub.macd ? 'h-[150px] opacity-100' : 'h-0 opacity-0 border-none'}`}>
          <div ref={macdRef} className="w-full h-full" />
        </div>

        <div className={`w-full rounded-lg overflow-hidden border border-slate-800 shadow-xl transition-all duration-300 ${showSub.stoch ? 'h-[150px] opacity-100' : 'h-0 opacity-0 border-none'}`}>
          <div ref={stochRef} className="w-full h-full" />
        </div>
      </div>

    </div>
  );
};