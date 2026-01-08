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
import { Settings2, Activity } from "lucide-react";

// Types
interface KlineData {
  t: number; o: number; h: number; l: number; c: number; v: number;
}

interface KlineWithIndicator extends KlineData {
  symbol: string;
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

export const TradingChart = ({ symbol }: { symbol: string }) => {
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
  const [showSub, setShowSub] = useState({ rsi: true, macd: false, stoch: false });

  // Update Visibility
  useEffect(() => {
    series.current.sma?.applyOptions({ visible: showMain.sma });
    series.current.upperBB?.applyOptions({ visible: showMain.bb });
    series.current.lowerBB?.applyOptions({ visible: showMain.bb });
  }, [showMain]);

  // Sync logic
  const syncCharts = useCallback(() => {
    const charts = [mainChart.current, rsiChart.current, macdChart.current, stochChart.current].filter(c => c !== null) as IChartApi[];
    charts.forEach(c1 => {
      c1.timeScale().subscribeVisibleLogicalRangeChange(range => {
        if (!range) return;
        charts.forEach(c2 => { if (c1 !== c2) c2.timeScale().setVisibleLogicalRange(range); });
      });
    });
  }, []);

  const fetchData = useCallback(async (p: number, m: number) => {
    try {
      const data = await invoke<KlineWithIndicator[]>("fetch_candles", { symbol, period: p, multiplier: m });
      
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
    } catch (e) { console.error(`Fetch error for ${symbol}:`, e); }
  }, [symbol]);

  useEffect(() => {
    if (!mainRef.current) return;
    const commonOptions = {
      layout: { background: { color: "#0f172a" }, textColor: "#94a3b8" },
      grid: { vertLines: { color: "#1e293b" }, horzLines: { color: "#1e293b" } },
      timeScale: { timeVisible: true, secondsVisible: false },
    };

    mainChart.current = createChart(mainRef.current, { ...commonOptions, width: mainRef.current.clientWidth, height: 250 });
    series.current.candle = mainChart.current.addSeries(CandlestickSeries, { upColor: "#10b981", downColor: "#ef4444" });
    series.current.sma = mainChart.current.addSeries(LineSeries, { color: "#3b82f6", lineWidth: 2, title: "SMA" });
    series.current.upperBB = mainChart.current.addSeries(LineSeries, { color: "rgba(168, 85, 247, 0.4)", lineWidth: 1, title: "BB Upper" });
    series.current.lowerBB = mainChart.current.addSeries(LineSeries, { color: "rgba(168, 85, 247, 0.4)", lineWidth: 1, title: "BB Lower" });

    if (rsiRef.current) {
      rsiChart.current = createChart(rsiRef.current, { ...commonOptions, width: rsiRef.current.clientWidth, height: 80 });
      series.current.rsi = rsiChart.current.addSeries(LineSeries, { color: "#f59e0b", lineWidth: 2, title: "RSI" });
    }
    if (macdRef.current) {
      macdChart.current = createChart(macdRef.current, { ...commonOptions, width: macdRef.current.clientWidth, height: 80 });
      series.current.macdHist = macdChart.current.addSeries(HistogramSeries, { title: "MACD Hist" });
      series.current.macd = macdChart.current.addSeries(LineSeries, { color: "#2962ff", lineWidth: 1 });
      series.current.macdSignal = macdChart.current.addSeries(LineSeries, { color: "#ff6d00", lineWidth: 1 });
    }
    if (stochRef.current) {
      stochChart.current = createChart(stochRef.current, { ...commonOptions, width: stochRef.current.clientWidth, height: 80 });
      series.current.stochK = stochChart.current.addSeries(LineSeries, { color: "#2962ff", lineWidth: 1, title: "%K" });
      series.current.stochD = stochChart.current.addSeries(LineSeries, { color: "#ff6d00", lineWidth: 1, title: "%D" });
    }

    syncCharts();
    fetchData(period, multiplier);

    const handleResize = () => {
      const w = mainRef.current?.clientWidth || 0;
      mainChart.current?.applyOptions({ width: w });
      rsiChart.current?.applyOptions({ width: w });
      macdChart.current?.applyOptions({ width: w });
      stochChart.current?.applyOptions({ width: w });
    };
    window.addEventListener("resize", handleResize);

    let unlisten: (() => void) | undefined;
    listen<KlineWithIndicator>("kline-update", (e) => {
      const d = e.payload;
      if (d.symbol !== symbol) return;
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
      mainChart.current?.remove(); rsiChart.current?.remove(); macdChart.current?.remove(); stochChart.current?.remove();
      if (unlisten) unlisten();
    };
  }, [symbol, fetchData, syncCharts]);

  return (
    <div className="bg-slate-900/80 backdrop-blur-sm rounded-xl border border-slate-800 overflow-hidden flex flex-col h-full shadow-lg transition-all hover:border-slate-700">
      <div className="p-3 border-b border-slate-800 flex items-center justify-between bg-slate-900/50">
        <h3 className="font-bold text-white flex items-center gap-2 text-sm uppercase tracking-wider">
          <span className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]"></span>
          {symbol}
        </h3>
        <div className="flex gap-2">
          <button onClick={() => setShowSub(s => ({ ...s, rsi: !s.rsi }))} className={`text-[9px] px-1.5 py-0.5 rounded border ${showSub.rsi ? 'bg-amber-500/20 border-amber-500/50 text-amber-500' : 'border-slate-700 text-slate-500'}`}>RSI</button>
          <button onClick={() => setShowSub(s => ({ ...s, macd: !s.macd }))} className={`text-[9px] px-1.5 py-0.5 rounded border ${showSub.macd ? 'bg-cyan-500/20 border-cyan-500/50 text-cyan-500' : 'border-slate-700 text-slate-500'}`}>MACD</button>
          <button onClick={() => setShowSub(s => ({ ...s, stoch: !s.stoch }))} className={`text-[9px] px-1.5 py-0.5 rounded border ${showSub.stoch ? 'bg-pink-500/20 border-pink-500/50 text-pink-500' : 'border-slate-700 text-slate-500'}`}>STOCH</button>
        </div>
      </div>
      <div className="flex-1 flex flex-col min-h-0">
        <div ref={mainRef} className="flex-1 min-h-[150px]" />
        {showSub.rsi && <div className="h-20 border-t border-slate-800"><div ref={rsiRef} className="w-full h-full" /></div>}
        {showSub.macd && <div className="h-20 border-t border-slate-800"><div ref={macdRef} className="w-full h-full" /></div>}
        {showSub.stoch && <div className="h-20 border-t border-slate-800"><div ref={stochRef} className="w-full h-full" /></div>}
      </div>
    </div>
  );
};
