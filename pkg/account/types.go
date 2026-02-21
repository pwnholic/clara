// Package account provides normalized account types for trading.
package account

import (
	"fmt"

	"github.com/pwnholic/clara/pkg/market"
	"github.com/quagmt/udecimal"
)

// Balance represents the balance of a single asset.
type Balance struct {
	// Asset is the asset symbol (e.g., "BTC", "USDT").
	Asset string

	// Free is the available balance for trading.
	Free udecimal.Decimal

	// Locked is the balance locked in open orders.
	Locked udecimal.Decimal

	// Total returns the total balance (Free + Locked).
	Total udecimal.Decimal
}

// AccountInfo represents the account information.
type AccountInfo struct {
	// Balances is the list of asset balances.
	Balances []Balance

	// MarginLevel is the margin level for futures accounts.
	MarginLevel udecimal.Decimal

	// TotalAssetValue is the total account value in quote currency.
	TotalAssetValue udecimal.Decimal
}

// GetBalance returns the balance for a specific asset.
func (a AccountInfo) GetBalance(asset string) (*Balance, error) {
	for i := range a.Balances {
		if a.Balances[i].Asset == asset {
			return &a.Balances[i], nil
		}
	}
	return nil, fmt.Errorf("balance not found for asset: %s", asset)
}

// PositionSide represents the position side for futures.
type PositionSide int

const (
	PositionSideBoth PositionSide = iota // One-way mode
	PositionSideLong                     // Hedge mode - long
	PositionSideShort                    // Hedge mode - short
)

// String returns the string representation of the position side.
func (p PositionSide) String() string {
	switch p {
	case PositionSideBoth:
		return "both"
	case PositionSideLong:
		return "long"
	case PositionSideShort:
		return "short"
	default:
		return "unknown"
	}
}

// Position represents a futures position.
type Position struct {
	// Symbol is the trading pair.
	Symbol market.Symbol

	// Side is the position side (long/short/both).
	Side PositionSide

	// Quantity is the position quantity (positive for long, negative for short).
	Quantity udecimal.Decimal

	// EntryPrice is the average entry price.
	EntryPrice udecimal.Decimal

	// MarkPrice is the current mark price.
	MarkPrice udecimal.Decimal

	// UnrealizedPnL is the unrealized profit/loss.
	UnrealizedPnL udecimal.Decimal

	// RealizedPnL is the realized profit/loss.
	RealizedPnL udecimal.Decimal

	// Leverage is the current leverage.
	Leverage udecimal.Decimal

	// LiquidationPrice is the estimated liquidation price.
	LiquidationPrice udecimal.Decimal

	// Margin is the position margin.
	Margin udecimal.Decimal

	// MarginMode is the margin mode (cross/isolated).
	MarginMode string
}

// IsLong returns true if this is a long position.
func (p Position) IsLong() bool {
	return p.Quantity.IsPos() || p.Side == PositionSideLong
}

// IsShort returns true if this is a short position.
func (p Position) IsShort() bool {
	return p.Quantity.IsNeg() || p.Side == PositionSideShort
}

// IsOpen returns true if the position has non-zero quantity.
func (p Position) IsOpen() bool {
	return !p.Quantity.IsZero()
}

// ROE (Return on Equity) calculates the return on equity percentage.
func (p Position) ROE() (udecimal.Decimal, error) {
	if p.Margin.IsZero() {
		return udecimal.Decimal{}, fmt.Errorf("margin is zero")
	}
	return p.UnrealizedPnL.Div(p.Margin)
}

// Leverage represents leverage settings for a symbol.
type Leverage struct {
	// Symbol is the trading pair.
	Symbol market.Symbol

	// Leverage is the current leverage multiplier.
	Leverage udecimal.Decimal

	// MaxLeverage is the maximum allowed leverage.
	MaxLeverage udecimal.Decimal
}
