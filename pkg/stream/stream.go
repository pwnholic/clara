// Package stream provides the core stream abstraction for the clara trading SDK.
// Streams are the primary interface for consuming market data, regardless of
// whether the underlying transport is WebSocket or REST polling.
package stream

import (
	"context"
	"sync"
	"sync/atomic"
	"time"

	"github.com/pwnholic/clara/pkg/errors"
)

// Stream is the primary interface for consuming typed data streams.
// Implementations may use WebSocket, REST polling, or other transports.
// All streams must be safe for concurrent use.
//
// Type parameter T represents the data type emitted by this stream.
type Stream[T any] interface {
	// Subscribe starts the stream and returns a channel for receiving data.
	// The returned channel is closed when Unsubscribe is called or context is cancelled.
	// Returns ErrAlreadySubscribed if already active.
	Subscribe(ctx context.Context) (<-chan T, error)

	// Unsubscribe stops the stream and closes the data channel.
	// Returns ErrNotSubscribed if not currently subscribed.
	Unsubscribe(ctx context.Context) error

	// Errors returns a channel for receiving non-fatal stream errors.
	// The channel is closed when the stream is unsubscribed.
	Errors() <-chan error

	// Done returns a channel that is closed when the stream is fully stopped.
	Done() <-chan struct{}

	// State returns the current stream state.
	State() State
}

// State represents the current state of a stream.
type State int32

const (
	StateIdle State = iota
	StateConnecting
	StateActive
	StateReconnecting
	StateClosing
	StateClosed
)

// String implements fmt.Stringer.
func (s State) String() string {
	switch s {
	case StateIdle:
		return "idle"
	case StateConnecting:
		return "connecting"
	case StateActive:
		return "active"
	case StateReconnecting:
		return "reconnecting"
	case StateClosing:
		return "closing"
	case StateClosed:
		return "closed"
	default:
		return "unknown"
	}
}

// Config holds common configuration for streams.
type Config struct {
	// BufferSize is the channel buffer size for emitted data.
	BufferSize int

	// Reconnect enables automatic reconnection on disconnect.
	Reconnect bool

	// MaxReconnectAttempts is the maximum reconnection attempts (0 = unlimited).
	MaxReconnectAttempts int

	// ReconnectBaseDelay is the initial delay between reconnection attempts.
	// Delay increases exponentially with each attempt.
	ReconnectBaseDelay time.Duration

	// ReconnectMaxDelay is the maximum delay between reconnection attempts.
	ReconnectMaxDelay time.Duration

	// PingInterval is the interval for sending keepalive pings.
	PingInterval time.Duration

	// PongTimeout is the timeout for receiving pong responses.
	PongTimeout time.Duration
}

// DefaultConfig returns a Config with sensible defaults.
func DefaultConfig() Config {
	return Config{
		BufferSize:           100,
		Reconnect:            true,
		MaxReconnectAttempts: 10,
		ReconnectBaseDelay:   time.Second,
		ReconnectMaxDelay:    30 * time.Second,
		PingInterval:         30 * time.Second,
		PongTimeout:          10 * time.Second,
	}
}

// Validate validates the configuration.
func (c Config) Validate() error {
	if c.BufferSize < 0 {
		return errors.NewValidationError("buffer_size", "must be non-negative")
	}
	if c.MaxReconnectAttempts < 0 {
		return errors.NewValidationError("max_reconnect_attempts", "must be non-negative")
	}
	if c.ReconnectBaseDelay < 0 {
		return errors.NewValidationError("reconnect_base_delay", "must be non-negative")
	}
	if c.ReconnectMaxDelay < c.ReconnectBaseDelay {
		return errors.NewValidationError("reconnect_max_delay", "must be >= reconnect_base_delay")
	}
	return nil
}

// BaseStream provides common functionality for stream implementations.
// Embed this in your stream implementations to get basic state management.
type BaseStream[T any] struct {
	mu       sync.RWMutex
	state    atomic.Int32
	config   Config
	dataCh   chan T
	errorCh  chan error
	doneCh   chan struct{}
	cancel   context.CancelFunc
}

// NewBaseStream creates a new BaseStream with the given configuration.
func NewBaseStream[T any](cfg Config) *BaseStream[T] {
	return &BaseStream[T]{
		config:  cfg,
		doneCh:  make(chan struct{}),
		errorCh: make(chan error, 10),
	}
}

// State returns the current stream state.
func (s *BaseStream[T]) State() State {
	return State(s.state.Load())
}

// setState updates the stream state atomically.
func (s *BaseStream[T]) setState(state State) {
	s.state.Store(int32(state))
}

// compareAndSwapState atomically compares and swaps the state.
func (s *BaseStream[T]) compareAndSwapState(old, new State) bool {
	return s.state.CompareAndSwap(int32(old), int32(new))
}

// DataChannel returns the data channel, creating it if necessary.
func (s *BaseStream[T]) DataChannel() chan T {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.dataCh == nil {
		bufSize := s.config.BufferSize
		if bufSize <= 0 {
			bufSize = 100
		}
		s.dataCh = make(chan T, bufSize)
	}
	return s.dataCh
}

// ErrorChannel returns the error channel.
func (s *BaseStream[T]) ErrorChannel() chan error {
	return s.errorCh
}

// Done returns the done channel.
func (s *BaseStream[T]) Done() <-chan struct{} {
	return s.doneCh
}

// Errors returns the error channel.
func (s *BaseStream[T]) Errors() <-chan error {
	return s.errorCh
}

// Emit sends data to the data channel. Non-blocking.
// Returns true if sent, false if channel full or closed.
func (s *BaseStream[T]) Emit(data T) bool {
	select {
	case s.DataChannel() <- data:
		return true
	default:
		return false
	}
}

// EmitError sends an error to the error channel. Non-blocking.
func (s *BaseStream[T]) EmitError(err error) {
	select {
	case s.errorCh <- err:
	default:
		// Error channel full, drop the error
	}
}

// Start begins the stream with the given run function.
// The run function should block until context is cancelled.
func (s *BaseStream[T]) Start(ctx context.Context, run func(ctx context.Context) error) error {
	if !s.compareAndSwapState(StateIdle, StateConnecting) {
		return errors.ErrCancelled // Already running
	}

	ctx, s.cancel = context.WithCancel(ctx)

	// Close channels when done
	go func() {
		<-ctx.Done()
		s.mu.Lock()
		if s.dataCh != nil {
			close(s.dataCh)
			s.dataCh = nil
		}
		s.mu.Unlock()
		close(s.errorCh)
		close(s.doneCh)
		s.setState(StateClosed)
	}()

	// Run the stream
	go func() {
		s.setState(StateActive)
		if err := run(ctx); err != nil && ctx.Err() == nil {
			s.EmitError(err)
		}
	}()

	return nil
}

// Stop stops the stream.
func (s *BaseStream[T]) Stop() error {
	if s.State() == StateClosed || s.State() == StateIdle {
		return nil
	}

	s.setState(StateClosing)
	if s.cancel != nil {
		s.cancel()
	}
	return nil
}

// Config returns the stream configuration.
func (s *BaseStream[T]) Config() Config {
	return s.config
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
