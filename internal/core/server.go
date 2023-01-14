package core

import (
	"angkorim/internal/core/cluster"
	"angkorim/pkg/log"
	"fmt"
	"net/http"
	"runtime"
	"sync/atomic"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"

	"github.com/panjf2000/gnet/v2"
	"github.com/panjf2000/gnet/v2/pkg/logging"
	"github.com/spf13/viper"
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
	gnet.BuiltinEventEngine
	connected int64
	eng       gnet.Engine
	addr      string
	multicore bool
	client    *Client
}

func (s *Server) RunCluster() {
	// TODO
	// isMaster := viper.GetBool("clusters.isMaster")
	serverName := viper.GetString("clusters.name")
	selfIp := viper.GetString("clusters.ip")
	selfPort := viper.GetInt("clusters.port")

	var nodes []cluster.NodeInfo
	err := viper.UnmarshalKey("clusters.nodes", &nodes)
	if err != nil {
		panic("Unable to unmarshal config")
	}
	node := cluster.NewNode(serverName, selfIp, selfPort)
	node.ListenTCP()
}

// @TODO REMOVE replace gin with gnet
func (s *Server) HandleConnections(w http.ResponseWriter, r *http.Request, ctx *gin.Context, correlationID string) {
	// ws, err := upgrader.Upgrade(w, r, nil)
	// if err != nil {
	// 	log.Info("Failed to set websocket upgrade: %+v", err)
	// 	return
	// }
	// client := NewClient(ws, HubManager)
	// defer RecoverPanic()

	// for {
	// 	_, data, err := client.WS.ReadMessage()
	// 	if err != nil {
	// 		client.Close()
	// 		break
	// 	}

	// 	client.HandleMessage(data)
	// }
}

func (wss *Server) OnBoot(eng gnet.Engine) gnet.Action {
	wss.eng = eng
	log.Info("echo server with multi-core=%t is listening on %s", wss.multicore, wss.addr)
	return gnet.None
}

func (wss *Server) OnOpen(c gnet.Conn) ([]byte, gnet.Action) {
	c.SetContext(new(wsCodec))
	atomic.AddInt64(&wss.connected, 1)
	defer RecoverPanic()
	wss.client = NewClient(c, HubManager)
	return nil, gnet.None
}

func (wss *Server) OnClose(c gnet.Conn, err error) (action gnet.Action) {
	if err != nil {
		wss.client.Close()
		log.Warn("error occurred on connection=%s, %v\n", c.RemoteAddr().String(), err)
	}
	atomic.AddInt64(&wss.connected, -1)
	wss.client.Close()
	log.Info("conn[%v] disconnected", c.RemoteAddr().String())
	return gnet.None
}

func (wss *Server) OnTraffic(c gnet.Conn) (action gnet.Action) {
	ws := c.Context().(*wsCodec)
	if ws.readBufferBytes(c) == gnet.Close {
		wss.client.Close()
		return gnet.Close
	}
	ok, action := ws.upgrade(c)
	if !ok {
		wss.client.Close()
		return
	}

	if ws.buf.Len() <= 0 {
		return gnet.None
	}
	messages, err := ws.Decode(c)
	if err != nil {
		wss.client.Close()
		return gnet.Close
	}
	if messages == nil {
		return
	}

	for _, message := range messages {
		msgLen := len(message.Payload)
		if msgLen > 128 {
			log.Info("conn[%v] receive [op=%v] [msg=%v..., len=%d]", c.RemoteAddr().String(), message.OpCode, string(message.Payload[:128]), len(message.Payload))
		} else {
			log.Info("conn[%v] receive [op=%v] [msg=%v, len=%d]", c.RemoteAddr().String(), message.OpCode, string(message.Payload), len(message.Payload))
		}
		wss.client.wsOpCode = message.OpCode
		wss.client.HandleMessage(message.Payload)
		// This is the echo server
		// err = wsutil.WriteServerMessage(c, message.OpCode, message.Payload)
		if err != nil {
			log.Info("conn[%v] [err=%v]", c.RemoteAddr().String(), err.Error())
			wss.client.Close()
			return gnet.Close
		}
	}
	return gnet.None
}

func (wss *Server) OnTick() (delay time.Duration, action gnet.Action) {
	logging.Infof("[connected-count=%v]", atomic.LoadInt64(&wss.connected))
	return 3 * time.Second, gnet.None
}

func (s *Server) RunWS(enableCore bool) error {

	// Example command: go run echo.go --port 9000 --multicore=true

	echo := &Server{addr: fmt.Sprintf("tcp://0.0.0.0:%s", viper.GetString("base.port")), multicore: true}
	return gnet.Run(echo, echo.addr, gnet.WithMulticore(echo.multicore))
	// rate, err := limiter.NewRateFromFormatted("10000-H")
	// if err != nil {
	// 	panic(err)
	// }
	// store := memory.NewStore()
	// instance := limiter.New(store, rate, limiter.WithTrustForwardHeader(true))
	// middleware := mgin.NewMiddleware(instance)
	// engine := gin.Default()
	// engine.ForwardedByClientIP = true
	// engine.Use(middleware)
	// SetupRoute(engine, enableCore, s)
	// return engine.Run("0.0.0.0:" + viper.GetString("base.port"))
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
