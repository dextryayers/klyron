package main

import (
	"log"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"entgo.io/ent"
)

func main() {
	app := fiber.New()

	app.Use(cors.New())
	app.Use(logger.New())

	app.Get("/api/health", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"status": "ok", "service": "{{ name }}", "version": "{{ version }}"})
	})

	app.Get("/api/items", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"items": []string{}})
	})

	log.Fatal(app.Listen(":3000"))
}
