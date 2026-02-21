// Package account provides normalized account and position types for trading.
package account

import (
	"fmt"
	"time"

	"github.com/pwnholic/clara/pkg/errors"
	"github.com/pwnholic/clara/pkg/market"
	"github.com/pwnholic/clara/pkg/order"
	"github.com/quagmt/udecimal"
)

// Info represents the account information.
type Info struct {
	Balances       []order.Balance   `json:"balances"`
	MarginLevel    udecimal.Decimal  `json:"margin_level,omitempty"`
	TotalValue     udecimal.Decimal  `json:"total_value"`
	UpdateTime     time.Time         `json:"update_time"`
}

// GetBalance returns the balance for a specific asset.
func (a *Info) GetBalance(asset string) (*order.Balance, error) {
	for i := range a.Balances {
		if a.Balances[i].Asset == asset {
			return &a.Balances[i], nil
		}
	}
	return nil, fmt.Errorf("%w: asset %s", errors.ErrNotFound, asset)
}

// HasBalance returns true if the account has a non-zero balance for the asset.
func (a *Info) HasBalance(asset string) bool {
	b, err := a.GetBalance(asset)
	if err != nil {
		return false
	}
	return !b.Total().IsZero()
}

// PositionSide represents the position side for futures.
type PositionSide int

const (
	PositionSideBoth PositionSide = iota // One-way mode
	PositionSideLong                      // Hedge mode - long
	PositionSideShort                     // Hedge mode - short
)

// String implements fmt.Stringer.
func (p PositionSide) String() string {
	switch p {
	case PositionSideBoth:
		return "BOTH"
	case PositionSideLong:
		return "LONG"
	case PositionSideShort:
		return "SHORT"
	default:
		return "UNKNOWN"
	}
}

// MarginMode represents the margin mode for futures.
type MarginMode int

const (
	MarginModeCross MarginMode = iota
	MarginModeIsolated
)

// String implements fmt.Stringer.
func (m MarginMode) String() string {
	switch m {
	case MarginModeCross:
		return "CROSS"
	case MarginModeIsolated:
		return "ISOLATED"
	default:
		return "UNKNOWN"
	}
}

// Position represents a futures position.
type Position struct {
	Symbol            market.Symbol    `json:"symbol"`
	Side              PositionSide     `json:"side"`
	Quantity          udecimal.Decimal `json:"quantity"`
	EntryPrice        udecimal.Decimal `json:"entry_price"`
	MarkPrice         udecimal.Decimal `json:"mark_price"`
	UnrealizedPnL     udecimal.Decimal `json:"unrealized_pnl"`
	RealizedPnL       udecimal.Decimal `json:"realized_pnl"`
	Leverage          udecimal.Decimal `json:"leverage"`
	LiquidationPrice  udecimal.Decimal `json:"liquidation_price"`
	Margin            udecimal.Decimal `json:"margin"`
	MarginMode        MarginMode       `json:"margin_mode"`
	UpdateTime        time.Time        `json:"update_time"`
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

// IsClosed returns true if the position is closed.
func (p Position) IsClosed() bool {
	return p.Quantity.IsZero()
}

// AbsQty returns the absolute quantity.
func (p Position) AbsQty() udecimal.Decimal {
	if p.Quantity.IsNeg() {
		return p.Quantity.Neg()
	}
	return p.Quantity
}

// Value returns the notional value of the position.
func (p Position) Value() udecimal.Decimal {
	return p.AbsQty().Mul(p.MarkPrice)
}

// EntryValue returns the entry value of the position.
func (p Position) EntryValue() udecimal.Decimal {
	return p.AbsQty().Mul(p.EntryPrice)
}

// ROE (Return on Equity) calculates the return on equity percentage (0-1).
func (p Position) ROE() (udecimal.Decimal, error) {
	if p.Margin.IsZero() {
		return udecimal.Decimal{}, errors.NewValidationError("margin", "margin is zero")
	}
	return p.UnrealizedPnL.Div(p.Margin)
}

// PnLPercent returns the PnL as a percentage of entry value.
func (p Position) PnLPercent() (udecimal.Decimal, error) {
	entryValue := p.EntryValue()
	if entryValue.IsZero() {
		return udecimal.Decimal{}, errors.NewValidationError("entry_value", "entry value is zero")
	}
	return p.UnrealizedPnL.Div(entryValue)
}

// LeverageSetting represents leverage settings for a symbol.
type LeverageSetting struct {
	Symbol      market.Symbol    `json:"symbol"`
	Leverage    udecimal.Decimal `json:"leverage"`
	MaxLeverage udecimal.Decimal `json:"max_leverage"`
}

// IsMaxed returns true if leverage is at maximum.
func (l LeverageSetting) IsMaxed() bool {
	return l.Leverage.GreaterThanOrEqual(l.MaxLeverage)
}
