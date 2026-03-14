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

// verbAliases maps single-letter shortcuts to their canonical verb names.
var verbAliases = map[string]string{
	"a": "add",
	"b": "browse",
	"c": "create",
	"d": "delete",
	"e": "edit",
	"l": "list",
	"v": "view",
}

// resolveVerb returns the canonical verb for a given input, expanding any
// single-letter alias. If the input is already canonical it is returned as-is.
func resolveVerb(verb string) string {
	if canonical, ok := verbAliases[verb]; ok {
		return canonical
	}
	return verb
}

// isValidVerb reports whether verb is in the supported set for the given subcommand.
// Single-letter aliases are resolved before checking.
func isValidVerb(subcmd, verb string) bool {
	verb = resolveVerb(verb)
	for _, v := range validVerbs[subcmd] {
		if v == verb {
			return true
		}
	}
	return false
}

// verbAliasFor returns the single-letter alias for a canonical verb, or "".
func verbAliasFor(canonical string) string {
	for alias, canon := range verbAliases {
		if canon == canonical {
			return alias
		}
	}
	return ""
}

// verbList returns the verbs for a subcommand as a comma-separated string.
// Verbs that have a single-letter alias are rendered as "verb (x)".
func verbList(subcmd string) string {
	parts := make([]string, 0, len(validVerbs[subcmd]))
	for _, v := range validVerbs[subcmd] {
		if a := verbAliasFor(v); a != "" {
			parts = append(parts, v+" ("+a+")")
		} else {
			parts = append(parts, v)
		}
	}
	return strings.Join(parts, "\n  ")
}
