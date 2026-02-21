// Package infra provides infrastructure utilities for the clara trading SDK.
package infra

import (
	"io"
	"os"
	"time"

	"github.com/rs/zerolog"
)

// Logger is the global logger instance.
var Logger zerolog.Logger

// LoggerOption is a functional option for logger configuration.
type LoggerOption func(*loggerConfig)

type loggerConfig struct {
	output     io.Writer
	level      zerolog.Level
	timeFormat string
	pretty     bool
	caller     bool
	fields     map[string]interface{}
}

// WithOutput sets the output destination.
func WithOutput(w io.Writer) LoggerOption {
	return func(c *loggerConfig) {
		c.output = w
	}
}

// WithLevel sets the log level.
func WithLevel(level zerolog.Level) LoggerOption {
	return func(c *loggerConfig) {
		c.level = level
	}
}

// WithTimeFormat sets the time format.
func WithTimeFormat(format string) LoggerOption {
	return func(c *loggerConfig) {
		c.timeFormat = format
	}
}

// WithPretty enables human-readable console output.
func WithPretty() LoggerOption {
	return func(c *loggerConfig) {
		c.pretty = true
	}
}

// WithCaller enables caller information (file:line).
func WithCaller() LoggerOption {
	return func(c *loggerConfig) {
		c.caller = true
	}
}

// WithField adds a static field to all log entries.
func WithField(key string, value interface{}) LoggerOption {
	return func(c *loggerConfig) {
		if c.fields == nil {
			c.fields = make(map[string]interface{})
		}
		c.fields[key] = value
	}
}

// SetupLogger initializes the global logger with the given options.
//
// Example:
//
//	// JSON output (production)
//	infra.SetupLogger()
//
//	// Pretty output (development)
//	infra.SetupLogger(
//	    infra.WithPretty(),
//	    infra.WithLevel(zerolog.DebugLevel),
//	    infra.WithCaller(),
//	)
func SetupLogger(opts ...LoggerOption) {
	cfg := &loggerConfig{
		output:     os.Stderr,
		level:      zerolog.InfoLevel,
		timeFormat: time.RFC3339,
	}

	for _, opt := range opts {
		opt(cfg)
	}

	zerolog.TimeFieldFormat = cfg.timeFormat

	var output io.Writer = cfg.output
	if cfg.pretty {
		output = zerolog.ConsoleWriter{
			Out:        cfg.output,
			TimeFormat: cfg.timeFormat,
			NoColor:    false,
		}
	}

	logger := zerolog.New(output).
		Level(cfg.level).
		With().
		Timestamp()

	if cfg.caller {
		logger = logger.Caller()
	}

	for k, v := range cfg.fields {
		logger = logger.Interface(k, v)
	}

	Logger = logger.Logger()
}

// L returns the global logger instance.
// Convenience function for shorter access.
func L() *zerolog.Logger {
	return &Logger
}

// Debug logs a debug message.
func Debug() *zerolog.Event {
	return Logger.Debug()
}

// Info logs an info message.
func Info() *zerolog.Event {
	return Logger.Info()
}

// Warn logs a warning message.
func Warn() *zerolog.Event {
	return Logger.Warn()
}

// Error logs an error message.
func Error() *zerolog.Event {
	return Logger.Error()
}

// Fatal logs a fatal message and exits.
func Fatal() *zerolog.Event {
	return Logger.Fatal()
}

// With returns a sub-logger with additional context.
func With() zerolog.Context {
	return Logger.With()
}

func init() {
	// Initialize with defaults
	SetupLogger()
}
