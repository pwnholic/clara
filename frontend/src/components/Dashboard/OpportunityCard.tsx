import clsx from "clsx";
import { TrendingUp, TrendingDown, Clock, AlertTriangle } from "lucide-react";
import type { MatchedMarketData } from "@/types";
import {
  formatPriceCents,
  formatPercent,
  formatCurrency,
  formatDateTime,
} from "@/utils/format";

interface OpportunityCardProps {
  market: MatchedMarketData;
  isSelected: boolean;
  onClick: () => void;
}

export function OpportunityCard({
  market,
  isSelected,
  onClick,
}: OpportunityCardProps) {
  const hasOpportunity = market.has_opportunity;
  const profitPositive = market.profit_margin >= 0;

  return (
    <div
      onClick={onClick}
      className={clsx(
        "p-3 rounded-lg cursor-pointer transition-all duration-200 border",
        isSelected
          ? "bg-blue-900/30 border-blue-500 ring-1 ring-blue-500"
          : "bg-gray-800 border-gray-700 hover:bg-gray-750 hover:border-gray-600",
        hasOpportunity && !isSelected && "border-l-4 border-l-green-500",
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium text-white truncate">
            {market.event_name}
          </div>
          {market.team_name && (
            <div className="text-xs text-yellow-400 mt-0.5">
              {market.team_name}
            </div>
          )}
        </div>

        <div className="flex-shrink-0 text-right">
          {hasOpportunity ? (
            <div className="flex items-center gap-1">
              <TrendingUp className="w-4 h-4 text-green-400" />
              <span className="text-sm font-bold text-green-400">
                {formatPercent(market.profit_margin)}
              </span>
            </div>
          ) : (
            <div className="flex items-center gap-1">
              <TrendingDown className="w-4 h-4 text-gray-500" />
              <span className="text-sm text-gray-500">
                {formatPercent(market.profit_margin)}
              </span>
            </div>
          )}
          <div
            className={clsx(
              "text-xs mt-0.5",
              profitPositive ? "text-green-400/70" : "text-gray-500",
            )}
          >
            Net: {formatCurrency(market.expected_profit)}
          </div>
        </div>
      </div>

      <div className="mt-2 flex items-center gap-3 text-xs">
        <div className="flex items-center gap-2">
          <span className="text-blue-400 font-medium">K:</span>
          <span className="text-green-400">
            {formatPriceCents(market.kalshi_yes_price)}¢
          </span>
          <span className="text-gray-500">/</span>
          <span className="text-red-400">
            {formatPriceCents(market.kalshi_no_price)}¢
          </span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-purple-400 font-medium">P:</span>
          <span className="text-green-400">
            {formatPriceCents(market.poly_yes_price)}¢
          </span>
          <span className="text-gray-500">/</span>
          <span className="text-red-400">
            {formatPriceCents(market.poly_no_price)}¢
          </span>
        </div>
      </div>

      <div className="mt-2 flex items-center gap-2 text-xs">
        <div className="flex items-center gap-1">
          <span
            className={clsx(
              "px-1.5 py-0.5 rounded text-[10px]",
              market.kalshi_ready
                ? "bg-blue-500/20 text-blue-400"
                : "bg-gray-700 text-gray-500",
            )}
          >
            K{market.kalshi_ready ? "✓" : "○"}
          </span>
          <span
            className={clsx(
              "px-1.5 py-0.5 rounded text-[10px]",
              market.poly_ready
                ? "bg-purple-500/20 text-purple-400"
                : "bg-gray-700 text-gray-500",
            )}
          >
            P{market.poly_ready ? "✓" : "○"}
          </span>
        </div>

        {market.end_time && (
          <div className="flex items-center gap-1 text-gray-500">
            <Clock className="w-3 h-3" />
            <span className="truncate">{formatDateTime(market.end_time)}</span>
          </div>
        )}

        {market.confidence < 0.8 && (
          <div className="flex items-center gap-1 text-yellow-500">
            <AlertTriangle className="w-3 h-3" />
            <span>{(market.confidence * 100).toFixed(0)}%</span>
          </div>
        )}
      </div>
    </div>
  );
}
