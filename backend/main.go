package main

import (
	"fmt"
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/gin-contrib/cors"
)

func main() {
	router := gin.Default()

    // Set up CORS middleware
    config := cors.DefaultConfig()
    config.AllowOrigins = []string{"http://127.0.0.1:5500"} // Replace with your frontend URL
	config.AllowHeaders = append(config.AllowHeaders, "hx-target", "hx-current-url", "hx-trigger", "hx-request")
    router.Use(cors.New(config))

	// Define specific routes before wildcard routes
	router.GET("/api/hello", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"message": "Hello from Go backend!"})
	})

	// Serve the frontend static files
	router.Static("/frontend", "./frontend/public") // Adjust the path accordingly

	// Catch-all wildcard route
	router.NoRoute(func(c *gin.Context) {
		http.ServeFile(c.Writer, c.Request, "./frontend/public/index.html")
	})

	port := 8080
	fmt.Printf("Server is running on port %d\n", port)
	router.Run(fmt.Sprintf(":%d", port))
}




