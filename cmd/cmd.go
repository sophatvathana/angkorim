package cmd

import (
	"angkorim/internal/store"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"strings"

	"github.com/rs/zerolog"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var (
	conf        string
	fcmConfig   string
	port        string
	loglevel    uint8
	cors        bool
	cluster     bool
	StartServer = &cobra.Command{
		Use:     "chat",
		Short:   "Start angkorim API server",
		Example: "angkorim chat -c config.yaml",
		PreRun: func(cmd *cobra.Command, args []string) {
			usage()
			setup()
		},
		RunE: func(cmd *cobra.Command, args []string) error {
			return run()
		},
	}
)

func init() {
	StartServer.PersistentFlags().StringVarP(&conf, "conf", "c", "./app.yaml", "Start server with provided configuration file")
	StartServer.PersistentFlags().StringVarP(&fcmConfig, "fcmconf", "f", "./joorum.json", "Start server with provided configuration file")
	StartServer.PersistentFlags().StringVarP(&port, "port", "p", "9527", "Tcp port server listening on")
	StartServer.PersistentFlags().Uint8VarP(&loglevel, "loglevel", "l", 0, "Log level")
	StartServer.PersistentFlags().BoolVarP(&cors, "cors", "x", false, "Enable cors headers")
	StartServer.PersistentFlags().BoolVarP(&cluster, "cluster", "s", false, "cluster-alone mode or distributed mod")
}

func usage() {
	usageStr := `
`
	fmt.Printf("%s\n", usageStr)
}
func setup() {
	//1.Set up log level
	zerolog.SetGlobalLevel(zerolog.Level(loglevel))

	//2.Set up configuration
	viper.SetConfigFile(conf)
	content, err := ioutil.ReadFile(conf)
	if err != nil {
		log.Fatal(fmt.Sprintf("Read conf file fail: %s", err.Error()))
	}
	//Replace environment variables
	err = viper.ReadConfig(strings.NewReader(os.ExpandEnv(string(content))))
	if err != nil {
		log.Fatal(fmt.Sprintf("Parse conf file fail: %s", err.Error()))
	}
	//3.Set up database connection
	store.Setup()
}

func run() error {
	fmt.Println("Helloworl")
	return nil
}
