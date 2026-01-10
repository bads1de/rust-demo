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
  ColorType,
  LineStyle,
} from "lightweight-charts";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Settings2 } from "lucide-react";

// Types
interface KlineData {
  t: number;
  o: number;
  h: number;
  l: number;
  c: number;
  v: number;
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

export const TradingChart = ({
  symbol,
  exchange,
}: {
  symbol: string;
  exchange: string;
}) => {
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
  const [period] = useState(20);
  const [multiplier] = useState(2.0);
  const [showMain] = useState({ sma: true, bb: true });
  const [showSub, setShowSub] = useState({
    rsi: true,
    macd: false,
    stoch: false,
  });

  // Update Visibility
  useEffect(() => {
    series.current.sma?.applyOptions({ visible: showMain.sma });
    series.current.upperBB?.applyOptions({ visible: showMain.bb });
    series.current.lowerBB?.applyOptions({ visible: showMain.bb });
  }, [showMain]);

  // Sync Logic
  const syncCharts = useCallback(() => {
    const charts = [
      mainChart.current,
      rsiChart.current,
      macdChart.current,
      stochChart.current,
    ].filter((c) => c !== null) as IChartApi[];
    charts.forEach((c1) => {
      c1.timeScale().subscribeVisibleLogicalRangeChange((range) => {
        if (!range) return;
        charts.forEach((c2) => {
          if (c1 !== c2) c2.timeScale().setVisibleLogicalRange(range);
        });
      });
    });
  }, []);

  const fetchData = useCallback(
    async (p: number, m: number) => {
      try {
        const data = await invoke<KlineWithIndicator[]>("fetch_candles", {
          exchange_name: exchange,
          symbol,
          period: p,
          multiplier: m,
        });
        // ... data parsing (omitted for brevity, same as before) ...
        // But need to reconstruct arrays
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

        data.forEach((d) => {
          const time = (d.t / 1000) as Time;
          arrays.candle.push({
            time,
            open: d.o,
            high: d.h,
            low: d.l,
            close: d.c,
          });
          if (d.sma) arrays.sma.push({ time, value: d.sma });
          if (d.upper_band) arrays.upper.push({ time, value: d.upper_band });
          if (d.lower_band) arrays.lower.push({ time, value: d.lower_band });
          if (d.rsi) arrays.rsi.push({ time, value: d.rsi });
          if (d.macd) arrays.macd.push({ time, value: d.macd });
          if (d.macd_signal) arrays.signal.push({ time, value: d.macd_signal });
          if (d.macd_hist)
            arrays.hist.push({
              time,
              value: d.macd_hist,
              color: d.macd_hist >= 0 ? "#26a69a" : "#ef5350",
            });
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
        console.error(e);
      }
    },
    [symbol, exchange]
  );

  useEffect(() => {
    if (!mainRef.current) return;

    // Theme Colors
    const chartBg = "#111620";
    const textColor = "#64748b";
    const gridColor = "#1e293b";

    const commonOptions = {
      layout: {
        background: { type: ColorType.Solid, color: chartBg },
        textColor,
      },
      grid: {
        vertLines: { color: gridColor },
        horzLines: { color: gridColor },
      },
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        borderColor: gridColor,
      },
      rightPriceScale: { borderColor: gridColor },
    };

    mainChart.current = createChart(mainRef.current, {
      ...commonOptions,
      width: mainRef.current.clientWidth,
      height: 250,
    });
    series.current.candle = mainChart.current.addSeries(CandlestickSeries, {
      upColor: "#10b981",
      downColor: "#ef4444",
      borderVisible: false,
      wickUpColor: "#10b981",
      wickDownColor: "#ef4444",
    });
    series.current.sma = mainChart.current.addSeries(LineSeries, {
      color: "#6366f1",
      lineWidth: 2,
      title: "SMA",
    }); // Indigo
    series.current.upperBB = mainChart.current.addSeries(LineSeries, {
      color: "rgba(99, 102, 241, 0.3)",
      lineWidth: 1,
      title: "",
    });
    series.current.lowerBB = mainChart.current.addSeries(LineSeries, {
      color: "rgba(99, 102, 241, 0.3)",
      lineWidth: 1,
      title: "",
    });

    // Helper to create sub-chart
    const createSubChart = (ref: React.RefObject<HTMLDivElement | null>) => {
      if (!ref.current) return null;
      return createChart(ref.current, {
        ...commonOptions,
        width: ref.current.clientWidth,
        height: 80,
      });
    };

    if (rsiRef.current) {
      rsiChart.current = createSubChart(rsiRef);
      if (rsiChart.current) {
        series.current.rsi = rsiChart.current.addSeries(LineSeries, {
          color: "#f59e0b",
          lineWidth: 2,
          title: "RSI",
        }); // Amber
        // Guides
        const lineOpts = {
          color: "rgba(255,255,255,0.1)",
          lineWidth: 1,
          lineStyle: LineStyle.Dashed,
          crosshairMarkerVisible: false,
        } as const;
        rsiChart.current
          .addSeries(LineSeries, { ...lineOpts })
          .setData(generateGuideLine(70));
        rsiChart.current
          .addSeries(LineSeries, { ...lineOpts })
          .setData(generateGuideLine(30));
      }
    }
    if (macdRef.current) {
      macdChart.current = createSubChart(macdRef);
      if (macdChart.current) {
        series.current.macdHist = macdChart.current.addSeries(HistogramSeries, {
          title: "Hist",
        });
        series.current.macd = macdChart.current.addSeries(LineSeries, {
          color: "#3b82f6",
          lineWidth: 1,
          title: "MACD",
        }); // Blue
        series.current.macdSignal = macdChart.current.addSeries(LineSeries, {
          color: "#f97316",
          lineWidth: 1,
          title: "Sig",
        }); // Orange
      }
    }
    if (stochRef.current) {
      stochChart.current = createSubChart(stochRef);
      if (stochChart.current) {
        series.current.stochK = stochChart.current.addSeries(LineSeries, {
          color: "#3b82f6",
          lineWidth: 1,
          title: "K",
        });
        series.current.stochD = stochChart.current.addSeries(LineSeries, {
          color: "#f97316",
          lineWidth: 1,
          title: "D",
        });
        // Guides
        const lineOpts = {
          color: "rgba(255,255,255,0.1)",
          lineWidth: 1,
          lineStyle: LineStyle.Dashed,
          crosshairMarkerVisible: false,
        } as const;
        stochChart.current
          .addSeries(LineSeries, { ...lineOpts })
          .setData(generateGuideLine(80));
        stochChart.current
          .addSeries(LineSeries, { ...lineOpts })
          .setData(generateGuideLine(20));
      }
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

    let isMounted = true;
    let unlisten: (() => void) | undefined;
    listen<KlineWithIndicator>("kline-update", (e) => {
      if (!isMounted) return; // Skip if component is unmounted
      const d = e.payload;
      if (d.symbol !== symbol) return;
      const t = (d.t / 1000) as Time;
      series.current.candle?.update({
        time: t,
        open: d.o,
        high: d.h,
        low: d.l,
        close: d.c,
      });
      if (d.sma) series.current.sma?.update({ time: t, value: d.sma });
      if (d.upper_band)
        series.current.upperBB?.update({ time: t, value: d.upper_band });
      if (d.lower_band)
        series.current.lowerBB?.update({ time: t, value: d.lower_band });
      if (d.rsi) series.current.rsi?.update({ time: t, value: d.rsi });
      if (d.macd) series.current.macd?.update({ time: t, value: d.macd });
      if (d.macd_signal)
        series.current.macdSignal?.update({ time: t, value: d.macd_signal });
      if (d.macd_hist)
        series.current.macdHist?.update({
          time: t,
          value: d.macd_hist,
          color: d.macd_hist >= 0 ? "#26a69a" : "#ef5350",
        });
      if (d.stoch_k)
        series.current.stochK?.update({ time: t, value: d.stoch_k });
      if (d.stoch_d)
        series.current.stochD?.update({ time: t, value: d.stoch_d });
    }).then((u) => (unlisten = u));

    return () => {
      isMounted = false; // Mark as unmounted first
      window.removeEventListener("resize", handleResize);
      // IMPORTANT: Unsubscribe from events BEFORE removing charts
      if (unlisten) unlisten();
      // Now safe to remove charts
      mainChart.current?.remove();
      rsiChart.current?.remove();
      macdChart.current?.remove();
      stochChart.current?.remove();
    };
  }, [symbol, exchange, fetchData, syncCharts, period, multiplier]);

  return (
    <div className="bg-[#111620] rounded-lg border border-white/5 overflow-hidden flex flex-col h-full shadow-lg group hover:border-white/10 transition-colors">
      {/* Header */}
      <div className="px-3 py-2 border-b border-white/5 flex items-center justify-between bg-[#151b26]">
        <div className="flex items-center gap-2">
          <h3 className="font-bold text-gray-200 text-sm">{symbol}</h3>
          <span className="text-[10px] text-emerald-500 bg-emerald-500/10 px-1.5 py-0.5 rounded font-medium">
            LIVE
          </span>
        </div>

        {/* Toggle Buttons */}
        <div className="flex gap-1.5 opacity-60 group-hover:opacity-100 transition-opacity">
          <Settings2 size={12} className="text-gray-500 mr-1" />
          <ToggleBtn
            label="RSI"
            active={showSub.rsi}
            onClick={() => setShowSub((s) => ({ ...s, rsi: !s.rsi }))}
            color="text-amber-500"
          />
          <ToggleBtn
            label="MACD"
            active={showSub.macd}
            onClick={() => setShowSub((s) => ({ ...s, macd: !s.macd }))}
            color="text-blue-500"
          />
          <ToggleBtn
            label="STOCH"
            active={showSub.stoch}
            onClick={() => setShowSub((s) => ({ ...s, stoch: !s.stoch }))}
            color="text-indigo-500"
          />
        </div>
      </div>

      {/* Chart Area */}
      <div className="flex-1 flex flex-col min-h-0 bg-[#0B0E14]">
        <div ref={mainRef} className="flex-1 min-h-37.5" />

        {/* Sub Charts Containers (Always rendered but hidden via CSS height) */}
        <div
          className={`transition-all duration-300 ease-in-out overflow-hidden border-t border-white/5 ${
            showSub.rsi ? "h-20 opacity-100" : "h-0 opacity-0 border-none"
          }`}
        >
          <div ref={rsiRef} className="w-full h-full" />
        </div>
        <div
          className={`transition-all duration-300 ease-in-out overflow-hidden border-t border-white/5 ${
            showSub.macd ? "h-20 opacity-100" : "h-0 opacity-0 border-none"
          }`}
        >
          <div ref={macdRef} className="w-full h-full" />
        </div>
        <div
          className={`transition-all duration-300 ease-in-out overflow-hidden border-t border-white/5 ${
            showSub.stoch ? "h-20 opacity-100" : "h-0 opacity-0 border-none"
          }`}
        >
          <div ref={stochRef} className="w-full h-full" />
        </div>
      </div>
    </div>
  );
};

// Sub-components
const ToggleBtn = ({
  label,
  active,
  onClick,
  color,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
  color: string;
}) => (
  <button
    onClick={onClick}
    className={`text-[9px] font-bold px-1.5 py-0.5 rounded transition-all ${
      active ? `bg-white/5 ${color}` : "text-gray-600 hover:text-gray-400"
    }`}
  >
    {label}
  </button>
);

// Helper
const generateGuideLine = (val: number) => {
  const now = Math.floor(Date.now() / 1000);
  return [
    { time: (now - 1000000) as Time, value: val },
    { time: (now + 1000000) as Time, value: val },
  ];
};
