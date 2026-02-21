// Package errors provides common error types for the clara trading SDK.
package errors

import (
	"errors"
	"fmt"
)

// Common errors that can be checked with errors.Is().
var (
	// ErrNotFound indicates the requested resource was not found.
	ErrNotFound = errors.New("resource not found")

	// ErrUnauthorized indicates authentication failed.
	ErrUnauthorized = errors.New("unauthorized")

	// ErrRateLimited indicates rate limit was exceeded.
	ErrRateLimited = errors.New("rate limit exceeded")

	// ErrTimeout indicates the operation timed out.
	ErrTimeout = errors.New("operation timed out")

	// ErrDisconnected indicates the connection was lost.
	ErrDisconnected = errors.New("disconnected")

	// ErrCancelled indicates the operation was cancelled.
	ErrCancelled = errors.New("operation cancelled")

	// ErrInvalidSymbol indicates an invalid trading symbol.
	ErrInvalidSymbol = errors.New("invalid symbol")

	// ErrInvalidOrder indicates an invalid order request.
	ErrInvalidOrder = errors.New("invalid order")

	// ErrInsufficientBalance indicates insufficient balance.
	ErrInsufficientBalance = errors.New("insufficient balance")

	// ErrOrderNotFound indicates the order was not found.
	ErrOrderNotFound = errors.New("order not found")

	// ErrOrderNotActive indicates the order is not active.
	ErrOrderNotActive = errors.New("order not active")
)

// ExchangeError represents an error returned by an exchange API.
type ExchangeError struct {
	Provider string // Exchange provider name
	Code     int    // Exchange-specific error code
	Message  string // Error message from exchange
	Err      error  // Underlying error
}

func (e *ExchangeError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("[%s] code=%d: %s: %v", e.Provider, e.Code, e.Message, e.Err)
	}
	return fmt.Sprintf("[%s] code=%d: %s", e.Provider, e.Code, e.Message)
}

func (e *ExchangeError) Unwrap() error {
	return e.Err
}

// NewExchangeError creates a new ExchangeError.
func NewExchangeError(provider string, code int, message string, err error) *ExchangeError {
	return &ExchangeError{
		Provider: provider,
		Code:     code,
		Message:  message,
		Err:      err,
	}
}

// ValidationError represents a validation error.
type ValidationError struct {
	Field   string // Field that failed validation
	Message string // Validation message
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("validation error: %s: %s", e.Field, e.Message)
}

// NewValidationError creates a new ValidationError.
func NewValidationError(field, message string) *ValidationError {
	return &ValidationError{
		Field:   field,
		Message: message,
	}
}

// StreamError represents an error from a data stream.
type StreamError struct {
	Provider string // Exchange provider name
	Stream   string // Stream name (e.g., "ticker", "orderbook")
	Message  string // Error message
	Err      error  // Underlying error
}

func (e *StreamError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("[%s] stream %s: %s: %v", e.Provider, e.Stream, e.Message, e.Err)
	}
	return fmt.Sprintf("[%s] stream %s: %s", e.Provider, e.Stream, e.Message)
}

func (e *StreamError) Unwrap() error {
	return e.Err
}

// NewStreamError creates a new StreamError.
func NewStreamError(provider, stream, message string, err error) *StreamError {
	return &StreamError{
		Provider: provider,
		Stream:   stream,
		Message:  message,
		Err:      err,
	}
}
