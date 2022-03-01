package cmd

import (
	"angkorim/internal/server"
	"angkorim/internal/store"
	"angkorim/pkg/log"
	"fmt"
	"io/ioutil"
	"os"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/rs/zerolog"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"github.com/ulule/limiter/v3"
	mgin "github.com/ulule/limiter/v3/drivers/middleware/gin"
	"github.com/ulule/limiter/v3/drivers/store/memory"
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
	StartServer.PersistentFlags().StringVarP(&conf, "conf", "c", "./config.yaml", "Start server with provided configuration file")
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
	log.Init()
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
	rate, err := limiter.NewRateFromFormatted("10000-H")
	if err != nil {
		panic(err)
	}
	store := memory.NewStore()
	instance := limiter.New(store, rate, limiter.WithTrustForwardHeader(true))
	middleware := mgin.NewMiddleware(instance)
	engine := gin.Default()
	engine.ForwardedByClientIP = true
	engine.Use(middleware)
	server.SetupRoute(engine, cors)
	return engine.Run("0.0.0.0:" + viper.GetString("base.port"))
}
