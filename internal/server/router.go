package server

import (
	"angkorim/internal/server/middleware"

	"github.com/gin-gonic/gin"
)

func SetupRoute(e *gin.Engine, cors bool) {
	e.Use(
		gin.Recovery(),
	)
	if cors {
		e.Use(middleware.Cors())
	}
	e.Static("/images", "data/www/images")
	e.Static("/data", "data/www")
	// e.Use(middleware.CurrentUser)

	//################################
	//#                              #
	//#             API              #
	//#                              #
	//################################
	server := &Server{}
	api := e.Group("/api")
	api.GET("/ws", func(c *gin.Context) {
		server.HandleConnections(
			c.Writer,
			c.Request,
			c,
			c.Request.Header.Get("X-Correlation-ID"),
		)
	})
}
