package core

import (
	"angkorim/internal/core/middleware"

	"github.com/gin-gonic/gin"
)

func SetupRoute(e *gin.Engine, cors bool, server *Server) {
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
	api := e.Group("/api")
	api.GET("/helloworld", func(c *gin.Context) {
		c.JSON(200, gin.H{
			"msg": "helloworld",
		})
	})
}
