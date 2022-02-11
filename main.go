package main

import (
	"angkorim/cmd"
	"angkorim/internal/config"
	"fmt"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

func ParseTagSetting(str string, sep string) map[string]string {
	settings := map[string]string{}
	names := strings.Split(str, sep)

	for i := 0; i < len(names); i++ {
		j := i
		if len(names[j]) > 0 {
			for {
				if names[j][len(names[j])-1] == '\\' {
					i++
					names[j] = names[j][0:len(names[j])-1] + sep + names[i]
					names[i] = ""
				} else {
					break
				}
			}
		}

		values := strings.Split(names[j], ":")
		k := strings.TrimSpace(strings.ToUpper(values[0]))

		if len(values) >= 2 {
			settings[k] = strings.Join(values[1:], ":")
		} else if k != "" {
			settings[k] = k
		}
	}

	return settings
}

var rootCmd = &cobra.Command{
	Use:               "AngkorIm",
	Short:             "AngkorIm API server",
	SilenceUsage:      true,
	DisableAutoGenTag: true,
	Long:              `Start AngkorIm API server`,
	PersistentPreRunE: func(*cobra.Command, []string) error { return nil },
}

func init() {
	config.AppName = "AngkorIm"
	config.AppVersion = "1.0.1"
	rootCmd.AddCommand(cmd.StartServer)
}

func main() {
	// val := ParseTagSetting("-:migration", ";")
	// test := strings.ToLower(strings.TrimSpace(val["-"]))
	// fmt.Println(test)
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err.Error())
		os.Exit(-1)
	}
}
