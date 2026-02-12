export type Platform = "kalshi" | "polymarket";
export type Side = "yes" | "no";
export type OrderStatus =
  | "pending"
  | "filled"
  | "cancelled"
  | "resting"
  | "executed";
export type ArbitrageType = "KalshiYesPolymarketNo" | "KalshiNoPolymarketYes";

export interface Market {
  platform: "Kalshi" | "Polymarket";
  event_id: string;
  event_name: string;
  yes_price: number;
  no_price: number;
  volume: number;
  team_name?: string;
  category: string;
  end_time?: string;
}

export interface MatchedMarketData {
  event_name: string;
  team_name: string;
  game_date?: string;
  kalshi_market_id: string;
  polymarket_market_id: string;
  poly_token_id?: string;
  poly_opponent_token_id?: string;
  kalshi_yes_price: number;
  kalshi_no_price: number;
  poly_yes_price: number;
  poly_no_price: number;
  kalshi_ready: boolean;
  poly_ready: boolean;
  both_ready: boolean;
  confidence: number;
  end_time?: string;
  has_opportunity: boolean;
  profit_margin: number;
  expected_profit: number;
  gross_profit?: number;
  kalshi_contracts?: number;
  kalshi_fee?: number;
  arbitrage_type?: string;
}

export interface ArbitrageOpportunity {
  kalshi_market: Market;
  polymarket_market: Market;
  arbitrage_type: ArbitrageType;
  profit_margin: number;
  optimal_bet: [number, number];
  expected_profit: number;
  match_confidence: number;
}

export interface ScanResponse {
  kalshi_markets: Market[];
  polymarket_markets: Market[];
  opportunities: ArbitrageOpportunity[];
  matched_count: number;
}

export interface WsMessage {
  type:
    | "opportunity"
    | "opportunities"
    | "opportunities_list"
    | "matched_markets_list"
    | "scan_started"
    | "scan_completed"
    | "log"
    | "ping"
    | "pong"
    | "connected"
    | "stats"
    | "metrics";
  data?:
    | ArbitrageOpportunity
    | ArbitrageOpportunity[]
    | MatchedMarketData[]
    | MetricsReport;
  kalshi_count?: number;
  polymarket_count?: number;
  matched_count?: number;
  opportunities_count?: number;
  count?: number;
  level?: string;
  message?: string;
  timestamp?: string;
}

export interface OperationStats {
  name: string;
  count: number;
  avg_ms: number;
  max_ms: number;
  total_ms: number;
}

export interface ApiLatency {
  kalshi_ms?: number;
  polymarket_ms?: number;
}

export interface MetricsReport {
  operations: OperationStats[];
  api_latency: ApiLatency;
}

export interface LogEntry {
  time: string;
  level: "info" | "success" | "warning" | "error";
  message: string;
}

export interface ProfitHistoryEntry {
  time: string;
  profit_margin: number;
  kalshi_price: number;
  polymarket_price: number;
}

export interface TrackingRecord {
  event_name: string;
  team_name: string;
  kalshi_market_id: string;
  polymarket_market_id: string;
  start_time: string;
  end_time: string | null;
  duration_seconds: number | null;
  duration_ms: number | null;
  max_profit_margin: number;
  max_profit_time: string | null;
  profit_history: ProfitHistoryEntry[];
  poly_ask_depth: number;
  kalshi_ask_depth: number;
}

export interface ActiveTracking {
  event_name: string;
  team_name: string;
  start_time: string;
  duration_seconds: number;
  max_profit_margin: number;
}

export interface TrackingStats {
  active_count: number;
  completed_count: number;
  active: ActiveTracking[];
  recent_completed: TrackingRecord[];
}

export interface DataCoverage {
  total_markets: number;
  kalshi_ready: number;
  polymarket_ready: number;
  both_ready: number;
  kalshi_coverage: string;
  polymarket_coverage: string;
  full_coverage: string;
  kalshi_connected: boolean;
  polymarket_connected: boolean;
  kalshi_latency_ms?: number;
  polymarket_latency_ms?: number;
}

export interface PlatformBalance {
  available: boolean;
  balance?: number;
  portfolio_value?: number;
  error?: string;
  pnl?: string;
  trades?: number;
  positions?: number;
  updated_ts?: number;
}

export interface AccountBalance {
  kalshi: PlatformBalance;
  polymarket: PlatformBalance;
}

export interface KalshiOrder {
  order_id: string;
  user_id?: string;
  client_order_id?: string;
  ticker: string;
  side: "yes" | "no";
  action: "buy" | "sell";
  type: "limit" | "market";
  status: "resting" | "canceled" | "executed" | "pending";
  yes_price?: number;
  no_price?: number;
  fill_count: number;
  remaining_count: number;
  initial_count: number;
  taker_fees?: number;
  maker_fees?: number;
  taker_fill_cost?: number;
  maker_fill_cost?: number;
  created_time: string;
  last_update_time?: string;
}

export interface KalshiPosition {
  ticker: string;
  event_ticker?: string;
  market_exposure: number;
  position: number;
  resting_orders_count: number;
  fees_paid?: number;
  total_traded?: number;
  realized_pnl?: number;
}

export interface KalshiOrderRequest {
  ticker: string;
  side: "yes" | "no";
  action: "buy" | "sell";
  count: number;
}

export type OrderRequest = KalshiOrderRequest;

export interface PolymarketOrderRequest {
  token_id: string;
  side: "buy" | "sell";
  amount: number;
}

export interface OrderResponse {
  success: boolean;
  order?: KalshiOrder;
  elapsed_ms?: number;
  error?: string;
}

export interface PolymarketOrderResponse {
  success: boolean;
  order_id?: string;
  status?: string;
  taking_amount?: string;
  making_amount?: string;
  elapsed_ms?: number;
  error?: string;
}

export interface PolymarketOrder {
  id: string;
  status: string;
  owner: string;
  maker_address: string;
  market: string;
  asset_id: string;
  side: string;
  original_size: string;
  size_matched: string;
  price: string;
  outcome: string;
  created_at: number;
  order_type: string;
}

export interface PolymarketPosition {
  id?: string;
  asset?: string;
  conditionId?: string;
  outcomeIndex?: number;
  size?: string;
  avgPrice?: string;
  curPrice?: string;
  value?: string;
  pnl?: string;
  pnlPercent?: string;
  title?: string;
  [key: string]: unknown;
}

export interface UnifiedPosition {
  id: string;
  platform: "kalshi" | "polymarket";
  ticker: string;
  title?: string;
  size: number;
  side?: "yes" | "no" | "buy" | "sell";
  avgPrice?: number;
  curPrice?: number;
  value?: number;
  pnl?: number;
  pnlPercent?: number;
}

export interface OrdersResponse {
  orders: KalshiOrder[];
  error?: string;
}

export interface PolymarketOrdersResponse {
  orders: PolymarketOrder[];
  error?: string;
}

export interface PositionsResponse {
  positions: KalshiPosition[];
  error?: string;
}

export interface ArbitrageExecuteRequest {
  kalshi_ticker: string;
  kalshi_side: "yes" | "no";
  kalshi_bet: number;
  kalshi_price: number;
  poly_token_id: string;
  poly_side: "buy" | "sell";
  poly_amount: number;
}

export interface ArbitrageExecuteResponse {
  success: boolean;
  error?: string;
  kalshi: {
    success: boolean;
    order?: Record<string, unknown>;
    elapsed_ms?: number;
    count?: number;
    error?: string;
  };
  polymarket: {
    success: boolean;
    order_id?: string;
    status?: string;
    elapsed_ms?: number;
    amount?: number;
    error?: string;
  };
}

export interface SideDepth {
  price?: number;
  size?: number;
}

export interface PlatformDepthDual {
  yes: SideDepth;
  no: SideDepth;
}

export interface OrderbookDepthResponse {
  kalshi?: PlatformDepthDual;
  polymarket?: PlatformDepthDual;
}

export interface AutoTradeStatus {
  enabled: boolean;
  trade_count: number;
  max_trade_count: number;
  remaining: number;
  max_amount: number;
  min_duration_ms: number;
  flexible_mode: boolean;
  max_contracts: number;
  min_contracts: number;
  last_trade_time: string | null;
}

export interface AppSettings {
  refresh_interval: number;
  min_profit_margin: number;
  default_bet_amount: number;
  tracking_threshold: number;
  updated_at: string | null;
}

export interface HistoryEntry {
  id: string;
  timestamp: string;
  event_name: string;
  team_name?: string;
  kalshi_ticker: string;
  poly_market_id: string;
  kalshi_side: Side;
  kalshi_price: number;
  kalshi_amount: number;
  poly_side: "buy" | "sell";
  poly_price: number;
  poly_amount: number;
  profit_margin: number;
  realized_profit?: number;
  status: "executed" | "partial" | "failed";
}
