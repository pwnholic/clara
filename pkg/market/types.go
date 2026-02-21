// Package market provides normalized market data types for trading.
// All price and quantity values use udecimal.Decimal for financial precision.
package market

import (
	"fmt"
	"time"

	"github.com/quagmt/udecimal"
)

// Symbol represents a trading pair identifier (e.g., "BTCUSDT").
type Symbol string

// String returns the string representation of the symbol.
func (s Symbol) String() string {
	return string(s)
}

// Side represents the order side (buy or sell).
type Side int

const (
	SideBuy Side = iota
	SideSell
)

// String returns the string representation of the side.
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

// ParseSide parses a string into a Side.
func ParseSide(s string) (Side, error) {
	switch s {
	case "buy", "BUY", "Buy":
		return SideBuy, nil
	case "sell", "SELL", "Sell":
		return SideSell, nil
	default:
		return SideBuy, fmt.Errorf("invalid side: %s", s)
	}
}

// Ticker represents normalized ticker data for a trading pair.
type Ticker struct {
	// Symbol is the trading pair identifier.
	Symbol Symbol

	// LastPrice is the last traded price.
	LastPrice udecimal.Decimal

	// BidPrice is the best bid price.
	BidPrice udecimal.Decimal

	// AskPrice is the best ask price.
	AskPrice udecimal.Decimal

	// BidQty is the quantity at best bid.
	BidQty udecimal.Decimal

	// AskQty is the quantity at best ask.
	AskQty udecimal.Decimal

	// High24h is the 24-hour high price.
	High24h udecimal.Decimal

	// Low24h is the 24-hour low price.
	Low24h udecimal.Decimal

	// Volume24h is the 24-hour trading volume.
	Volume24h udecimal.Decimal

	// QuoteVolume24h is the 24-hour quote volume.
	QuoteVolume24h udecimal.Decimal

	// Timestamp is when this ticker was generated.
	Timestamp time.Time
}

// Spread returns the bid-ask spread.
func (t Ticker) Spread() udecimal.Decimal {
	return t.AskPrice.Sub(t.BidPrice)
}

// SpreadPercent returns the spread as a percentage of mid-price.
func (t Ticker) SpreadPercent() (udecimal.Decimal, error) {
	mid, err := t.BidPrice.Add(t.AskPrice).Div64(2)
	if err != nil {
		return udecimal.Decimal{}, fmt.Errorf("cannot calculate mid price: %w", err)
	}
	if mid.IsZero() {
		return udecimal.Decimal{}, fmt.Errorf("cannot calculate spread percent: mid price is zero")
	}
	return t.Spread().Div(mid)
}

// OrderBookEntry represents a single price level in the order book.
type OrderBookEntry struct {
	Price udecimal.Decimal
	Qty   udecimal.Decimal
}

// OrderBook represents a normalized order book (depth snapshot).
type OrderBook struct {
	// Symbol is the trading pair identifier.
	Symbol Symbol

	// Bids are buy orders sorted by price descending.
	Bids []OrderBookEntry

	// Asks are sell orders sorted by price ascending.
	Asks []OrderBookEntry

	// Timestamp is when this order book snapshot was taken.
	Timestamp time.Time

	// Sequence is the update sequence number (for detecting gaps).
	Sequence uint64
}

// Trade represents a normalized trade execution.
type Trade struct {
	// ID is the trade identifier.
	ID string

	// Symbol is the trading pair identifier.
	Symbol Symbol

	// Price is the execution price.
	Price udecimal.Decimal

	// Qty is the executed quantity.
	Qty udecimal.Decimal

	// Side indicates if this was a buy or sell (taker side).
	Side Side

	// Timestamp is when the trade occurred.
	Timestamp time.Time

	// IsBuyerMaker indicates if the buyer was the maker.
	IsBuyerMaker bool
}

// Kline represents a normalized candlestick/OHLCV data point.
type Kline struct {
	// Symbol is the trading pair identifier.
	Symbol Symbol

	// Interval is the kline interval (e.g., "1m", "5m", "1h", "1d").
	Interval string

	// OpenTime is the start time of this candle.
	OpenTime time.Time

	// CloseTime is the end time of this candle.
	CloseTime time.Time

	// Open is the opening price.
	Open udecimal.Decimal

	// High is the highest price.
	High udecimal.Decimal

	// Low is the lowest price.
	Low udecimal.Decimal

	// Close is the closing price.
	Close udecimal.Decimal

	// Volume is the trading volume.
	Volume udecimal.Decimal

	// QuoteVolume is the quote asset volume.
	QuoteVolume udecimal.Decimal

	// TradeCount is the number of trades.
	TradeCount uint64
}
