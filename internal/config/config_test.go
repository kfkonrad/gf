package config_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/derkev/gf/internal/config"
)

// TestDir_XDG verifies the XDG_CONFIG_HOME override on Linux.
func TestDir_XDG(t *testing.T) {
	t.Setenv("XDG_CONFIG_HOME", "/tmp/xdg-test")
	got, err := config.Dir()
	if err != nil {
		t.Fatal(err)
	}
	want := "/tmp/xdg-test/gf"
	if got != want {
		t.Errorf("Dir() = %q, want %q", got, want)
	}
}

// TestDir_Default verifies the fallback to ~/.config/gf when XDG_CONFIG_HOME is unset.
func TestDir_Default(t *testing.T) {
	t.Setenv("XDG_CONFIG_HOME", "")
	got, err := config.Dir()
	if err != nil {
		t.Fatal(err)
	}
	home, _ := os.UserHomeDir()
	want := filepath.Join(home, ".config", "gf")
	if got != want {
		t.Errorf("Dir() = %q, want %q", got, want)
	}
}

// TestLoad_Missing bootstraps the default config when the file does not exist.
func TestLoad_Missing(t *testing.T) {
	t.Setenv("XDG_CONFIG_HOME", t.TempDir())
	cfg, err := config.Load()
	if err != nil {
		t.Fatal(err)
	}
	want := map[string]struct {
		typ string
		cli string
	}{
		"github.com":  {"github", "gh"},
		"gitlab.com":  {"gitlab", "glab"},
		"gitea.com":   {"gitea", "tea"},
		"forgejo.org": {"forgejo", "fj"},
	}
	if len(cfg.Forges) != len(want) {
		t.Fatalf("expected %d default forges, got %d: %v", len(want), len(cfg.Forges), cfg.Forges)
	}
	for host, w := range want {
		e, ok := cfg.Forges[host]
		if !ok {
			t.Errorf("missing default forge %q", host)
			continue
		}
		if e.Type != w.typ {
			t.Errorf("%s: type = %q, want %q", host, e.Type, w.typ)
		}
		if e.CLI != w.cli {
			t.Errorf("%s: cli = %q, want %q", host, e.CLI, w.cli)
		}
	}

	// Config file must have been written to disk.
	dir, _ := config.Dir()
	if _, err := os.Stat(filepath.Join(dir, "config.yaml")); err != nil {
		t.Errorf("config file not created on first load: %v", err)
	}
}

// TestLoad_Valid parses a well-formed config file.
func TestLoad_Valid(t *testing.T) {
	dir := t.TempDir()
	t.Setenv("XDG_CONFIG_HOME", dir)

	yaml := `forges:
  github.com:
    type: github
  git.corp.com:
    type: gitea
    cli: /usr/local/bin/tea
`
	if err := os.MkdirAll(filepath.Join(dir, "gf"), 0700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "gf", "config.yaml"), []byte(yaml), 0600); err != nil {
		t.Fatal(err)
	}

	cfg, err := config.Load()
	if err != nil {
		t.Fatal(err)
	}
	if len(cfg.Forges) != 2 {
		t.Fatalf("expected 2 forges, got %d", len(cfg.Forges))
	}
	gh := cfg.Forges["github.com"]
	if gh.Type != "github" || gh.CLI != "" {
		t.Errorf("github.com entry: got %+v", gh)
	}
	corp := cfg.Forges["git.corp.com"]
	if corp.Type != "gitea" || corp.CLI != "/usr/local/bin/tea" {
		t.Errorf("git.corp.com entry: got %+v", corp)
	}
}

// TestPath_EnvOverride verifies that GF_CONFIG overrides the platform default.
func TestPath_EnvOverride(t *testing.T) {
	custom := filepath.Join(t.TempDir(), "custom.yaml")
	t.Setenv("GF_CONFIG", custom)
	got, err := config.Path()
	if err != nil {
		t.Fatal(err)
	}
	if got != custom {
		t.Errorf("Path() = %q, want %q", got, custom)
	}
}

// TestLoad_EnvOverride uses GF_CONFIG to point Load/Save at a custom path.
func TestLoad_EnvOverride(t *testing.T) {
	custom := filepath.Join(t.TempDir(), "subdir", "my.yaml")
	t.Setenv("GF_CONFIG", custom)

	// First load: file absent → defaults written to custom path.
	cfg, err := config.Load()
	if err != nil {
		t.Fatal(err)
	}
	if _, ok := cfg.Forges["github.com"]; !ok {
		t.Fatal("expected default forges to be present")
	}
	if _, err := os.Stat(custom); err != nil {
		t.Errorf("config not written to custom path: %v", err)
	}

	// Second load: reads back from the custom path.
	cfg2, err := config.Load()
	if err != nil {
		t.Fatal(err)
	}
	if len(cfg2.Forges) != len(cfg.Forges) {
		t.Errorf("reload mismatch: got %d forges, want %d", len(cfg2.Forges), len(cfg.Forges))
	}
}

// TestLoad_InvalidYAML returns an error for malformed YAML.
func TestLoad_InvalidYAML(t *testing.T) {
	dir := t.TempDir()
	t.Setenv("XDG_CONFIG_HOME", dir)

	if err := os.MkdirAll(filepath.Join(dir, "gf"), 0700); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "gf", "config.yaml"), []byte(":\nbroken: [yaml"), 0600); err != nil {
		t.Fatal(err)
	}

	_, err := config.Load()
	if err == nil {
		t.Fatal("expected error for invalid YAML")
	}
}

// TestSave roundtrips a config through Save then Load.
func TestSave(t *testing.T) {
	dir := t.TempDir()
	t.Setenv("XDG_CONFIG_HOME", dir)

	original := &config.Config{
		Forges: map[string]config.ForgeEntry{
			"github.com":   {Type: "github"},
			"codeberg.org": {Type: "forgejo", CLI: "fj"},
		},
	}

	if err := config.Save(original); err != nil {
		t.Fatal(err)
	}

	// Check file permissions.
	info, err := os.Stat(filepath.Join(dir, "gf", "config.yaml"))
	if err != nil {
		t.Fatal(err)
	}
	if perm := info.Mode().Perm(); perm != 0600 {
		t.Errorf("file permissions = %o, want 0600", perm)
	}

	// Reload and compare.
	loaded, err := config.Load()
	if err != nil {
		t.Fatal(err)
	}
	if len(loaded.Forges) != 2 {
		t.Fatalf("expected 2 forges after reload, got %d", len(loaded.Forges))
	}
	if loaded.Forges["github.com"].Type != "github" {
		t.Errorf("github.com type mismatch")
	}
	if loaded.Forges["codeberg.org"].CLI != "fj" {
		t.Errorf("codeberg.org CLI mismatch")
	}
}

// TestSave_CreatesDir verifies Save creates the config directory if absent.
func TestSave_CreatesDir(t *testing.T) {
	dir := t.TempDir()
	// Point XDG at a subdirectory that doesn't exist yet.
	t.Setenv("XDG_CONFIG_HOME", filepath.Join(dir, "nonexistent"))

	cfg := &config.Config{Forges: map[string]config.ForgeEntry{
		"gitlab.com": {Type: "gitlab"},
	}}
	if err := config.Save(cfg); err != nil {
		t.Fatalf("Save failed: %v", err)
	}
}

// TestEffectiveCLI returns the override CLI when set, otherwise the default.
func TestEffectiveCLI(t *testing.T) {
	cases := []struct {
		entry config.ForgeEntry
		want  string
	}{
		{config.ForgeEntry{Type: "github"}, "gh"},
		{config.ForgeEntry{Type: "gitlab"}, "glab"},
		{config.ForgeEntry{Type: "gitea"}, "tea"},
		{config.ForgeEntry{Type: "forgejo"}, "fj"},
		{config.ForgeEntry{Type: "github", CLI: "gh-enterprise"}, "gh-enterprise"},
		{config.ForgeEntry{Type: "gitea", CLI: "/usr/local/bin/tea"}, "/usr/local/bin/tea"},
	}
	for _, tc := range cases {
		got := config.EffectiveCLI(tc.entry)
		if got != tc.want {
			t.Errorf("EffectiveCLI(%+v) = %q, want %q", tc.entry, got, tc.want)
		}
	}
}
