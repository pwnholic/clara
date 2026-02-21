// Package exchange provides the exchange client interface and registry
// for the clara trading SDK.
package exchange

import (
	"context"
	"fmt"

	"github.com/pwnholic/clara/pkg/market"
	"github.com/pwnholic/clara/pkg/stream"
)

// ProviderName identifies a specific exchange provider.
type ProviderName string

const (
	// ProviderBinance is the Binance exchange provider.
	ProviderBinance ProviderName = "binance"

	// ProviderBybit is the Bybit exchange provider.
	ProviderBybit ProviderName = "bybit"
)

// String returns the string representation of the provider name.
func (p ProviderName) String() string {
	return string(p)
}

// ClientConfig holds configuration for creating an exchange client.
type ClientConfig struct {
	// Provider is the exchange provider to use (required).
	Provider ProviderName

	// APIKey is the API key for authenticated requests.
	APIKey string

	// APISecret is the API secret for signing requests.
	APISecret string

	// Passphrase is required by some exchanges (e.g., OKX).
	Passphrase string

	// Testnet enables testnet mode if supported.
	Testnet bool

	// Timeout is the request timeout in milliseconds.
	// Default: 10000 (10 seconds)
	Timeout int

	// StreamConfig is the configuration for stream connections.
	StreamConfig stream.StreamConfig

	// Debug enables debug logging.
	Debug bool
}

// Validate validates the client configuration.
func (c ClientConfig) Validate() error {
	if c.Provider == "" {
		return fmt.Errorf("provider is required")
	}

	// Validate provider is registered
	if !IsProviderRegistered(c.Provider) {
		return fmt.Errorf("provider %q is not registered", c.Provider)
	}

	if c.Timeout < 0 {
		return fmt.Errorf("timeout must be non-negative, got %d", c.Timeout)
	}

	return c.StreamConfig.Validate()
}

// ExchangeClient is the primary interface for interacting with exchanges.
// All methods return normalized types from the market package.
//
// Usage:
//
//	cfg := exchange.ClientConfig{
//	    Provider:  exchange.ProviderBinance,
//	    APIKey:    "your-api-key",
//	    APISecret: "your-api-secret",
//	}
//	client, err := exchange.NewClient(cfg)
//	if err != nil {
//	    return err
//	}
//	defer client.Close()
type ExchangeClient interface {
	// Provider returns the name of this exchange provider.
	Provider() ProviderName

	// Connect establishes connections to the exchange.
	// Must be called before using stream methods.
	Connect(ctx context.Context) error

	// Close closes all connections and releases resources.
	Close() error

	// --- Market Data Streams ---

	// TickerStream returns a stream of ticker updates for the given symbol.
	TickerStream(symbol market.Symbol) stream.Stream[market.Ticker]

	// OrderBookStream returns a stream of order book updates.
	// Depth specifies the number of price levels (0 = full depth).
	OrderBookStream(symbol market.Symbol, depth int) stream.Stream[market.OrderBook]

	// TradeStream returns a stream of public trades.
	TradeStream(symbol market.Symbol) stream.Stream[market.Trade]

	// KlineStream returns a stream of kline/candlestick updates.
	// Interval specifies the kline interval (e.g., "1m", "5m", "1h", "1d").
	KlineStream(symbol market.Symbol, interval string) stream.Stream[market.Kline]

	// --- REST API Methods ---

	// GetTicker fetches the current ticker for a symbol.
	GetTicker(ctx context.Context, symbol market.Symbol) (*market.Ticker, error)

	// GetOrderBook fetches the current order book snapshot.
	GetOrderBook(ctx context.Context, symbol market.Symbol, depth int) (*market.OrderBook, error)

	// GetTrades fetches recent public trades.
	GetTrades(ctx context.Context, symbol market.Symbol, limit int) ([]market.Trade, error)

	// GetKlines fetches historical kline data.
	GetKlines(ctx context.Context, symbol market.Symbol, interval string, limit int) ([]market.Kline, error)
}

// ProviderFactory is a function that creates an ExchangeClient for a specific provider.
type ProviderFactory func(cfg ClientConfig) (ExchangeClient, error)

// registry holds registered provider factories.
var registry = make(map[ProviderName]ProviderFactory)

// RegisterProvider registers a provider factory.
// Panics if the provider is already registered.
// This should be called in init() of provider packages.
func RegisterProvider(name ProviderName, factory ProviderFactory) {
	if _, exists := registry[name]; exists {
		panic(fmt.Sprintf("provider %q already registered", name))
	}
	registry[name] = factory
}

// IsProviderRegistered returns true if the provider is registered.
func IsProviderRegistered(name ProviderName) bool {
	_, exists := registry[name]
	return exists
}

// RegisteredProviders returns a list of all registered provider names.
func RegisteredProviders() []ProviderName {
	providers := make([]ProviderName, 0, len(registry))
	for name := range registry {
		providers = append(providers, name)
	}
	return providers
}

// NewClient creates a new ExchangeClient for the given configuration.
// The provider must be registered before calling this function.
func NewClient(cfg ClientConfig) (ExchangeClient, error) {
	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("invalid config: %w", err)
	}

	factory, exists := registry[cfg.Provider]
	if !exists {
		return nil, fmt.Errorf("provider %q is not registered", cfg.Provider)
	}

	return factory(cfg)
}
