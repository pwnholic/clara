// Package market provides normalized market data types for trading.
// All price and quantity values use udecimal.Decimal for financial precision.
package market

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/pwnholic/clara/pkg/errors"
	"github.com/quagmt/udecimal"
)

// Symbol represents a trading pair identifier (e.g., "BTCUSDT").
// Symbols are normalized to uppercase internally.
type Symbol string

// NewSymbol creates a normalized Symbol from a string.
func NewSymbol(s string) Symbol {
	return Symbol(strings.ToUpper(strings.TrimSpace(s)))
}

// String returns the string representation of the symbol.
func (s Symbol) String() string {
	return string(s)
}

// IsValid returns true if the symbol is valid (non-empty).
func (s Symbol) IsValid() bool {
	return len(strings.TrimSpace(string(s))) > 0
}

// Base returns the base asset (e.g., "BTC" from "BTCUSDT").
// This is a simple heuristic - may not work for all exchanges.
func (s Symbol) Base() string {
	str := string(s)
	// Common quote assets
	quotes := []string{"USDT", "USDC", "USD", "BTC", "ETH", "BNB", "BUSD"}
	for _, q := range quotes {
		if strings.HasSuffix(str, q) {
			return strings.TrimSuffix(str, q)
		}
	}
	return str
}

// Quote returns the quote asset (e.g., "USDT" from "BTCUSDT").
func (s Symbol) Quote() string {
	str := string(s)
	quotes := []string{"USDT", "USDC", "USD", "BTC", "ETH", "BNB", "BUSD"}
	for _, q := range quotes {
		if strings.HasSuffix(str, q) {
			return q
		}
	}
	return ""
}

// MarshalJSON implements json.Marshaler.
func (s Symbol) MarshalJSON() ([]byte, error) {
	return json.Marshal(string(s))
}

// UnmarshalJSON implements json.Unmarshaler.
func (s *Symbol) UnmarshalJSON(data []byte) error {
	var str string
	if err := json.Unmarshal(data, &str); err != nil {
		return err
	}
	*s = NewSymbol(str)
	return nil
}

// Side represents the order side (buy or sell).
type Side int

const (
	SideBuy Side = iota
	SideSell
)

// String implements fmt.Stringer.
func (s Side) String() string {
	switch s {
	case SideBuy:
		return "buy"
	case SideSell:
		return "sell"
	default:
		return "unknown"
	}
}

// MarshalText implements encoding.TextMarshaler.
func (s Side) MarshalText() ([]byte, error) {
	return []byte(s.String()), nil
}

// UnmarshalText implements encoding.TextUnmarshaler.
func (s *Side) UnmarshalText(text []byte) error {
	switch strings.ToLower(string(text)) {
	case "buy", "bid":
		*s = SideBuy
	case "sell", "ask":
		*s = SideSell
	default:
		return errors.NewValidationError("side", fmt.Sprintf("invalid side: %s", string(text)))
	}
	return nil
}

// ParseSide parses a string into a Side.
func ParseSide(s string) (Side, error) {
	var side Side
	if err := side.UnmarshalText([]byte(s)); err != nil {
		return SideBuy, err
	}
	return side, nil
}

// Ticker represents normalized ticker data for a trading pair.
type Ticker struct {
	Symbol        Symbol          `json:"symbol"`
	LastPrice     udecimal.Decimal `json:"last_price"`
	BidPrice      udecimal.Decimal `json:"bid_price"`
	AskPrice      udecimal.Decimal `json:"ask_price"`
	BidQty        udecimal.Decimal `json:"bid_qty"`
	AskQty        udecimal.Decimal `json:"ask_qty"`
	High24h       udecimal.Decimal `json:"high_24h"`
	Low24h        udecimal.Decimal `json:"low_24h"`
	Volume24h     udecimal.Decimal `json:"volume_24h"`
	QuoteVolume24h udecimal.Decimal `json:"quote_volume_24h"`
	PriceChange   udecimal.Decimal `json:"price_change"`
	PriceChangePercent udecimal.Decimal `json:"price_change_percent"`
	Timestamp     time.Time       `json:"timestamp"`
}

// Spread returns the bid-ask spread (ask - bid).
func (t Ticker) Spread() udecimal.Decimal {
	return t.AskPrice.Sub(t.BidPrice)
}

// MidPrice returns the mid-price ((bid + ask) / 2).
func (t Ticker) MidPrice() (udecimal.Decimal, error) {
	sum := t.BidPrice.Add(t.AskPrice)
	return sum.Div64(2)
}

// SpreadPercent returns the spread as a percentage of mid-price.
func (t Ticker) SpreadPercent() (udecimal.Decimal, error) {
	mid, err := t.MidPrice()
	if err != nil {
		return udecimal.Decimal{}, fmt.Errorf("calculate mid price: %w", err)
	}
	if mid.IsZero() {
		return udecimal.Decimal{}, errors.NewValidationError("mid_price", "mid price is zero")
	}
	return t.Spread().Div(mid)
}

// OrderBookEntry represents a single price level in the order book.
type OrderBookEntry struct {
	Price udecimal.Decimal `json:"price"`
	Qty   udecimal.Decimal `json:"qty"`
}

// Value returns the total value at this price level (price * qty).
func (e OrderBookEntry) Value() udecimal.Decimal {
	return e.Price.Mul(e.Qty)
}

// OrderBook represents a normalized order book (depth snapshot).
type OrderBook struct {
	Symbol    Symbol             `json:"symbol"`
	Bids      []OrderBookEntry   `json:"bids"` // Sorted by price descending
	Asks      []OrderBookEntry   `json:"asks"` // Sorted by price ascending
	Timestamp time.Time          `json:"timestamp"`
	Sequence  uint64             `json:"sequence"`
}

// BestBid returns the best (highest) bid entry, or nil if empty.
func (ob OrderBook) BestBid() *OrderBookEntry {
	if len(ob.Bids) == 0 {
		return nil
	}
	return &ob.Bids[0]
}

// BestAsk returns the best (lowest) ask entry, or nil if empty.
func (ob OrderBook) BestAsk() *OrderBookEntry {
	if len(ob.Asks) == 0 {
		return nil
	}
	return &ob.Asks[0]
}

// Spread returns the bid-ask spread.
func (ob OrderBook) Spread() (udecimal.Decimal, error) {
	bestBid := ob.BestBid()
	bestAsk := ob.BestAsk()
	if bestBid == nil || bestAsk == nil {
		return udecimal.Decimal{}, errors.NewValidationError("orderbook", "insufficient depth")
	}
	return bestAsk.Price.Sub(bestBid.Price), nil
}

// Depth returns the number of bid and ask levels.
func (ob OrderBook) Depth() (bids, asks int) {
	return len(ob.Bids), len(ob.Asks)
}

// Trade represents a normalized trade execution.
type Trade struct {
	ID            string           `json:"id"`
	Symbol        Symbol           `json:"symbol"`
	Price         udecimal.Decimal `json:"price"`
	Qty           udecimal.Decimal `json:"qty"`
	Side          Side             `json:"side"`
	Timestamp     time.Time        `json:"timestamp"`
	IsBuyerMaker  bool             `json:"is_buyer_maker"`
}

// Value returns the trade value (price * qty).
func (t Trade) Value() udecimal.Decimal {
	return t.Price.Mul(t.Qty)
}

// KlineInterval represents a kline/candlestick interval.
type KlineInterval string

const (
	Interval1m  KlineInterval = "1m"
	Interval3m  KlineInterval = "3m"
	Interval5m  KlineInterval = "5m"
	Interval15m KlineInterval = "15m"
	Interval30m KlineInterval = "30m"
	Interval1h  KlineInterval = "1h"
	Interval2h  KlineInterval = "2h"
	Interval4h  KlineInterval = "4h"
	Interval6h  KlineInterval = "6h"
	Interval8h  KlineInterval = "8h"
	Interval12h KlineInterval = "12h"
	Interval1d  KlineInterval = "1d"
	Interval3d  KlineInterval = "3d"
	Interval1w  KlineInterval = "1w"
	Interval1M  KlineInterval = "1M"
)

// String implements fmt.Stringer.
func (i KlineInterval) String() string {
	return string(i)
}

// Kline represents a normalized candlestick/OHLCV data point.
type Kline struct {
	Symbol      Symbol           `json:"symbol"`
	Interval    KlineInterval    `json:"interval"`
	OpenTime    time.Time        `json:"open_time"`
	CloseTime   time.Time        `json:"close_time"`
	Open        udecimal.Decimal `json:"open"`
	High        udecimal.Decimal `json:"high"`
	Low         udecimal.Decimal `json:"low"`
	Close       udecimal.Decimal `json:"close"`
	Volume      udecimal.Decimal `json:"volume"`
	QuoteVolume udecimal.Decimal `json:"quote_volume"`
	TradeCount  uint64           `json:"trade_count"`
	IsClosed    bool             `json:"is_closed"`
}

// Change returns the price change (close - open).
func (k Kline) Change() udecimal.Decimal {
	return k.Close.Sub(k.Open)
}

// ChangePercent returns the price change as a percentage.
func (k Kline) ChangePercent() (udecimal.Decimal, error) {
	if k.Open.IsZero() {
		return udecimal.Decimal{}, errors.NewValidationError("open", "open price is zero")
	}
	return k.Change().Div(k.Open)
}

// Range returns the price range (high - low).
func (k Kline) Range() udecimal.Decimal {
	return k.High.Sub(k.Low)
}

// VWAP returns the volume-weighted average price.
func (k Kline) VWAP() (udecimal.Decimal, error) {
	if k.Volume.IsZero() {
		return udecimal.Decimal{}, errors.NewValidationError("volume", "volume is zero")
	}
	return k.QuoteVolume.Div(k.Volume)
}

// IsBullish returns true if the candle is bullish (close > open).
func (k Kline) IsBullish() bool {
	return k.Close.GreaterThan(k.Open)
}

// IsBearish returns true if the candle is bearish (close < open).
func (k Kline) IsBearish() bool {
	return k.Close.LessThan(k.Open)
}
