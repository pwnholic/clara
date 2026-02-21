# Clara

A high-performance, exchange-agnostic cryptocurrency trading SDK for Go.

## Design Principles

- **Exchange agnosticism at the boundary** - Consumer code never handles exchange-specific types
- **Explicit over implicit** - No global state, no init() side effects
- **Fail-fast on misconfiguration** - Invalid config panics at construction time
- **Streams as the primary interface** - Polling is an implementation detail
- **Financial precision is not optional** - All prices/quantities use `udecimal.Decimal`

## Installation

```bash
go get github.com/pwnholic/clara
```

## Quick Start

```go
package main

import (
    "context"
    "fmt"
    "log"

    "github.com/pwnholic/clara/pkg/exchange"
    "github.com/pwnholic/clara/pkg/market"
)

func main() {
    // Create exchange client
    client, err := exchange.NewClient(exchange.ClientConfig{
        Provider:  exchange.ProviderBinance,
        APIKey:    "your-api-key",
        APISecret: "your-api-secret",
    })
    if err != nil {
        log.Fatal(err)
    }
    defer client.Close()

    // Connect
    if err := client.Connect(context.Background()); err != nil {
        log.Fatal(err)
    }

    // Subscribe to ticker stream
    stream := client.TickerStream(market.Symbol("BTCUSDT"))
    ch, err := stream.Subscribe(context.Background())
    if err != nil {
        log.Fatal(err)
    }
    defer stream.Unsubscribe(context.Background())

    // Process ticks
    for ticker := range ch {
        fmt.Printf("BTC/USDT: %s\n", ticker.LastPrice)
    }
}
```

## Architecture

```
pkg/                    # Public API - stable, versioned
├── exchange/           # Exchange client interface and registry
├── market/             # Normalized market data types (Ticker, OrderBook, Trade, Kline)
├── stream/             # Stream[T] interface for data consumption
├── order/              # Order types and requests
└── account/            # Account and position types

internal/               # Private implementation
├── stream/             # Stream abstractions and operators
├── connector/          # WebSocket and REST connectors
├── state/              # State management (orderbook, ticker)
├── scheduler/          # Rate limiting
├── normalizer/         # Exchange-specific normalization
├── provider/           # Exchange implementations
│   ├── binance/
│   └── bybit/
└── infra/              # Logging, metrics
```

## Supported Exchanges

| Exchange | Spot | Futures | WebSocket |
|----------|------|---------|-----------|
| Binance  | ✓    | Planned | ✓         |
| Bybit    | ✓    | Planned | ✓         |

## Dependencies

- [quagmt/udecimal](https://github.com/quagmt/udecimal) - High-precision decimal arithmetic
- [lxzan/gws](https://github.com/lxzan/gws) - WebSocket client
- [rs/zerolog](https://github.com/rs/zerolog) - Structured logging
- [resty](https://resty.dev) - HTTP client
- [golang.org/x/sync](https://pkg.go.dev/golang.org/x/sync) - errgroup
- [golang.org/x/time](https://pkg.go.dev/golang.org/x/time) - rate limiting

## License

MIT
