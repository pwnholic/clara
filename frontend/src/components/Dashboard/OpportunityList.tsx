import { useState, useMemo } from "react";
import { Search, Filter, RefreshCw } from "lucide-react";
import type { MatchedMarketData } from "@/types";
import { OpportunityCard } from "./OpportunityCard";
import { Button } from "@/components/common/Button";

interface OpportunityListProps {
  markets: MatchedMarketData[];
  onSelectMarket: (market: MatchedMarketData) => void;
  selectedMarketId?: string;
  onRefresh?: () => void;
  isLoading?: boolean;
}

type SortKey = "profit" | "kalshi_yes" | "poly_yes" | "updated";

export function OpportunityList({
  markets,
  onSelectMarket,
  selectedMarketId,
  onRefresh,
  isLoading,
}: OpportunityListProps) {
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState<SortKey>("profit");
  const [showOpportunitiesOnly, setShowOpportunitiesOnly] = useState(false);

  const filteredMarkets = useMemo(() => {
    let result = [...markets];

    if (search) {
      const searchLower = search.toLowerCase();
      result = result.filter(
        (m) =>
          m.event_name.toLowerCase().includes(searchLower) ||
          m.team_name.toLowerCase().includes(searchLower),
      );
    }

    if (showOpportunitiesOnly) {
      result = result.filter((m) => m.has_opportunity);
    }

    result.sort((a, b) => {
      switch (sortBy) {
        case "profit":
          return b.profit_margin - a.profit_margin;
        case "kalshi_yes":
          return b.kalshi_yes_price - a.kalshi_yes_price;
        case "poly_yes":
          return b.poly_yes_price - a.poly_yes_price;
        default:
          return 0;
      }
    });

    return result;
  }, [markets, search, sortBy, showOpportunitiesOnly]);

  const opportunityCount = markets.filter((m) => m.has_opportunity).length;

  return (
    <div className="h-full flex flex-col bg-gray-900">
      <div className="flex-shrink-0 p-3 border-b border-gray-700 space-y-2">
        <div className="flex items-center gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input
              type="text"
              placeholder="Search markets..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full pl-9 pr-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-sm text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          {onRefresh && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onRefresh}
              disabled={isLoading}
              loading={isLoading}
            >
              <RefreshCw className="w-4 h-4" />
            </Button>
          )}
        </div>

        <div className="flex items-center gap-2 text-xs">
          <button
            onClick={() => setShowOpportunitiesOnly(!showOpportunitiesOnly)}
            className={`px-2 py-1 rounded transition-colors ${
              showOpportunitiesOnly
                ? "bg-green-500/20 text-green-400 border border-green-500/30"
                : "bg-gray-800 text-gray-400 hover:text-gray-300"
            }`}
          >
            <Filter className="w-3 h-3 inline mr-1" />
            Opportunities ({opportunityCount})
          </button>

          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as SortKey)}
            className="px-2 py-1 bg-gray-800 border border-gray-700 rounded text-gray-400 text-xs"
          >
            <option value="profit">Sort: Profit</option>
            <option value="kalshi_yes">Sort: Kalshi Yes</option>
            <option value="poly_yes">Sort: Poly Yes</option>
          </select>

          <span className="text-gray-500 ml-auto">
            {filteredMarkets.length} markets
          </span>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {filteredMarkets.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            {search ? "No markets match your search" : "No markets available"}
          </div>
        ) : (
          filteredMarkets.map((market) => (
            <OpportunityCard
              key={`${market.kalshi_market_id}_${market.polymarket_market_id}`}
              market={market}
              isSelected={
                selectedMarketId ===
                `${market.kalshi_market_id}_${market.polymarket_market_id}`
              }
              onClick={() => onSelectMarket(market)}
            />
          ))
        )}
      </div>
    </div>
  );
}
