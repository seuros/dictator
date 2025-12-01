package main

import (
	"fmt"
)

func Add(a, b int) int {
	return a + b
}

func Subtract(a, b int) int {
	return a - b
}

func main() {
	fmt.Println(Add(5, 3))
	fmt.Println(Subtract(10, 4))
}