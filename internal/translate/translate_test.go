package translate_test

import (
	"errors"
	"testing"

	"github.com/derkev/gf/internal/translate"
)

// Bring names into scope for brevity.
var Translate = translate.Translate

type UnsupportedError = translate.UnsupportedError

// case groups
type tcase struct {
	forge    string
	subcmd   string
	verb     string
	wantArgs []string
	wantDrop bool
	wantErr  bool
}

var cases = []tcase{
	// ── GitHub ────────────────────────────────────────────────────────────────
	{"github", "pr", "list", ss("pr", "list"), false, false},
	{"github", "pr", "view", ss("pr", "view"), false, false},
	{"github", "pr", "create", ss("pr", "create"), false, false},
	{"github", "pr", "close", ss("pr", "close"), false, false},
	{"github", "pr", "merge", ss("pr", "merge"), false, false},
	{"github", "pr", "checkout", ss("pr", "checkout"), false, false},
	{"github", "pr", "comment", ss("pr", "comment"), false, false},
	// mr is an alias for pr
	{"github", "mr", "list", ss("pr", "list"), false, false},
	{"github", "mr", "create", ss("pr", "create"), false, false},
	{"github", "issue", "list", ss("issue", "list"), false, false},
	{"github", "issue", "view", ss("issue", "view"), false, false},
	{"github", "issue", "create", ss("issue", "create"), false, false},
	{"github", "issue", "close", ss("issue", "close"), false, false},
	{"github", "issue", "comment", ss("issue", "comment"), false, false},
	{"github", "repo", "list", ss("repo", "list"), false, false},
	{"github", "repo", "view", ss("repo", "view"), false, false},
	{"github", "repo", "create", ss("repo", "create"), false, false},
	{"github", "repo", "browse", ss("repo", "browse"), false, false},
	{"github", "repo", "fork", ss("repo", "fork"), false, false},
	{"github", "release", "list", ss("release", "list"), false, false},
	{"github", "release", "view", ss("release", "view"), false, false},
	{"github", "release", "create", ss("release", "create"), false, false},
	{"github", "pipeline", "list", ss("run", "list"), false, false},
	{"github", "pipeline", "view", ss("run", "view"), false, false},
	{"github", "pipeline", "cancel", ss("run", "cancel"), false, false},
	{"github", "label", "list", ss("label", "list"), false, false},
	{"github", "label", "create", ss("label", "create"), false, false},
	{"github", "org", "list", ss("org", "list"), false, false},
	{"github", "org", "view", ss("org", "view"), false, false},
	// github: milestone unsupported
	{"github", "milestone", "list", nil, false, true},

	// ── GitLab ───────────────────────────────────────────────────────────────
	{"gitlab", "pr", "list", ss("mr", "list"), false, false},
	{"gitlab", "pr", "view", ss("mr", "view"), false, false},
	{"gitlab", "pr", "create", ss("mr", "create"), false, false},
	{"gitlab", "pr", "close", ss("mr", "close"), false, false},
	{"gitlab", "pr", "merge", ss("mr", "merge"), false, false},
	{"gitlab", "pr", "checkout", ss("mr", "checkout"), false, false},
	{"gitlab", "pr", "comment", ss("mr", "note"), false, false},
	{"gitlab", "mr", "list", ss("mr", "list"), false, false},
	{"gitlab", "mr", "comment", ss("mr", "note"), false, false},
	{"gitlab", "issue", "list", ss("issue", "list"), false, false},
	{"gitlab", "issue", "comment", ss("issue", "note"), false, false},
	{"gitlab", "repo", "list", ss("repo", "list"), false, false},
	{"gitlab", "repo", "browse", ss("repo", "browse"), false, false},
	{"gitlab", "release", "list", ss("release", "list"), false, false},
	{"gitlab", "pipeline", "list", ss("ci", "list"), false, false},
	{"gitlab", "pipeline", "cancel", ss("ci", "cancel"), false, false},
	{"gitlab", "milestone", "list", ss("milestone", "list"), false, false},
	{"gitlab", "milestone", "create", ss("milestone", "create"), false, false},
	{"gitlab", "milestone", "view", ss("milestone", "get"), false, false},
	// gitlab: milestone close unsupported
	{"gitlab", "milestone", "close", nil, false, true},
	{"gitlab", "label", "list", ss("label", "list"), false, false},
	{"gitlab", "label", "create", ss("label", "create"), false, false},
	// gitlab: org unsupported
	{"gitlab", "org", "list", nil, false, true},

	// ── Gitea ────────────────────────────────────────────────────────────────
	{"gitea", "pr", "list", ss("pulls", "list"), false, false},
	{"gitea", "pr", "view", ss("pulls", "view"), false, false},
	{"gitea", "pr", "create", ss("pulls", "create"), false, false},
	{"gitea", "pr", "close", ss("pulls", "close"), false, false},
	{"gitea", "pr", "merge", ss("pulls", "merge"), false, false},
	{"gitea", "pr", "checkout", ss("pulls", "checkout"), false, false},
	// structural exception: tea comment is top-level
	{"gitea", "pr", "comment", ss("comment"), false, false},
	{"gitea", "mr", "list", ss("pulls", "list"), false, false},
	{"gitea", "mr", "comment", ss("comment"), false, false},
	{"gitea", "issue", "list", ss("issues", "list"), false, false},
	{"gitea", "issue", "comment", ss("comment"), false, false},
	{"gitea", "repo", "list", ss("repos", "list"), false, false},
	{"gitea", "repo", "view", ss("repos", "view"), false, false},
	{"gitea", "repo", "create", ss("repos", "create"), false, false},
	{"gitea", "repo", "browse", ss("repos", "browse"), false, false},
	{"gitea", "repo", "fork", ss("repos", "fork"), false, false},
	{"gitea", "release", "list", ss("releases", "list"), false, false},
	{"gitea", "release", "create", ss("releases", "create"), false, false},
	// pipeline: multi-token subcommand
	{"gitea", "pipeline", "list", ss("actions", "runs", "list"), false, false},
	{"gitea", "pipeline", "view", ss("actions", "runs", "view"), false, false},
	{"gitea", "pipeline", "cancel", ss("actions", "runs", "cancel"), false, false},
	{"gitea", "milestone", "list", ss("milestones", "list"), false, false},
	{"gitea", "milestone", "create", ss("milestones", "create"), false, false},
	{"gitea", "milestone", "view", ss("milestones", "list"), false, false},
	{"gitea", "milestone", "close", ss("milestones", "close"), false, false},
	{"gitea", "label", "list", ss("labels", "list"), false, false},
	{"gitea", "label", "create", ss("labels", "create"), false, false},
	// gitea: org unsupported
	{"gitea", "org", "list", nil, false, true},

	// ── Forgejo ──────────────────────────────────────────────────────────────
	// list translates to search for pr/issue/mr
	{"forgejo", "pr", "list", ss("pr", "search"), false, false},
	{"forgejo", "pr", "view", ss("pr", "view"), false, false},
	{"forgejo", "pr", "create", ss("pr", "create"), false, false},
	{"forgejo", "pr", "comment", ss("pr", "comment"), false, false},
	{"forgejo", "mr", "list", ss("pr", "search"), false, false},
	{"forgejo", "mr", "create", ss("pr", "create"), false, false},
	{"forgejo", "issue", "list", ss("issue", "search"), false, false},
	{"forgejo", "issue", "view", ss("issue", "view"), false, false},
	{"forgejo", "issue", "create", ss("issue", "create"), false, false},
	{"forgejo", "issue", "comment", ss("issue", "comment"), false, false},
	{"forgejo", "repo", "view", ss("repo", "view"), false, false},
	{"forgejo", "repo", "create", ss("repo", "create"), false, false},
	{"forgejo", "repo", "browse", ss("repo", "browse"), false, false},
	{"forgejo", "repo", "fork", ss("repo", "fork"), false, false},
	// forgejo: repo list unsupported
	{"forgejo", "repo", "list", nil, false, true},
	{"forgejo", "release", "list", ss("release", "list"), false, false},
	{"forgejo", "release", "create", ss("release", "create"), false, false},
	{"forgejo", "org", "list", ss("org", "list"), false, false},
	{"forgejo", "org", "view", ss("org", "view"), false, false},
	// forgejo: pipeline, milestone, label all unsupported
	{"forgejo", "pipeline", "list", nil, false, true},
	{"forgejo", "milestone", "list", nil, false, true},
	{"forgejo", "label", "list", nil, false, true},

	// ── Unknown subcommand ───────────────────────────────────────────────────
	{"github", "unknown", "list", nil, false, true},
}

func TestTranslate(t *testing.T) {
	for _, tc := range cases {
		name := tc.forge + "/" + tc.subcmd + "/" + tc.verb
		t.Run(name, func(t *testing.T) {
			got, err := Translate(tc.forge, tc.subcmd, tc.verb)
			if tc.wantErr {
				if err == nil {
					t.Fatalf("expected error, got args %v", got.Args)
				}
				return
			}
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if !sliceEqual(got.Args, tc.wantArgs) {
				t.Errorf("Args: got %v, want %v", got.Args, tc.wantArgs)
			}
			if got.DropArgs != tc.wantDrop {
				t.Errorf("DropArgs: got %v, want %v", got.DropArgs, tc.wantDrop)
			}
		})
	}
}

func TestUnsupportedErrorType(t *testing.T) {
	// Unsupported errors must be *UnsupportedError so dispatch can check them.
	cases := []struct{ forge, subcmd, verb string }{
		{"github", "milestone", "list"},
		{"forgejo", "pipeline", "list"},
		{"gitlab", "milestone", "close"},
		{"forgejo", "repo", "list"},
	}
	for _, tc := range cases {
		_, err := Translate(tc.forge, tc.subcmd, tc.verb)
		if err == nil {
			t.Errorf("%s/%s/%s: expected error", tc.forge, tc.subcmd, tc.verb)
			continue
		}
		var ue *UnsupportedError
		if !errors.As(err, &ue) {
			t.Errorf("%s/%s/%s: error is %T, want *UnsupportedError", tc.forge, tc.subcmd, tc.verb, err)
		}
	}
}

// ss is shorthand for []string{...}.
func ss(v ...string) []string { return v }

func sliceEqual(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
