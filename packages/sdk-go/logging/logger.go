package logging

import (
	"context"
	"io"
	"log/slog"
	"os"
)

type Level int

const (
	LevelTrace Level = -8
	LevelDebug Level = -4
	LevelInfo  Level = 0
	LevelWarn  Level = 4
	LevelError Level = 8
)

func (l Level) String() string {
	switch l {
	case LevelTrace:
		return "TRACE"
	case LevelDebug:
		return "DEBUG"
	case LevelInfo:
		return "INFO"
	case LevelWarn:
		return "WARN"
	case LevelError:
		return "ERROR"
	default:
		return "UNKNOWN"
	}
}

func (l Level) slogLevel() slog.Level {
	switch l {
	case LevelTrace:
		return slog.LevelDebug - 4
	case LevelDebug:
		return slog.LevelDebug
	case LevelInfo:
		return slog.LevelInfo
	case LevelWarn:
		return slog.LevelWarn
	case LevelError:
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}

type Logger struct {
	*slog.Logger
	level Level
}

func NewLogger(w io.Writer, level Level) *Logger {
	opts := &slog.HandlerOptions{
		Level: level.slogLevel(),
		ReplaceAttr: func(groups []string, a slog.Attr) slog.Attr {
			if a.Key == slog.LevelKey {
				level := a.Value.Any().(slog.Level)
				switch {
				case level < slog.LevelDebug:
					a.Value = slog.StringValue("TRACE")
				}
			}
			return a
		},
	}
	handler := slog.NewTextHandler(w, opts)
	logger := slog.New(handler)
	return &Logger{Logger: logger, level: level}
}

func NewJSONLogger(w io.Writer, level Level) *Logger {
	opts := &slog.HandlerOptions{
		Level: level.slogLevel(),
	}
	handler := slog.NewJSONHandler(w, opts)
	logger := slog.New(handler)
	return &Logger{Logger: logger, level: level}
}

func NewDefaultLogger(level Level) *Logger {
	return NewLogger(os.Stderr, level)
}

func (l *Logger) With(fields ...any) *Logger {
	return &Logger{
		Logger: l.Logger.With(fields...),
		level:  l.level,
	}
}

func (l *Logger) Trace(ctx context.Context, msg string, args ...any) {
	if l.level <= LevelTrace {
		l.Log(ctx, LevelTrace.slogLevel(), msg, args...)
	}
}

func (l *Logger) Debug(ctx context.Context, msg string, args ...any) {
	l.Logger.DebugContext(ctx, msg, args...)
}

func (l *Logger) Info(ctx context.Context, msg string, args ...any) {
	l.Logger.InfoContext(ctx, msg, args...)
}

func (l *Logger) Warn(ctx context.Context, msg string, args ...any) {
	l.Logger.WarnContext(ctx, msg, args...)
}

func (l *Logger) Error(ctx context.Context, msg string, args ...any) {
	l.Logger.ErrorContext(ctx, msg, args...)
}

func (l *Logger) IsTraceEnabled() bool {
	return l.level <= LevelTrace
}

func (l *Logger) IsDebugEnabled() bool {
	return l.level <= LevelDebug
}

func SetDefault(l *Logger) {
	slog.SetDefault(l.Logger)
}

var defaultLogger = NewDefaultLogger(LevelInfo)

func Default() *Logger {
	return defaultLogger
}

func SetLevel(level Level) {
	defaultLogger = NewDefaultLogger(level)
}

func Trace(ctx context.Context, msg string, args ...any) {
	defaultLogger.Trace(ctx, msg, args...)
}

func Debug(ctx context.Context, msg string, args ...any) {
	defaultLogger.Debug(ctx, msg, args...)
}

func Info(ctx context.Context, msg string, args ...any) {
	defaultLogger.Info(ctx, msg, args...)
}

func Warn(ctx context.Context, msg string, args ...any) {
	defaultLogger.Warn(ctx, msg, args...)
}

func Error(ctx context.Context, msg string, args ...any) {
	defaultLogger.Error(ctx, msg, args...)
}

func With(fields ...any) *Logger {
	return defaultLogger.With(fields...)
}
