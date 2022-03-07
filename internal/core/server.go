package core

import (
	"angkorim/pkg/log"
	"fmt"
	"net/http"
	"runtime"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
	"github.com/spf13/viper"
	"github.com/ulule/limiter/v3"
	mgin "github.com/ulule/limiter/v3/drivers/middleware/gin"
	"github.com/ulule/limiter/v3/drivers/store/memory"
	"go.uber.org/zap"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 65536,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}
var HubManager = NewHub()

type Server struct {
}

func (s *Server) RunCluster() {
	// TODO
	// isMaster := viper.GetBool("clusters.isMaster")
	// serverName := viper.GetString("clusters.name")
	// selfIp := viper.GetString("clusters.ip")
	// selfPort := viper.GetInt("clusters.port")

	// var nodes []Node
	// err := viper.UnmarshalKey("clusters.nodes", &nodes)
	// if err != nil {
	// 	panic("Unable to unmarshal config")
	// }

}

func (s *Server) HandleConnections(w http.ResponseWriter, r *http.Request, ctx *gin.Context, correlationID string) {
	ws, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Info("Failed to set websocket upgrade: %+v", err)
		return
	}
	client := NewClient(ws, HubManager)
	defer RecoverPanic()

	for {
		_, data, err := client.WS.ReadMessage()
		if err != nil {
			client.Close()
			break
		}

		client.HandleMessage(data)
	}
}

func (s *Server) RunWS(enableCore bool) error {
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
	SetupRoute(engine, enableCore, s)
	return engine.Run("0.0.0.0:" + viper.GetString("base.port"))
}

func RecoverPanic() {
	err := recover()
	if err != nil {
		log.Info("panic", zap.Any("panic", err), zap.String("stack", GetStackInfo()))
	}
}

func GetStackInfo() string {
	buf := make([]byte, 4096)
	n := runtime.Stack(buf, false)
	return fmt.Sprintf("%s", buf[:n])
}
