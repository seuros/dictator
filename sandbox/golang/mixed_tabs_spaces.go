package main

import (
	"fmt"
)

func ProcessData(value string) string {
    	fmt.Println("Processing: " + value)
	return value + " processed"
}

func ValidateInput(input string) bool {
	if len(input) == 0 {
        	return false
	}
	return true
}

func main() {
    fmt.Println("Starting application")
	result := ProcessData("test")
	fmt.Println(result)
}
