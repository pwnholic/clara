// Package order provides normalized order types for trading.
package order

import (
	"fmt"
	"time"

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

// String returns the string representation of the order type.
func (t Type) String() string {
	switch t {
	case TypeLimit:
		return "limit"
	case TypeMarket:
		return "market"
	case TypeStopLoss:
		return "stop_loss"
	case TypeStopLossLimit:
		return "stop_loss_limit"
	case TypeTakeProfit:
		return "take_profit"
	case TypeTakeProfitLimit:
		return "take_profit_limit"
	case TypeTrailingStop:
		return "trailing_stop"
	default:
		return "unknown"
	}
}

// ParseType parses a string into a Type.
func ParseType(s string) (Type, error) {
	switch s {
	case "limit", "LIMIT", "Limit":
		return TypeLimit, nil
	case "market", "MARKET", "Market":
		return TypeMarket, nil
	case "stop_loss", "STOP_LOSS":
		return TypeStopLoss, nil
	case "stop_loss_limit", "STOP_LOSS_LIMIT":
		return TypeStopLossLimit, nil
	case "take_profit", "TAKE_PROFIT":
		return TypeTakeProfit, nil
	case "take_profit_limit", "TAKE_PROFIT_LIMIT":
		return TypeTakeProfitLimit, nil
	case "trailing_stop", "TRAILING_STOP":
		return TypeTrailingStop, nil
	default:
		return TypeLimit, fmt.Errorf("invalid order type: %s", s)
	}
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
)

// String returns the string representation of the order status.
func (s Status) String() string {
	switch s {
	case StatusNew:
		return "new"
	case StatusPartiallyFilled:
		return "partially_filled"
	case StatusFilled:
		return "filled"
	case StatusCancelled:
		return "cancelled"
	case StatusRejected:
		return "rejected"
	case StatusExpired:
		return "expired"
	default:
		return "unknown"
	}
}

// ParseStatus parses a string into a Status.
func ParseStatus(s string) (Status, error) {
	switch s {
	case "new", "NEW", "New":
		return StatusNew, nil
	case "partially_filled", "PARTIALLY_FILLED", "PartiallyFilled":
		return StatusPartiallyFilled, nil
	case "filled", "FILLED", "Filled":
		return StatusFilled, nil
	case "cancelled", "CANCELED", "CANCELLED", "Cancelled":
		return StatusCancelled, nil
	case "rejected", "REJECTED", "Rejected":
		return StatusRejected, nil
	case "expired", "EXPIRED", "Expired":
		return StatusExpired, nil
	default:
		return StatusNew, fmt.Errorf("invalid order status: %s", s)
	}
}

// TimeInForce represents the time in force for an order.
type TimeInForce int

const (
	TIFGoodTillCancel TimeInForce = iota // GTC
	TIFImmediateOrCancel                 // IOC
	TIFillOrKill                         // FOK
	TIFGoodTillCrossing                  // GTX (Post Only)
)

// String returns the string representation of the time in force.
func (t TimeInForce) String() string {
	switch t {
	case TIFGoodTillCancel:
		return "GTC"
	case TIFImmediateOrCancel:
		return "IOC"
	case TIFillOrKill:
		return "FOK"
	case TIFGoodTillCrossing:
		return "GTX"
	default:
		return "UNKNOWN"
	}
}

// Order represents a normalized order.
type Order struct {
	// ID is the exchange-assigned order ID.
	ID string

	// ClientID is the client-specified order ID (optional).
	ClientID string

	// Symbol is the trading pair.
	Symbol market.Symbol

	// Side is the order side (buy/sell).
	Side market.Side

	// Type is the order type.
	Type Type

	// Status is the current order status.
	Status Status

	// Price is the limit price (for limit orders).
	Price udecimal.Decimal

	// Quantity is the original order quantity.
	Quantity udecimal.Decimal

	// ExecutedQty is the quantity that has been filled.
	ExecutedQty udecimal.Decimal

	// RemainingQty is the quantity remaining to be filled.
	RemainingQty udecimal.Decimal

	// AvgPrice is the average execution price.
	AvgPrice udecimal.Decimal

	// StopPrice is the trigger price for stop orders.
	StopPrice udecimal.Decimal

	// TimeInForce is the time in force policy.
	TimeInForce TimeInForce

	// CreatedAt is when the order was created.
	CreatedAt time.Time

	// UpdatedAt is when the order was last updated.
	UpdatedAt time.Time
}

// IsFilled returns true if the order is fully filled.
func (o Order) IsFilled() bool {
	return o.Status == StatusFilled
}

// IsOpen returns true if the order is still active.
func (o Order) IsOpen() bool {
	return o.Status == StatusNew || o.Status == StatusPartiallyFilled
}

// IsCancelled returns true if the order is cancelled.
func (o Order) IsCancelled() bool {
	return o.Status == StatusCancelled
}

// FillPercent returns the percentage of the order that has been filled.
func (o Order) FillPercent() (udecimal.Decimal, error) {
	if o.Quantity.IsZero() {
		return udecimal.Decimal{}, fmt.Errorf("quantity is zero")
	}
	return o.ExecutedQty.Div(o.Quantity)
}

// OrderRequest represents a request to place a new order.
type OrderRequest struct {
	// Symbol is the trading pair (required).
	Symbol market.Symbol

	// Side is the order side (required).
	Side market.Side

	// Type is the order type (required).
	Type Type

	// Quantity is the order quantity (required).
	Quantity udecimal.Decimal

	// Price is the limit price (required for limit orders).
	Price udecimal.Decimal

	// StopPrice is the trigger price (required for stop orders).
	StopPrice udecimal.Decimal

	// TimeInForce is the time in force policy.
	TimeInForce TimeInForce

	// ClientID is a client-specified order ID (optional).
	ClientID string

	// ReduceOnly marks the order as reduce-only for futures.
	ReduceOnly bool
}

// Validate validates the order request.
func (r OrderRequest) Validate() error {
	if r.Symbol == "" {
		return fmt.Errorf("symbol is required")
	}
	if r.Quantity.IsZero() {
		return fmt.Errorf("quantity is required")
	}
	if r.Type == TypeLimit && r.Price.IsZero() {
		return fmt.Errorf("price is required for limit orders")
	}
	return nil
}

// CancelRequest represents a request to cancel an order.
type CancelRequest struct {
	// Symbol is the trading pair (required).
	Symbol market.Symbol

	// OrderID is the exchange-assigned order ID.
	OrderID string

	// ClientID is the client-specified order ID.
	ClientID string
}

// Validate validates the cancel request.
func (r CancelRequest) Validate() error {
	if r.Symbol == "" {
		return fmt.Errorf("symbol is required")
	}
	if r.OrderID == "" && r.ClientID == "" {
		return fmt.Errorf("order_id or client_id is required")
	}
	return nil
}
