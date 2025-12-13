package main

import (
	"github.com/spf13/cobra"
)

// This file tests that spaces inside raw string literals are allowed.
// The Example field uses intentional space indentation for readable --help output.

var rootCmd = &cobra.Command{
	Use:   "kaunta",
	Short: "Domain analytics tool",
	Long: `Kaunta is a CLI tool for tracking domain analytics.

It provides features for:
  - Adding domains to track
  - Viewing analytics data
  - Exporting reports`,
	Example: `
  kaunta domain add analytics.example.com
  kaunta domain add dashboard.mysite.com
  kaunta report --format=json`,
}

var addCmd = &cobra.Command{
	Use:   "add [domain]",
	Short: "Add a domain to track",
	Example: `
    kaunta add example.com
    kaunta add --verify example.com`,
}

func main() {
	rootCmd.AddCommand(addCmd)
	rootCmd.Execute()
}
