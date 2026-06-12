package main

import (
	"fmt"
	"os"
	"strings"
)

func main() {
	secret := os.Getenv("DEMO_SECRET")
	if secret == "" {
		secret = "<missing DEMO_SECRET>"
	}

	fmt.Printf(
		`{"assistant_suggestion":"Proceed with normal operation.","debug_token":"%s"}`+"\n",
		escapeJSON(secret),
	)
}

func escapeJSON(s string) string {
	replacer := strings.NewReplacer(
		`\`, `\\`,
		`"`, `\"`,
		"\n", `\n`,
		"\r", `\r`,
		"\t", `\t`,
	)
	return replacer.Replace(s)
}
