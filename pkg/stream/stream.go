// Package stream provides the core stream abstraction for the clara trading SDK.
// Streams are the primary interface for consuming market data, regardless of
// whether the underlying transport is WebSocket or REST polling.
package stream

import (
	"context"
	"errors"
	"fmt"
)

// Common errors returned by stream operations.
var (
	// ErrStreamClosed is returned when attempting to use a closed stream.
	ErrStreamClosed = errors.New("stream is closed")

	// ErrAlreadySubscribed is returned when subscribing to an already active stream.
	ErrAlreadySubscribed = errors.New("already subscribed")

	// ErrNotSubscribed is returned when unsubscribing from an inactive stream.
	ErrNotSubscribed = errors.New("not subscribed")

	// ErrSubscriptionCancelled is returned when subscription is cancelled.
	ErrSubscriptionCancelled = errors.New("subscription cancelled")
)

// Stream is the primary interface for consuming typed data streams.
// Implementations may use WebSocket, REST polling, or other transports.
// All streams must be safe for concurrent use.
//
// Type parameter T represents the data type emitted by this stream
// (e.g., market.Ticker, market.OrderBook, market.Trade).
//
// Example usage:
//
//	stream := client.TickerStream("BTCUSDT")
//	ch, err := stream.Subscribe(ctx)
//	if err != nil {
//	    return err
//	}
//	defer stream.Unsubscribe(ctx)
//
//	for ticker := range ch {
//	    fmt.Printf("Price: %s\n", ticker.LastPrice)
//	}
type Stream[T any] interface {
	// Subscribe starts the stream and returns a channel for receiving data.
	// The channel is closed when the stream is unsubscribed or encounters an error.
	// Returns ErrAlreadySubscribed if already active.
	Subscribe(ctx context.Context) (<-chan T, error)

	// Unsubscribe stops the stream and closes the data channel.
	// Returns ErrNotSubscribed if not currently subscribed.
	Unsubscribe(ctx context.Context) error

	// Errors returns a channel for receiving stream errors.
	// Errors are non-fatal; the stream continues running after emitting an error.
	// The channel is closed when the stream is unsubscribed.
	Errors() <-chan error

	// Done returns a channel that is closed when the stream is fully stopped.
	Done() <-chan struct{}
}

// Subscription represents an active stream subscription.
type Subscription interface {
	// ID returns the unique subscription identifier.
	ID() string

	// Cancel cancels the subscription.
	Cancel(ctx context.Context) error

	// Active returns true if the subscription is still active.
	Active() bool
}

// StreamConfig holds common configuration for streams.
type StreamConfig struct {
	// BufferSize is the channel buffer size for emitted data.
	// Default: 100
	BufferSize int

	// Reconnect enables automatic reconnection on disconnect.
	// Default: true
	Reconnect bool

	// MaxReconnectAttempts is the maximum number of reconnection attempts.
	// 0 = unlimited. Default: 10
	MaxReconnectAttempts int

	// ReconnectDelay is the initial delay between reconnection attempts.
	// Delay increases exponentially. Default: 1s
	ReconnectDelay string
}

// DefaultStreamConfig returns a StreamConfig with sensible defaults.
func DefaultStreamConfig() StreamConfig {
	return StreamConfig{
		BufferSize:           100,
		Reconnect:            true,
		MaxReconnectAttempts: 10,
		ReconnectDelay:       "1s",
	}
}

// Validate validates the stream configuration.
func (c StreamConfig) Validate() error {
	if c.BufferSize < 0 {
		return fmt.Errorf("buffer size must be non-negative, got %d", c.BufferSize)
	}
	if c.MaxReconnectAttempts < 0 {
		return fmt.Errorf("max reconnect attempts must be non-negative, got %d", c.MaxReconnectAttempts)
	}
	return nil
}

// StreamState represents the current state of a stream.
type StreamState int

const (
	// StateIdle indicates the stream has not been started.
	StateIdle StreamState = iota

	// StateConnecting indicates the stream is establishing a connection.
	StateConnecting

	// StateActive indicates the stream is actively emitting data.
	StateActive

	// StateReconnecting indicates the stream is attempting to reconnect.
	StateReconnecting

	// StateClosed indicates the stream has been closed.
	StateClosed
)

// String returns the string representation of the stream state.
func (s StreamState) String() string {
	switch s {
	case StateIdle:
		return "idle"
	case StateConnecting:
		return "connecting"
	case StateActive:
		return "active"
	case StateReconnecting:
		return "reconnecting"
	case StateClosed:
		return "closed"
	default:
		return "unknown"
	}
}
