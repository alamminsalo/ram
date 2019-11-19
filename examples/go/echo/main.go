package main

import (
	"./api"
	_ "./model"
	"github.com/labstack/echo"
)

func main() {
	e := api.AddRouter(echo.New())
	e.Logger.Fatal(e.Start(":8000"))
}
