package forge_test

import (
	"testing"

	"gf/internal/forge"
)

func TestParseHost(t *testing.T) {
	cases := []struct {
		input   string
		want    string
		wantErr bool
	}{
		// HTTPS URLs
		{"https://github.com/user/repo.git", "github.com", false},
		{"https://gitlab.com/user/repo", "gitlab.com", false},
		{"https://git.example.com/user/repo.git", "git.example.com", false},
		// HTTPS with port
		{"https://git.corp.com:8080/user/repo.git", "git.corp.com", false},
		// SCP-style git@ URLs
		{"git@github.com:user/repo.git", "github.com", false},
		{"git@gitlab.com:org/repo.git", "gitlab.com", false},
		{"git@codeberg.org:user/repo.git", "codeberg.org", false},
		// SSH URLs
		{"ssh://git@github.com/user/repo.git", "github.com", false},
		{"ssh://git@git.corp.com:2222/user/repo.git", "git.corp.com", false},
		// Trailing whitespace/newlines (from git output)
		{"https://github.com/user/repo.git\n", "github.com", false},
		{"git@github.com:user/repo.git\n", "github.com", false},
		// Errors
		{"git@missing-colon", "", true},
		{"not-a-url", "", true},
	}

	for _, tc := range cases {
		t.Run(tc.input, func(t *testing.T) {
			got, err := forge.ParseHost(tc.input)
			if tc.wantErr {
				if err == nil {
					t.Fatalf("expected error, got %q", got)
				}
				return
			}
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if got != tc.want {
				t.Errorf("got %q, want %q", got, tc.want)
			}
		})
	}
}
