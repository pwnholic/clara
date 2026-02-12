import type {
  Market,
  ScanResponse,
  OrderRequest,
  OrderResponse,
  OrdersResponse,
  PositionsResponse,
  PolymarketOrderRequest,
  PolymarketOrderResponse,
  PolymarketOrdersResponse,
  PolymarketPosition,
  ArbitrageExecuteRequest,
  ArbitrageExecuteResponse,
  AutoTradeStatus,
  AppSettings,
  AccountBalance,
  OrderbookDepthResponse,
  HistoryEntry,
} from "@/types";

const API_BASE = "/api";

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }
  return response.json();
}

export const api = {
  markets: {
    kalshi: {
      list: () => fetchJson<Market[]>(`${API_BASE}/markets/kalshi`),
    },
    polymarket: {
      list: () => fetchJson<Market[]>(`${API_BASE}/markets/polymarket`),
    },
  },

  opportunities: {
    list: () => fetchJson<ScanResponse>(`${API_BASE}/opportunities`),
    scan: () => fetchJson<ScanResponse>(`${API_BASE}/scan`, { method: "POST" }),
  },

  orderbook: {
    depth: (params: {
      kalshi_ticker?: string;
      poly_token_id?: string;
      poly_opponent_token_id?: string;
    }) => {
      const searchParams = new URLSearchParams();
      if (params.kalshi_ticker)
        searchParams.append("kalshi_ticker", params.kalshi_ticker);
      if (params.poly_token_id)
        searchParams.append("poly_token_id", params.poly_token_id);
      if (params.poly_opponent_token_id)
        searchParams.append(
          "poly_opponent_token_id",
          params.poly_opponent_token_id,
        );
      return fetchJson<OrderbookDepthResponse>(
        `${API_BASE}/orderbook/depth?${searchParams}`,
      );
    },
  },

  orders: {
    kalshi: {
      list: (status?: string) => {
        const url = status
          ? `${API_BASE}/orders/kalshi?status=${status}`
          : `${API_BASE}/orders/kalshi`;
        return fetchJson<OrdersResponse>(url);
      },
      create: (data: OrderRequest) =>
        fetchJson<OrderResponse>(`${API_BASE}/order/kalshi`, {
          method: "POST",
          body: JSON.stringify(data),
        }),
      cancel: (orderId: string) =>
        fetchJson<{ success: boolean; error?: string }>(
          `${API_BASE}/orders/kalshi/${orderId}`,
          {
            method: "DELETE",
          },
        ),
    },
    polymarket: {
      list: () =>
        fetchJson<PolymarketOrdersResponse>(`${API_BASE}/orders/polymarket`),
      create: (data: PolymarketOrderRequest) =>
        fetchJson<PolymarketOrderResponse>(`${API_BASE}/order/polymarket`, {
          method: "POST",
          body: JSON.stringify(data),
        }),
      cancel: (orderId: string) =>
        fetchJson<{ success: boolean; error?: string }>(
          `${API_BASE}/orders/polymarket/${orderId}`,
          {
            method: "DELETE",
          },
        ),
    },
  },

  positions: {
    kalshi: {
      list: () => fetchJson<PositionsResponse>(`${API_BASE}/positions/kalshi`),
    },
    polymarket: {
      list: () =>
        fetchJson<{ positions: PolymarketPosition[]; error?: string }>(
          `${API_BASE}/positions/polymarket`,
        ),
    },
  },

  accounts: {
    balance: () => fetchJson<AccountBalance>(`${API_BASE}/accounts/balance`),
  },

  arbitrage: {
    execute: (data: ArbitrageExecuteRequest) =>
      fetchJson<ArbitrageExecuteResponse>(`${API_BASE}/arbitrage/execute`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
  },

  autoTrade: {
    status: () => fetchJson<AutoTradeStatus>(`${API_BASE}/auto-trade/status`),
    enable: () =>
      fetchJson<{ success: boolean; message?: string; error?: string }>(
        `${API_BASE}/auto-trade/enable`,
        {
          method: "POST",
        },
      ),
    disable: () =>
      fetchJson<{ success: boolean; message?: string; error?: string }>(
        `${API_BASE}/auto-trade/disable`,
        {
          method: "POST",
        },
      ),
    reset: () =>
      fetchJson<{ success: boolean; message?: string; error?: string }>(
        `${API_BASE}/auto-trade/reset`,
        {
          method: "POST",
        },
      ),
    updateSettings: (settings: {
      max_amount?: number;
      min_duration_ms?: number;
      max_trade_count?: number;
      flexible_mode?: boolean;
      max_contracts?: number;
      min_contracts?: number;
    }) =>
      fetchJson<{ success: boolean; message?: string; error?: string }>(
        `${API_BASE}/auto-trade/settings`,
        {
          method: "PUT",
          body: JSON.stringify(settings),
        },
      ),
  },

  settings: {
    get: () => fetchJson<AppSettings>(`${API_BASE}/settings`),
    update: (settings: {
      refresh_interval?: number;
      min_profit_margin?: number;
      default_bet_amount?: number;
      tracking_threshold?: number;
    }) =>
      fetchJson<{ success: boolean; message?: string; error?: string }>(
        `${API_BASE}/settings`,
        {
          method: "PUT",
          body: JSON.stringify(settings),
        },
      ),
  },

  history: {
    list: (limit = 50, offset = 0) =>
      fetchJson<{ entries: HistoryEntry[]; total: number }>(
        `${API_BASE}/history?limit=${limit}&offset=${offset}`,
      ),
  },

  dataCoverage: {
    get: () =>
      fetchJson<{
        total_markets: number;
        kalshi_ready: number;
        polymarket_ready: number;
        both_ready: number;
        kalshi_coverage: string;
        polymarket_coverage: string;
        full_coverage: string;
        kalshi_connected: boolean;
        polymarket_connected: boolean;
      }>(`${API_BASE}/data-coverage`),
  },

  health: () =>
    fetchJson<{ status: string; version: string }>(`${API_BASE}/health`),
};

export default api;
