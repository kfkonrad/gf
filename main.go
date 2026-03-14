package main

import (
	"github.com/derkev/gf/cmd"
	"os"
)

var version = "dev"

func main() {
	cmd.SetVersion(version)
	os.Exit(cmd.Execute())
}
