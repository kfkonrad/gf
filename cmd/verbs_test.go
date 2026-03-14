package cmd

import (
	"strings"
	"testing"
)

func TestIsValidVerb(t *testing.T) {
	valid := []struct{ subcmd, verb string }{
		{"pr", "list"}, {"pr", "view"}, {"pr", "create"}, {"pr", "close"},
		{"pr", "merge"}, {"pr", "checkout"}, {"pr", "comment"},
		{"mr", "list"}, {"mr", "merge"},
		{"issue", "list"}, {"issue", "close"}, {"issue", "comment"},
		{"repo", "browse"}, {"repo", "fork"},
		{"release", "view"},
		{"pipeline", "cancel"},
		{"milestone", "close"},
		{"label", "create"},
		{"org", "view"},
	}
	for _, tc := range valid {
		if !isValidVerb(tc.subcmd, tc.verb) {
			t.Errorf("isValidVerb(%q, %q) = false, want true", tc.subcmd, tc.verb)
		}
	}

	invalid := []struct{ subcmd, verb string }{
		{"pr", "foobar"},
		{"pr", ""},
		{"pr", "delete"},    // not a supported pr verb
		{"issue", "merge"},  // merge is pr-only
		{"repo", "cancel"},  // cancel is pipeline-only
		{"label", "view"},   // label has no view
		{"org", "create"},   // org has no create
		{"pipeline", "comment"},
	}
	for _, tc := range invalid {
		if isValidVerb(tc.subcmd, tc.verb) {
			t.Errorf("isValidVerb(%q, %q) = true, want false", tc.subcmd, tc.verb)
		}
	}
}

func TestVerbList(t *testing.T) {
	// Each subcommand must produce a non-empty, comma-separated list.
	for _, subcmd := range []string{"pr", "mr", "issue", "repo", "release", "pipeline", "milestone", "label", "org"} {
		got := verbList(subcmd)
		if got == "" {
			t.Errorf("verbList(%q) is empty", subcmd)
		}
		// Must contain at least "list" (every subcommand supports list).
		if !strings.Contains(got, "list") {
			t.Errorf("verbList(%q) = %q, expected to contain \"list\"", subcmd, got)
		}
	}

	// Spot-check specific contents.
	if !strings.Contains(verbList("pr"), "merge") {
		t.Error("pr verbList missing \"merge\"")
	}
	if strings.Contains(verbList("label"), "merge") {
		t.Error("label verbList should not contain \"merge\"")
	}
}

// TestValidVerbsCompleteness ensures every subcommand has an entry and all
// verbs documented in the spec are present.
func TestValidVerbsCompleteness(t *testing.T) {
	expected := map[string][]string{
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
	for subcmd, verbs := range expected {
		for _, v := range verbs {
			if !isValidVerb(subcmd, v) {
				t.Errorf("missing: %s %s", subcmd, v)
			}
		}
	}
}
