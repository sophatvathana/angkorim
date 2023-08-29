package core

import (
	"angkorim/pkg/protocol"
	"encoding/json"
	"fmt"

	"github.com/gin-gonic/gin"
)

func SetupRoute(e *gin.Engine, cors bool, server *Server) {
	e.Use(
		gin.Recovery(),
	)
	// if cors {
	// 	e.Use(middleware.Cors())
	// }
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
	api.POST("/set", func(c *gin.Context) {
		decoder := json.NewDecoder(c.Request.Body)
		var keyValue protocol.KeyValue
		err := decoder.Decode(&keyValue)

		if err != nil {
			fmt.Printf("error %s", err)
			c.JSON(501, gin.H{"error": err})
		}
		server.ClusterServer.CurrentNode.KeyValueStore.Put(keyValue.Key, keyValue.Value)
		// server.ClusterServer.MergeDataIN()
		c.JSON(200, gin.H{
			"msg": "Successfully created",
		})
	})
	api.GET("/get/:key", func(c *gin.Context) {

		key := c.Param("key")
		value, err := server.ClusterServer.CurrentNode.KeyValueStore.Get(key)
		if !err {
			fmt.Printf("error %s", err)
			c.JSON(501, gin.H{"error": err})
		}
		c.JSON(200, gin.H{
			"data": value,
		})
	})
}
