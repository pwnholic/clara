export const API_BASE_URL = "";
export const WS_URL = "";

export const PLATFORM_COLORS = {
  kalshi: {
    primary: "#3B82F6",
    bg: "bg-blue-500/20",
    text: "text-blue-400",
    border: "border-blue-500/30",
  },
  polymarket: {
    primary: "#8B5CF6",
    bg: "bg-purple-500/20",
    text: "text-purple-400",
    border: "border-purple-500/30",
  },
} as const;

export const STATUS_COLORS: Record<string, string> = {
  pending: "bg-yellow-500/20 text-yellow-400",
  resting: "bg-blue-500/20 text-blue-400",
  filled: "bg-green-500/20 text-green-400",
  executed: "bg-green-500/20 text-green-400",
  cancelled: "bg-red-500/20 text-red-400",
  canceled: "bg-red-500/20 text-red-400",
  partial: "bg-orange-500/20 text-orange-400",
  failed: "bg-red-500/20 text-red-400",
};

export const REFRESH_INTERVALS = {
  fast: 1000,
  normal: 3000,
  slow: 5000,
} as const;

export const DEFAULT_BET_AMOUNT = 100;
export const MIN_PROFIT_MARGIN = 0.5;
export const MAX_TRADE_COUNT = 10;
