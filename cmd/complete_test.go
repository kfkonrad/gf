package cmd

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/spf13/cobra"
)

// ── parseCobraOutput ─────────────────────────────────────────────────────────

func TestParseCobraOutput(t *testing.T) {
	cases := []struct {
		name      string
		input     string
		wantComps []string
		wantDir   cobra.ShellCompDirective
	}{
		{
			name:      "basic completions with directive",
			input:     "--repo\n--state\n:4\n",
			wantComps: []string{"--repo", "--state"},
			wantDir:   cobra.ShellCompDirectiveNoFileComp,
		},
		{
			name:      "completions with tab-separated descriptions",
			input:     "--repo\tRepository to use\n--state\tFilter by state\n:4\n",
			wantComps: []string{"--repo\tRepository to use", "--state\tFilter by state"},
			wantDir:   cobra.ShellCompDirectiveNoFileComp,
		},
		{
			name:      "directive 0 (ShellCompDirectiveDefault)",
			input:     "foo\nbar\n:0\n",
			wantComps: []string{"foo", "bar"},
			wantDir:   cobra.ShellCompDirectiveDefault,
		},
		{
			name:      "no completions, only directive",
			input:     ":4\n",
			wantComps: []string{},
			wantDir:   cobra.ShellCompDirectiveNoFileComp,
		},
		{
			name:      "no trailing newline",
			input:     "--flag\n:4",
			wantComps: []string{"--flag"},
			wantDir:   cobra.ShellCompDirectiveNoFileComp,
		},
		{
			name:      "missing directive line — defaults to NoFileComp",
			input:     "--flag\n",
			wantComps: []string{"--flag"},
			wantDir:   cobra.ShellCompDirectiveNoFileComp,
		},
		{
			name:      "empty output",
			input:     "",
			wantComps: []string{},
			wantDir:   cobra.ShellCompDirectiveNoFileComp,
		},
		{
			name:      "blank lines are dropped",
			input:     "\n--foo\n\n:4\n",
			wantComps: []string{"--foo"},
			wantDir:   cobra.ShellCompDirectiveNoFileComp,
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			gotComps, gotDir := parseCobraOutput([]byte(tc.input))

			if len(gotComps) != len(tc.wantComps) {
				t.Fatalf("completions: got %v, want %v", gotComps, tc.wantComps)
			}
			for i := range gotComps {
				if gotComps[i] != tc.wantComps[i] {
					t.Errorf("completions[%d]: got %q, want %q", i, gotComps[i], tc.wantComps[i])
				}
			}
			if gotDir != tc.wantDir {
				t.Errorf("directive: got %d, want %d", gotDir, tc.wantDir)
			}
		})
	}
}

// ── parseHelpOutput ──────────────────────────────────────────────────────────

// teaHelp is a representative excerpt of `tea pulls list --help` output.
const teaHelp = `NAME:
   tea pulls list - List pull requests of the repository

USAGE:
   tea pulls list [options]

OPTIONS:
   --fields string, -f string  Comma-separated list of fields to print
   --state string              Filter by state (all|open|closed) (default: open)
   --page int, -p int          specify page (default: 1)
   --limit int, --lm int       specify limit of items per page (default: 30)
   --repo string, -r string    Override local repository path
   --login string, -l string   Use a different Gitea Login. Optional
   --output string, -o string  Output format. (simple, table, csv, tsv, yaml, json)
   --help, -h                  show help

GLOBAL OPTIONS:
   --debug, --vvv  Enable debug mode
`

// fjHelp is a representative excerpt of `fj issue create --help` output.
const fjHelp = `Create a new issue on a repo

Usage: fj issue create [OPTIONS] [TITLE]

Arguments:
  [TITLE]
          Title of the issue

Options:
      --body <BODY>
          The text body of the issue

          Leaving this out will open your editor, unless --body-file is specified.

  -R, --remote <REMOTE>
          The local git remote that points to the repo to operate on

      --body-file <BODY_FILE>
          The text body of the issue, to read from a file

  -r, --repo <REPO>
          The repo to create this issue on

      --web
          Open the issue creation page in your web browser

  -h, --help
          Print help
`

// fjHelpSingleLine is a representative excerpt of `fj repo view --help` output
// where fj uses a compact single-line format for some commands.
const fjHelpSingleLine = `View a repo's info

Usage: fj repo view [OPTIONS] [NAME]

Arguments:
  [NAME]

Options:
  -R, --remote <REMOTE>  The local git remote
  -r, --repo <REPO>      The repo to view
  -h, --help             Print help
`

func TestParseHelpOutput(t *testing.T) {
	t.Run("tea format — all flags, no filter", func(t *testing.T) {
		got := parseHelpOutput([]byte(teaHelp), "")
		want := map[string]bool{
			"--fields": true, "-f": true,
			"--state": true,
			"--page": true, "-p": true,
			"--limit": true, "--lm": true,
			"--repo": true, "-r": true,
			"--login": true, "-l": true,
			"--output": true, "-o": true,
			"--help": true, "-h": true,
			"--debug": true, "--vvv": true,
		}
		for _, c := range got {
			if !want[c] {
				t.Errorf("unexpected completion %q", c)
			}
			delete(want, c)
		}
		for missing := range want {
			t.Errorf("missing completion %q", missing)
		}
	})

	t.Run("tea format — long flag prefix filter", func(t *testing.T) {
		got := parseHelpOutput([]byte(teaHelp), "--l")
		want := map[string]bool{"--limit": true, "--lm": true, "--login": true}
		for _, c := range got {
			if !want[c] {
				t.Errorf("unexpected: %q", c)
			}
			delete(want, c)
		}
		for missing := range want {
			t.Errorf("missing: %q", missing)
		}
	})

	t.Run("tea format — short flag filter", func(t *testing.T) {
		got := parseHelpOutput([]byte(teaHelp), "-p")
		if len(got) != 1 || got[0] != "-p" {
			t.Errorf("got %v, want [-p]", got)
		}
	})

	t.Run("tea format — -- prefix returns only long flags", func(t *testing.T) {
		got := parseHelpOutput([]byte(teaHelp), "--")
		for _, c := range got {
			if !strings.HasPrefix(c, "--") {
				t.Errorf("got short flag %q with '--' prefix", c)
			}
		}
		if len(got) == 0 {
			t.Error("expected long flags, got none")
		}
	})

	t.Run("fj multi-line format — all flags, no filter", func(t *testing.T) {
		got := parseHelpOutput([]byte(fjHelp), "")
		want := map[string]bool{
			"--body": true,
			"--remote": true, "-R": true,
			"--body-file": true,
			"--repo": true, "-r": true,
			"--web": true,
			"--help": true, "-h": true,
		}
		for _, c := range got {
			if !want[c] {
				t.Errorf("unexpected completion %q", c)
			}
			delete(want, c)
		}
		for missing := range want {
			t.Errorf("missing completion %q", missing)
		}
	})

	t.Run("fj multi-line format — no false positive from description text", func(t *testing.T) {
		// "--body-file" appears in the description of --body but must NOT be
		// extracted from that description line (only from its own definition line).
		got := parseHelpOutput([]byte(fjHelp), "")
		count := 0
		for _, c := range got {
			if c == "--body-file" {
				count++
			}
		}
		if count != 1 {
			t.Errorf("--body-file should appear exactly once, got %d times in %v", count, got)
		}
	})

	t.Run("fj single-line format — all flags", func(t *testing.T) {
		got := parseHelpOutput([]byte(fjHelpSingleLine), "")
		want := map[string]bool{
			"--remote": true, "-R": true,
			"--repo": true, "-r": true,
			"--help": true, "-h": true,
		}
		for _, c := range got {
			if !want[c] {
				t.Errorf("unexpected completion %q", c)
			}
			delete(want, c)
		}
		for missing := range want {
			t.Errorf("missing completion %q", missing)
		}
	})

	t.Run("no flags in output", func(t *testing.T) {
		got := parseHelpOutput([]byte("Usage: foo\nDo something\n"), "")
		if len(got) != 0 {
			t.Errorf("expected no completions, got %v", got)
		}
	})

	t.Run("deduplication", func(t *testing.T) {
		// --help and -h appear twice (OPTIONS + GLOBAL OPTIONS)
		input := "OPTIONS:\n   --help, -h  show help\nGLOBAL OPTIONS:\n   --help, -h  show help\n"
		got := parseHelpOutput([]byte(input), "")
		counts := make(map[string]int)
		for _, c := range got {
			counts[c]++
		}
		for flag, n := range counts {
			if n > 1 {
				t.Errorf("flag %q appears %d times, want 1", flag, n)
			}
		}
	})
}

// ── ValidArgsFunction verb filtering ─────────────────────────────────────────

func TestVerbCompletion_Filtering(t *testing.T) {
	var prCmd *cobra.Command
	for _, c := range rootCmd.Commands() {
		if c.Name() == "pr" {
			prCmd = c
			break
		}
	}
	if prCmd == nil {
		t.Fatal("pr command not found")
	}
	fn := prCmd.ValidArgsFunction
	if fn == nil {
		t.Fatal("pr command has no ValidArgsFunction")
	}

	cases := []struct {
		toComplete string
		want       []string
	}{
		{"", []string{"list", "view", "create", "close", "merge", "checkout", "comment"}},
		{"c", []string{"create", "close", "checkout", "comment"}},
		{"co", []string{"comment"}},
		{"cr", []string{"create"}},
		{"list", []string{"list"}},
		{"x", nil},
	}

	for _, tc := range cases {
		t.Run("prefix="+tc.toComplete, func(t *testing.T) {
			got, dir := fn(prCmd, nil, tc.toComplete)
			if dir != cobra.ShellCompDirectiveNoFileComp {
				t.Errorf("directive: got %d, want NoFileComp", dir)
			}
			if len(got) != len(tc.want) {
				t.Fatalf("got %v, want %v", got, tc.want)
			}
			wantSet := make(map[string]bool, len(tc.want))
			for _, w := range tc.want {
				wantSet[w] = true
			}
			for _, g := range got {
				if !wantSet[g] {
					t.Errorf("unexpected completion %q", g)
				}
			}
		})
	}
}

func TestVerbCompletion_InvalidVerbNoDelegate(t *testing.T) {
	var issueCmd *cobra.Command
	for _, c := range rootCmd.Commands() {
		if c.Name() == "issue" {
			issueCmd = c
			break
		}
	}
	if issueCmd == nil {
		t.Fatal("issue command not found")
	}

	got, dir := issueCmd.ValidArgsFunction(issueCmd, []string{"bogus"}, "--")
	if len(got) != 0 {
		t.Errorf("expected no completions for invalid verb, got %v", got)
	}
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("directive: got %d, want NoFileComp", dir)
	}
}

// ── cobraDelegate with a mock binary ─────────────────────────────────────────

func buildCobraMockCLI(t *testing.T, output string) string {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("mock CLI build skipped on Windows")
	}

	src := fmt.Sprintf(`package main
import (
	"fmt"
	"os"
)
func main() {
	if len(os.Args) > 1 && os.Args[1] == "__complete" {
		fmt.Print(%q)
		os.Exit(0)
	}
	os.Exit(1)
}
`, output)

	dir := t.TempDir()
	srcFile := filepath.Join(dir, "mockcobra.go")
	if err := os.WriteFile(srcFile, []byte(src), 0600); err != nil {
		t.Fatal(err)
	}
	bin := filepath.Join(dir, "mockcobra")
	if out, err := exec.Command("go", "build", "-o", bin, srcFile).CombinedOutput(); err != nil {
		t.Fatalf("build mock CLI: %v\n%s", err, out)
	}
	return bin
}

func TestCobraDelegate(t *testing.T) {
	bin := buildCobraMockCLI(t, "--repo\tRepository\n--state\tFilter state\n:4\n")

	comps, dir := cobraDelegate(bin, []string{"pr", "list"}, "--")

	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("directive: got %d, want NoFileComp", dir)
	}
	if len(comps) != 2 {
		t.Fatalf("expected 2 completions, got %v", comps)
	}
	if !strings.Contains(comps[0], "--repo") {
		t.Errorf("first completion: got %q, want --repo", comps[0])
	}
}

func TestCobraDelegate_EmptyOutput(t *testing.T) {
	bin := buildCobraMockCLI(t, "")
	comps, dir := cobraDelegate(bin, []string{"pr", "list"}, "--")
	if len(comps) != 0 {
		t.Errorf("expected no completions, got %v", comps)
	}
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("unexpected directive %d", dir)
	}
}

// ── helpDelegate with a mock binary ──────────────────────────────────────────

// buildHelpMockCLI compiles a mock binary that prints helpOutput when --help
// is among its arguments.
func buildHelpMockCLI(t *testing.T, helpOutput string) string {
	t.Helper()
	if runtime.GOOS == "windows" {
		t.Skip("mock CLI build skipped on Windows")
	}

	src := fmt.Sprintf(`package main
import (
	"fmt"
	"os"
)
func main() {
	for _, a := range os.Args[1:] {
		if a == "--help" {
			fmt.Print(%q)
			os.Exit(0)
		}
	}
	os.Exit(1)
}
`, helpOutput)

	dir := t.TempDir()
	srcFile := filepath.Join(dir, "mockhelp.go")
	if err := os.WriteFile(srcFile, []byte(src), 0600); err != nil {
		t.Fatal(err)
	}
	bin := filepath.Join(dir, "mockhelp")
	if out, err := exec.Command("go", "build", "-o", bin, srcFile).CombinedOutput(); err != nil {
		t.Fatalf("build mock help CLI: %v\n%s", err, out)
	}
	return bin
}

func TestHelpDelegate_TeaFormat(t *testing.T) {
	bin := buildHelpMockCLI(t, teaHelp)

	cases := []struct {
		toComplete string
		mustContain []string
		mustExclude []string
	}{
		{"--", []string{"--state", "--limit", "--fields"}, []string{"-f", "-p"}},
		{"-f", []string{"-f"}, []string{"--fields", "--state"}},
		{"--l", []string{"--limit", "--lm", "--login"}, []string{"--state", "-l"}},
		{"", []string{"--state", "-f", "--debug", "--vvv"}, nil},
	}

	for _, tc := range cases {
		t.Run("toComplete="+tc.toComplete, func(t *testing.T) {
			comps, dir := helpDelegate(bin, []string{"pulls", "list"}, tc.toComplete)
			if dir != cobra.ShellCompDirectiveNoFileComp {
				t.Errorf("directive: got %d, want NoFileComp", dir)
			}
			got := make(map[string]bool, len(comps))
			for _, c := range comps {
				got[c] = true
			}
			for _, must := range tc.mustContain {
				if !got[must] {
					t.Errorf("missing %q in %v", must, comps)
				}
			}
			for _, must := range tc.mustExclude {
				if got[must] {
					t.Errorf("should not contain %q in %v", must, comps)
				}
			}
		})
	}
}

func TestHelpDelegate_FjFormat(t *testing.T) {
	bin := buildHelpMockCLI(t, fjHelp)

	comps, dir := helpDelegate(bin, []string{"issue", "create"}, "")
	if dir != cobra.ShellCompDirectiveNoFileComp {
		t.Errorf("directive: got %d, want NoFileComp", dir)
	}
	got := make(map[string]bool, len(comps))
	for _, c := range comps {
		got[c] = true
	}
	for _, want := range []string{"--body", "--remote", "-R", "--body-file", "--repo", "-r", "--web", "--help", "-h"} {
		if !got[want] {
			t.Errorf("missing %q; got %v", want, comps)
		}
	}
}

func TestHelpDelegate_NoHelp(t *testing.T) {
	// Mock that doesn't respond to --help: should return no completions.
	if runtime.GOOS == "windows" {
		t.Skip("mock CLI build skipped on Windows")
	}
	src := `package main; import "os"; func main() { os.Exit(1) }`
	dir := t.TempDir()
	srcFile := filepath.Join(dir, "noop.go")
	if err := os.WriteFile(srcFile, []byte(src), 0600); err != nil {
		t.Fatal(err)
	}
	bin := filepath.Join(dir, "noop")
	if out, err := exec.Command("go", "build", "-o", bin, srcFile).CombinedOutput(); err != nil {
		t.Fatalf("build noop binary: %v\n%s", err, out)
	}

	comps, _ := helpDelegate(bin, []string{"issues"}, "")
	if len(comps) != 0 {
		t.Errorf("expected no completions, got %v", comps)
	}
}
