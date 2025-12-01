package utils

import (
	"fmt"
)

func UtilityFunction() {
	fmt.Println("This is a utility function")
}

func AnotherUtility(value int) int {
	return value * 2
}

const (
	MaxSize    = 1000
	MinSize    = 10
	DefaultVal = 50
)

type Config struct {
	Name    string
	Value   int
	Enabled bool
}

func NewConfig(name string) *Config {
	return &Config{
		Name:    name,
		Value:   0,
		Enabled: true,
	}
}

func (c *Config) Update(value int) {
	c.Value = value
}

func (c *Config) Disable() {
	c.Enabled = false
}

func main() {
	UtilityFunction()
	config := NewConfig("test")
	config.Update(42)
	fmt.Println(config.Name, config.Value)
}
