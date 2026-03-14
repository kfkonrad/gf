package forge_test

import (
	"testing"

	"github.com/derkev/gf/internal/forge"
)

func TestParseRepo(t *testing.T) {
	cases := []struct {
		input        string
		wantHost     string
		wantRepoPath string
		wantErr      bool
	}{
		{"https://github.com/owner/repo.git", "github.com", "owner/repo", false},
		{"https://github.com/owner/repo", "github.com", "owner/repo", false},
		{"git@github.com:owner/repo.git", "github.com", "owner/repo", false},
		{"git@gitlab.com:group/subgroup/repo.git", "gitlab.com", "group/subgroup/repo", false},
		{"ssh://git@github.com/owner/repo.git", "github.com", "owner/repo", false},
		{"https://git.corp.com:8080/owner/repo.git", "git.corp.com", "owner/repo", false},
		// Trailing newline from git output
		{"git@github.com:owner/repo.git\n", "github.com", "owner/repo", false},
		// Errors
		{"git@missing-colon", "", "", true},
		{"not-a-url", "", "", true},
	}

	for _, tc := range cases {
		t.Run(tc.input, func(t *testing.T) {
			host, repoPath, err := forge.ParseRepo(tc.input)
			if tc.wantErr {
				if err == nil {
					t.Fatalf("expected error, got host=%q repoPath=%q", host, repoPath)
				}
				return
			}
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if host != tc.wantHost {
				t.Errorf("host: got %q, want %q", host, tc.wantHost)
			}
			if repoPath != tc.wantRepoPath {
				t.Errorf("repoPath: got %q, want %q", repoPath, tc.wantRepoPath)
			}
		})
	}
}

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
