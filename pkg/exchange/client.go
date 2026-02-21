// Package exchange provides the exchange client interface and registry
// for the clara trading SDK.
package exchange

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/pwnholic/clara/pkg/errors"
	"github.com/pwnholic/clara/pkg/market"
	"github.com/pwnholic/clara/pkg/order"
	"github.com/pwnholic/clara/pkg/stream"
)

// Provider identifies a specific exchange provider.
type Provider string

const (
	ProviderBinance Provider = "binance"
	ProviderBybit   Provider = "bybit"
)

// String implements fmt.Stringer.
func (p Provider) String() string {
	return string(p)
}

// IsValid returns true if the provider is valid.
func (p Provider) IsValid() bool {
	switch p {
	case ProviderBinance, ProviderBybit:
		return true
	default:
		return false
	}
}

// Option is a functional option for configuring the client.
type Option func(*Options)

// Options holds all configuration options for the exchange client.
type Options struct {
	// Authentication
	APIKey    string
	APISecret string
	Passphrase string // Required for some exchanges (e.g., OKX)

	// Environment
	Testnet bool

	// HTTP settings
	HTTPClient *http.Client
	Timeout    time.Duration
	RetryCount int
	RetryDelay time.Duration

	// Stream settings
	StreamConfig stream.Config

	// Debug
	Debug bool
	Logger interface{ Debug(msg string, fields ...interface{}) }
}

// DefaultOptions returns Options with sensible defaults.
func DefaultOptions() Options {
	return Options{
		Timeout:      30 * time.Second,
		RetryCount:   3,
		RetryDelay:   time.Second,
		StreamConfig: stream.DefaultConfig(),
	}
}

// WithAPIKey sets the API key.
func WithAPIKey(key string) Option {
	return func(o *Options) {
		o.APIKey = key
	}
}

// WithAPISecret sets the API secret.
func WithAPISecret(secret string) Option {
	return func(o *Options) {
		o.APISecret = secret
	}
}

// WithPassphrase sets the passphrase (for exchanges that require it).
func WithPassphrase(passphrase string) Option {
	return func(o *Options) {
		o.Passphrase = passphrase
	}
}

// WithTestnet enables testnet mode.
func WithTestnet() Option {
	return func(o *Options) {
		o.Testnet = true
	}
}

// WithHTTPClient sets a custom HTTP client.
func WithHTTPClient(client *http.Client) Option {
	return func(o *Options) {
		o.HTTPClient = client
	}
}

// WithTimeout sets the request timeout.
func WithTimeout(timeout time.Duration) Option {
	return func(o *Options) {
		o.Timeout = timeout
	}
}

// WithRetry sets the retry configuration.
func WithRetry(count int, delay time.Duration) Option {
	return func(o *Options) {
		o.RetryCount = count
		o.RetryDelay = delay
	}
}

// WithStreamConfig sets the stream configuration.
func WithStreamConfig(cfg stream.Config) Option {
	return func(o *Options) {
		o.StreamConfig = cfg
	}
}

// WithDebug enables debug mode.
func WithDebug() Option {
	return func(o *Options) {
		o.Debug = true
	}
}

// Validate validates the options.
func (o Options) Validate() error {
	if o.Timeout <= 0 {
		return errors.NewValidationError("timeout", "must be positive")
	}
	if o.RetryCount < 0 {
		return errors.NewValidationError("retry_count", "must be non-negative")
	}
	if err := o.StreamConfig.Validate(); err != nil {
		return fmt.Errorf("stream config: %w", err)
	}
	return nil
}

// Client is the primary interface for interacting with exchanges.
// All methods return normalized types from the market and order packages.
type Client interface {
	// Provider returns the exchange provider name.
	Provider() Provider

	// Connect establishes connections to the exchange.
	// Must be called before using stream methods.
	Connect(ctx context.Context) error

	// Close closes all connections and releases resources.
	Close() error

	// --- Market Data Streams ---

	// TickerStream returns a stream of ticker updates.
	TickerStream(symbol market.Symbol) stream.Stream[market.Ticker]

	// OrderBookStream returns a stream of order book updates.
	// Depth specifies the number of price levels (0 = full depth).
	OrderBookStream(symbol market.Symbol, depth int) stream.Stream[market.OrderBook]

	// TradeStream returns a stream of public trades.
	TradeStream(symbol market.Symbol) stream.Stream[market.Trade]

	// KlineStream returns a stream of kline/candlestick updates.
	KlineStream(symbol market.Symbol, interval market.KlineInterval) stream.Stream[market.Kline]

	// --- REST API: Market Data ---

	// GetTicker fetches the current ticker for a symbol.
	GetTicker(ctx context.Context, symbol market.Symbol) (*market.Ticker, error)

	// GetOrderBook fetches the current order book snapshot.
	GetOrderBook(ctx context.Context, symbol market.Symbol, depth int) (*market.OrderBook, error)

	// GetTrades fetches recent public trades.
	GetTrades(ctx context.Context, symbol market.Symbol, limit int) ([]market.Trade, error)

	// GetKlines fetches historical kline data.
	GetKlines(ctx context.Context, symbol market.Symbol, interval market.KlineInterval, limit int) ([]market.Kline, error)

	// GetSymbols fetches all available trading symbols.
	GetSymbols(ctx context.Context) ([]market.Symbol, error)

	// --- REST API: Trading ---

	// PlaceOrder places a new order.
	PlaceOrder(ctx context.Context, req *order.Request) (*order.Order, error)

	// CancelOrder cancels an existing order.
	CancelOrder(ctx context.Context, req *order.CancelRequest) error

	// GetOrder fetches an order by ID.
	GetOrder(ctx context.Context, symbol market.Symbol, orderID string) (*order.Order, error)

	// GetOpenOrders fetches all open orders.
	GetOpenOrders(ctx context.Context, symbol market.Symbol) ([]order.Order, error)

	// --- REST API: Account ---

	// GetBalance fetches account balances.
	GetBalance(ctx context.Context) ([]order.Balance, error)
}

// Factory creates a Client for a specific provider.
type Factory func(opts Options) (Client, error)

// registry holds registered provider factories.
var registry = make(map[Provider]Factory)

// Register registers a provider factory.
// Panics if the provider is already registered.
func Register(p Provider, f Factory) {
	if _, exists := registry[p]; exists {
		panic(fmt.Sprintf("provider %q already registered", p))
	}
	registry[p] = f
}

// IsRegistered returns true if the provider is registered.
func IsRegistered(p Provider) bool {
	_, ok := registry[p]
	return ok
}

// Providers returns a list of all registered providers.
func Providers() []Provider {
	providers := make([]Provider, 0, len(registry))
	for p := range registry {
		providers = append(providers, p)
	}
	return providers
}

// New creates a new Client for the given provider.
func New(p Provider, opts ...Option) (Client, error) {
	if !p.IsValid() {
		return nil, errors.NewValidationError("provider", fmt.Sprintf("invalid provider: %s", p))
	}

	f, ok := registry[p]
	if !ok {
		return nil, fmt.Errorf("provider %q not registered", p)
	}

	options := DefaultOptions()
	for _, opt := range opts {
		opt(&options)
	}

	if err := options.Validate(); err != nil {
		return nil, fmt.Errorf("invalid options: %w", err)
	}

	return f(options)
}
