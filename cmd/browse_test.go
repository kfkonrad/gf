package cmd

import (
	"strings"
	"testing"

	"github.com/spf13/cobra"
)

func TestBuildBrowseURL(t *testing.T) {
	cases := []struct {
		name      string
		forgeType string
		host      string
		repoPath  string
		branch    string
		commit    string
		filePath  string
		line      int
		want      string
	}{
		// ── GitHub ───────────────────────────────────────────────────────────
		{"github root", "github", "github.com", "owner/repo", "", "", "", 0,
			"https://github.com/owner/repo"},
		{"github branch", "github", "github.com", "owner/repo", "main", "", "", 0,
			"https://github.com/owner/repo/tree/main"},
		{"github branch+path", "github", "github.com", "owner/repo", "main", "", "src/foo.go", 0,
			"https://github.com/owner/repo/tree/main/src/foo.go"},
		{"github branch+path+line", "github", "github.com", "owner/repo", "main", "", "src/foo.go", 42,
			"https://github.com/owner/repo/blob/main/src/foo.go#L42"},
		{"github commit", "github", "github.com", "owner/repo", "", "abc123", "", 0,
			"https://github.com/owner/repo/commit/abc123"},
		{"github commit+path", "github", "github.com", "owner/repo", "", "abc123", "src/foo.go", 0,
			"https://github.com/owner/repo/blob/abc123/src/foo.go"},
		{"github commit+path+line", "github", "github.com", "owner/repo", "", "abc123", "src/foo.go", 10,
			"https://github.com/owner/repo/blob/abc123/src/foo.go#L10"},

		// ── GitLab ───────────────────────────────────────────────────────────
		{"gitlab root", "gitlab", "gitlab.com", "group/repo", "", "", "", 0,
			"https://gitlab.com/group/repo"},
		{"gitlab branch", "gitlab", "gitlab.com", "group/repo", "main", "", "", 0,
			"https://gitlab.com/group/repo/-/tree/main"},
		{"gitlab branch+path", "gitlab", "gitlab.com", "group/repo", "main", "", "lib/a.rb", 0,
			"https://gitlab.com/group/repo/-/tree/main/lib/a.rb"},
		{"gitlab branch+path+line", "gitlab", "gitlab.com", "group/repo", "main", "", "lib/a.rb", 5,
			"https://gitlab.com/group/repo/-/blob/main/lib/a.rb#L5"},
		{"gitlab commit", "gitlab", "gitlab.com", "group/repo", "", "deadbeef", "", 0,
			"https://gitlab.com/group/repo/-/commit/deadbeef"},
		{"gitlab commit+path", "gitlab", "gitlab.com", "group/repo", "", "deadbeef", "lib/a.rb", 0,
			"https://gitlab.com/group/repo/-/blob/deadbeef/lib/a.rb"},
		{"gitlab subgroup", "gitlab", "gitlab.com", "org/sub/repo", "main", "", "", 0,
			"https://gitlab.com/org/sub/repo/-/tree/main"},

		// ── Gitea ────────────────────────────────────────────────────────────
		{"gitea root", "gitea", "gitea.com", "user/repo", "", "", "", 0,
			"https://gitea.com/user/repo"},
		{"gitea branch", "gitea", "gitea.com", "user/repo", "develop", "", "", 0,
			"https://gitea.com/user/repo/src/branch/develop"},
		{"gitea branch+path", "gitea", "gitea.com", "user/repo", "develop", "", "main.go", 0,
			"https://gitea.com/user/repo/src/branch/develop/main.go"},
		{"gitea branch+path+line", "gitea", "gitea.com", "user/repo", "develop", "", "main.go", 7,
			"https://gitea.com/user/repo/src/branch/develop/main.go#L7"},
		{"gitea commit", "gitea", "gitea.com", "user/repo", "", "cafebabe", "", 0,
			"https://gitea.com/user/repo/src/commit/cafebabe"},
		{"gitea commit+path", "gitea", "gitea.com", "user/repo", "", "cafebabe", "main.go", 0,
			"https://gitea.com/user/repo/src/commit/cafebabe/main.go"},

		// ── Forgejo (same URL structure as Gitea) ────────────────────────────
		{"forgejo root", "forgejo", "codeberg.org", "user/repo", "", "", "", 0,
			"https://codeberg.org/user/repo"},
		{"forgejo branch", "forgejo", "codeberg.org", "user/repo", "feature", "", "", 0,
			"https://codeberg.org/user/repo/src/branch/feature"},
		{"forgejo commit+path+line", "forgejo", "codeberg.org", "user/repo", "", "deadc0de", "a/b.go", 3,
			"https://codeberg.org/user/repo/src/commit/deadc0de/a/b.go#L3"},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := buildBrowseURL(tc.forgeType, tc.host, tc.repoPath, tc.branch, tc.commit, tc.filePath, tc.line)
			if got != tc.want {
				t.Errorf("got  %q\nwant %q", got, tc.want)
			}
		})
	}
}

func TestParseBrowsePath(t *testing.T) {
	// Only test the ":/" prefix branch, which doesn't need git to resolve.
	cases := []struct {
		raw      string
		wantPath string
		wantLine int
	}{
		{":/src/main.go", "src/main.go", 0},
		{":/src/main.go:42", "src/main.go", 42},
		{":/dir/sub/file.go:1", "dir/sub/file.go", 1},
		// Not a valid line number — treated as part of path name
		{":/file.go:notanumber", "file.go:notanumber", 0},
		// Line 0 is invalid — treated as part of path
		{":/file.go:0", "file.go:0", 0},
	}

	for _, tc := range cases {
		t.Run(tc.raw, func(t *testing.T) {
			path, line, err := parseBrowsePath(tc.raw)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if path != tc.wantPath {
				t.Errorf("path: got %q, want %q", path, tc.wantPath)
			}
			if line != tc.wantLine {
				t.Errorf("line: got %d, want %d", line, tc.wantLine)
			}
		})
	}
}

func TestBrowseCompletions_FlagNames(t *testing.T) {
	all, dir := browseCompletions(nil, "")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("directive: got %v, want NoFileComp", dir)
	}
	expected := map[string]bool{"--branch": true, "--commit": true, "--no-browser": true, "--path": true}
	if len(all) != len(expected) {
		t.Fatalf("got %d completions, want %d: %v", len(all), len(expected), all)
	}
	for _, c := range all {
		if !expected[c] {
			t.Errorf("unexpected completion %q", c)
		}
	}

	// Prefix filtering
	filtered, _ := browseCompletions(nil, "--b")
	if len(filtered) != 1 || filtered[0] != "--branch" {
		t.Errorf("prefix --b: got %v, want [--branch]", filtered)
	}

	filtered, _ = browseCompletions(nil, "--no")
	if len(filtered) != 1 || filtered[0] != "--no-browser" {
		t.Errorf("prefix --no: got %v, want [--no-browser]", filtered)
	}
}

func TestBrowseCompletions_AlreadyUsedFlags(t *testing.T) {
	// After --branch <value>, --branch should not appear in flag suggestions.
	comps, _ := browseCompletions([]string{"--branch", "main"}, "")
	for _, c := range comps {
		if c == "--branch" {
			t.Errorf("--branch appeared again after it was already used")
		}
	}

	// After -b <value>, --branch still gone.
	comps, _ = browseCompletions([]string{"-b", "main"}, "")
	for _, c := range comps {
		if c == "--branch" {
			t.Errorf("--branch appeared again after -b was used")
		}
	}

	// After --no-browser, it should be gone.
	comps, _ = browseCompletions([]string{"--no-browser"}, "")
	for _, c := range comps {
		if c == "--no-browser" {
			t.Errorf("--no-browser appeared again after it was already used")
		}
	}

	// After --branch=main (= style), --branch should be gone.
	comps, _ = browseCompletions([]string{"--branch=main"}, "")
	for _, c := range comps {
		if c == "--branch" {
			t.Errorf("--branch appeared again after --branch= was used")
		}
	}

	// Remaining flags should still appear.
	comps, _ = browseCompletions([]string{"--branch", "main"}, "")
	remaining := map[string]bool{}
	for _, c := range comps {
		remaining[c] = true
	}
	for _, want := range []string{"--no-browser", "--path"} {
		if !remaining[want] {
			t.Errorf("%q missing from completions after --branch was used; got %v", want, comps)
		}
	}
}

func TestBrowseCompletions_MutualExclusion(t *testing.T) {
	// After --branch, --commit must not appear.
	comps, _ := browseCompletions([]string{"--branch", "main"}, "")
	for _, c := range comps {
		if c == "--commit" {
			t.Errorf("--commit appeared after --branch was used")
		}
	}

	// After -b, --commit must not appear.
	comps, _ = browseCompletions([]string{"-b", "main"}, "")
	for _, c := range comps {
		if c == "--commit" {
			t.Errorf("--commit appeared after -b was used")
		}
	}

	// After --commit, --branch must not appear.
	comps, _ = browseCompletions([]string{"--commit", "abc123"}, "")
	for _, c := range comps {
		if c == "--branch" {
			t.Errorf("--branch appeared after --commit was used")
		}
	}

	// After --branch=main (= style), --commit must not appear.
	comps, _ = browseCompletions([]string{"--branch=main"}, "")
	for _, c := range comps {
		if c == "--commit" {
			t.Errorf("--commit appeared after --branch= was used")
		}
	}
}

func TestBrowseCompletions_ValueDirectives(t *testing.T) {
	// --branch value (space style): NoFileComp, no completions in test env (no git)
	_, dir := browseCompletions([]string{"--branch"}, "")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("--branch value: want NoFileComp, got %v", dir)
	}
	_, dir = browseCompletions([]string{"-b"}, "")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("-b value: want NoFileComp, got %v", dir)
	}

	// --path :/ value → NoFileComp (repo tree completion)
	_, dir = browseCompletions([]string{"--path"}, ":/")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("--path :/ value: want NoFileComp, got %v", dir)
	}
	_, dir = browseCompletions([]string{"-p"}, ":/src/")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("-p :/ value: want NoFileComp, got %v", dir)
	}

	// --path relative value → Default (filesystem completion)
	_, dir = browseCompletions([]string{"--path"}, "src/")
	if dir != cobra.ShellCompDirectiveDefault {
		t.Errorf("--path relative value: want Default, got %v", dir)
	}
	_, dir = browseCompletions([]string{"--path"}, "")
	if dir != cobra.ShellCompDirectiveDefault {
		t.Errorf("--path empty value: want Default, got %v", dir)
	}

	// --commit value → NoFileComp, no completions
	comps, dir := browseCompletions([]string{"--commit"}, "")
	if dir != cobra.ShellCompDirectiveNoFileComp || len(comps) != 0 {
		t.Errorf("--commit value: want (nil, NoFileComp), got (%v, %v)", comps, dir)
	}
	_, dir = browseCompletions([]string{"-c"}, "")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("-c value: want NoFileComp, got %v", dir)
	}
}

func TestRepoPathCompletions_Paths(t *testing.T) {
	// Run against the actual repo. git ls-tree returns full repo-root-relative
	// paths; completions must have the form ":/<full-path>", not ":/dir/dir/file".
	comps, dir := repoPathCompletions(":/")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("directive: want NoFileComp, got %v", dir)
	}
	if len(comps) == 0 {
		t.Skip("no completions returned (not a git repo or no HEAD?)")
	}
	for _, c := range comps {
		if !strings.HasPrefix(c, ":/") {
			t.Errorf("completion %q does not start with :/", c)
		}
		// Must not contain a doubled path segment like ":/cmd/cmd/"
		inner := strings.TrimPrefix(c, ":/")
		parts := strings.SplitN(inner, "/", 2)
		if len(parts) == 2 && parts[0] == parts[1] {
			t.Errorf("doubled path segment in %q", c)
		}
	}

	// Subdirectory: all results must start with ":/cmd/"
	cmdComps, _ := repoPathCompletions(":/cmd/")
	for _, c := range cmdComps {
		if !strings.HasPrefix(c, ":/cmd/") {
			t.Errorf(":/cmd/ completion %q does not start with :/cmd/", c)
		}
	}
	if len(cmdComps) == 0 {
		t.Error("expected completions for :/cmd/")
	}
}

func TestBrowseCompletions_EqualStyle(t *testing.T) {
	// --branch=VALUE style: NoFileComp
	_, dir := browseCompletions(nil, "--branch=")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("--branch= style: want NoFileComp, got %v", dir)
	}

	// --path=:/ style: NoFileComp (repo tree)
	_, dir = browseCompletions(nil, "--path=:/")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("--path=:/ style: want NoFileComp, got %v", dir)
	}

	// --path=relative style: Default (filesystem)
	_, dir = browseCompletions(nil, "--path=src/")
	if dir != cobra.ShellCompDirectiveDefault {
		t.Errorf("--path=relative style: want Default, got %v", dir)
	}
	_, dir = browseCompletions(nil, "--path=")
	if dir != cobra.ShellCompDirectiveDefault {
		t.Errorf("--path= empty style: want Default, got %v", dir)
	}
}
