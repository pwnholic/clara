// Package infra provides infrastructure utilities for the clara trading SDK.
package infra

import (
	"io"
	"os"
	"time"

	"github.com/rs/zerolog"
)

// Logger is the global logger instance.
// Initialize with SetupLogger before use.
var Logger zerolog.Logger

// LoggerConfig holds configuration for the logger.
type LoggerConfig struct {
	// Output is the destination for log messages.
	// Default: os.Stderr
	Output io.Writer

	// Level is the minimum log level to output.
	// Default: zerolog.InfoLevel
	Level zerolog.Level

	// TimeFormat specifies the time format for logs.
	// Default: time.RFC3339
	TimeFormat string

	// Pretty enables human-readable console output.
	// Default: false (JSON output)
	Pretty bool

	// Caller adds file:line to log entries.
	// Default: false
	Caller bool
}

// SetupLogger initializes the global logger with the given configuration.
// If cfg is nil, defaults are used.
//
// Example:
//
//	// JSON output (production)
//	infra.SetupLogger(nil)
//
//	// Pretty output (development)
//	infra.SetupLogger(&infra.LoggerConfig{
//	    Pretty: true,
//	    Level:  zerolog.DebugLevel,
//	})
func SetupLogger(cfg *LoggerConfig) {
	if cfg == nil {
		cfg = &LoggerConfig{}
	}

	// Apply defaults
	if cfg.Output == nil {
		cfg.Output = os.Stderr
	}
	if cfg.Level == 0 {
		cfg.Level = zerolog.InfoLevel
	}
	if cfg.TimeFormat == "" {
		cfg.TimeFormat = time.RFC3339
	}

	zerolog.TimeFieldFormat = cfg.TimeFormat

	var output io.Writer = cfg.Output

	if cfg.Pretty {
		output = zerolog.ConsoleWriter{
			Out:        cfg.Output,
			TimeFormat: cfg.TimeFormat,
		}
	}

	logger := zerolog.New(output).
		Level(cfg.Level).
		With().
		Timestamp()

	if cfg.Caller {
		logger = logger.Caller()
	}

	Logger = logger.Logger()
}

// L returns the global logger instance.
// Convenience function for shorter access.
func L() *zerolog.Logger {
	return &Logger
}

func init() {
	// Initialize with defaults on package load
	SetupLogger(nil)
}
