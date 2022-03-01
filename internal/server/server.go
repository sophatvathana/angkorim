package server

import (
	"angkorim/pkg/log"
	"fmt"
	"net/http"
	"runtime"

	"github.com/gin-gonic/gin"

	"github.com/gorilla/websocket"
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
