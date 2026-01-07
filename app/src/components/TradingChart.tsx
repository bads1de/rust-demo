import { useEffect, useRef } from "react";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type CandlestickData,
  type Time,
  CandlestickSeries,
} from "lightweight-charts";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface KlineData {
  t: number;
  o: string;
  h: string;
  l: string;
  c: string;
}

export const TradingChart = () => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);

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

    const series = chart.addSeries(CandlestickSeries, {
      upColor: "#10b981",
      downColor: "#ef4444",
      borderVisible: false,
      wickUpColor: "#10b981",
      wickDownColor: "#ef4444",
    });

    chartRef.current = chart;
    seriesRef.current = series;

    // Fetch Historical Data
    invoke<KlineData[]>("fetch_candles")
      .then((data) => {
        const candleData = data.map((d) => ({
          time: (d.t / 1000) as Time,
          open: parseFloat(d.o),
          high: parseFloat(d.h),
          low: parseFloat(d.l),
          close: parseFloat(d.c),
        }));
        series.setData(candleData);
      })
      .catch((e) => {
        console.error("Failed to fetch historical data", e);
      });

    // Handle Resize
    const handleResize = () => {
      if (chartContainerRef.current) {
        chart.applyOptions({ width: chartContainerRef.current.clientWidth });
      }
    };
    window.addEventListener("resize", handleResize);

    // Tauri Event Listener
    let unlistenFunction: (() => void) | undefined;

    listen<KlineData>("kline-update", (event) => {
      const { t, o, h, l, c } = event.payload;

      const newData: CandlestickData<Time> = {
        time: (t / 1000) as Time,
        open: parseFloat(o),
        high: parseFloat(h),
        low: parseFloat(l),
        close: parseFloat(c),
      };

      series.update(newData);
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
      if (unlistenFunction) {
        unlistenFunction();
      }
    };
  }, []);

  return (
    <div className="w-full bg-slate-900/50 backdrop-blur-md rounded-2xl border border-slate-800 overflow-hidden shadow-2xl p-4">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-slate-200 font-bold flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          BTC / USDT 1m
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