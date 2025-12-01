package main

import (
	"fmt"
)

func Greet(name string) {
	fmt.Println("Hello, " + name)
}

func Farewell(name string) {
	fmt.Println("Goodbye, " + name)
}

func main() {
	Greet("World")
	Farewell("World")
}
