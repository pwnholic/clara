import { useEffect, useRef, useState, useCallback } from "react";
import type {
  WsMessage,
  LogEntry,
  MatchedMarketData,
  MetricsReport,
  DataCoverage,
} from "@/types";

interface UseWebSocketReturn {
  matchedMarkets: MatchedMarketData[];
  logs: LogEntry[];
  isConnected: boolean;
  isReceivingData: boolean;
  lastUpdateTime: Date | null;
  updateCount: number;
  stats: {
    kalshiCount: number;
    polymarketCount: number;
    matchedCount: number;
    opportunitiesCount: number;
  };
  dataCoverage: DataCoverage;
  metrics: MetricsReport | null;
}

export function useWebSocket(url: string): UseWebSocketReturn {
  const [matchedMarkets, setMatchedMarkets] = useState<MatchedMarketData[]>([]);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [isReceivingData, setIsReceivingData] = useState(false);
  const [lastUpdateTime, setLastUpdateTime] = useState<Date | null>(null);
  const [updateCount, setUpdateCount] = useState(0);
  const [stats, setStats] = useState({
    kalshiCount: 0,
    polymarketCount: 0,
    matchedCount: 0,
    opportunitiesCount: 0,
  });
  const [dataCoverage, setDataCoverage] = useState<DataCoverage>({
    total_markets: 0,
    kalshi_ready: 0,
    polymarket_ready: 0,
    both_ready: 0,
    kalshi_coverage: "0/0",
    polymarket_coverage: "0/0",
    full_coverage: "0/0",
    kalshi_connected: false,
    polymarket_connected: false,
  });
  const [metrics, setMetrics] = useState<MetricsReport | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const prevPricesRef = useRef<
    Map<string, { k_yes: number; k_no: number; p_yes: number; p_no: number }>
  >(new Map());

  const addLog = useCallback((level: LogEntry["level"], message: string) => {
    const entry: LogEntry = {
      time: new Date().toISOString(),
      level,
      message,
    };
    setLogs((prev) => [entry, ...prev].slice(0, 100));
  }, []);

  useEffect(() => {
    const handleMessage = (message: WsMessage) => {
      switch (message.type) {
        case "opportunity":
          if (message.data) {
            const opp = message.data as unknown as MatchedMarketData;
            addLog(
              "success",
              `Found: ${opp.event_name} ${opp.team_name} - ${opp.profit_margin.toFixed(2)}%`,
            );
          }
          break;

        case "opportunities":
          if (message.data && Array.isArray(message.data)) {
            setIsReceivingData(true);
            const opportunities = message.data as MatchedMarketData[];
            setStats((prev) => ({
              ...prev,
              opportunitiesCount: opportunities.length,
            }));
            setLastUpdateTime(new Date());
            setUpdateCount((prev) => prev + 1);
          }
          break;

        case "stats":
          if (message.data) {
            const statsData = message.data as {
              total_kalshi_markets?: number;
              total_polymarket_markets?: number;
              matched_markets?: number;
              arbitrage_opportunities?: number;
            };
            setStats({
              kalshiCount: statsData.total_kalshi_markets || 0,
              polymarketCount: statsData.total_polymarket_markets || 0,
              matchedCount: statsData.matched_markets || 0,
              opportunitiesCount: statsData.arbitrage_opportunities || 0,
            });
          }
          break;

        case "log":
          if (message.level && message.message) {
            addLog(message.level as LogEntry["level"], message.message);
          }
          break;

        case "matched_markets_list":
          if (message.data && Array.isArray(message.data)) {
            setIsReceivingData(true);
            const marketsData = message.data as MatchedMarketData[];

            const newPricesMap = new Map<
              string,
              { k_yes: number; k_no: number; p_yes: number; p_no: number }
            >();
            marketsData.forEach((m) => {
              const key = `${m.kalshi_market_id}_${m.polymarket_market_id}`;
              newPricesMap.set(key, {
                k_yes: m.kalshi_yes_price,
                k_no: m.kalshi_no_price,
                p_yes: m.poly_yes_price,
                p_no: m.poly_no_price,
              });
            });
            prevPricesRef.current = newPricesMap;

            setMatchedMarkets(marketsData);
            setLastUpdateTime(new Date());
            setUpdateCount((prev) => prev + 1);

            if (message.count !== undefined) {
              setStats((prev) => ({
                ...prev,
                matchedCount: message.count!,
                opportunitiesCount:
                  message.opportunities_count || prev.opportunitiesCount,
              }));
            }
          }
          break;

        case "metrics":
          if (
            message.data &&
            typeof message.data === "object" &&
            "api_latency" in message.data
          ) {
            setMetrics(message.data as MetricsReport);
          }
          break;
      }
    };

    const connect = () => {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        setIsConnected(true);
        addLog("success", "WebSocket connected");
      };

      ws.onmessage = (event) => {
        try {
          const message: WsMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error("Failed to parse WebSocket message:", error);
        }
      };

      ws.onerror = () => {
        addLog("error", "WebSocket connection error");
      };

      ws.onclose = () => {
        setIsConnected(false);
        addLog("warning", "WebSocket disconnected, reconnecting in 5s...");
        setTimeout(connect, 5000);
      };
    };

    connect();

    return () => {
      wsRef.current?.close();
    };
  }, [url, addLog]);

  useEffect(() => {
    const fetchCoverage = async () => {
      try {
        const apiUrl = url
          .replace("ws://", "http://")
          .replace("wss://", "https://")
          .replace("/ws", "");
        const response = await fetch(`${apiUrl}/api/data-coverage`);
        if (response.ok) {
          const data = await response.json();
          setDataCoverage(data);
        }
      } catch {
        // Silent fail
      }
    };

    fetchCoverage();
    const interval = setInterval(fetchCoverage, 3000);
    return () => clearInterval(interval);
  }, [url]);

  return {
    matchedMarkets,
    logs,
    isConnected,
    isReceivingData,
    lastUpdateTime,
    updateCount,
    stats,
    dataCoverage,
    metrics,
  };
}
