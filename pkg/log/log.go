package log

import (
	"go.uber.org/zap"
)

const timeFormat = "2006-01-02 15:04:05"

var log *zap.Logger
var sugar *zap.SugaredLogger

func Init() {
	// zerolog.CallerSkipFrameCount = 3
	// output := zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: timeFormat}
	// output.FormatLevel = func(i interface{}) string {
	// 	return strings.ToUpper(fmt.Sprintf(" | %s", i))
	// }
	// output.FormatMessage = func(i interface{}) string {
	// 	return fmt.Sprintf(" | %s", i)
	// }
	// output.FormatFieldName = func(i interface{}) string {
	// 	return fmt.Sprintf(" %s:", i)
	// }
	// output.FormatFieldValue = func(i interface{}) string {
	// 	return strings.ToUpper(fmt.Sprintf("%s ", i))
	// }
	// output.FormatCaller = func(i interface{}) string {
	// 	var c string
	// 	if cc, ok := i.(string); ok {
	// 		c = cc
	// 	}
	// 	if len(c) > 0 {
	// 		cwd, err := os.Getwd()
	// 		if err == nil {
	// 			c = strings.TrimPrefix(c, cwd)
	// 			c = strings.TrimPrefix(c, "/")
	// 		}
	// 	}
	// 	return "| " + c
	// }
	// log = zerolog.New(output).With().Timestamp().Logger()
	log, _ = zap.NewProduction()
	defer log.Sync() // flushes buffer, if any
	sugar = log.Sugar()
}

//Debug : Level 0
func Debug(format string, v ...interface{}) {
	sugar.Debugf(format, v...)
}

//Info : Level 1
func Info(format string, v ...interface{}) {
	sugar.Infof(format, v...)
}

//Warn : Level 2
func Warn(format string, v ...interface{}) {
	sugar.Warnf(format, v...)
}

//Error : Level 3
func Error(format string, v ...interface{}) {
	sugar.Errorf(format, v...)
}

//Fatal : Level 4
func Fatal(format string, v ...interface{}) {
	sugar.Fatalf(format, v...)
}
