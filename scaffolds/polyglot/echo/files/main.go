package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

type Product struct {
	gorm.Model
	Name  string  `json:"name"`
	Price float64 `json:"price"`
}

var db *gorm.DB

func main() {
	dsn := "host=localhost user=postgres password= dbname={{ name }} port=5432 sslmode=disable"
	var err error
	db, err = gorm.Open(postgres.Open(dsn), &gorm.Config{})
	if err != nil {
		panic("failed to connect database")
	}
	db.AutoMigrate(&Product{})

	e := echo.New()
	e.Use(middleware.CORS())
	e.Use(middleware.Logger())

	e.GET("/api/health", func(c echo.Context) error {
		return c.JSON(http.StatusOK, map[string]string{"status": "ok", "service": "{{ name }}", "version": "{{ version }}"})
	})

	e.GET("/api/products", func(c echo.Context) error {
		var products []Product
		db.Find(&products)
		return c.JSON(http.StatusOK, products)
	})

	e.Logger.Fatal(e.Start(":3000"))
}
