// Package order provides normalized order types for trading.
package order

import (
	"fmt"
	"strings"
	"time"

	"github.com/pwnholic/clara/pkg/errors"
	"github.com/pwnholic/clara/pkg/market"
	"github.com/quagmt/udecimal"
)

// Type represents the order type.
type Type int

const (
	TypeLimit Type = iota
	TypeMarket
	TypeStopLoss
	TypeStopLossLimit
	TypeTakeProfit
	TypeTakeProfitLimit
	TypeTrailingStop
)

// String implements fmt.Stringer.
func (t Type) String() string {
	switch t {
	case TypeLimit:
		return "LIMIT"
	case TypeMarket:
		return "MARKET"
	case TypeStopLoss:
		return "STOP_LOSS"
	case TypeStopLossLimit:
		return "STOP_LOSS_LIMIT"
	case TypeTakeProfit:
		return "TAKE_PROFIT"
	case TypeTakeProfitLimit:
		return "TAKE_PROFIT_LIMIT"
	case TypeTrailingStop:
		return "TRAILING_STOP"
	default:
		return "UNKNOWN"
	}
}

// MarshalText implements encoding.TextMarshaler.
func (t Type) MarshalText() ([]byte, error) {
	return []byte(t.String()), nil
}

// UnmarshalText implements encoding.TextUnmarshaler.
func (t *Type) UnmarshalText(text []byte) error {
	switch strings.ToUpper(string(text)) {
	case "LIMIT":
		*t = TypeLimit
	case "MARKET":
		*t = TypeMarket
	case "STOP_LOSS", "STOP", "STOP_MARKET":
		*t = TypeStopLoss
	case "STOP_LOSS_LIMIT", "STOP_LIMIT":
		*t = TypeStopLossLimit
	case "TAKE_PROFIT", "TAKE_PROFIT_MARKET":
		*t = TypeTakeProfit
	case "TAKE_PROFIT_LIMIT":
		*t = TypeTakeProfitLimit
	case "TRAILING_STOP":
		*t = TypeTrailingStop
	default:
		return errors.NewValidationError("type", fmt.Sprintf("unknown order type: %s", string(text)))
	}
	return nil
}

// IsLimit returns true if this is a limit-type order.
func (t Type) IsLimit() bool {
	return t == TypeLimit || t == TypeStopLossLimit || t == TypeTakeProfitLimit
}

// IsMarket returns true if this is a market-type order.
func (t Type) IsMarket() bool {
	return t == TypeMarket || t == TypeStopLoss || t == TypeTakeProfit
}

// IsTrigger returns true if this is a trigger/conditional order.
func (t Type) IsTrigger() bool {
	return t == TypeStopLoss || t == TypeStopLossLimit || t == TypeTakeProfit || t == TypeTakeProfitLimit || t == TypeTrailingStop
}

// Status represents the order status.
type Status int

const (
	StatusNew Status = iota
	StatusPartiallyFilled
	StatusFilled
	StatusCancelled
	StatusRejected
	StatusExpired
	StatusPendingCancel
)

// String implements fmt.Stringer.
func (s Status) String() string {
	switch s {
	case StatusNew:
		return "NEW"
	case StatusPartiallyFilled:
		return "PARTIALLY_FILLED"
	case StatusFilled:
		return "FILLED"
	case StatusCancelled:
		return "CANCELLED"
	case StatusRejected:
		return "REJECTED"
	case StatusExpired:
		return "EXPIRED"
	case StatusPendingCancel:
		return "PENDING_CANCEL"
	default:
		return "UNKNOWN"
	}
}

// MarshalText implements encoding.TextMarshaler.
func (s Status) MarshalText() ([]byte, error) {
	return []byte(s.String()), nil
}

// UnmarshalText implements encoding.TextUnmarshaler.
func (s *Status) UnmarshalText(text []byte) error {
	switch strings.ToUpper(string(text)) {
	case "NEW", "PENDING":
		*s = StatusNew
	case "PARTIALLY_FILLED", "PARTIALLYFILLED":
		*s = StatusPartiallyFilled
	case "FILLED", "FILL":
		*s = StatusFilled
	case "CANCELLED", "CANCELED":
		*s = StatusCancelled
	case "REJECTED":
		*s = StatusRejected
	case "EXPIRED":
		*s = StatusExpired
	case "PENDING_CANCEL":
		*s = StatusPendingCancel
	default:
		return errors.NewValidationError("status", fmt.Sprintf("unknown order status: %s", string(text)))
	}
	return nil
}

// IsActive returns true if the order is still active.
func (s Status) IsActive() bool {
	return s == StatusNew || s == StatusPartiallyFilled || s == StatusPendingCancel
}

// IsTerminal returns true if the order is in a terminal state.
func (s Status) IsTerminal() bool {
	return s == StatusFilled || s == StatusCancelled || s == StatusRejected || s == StatusExpired
}

// TimeInForce represents the time in force for an order.
type TimeInForce int

const (
	GTC TimeInForce = iota // Good Till Cancel
	IOC                     // Immediate or Cancel
	FOK                     // Fill or Kill
	GTX                     // Good Till Crossing (Post Only)
)

// String implements fmt.Stringer.
func (t TimeInForce) String() string {
	switch t {
	case GTC:
		return "GTC"
	case IOC:
		return "IOC"
	case FOK:
		return "FOK"
	case GTX:
		return "GTX"
	default:
		return "UNKNOWN"
	}
}

// MarshalText implements encoding.TextMarshaler.
func (t TimeInForce) MarshalText() ([]byte, error) {
	return []byte(t.String()), nil
}

// UnmarshalText implements encoding.TextUnmarshaler.
func (t *TimeInForce) UnmarshalText(text []byte) error {
	switch strings.ToUpper(string(text)) {
	case "GTC":
		*t = GTC
	case "IOC":
		*t = IOC
	case "FOK":
		*t = FOK
	case "GTX", "POST_ONLY":
		*t = GTX
	default:
		return errors.NewValidationError("time_in_force", fmt.Sprintf("unknown time in force: %s", string(text)))
	}
	return nil
}

// Order represents a normalized order.
type Order struct {
	ID           string           `json:"id"`
	ClientID     string           `json:"client_id,omitempty"`
	Symbol       market.Symbol    `json:"symbol"`
	Side         market.Side      `json:"side"`
	Type         Type             `json:"type"`
	Status       Status           `json:"status"`
	Price        udecimal.Decimal `json:"price"`
	Quantity     udecimal.Decimal `json:"quantity"`
	ExecutedQty  udecimal.Decimal `json:"executed_qty"`
	AvgPrice     udecimal.Decimal `json:"avg_price"`
	StopPrice    udecimal.Decimal `json:"stop_price,omitempty"`
	TimeInForce  TimeInForce      `json:"time_in_force"`
	ReduceOnly   bool             `json:"reduce_only"`
	CreatedAt    time.Time        `json:"created_at"`
	UpdatedAt    time.Time        `json:"updated_at"`
}

// RemainingQty returns the remaining quantity to be filled.
func (o Order) RemainingQty() udecimal.Decimal {
	return o.Quantity.Sub(o.ExecutedQty)
}

// IsFilled returns true if the order is fully filled.
func (o Order) IsFilled() bool {
	return o.Status == StatusFilled
}

// IsOpen returns true if the order is still active.
func (o Order) IsOpen() bool {
	return o.Status.IsActive()
}

// IsCancelled returns true if the order is cancelled.
func (o Order) IsCancelled() bool {
	return o.Status == StatusCancelled
}

// FillPercent returns the percentage of the order that has been filled (0-1).
func (o Order) FillPercent() (udecimal.Decimal, error) {
	if o.Quantity.IsZero() {
		return udecimal.Decimal{}, errors.NewValidationError("quantity", "quantity is zero")
	}
	return o.ExecutedQty.Div(o.Quantity)
}

// IsFilledPercent returns true if at least pct% of the order is filled.
func (o Order) IsFilledPercent(pct udecimal.Decimal) (bool, error) {
	fillPct, err := o.FillPercent()
	if err != nil {
		return false, err
	}
	return fillPct.GreaterThanOrEqual(pct), nil
}

// Request represents a request to place a new order.
type Request struct {
	Symbol      market.Symbol    `json:"symbol"`
	Side        market.Side      `json:"side"`
	Type        Type             `json:"type"`
	Quantity    udecimal.Decimal `json:"quantity"`
	Price       udecimal.Decimal `json:"price,omitempty"`
	StopPrice   udecimal.Decimal `json:"stop_price,omitempty"`
	TimeInForce TimeInForce      `json:"time_in_force,omitempty"`
	ClientID    string           `json:"client_id,omitempty"`
	ReduceOnly  bool             `json:"reduce_only,omitempty"`
}

// Validate validates the order request.
func (r *Request) Validate() error {
	if !r.Symbol.IsValid() {
		return errors.NewValidationError("symbol", "symbol is required")
	}
	if r.Quantity.IsZero() {
		return errors.NewValidationError("quantity", "quantity is required and must be positive")
	}
	if r.Quantity.IsNeg() {
		return errors.NewValidationError("quantity", "quantity must be positive")
	}
	if r.Type.IsLimit() && r.Price.IsZero() {
		return errors.NewValidationError("price", "price is required for limit orders")
	}
	if r.Type.IsTrigger() && r.StopPrice.IsZero() {
		return errors.NewValidationError("stop_price", "stop price is required for trigger orders")
	}
	return nil
}

// CancelRequest represents a request to cancel an order.
type CancelRequest struct {
	Symbol   market.Symbol `json:"symbol"`
	OrderID  string        `json:"order_id,omitempty"`
	ClientID string        `json:"client_id,omitempty"`
}

// Validate validates the cancel request.
func (r *CancelRequest) Validate() error {
	if !r.Symbol.IsValid() {
		return errors.NewValidationError("symbol", "symbol is required")
	}
	if r.OrderID == "" && r.ClientID == "" {
		return errors.NewValidationError("order_id", "order_id or client_id is required")
	}
	return nil
}

// Balance represents the balance of a single asset.
type Balance struct {
	Asset  string           `json:"asset"`
	Free   udecimal.Decimal `json:"free"`
	Locked udecimal.Decimal `json:"locked"`
}

// Total returns the total balance (free + locked).
func (b Balance) Total() udecimal.Decimal {
	return b.Free.Add(b.Locked)
}
