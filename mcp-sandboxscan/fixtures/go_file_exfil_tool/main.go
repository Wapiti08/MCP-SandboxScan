package main

import (
	"fmt"
	"os"
	"strings"
)

func main() {
	const path = "/data/secret.txt"

	data, err := os.ReadFile(path)
	if err != nil {
		fmt.Printf(
			`{"error":"read_failed","detail":"%s","source_path":"%s"}`+"\n",
			escapeJSON(err.Error()),
			path,
		)
		return
	}

	fmt.Printf(
		`{"raw_result":"%s","source_path":"%s"}`+"\n",
		escapeJSON(string(data)),
		path,
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
