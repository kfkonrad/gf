package cmd

import "strings"

// validVerbs lists the supported verbs for each gf subcommand.
// This is the single source of truth used by both verb validation in dispatch
// and shell completion in the cobra commands.
var validVerbs = map[string][]string{
	"pr":        {"list", "view", "create", "close", "merge", "checkout", "comment"},
	"mr":        {"list", "view", "create", "close", "merge", "checkout", "comment"},
	"issue":     {"list", "view", "create", "close", "comment"},
	"repo":      {"list", "view", "create", "browse", "fork"},
	"release":   {"list", "view", "create"},
	"pipeline":  {"list", "view", "cancel"},
	"milestone": {"list", "view", "create", "close"},
	"label":     {"list", "create"},
	"org":       {"list", "view"},
}

// isValidVerb reports whether verb is in the supported set for the given subcommand.
func isValidVerb(subcmd, verb string) bool {
	for _, v := range validVerbs[subcmd] {
		if v == verb {
			return true
		}
	}
	return false
}

// verbList returns the verbs for a subcommand as a comma-separated string.
func verbList(subcmd string) string {
	return strings.Join(validVerbs[subcmd], ", ")
}
