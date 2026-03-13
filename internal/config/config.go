package config

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"gopkg.in/yaml.v3"
)

// ForgeEntry is a single forge configuration entry.
type ForgeEntry struct {
	Type string `yaml:"type"`
	CLI  string `yaml:"cli,omitempty"`
}

// Config is the top-level configuration.
type Config struct {
	Forges map[string]ForgeEntry `yaml:"forges"`
}

// DefaultCLIs maps forge type to default CLI binary name.
var DefaultCLIs = map[string]string{
	"github":  "gh",
	"gitlab":  "glab",
	"gitea":   "tea",
	"forgejo": "fj",
}

// ValidTypes is the set of valid forge types.
var ValidTypes = []string{"github", "gitlab", "gitea", "forgejo"}

// Dir returns the gf config directory.
func Dir() (string, error) {
	switch runtime.GOOS {
	case "windows":
		appData := os.Getenv("APPDATA")
		if appData == "" {
			return "", fmt.Errorf("APPDATA environment variable not set")
		}
		return filepath.Join(appData, "gf"), nil
	case "darwin":
		home, err := os.UserHomeDir()
		if err != nil {
			return "", err
		}
		return filepath.Join(home, "Library", "Application Support", "gf"), nil
	default: // Linux and others
		xdgConfigHome := os.Getenv("XDG_CONFIG_HOME")
		if xdgConfigHome != "" {
			return filepath.Join(xdgConfigHome, "gf"), nil
		}
		home, err := os.UserHomeDir()
		if err != nil {
			return "", err
		}
		return filepath.Join(home, ".config", "gf"), nil
	}
}

// Load reads the config file. Returns an empty Config if the file doesn't exist.
func Load() (*Config, error) {
	dir, err := Dir()
	if err != nil {
		return nil, err
	}
	path := filepath.Join(dir, "config.yaml")

	data, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return &Config{Forges: make(map[string]ForgeEntry)}, nil
	}
	if err != nil {
		return nil, fmt.Errorf("read config: %w", err)
	}

	var cfg Config
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("parse config: %w", err)
	}
	if cfg.Forges == nil {
		cfg.Forges = make(map[string]ForgeEntry)
	}
	return &cfg, nil
}

// Save writes the config file with 0600 permissions.
func Save(cfg *Config) error {
	dir, err := Dir()
	if err != nil {
		return err
	}
	if err := os.MkdirAll(dir, 0700); err != nil {
		return fmt.Errorf("create config dir: %w", err)
	}

	data, err := yaml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("marshal config: %w", err)
	}

	path := filepath.Join(dir, "config.yaml")
	return os.WriteFile(path, data, 0600)
}

// EffectiveCLI returns the CLI to use for a forge entry (override or default).
func EffectiveCLI(entry ForgeEntry) string {
	if entry.CLI != "" {
		return entry.CLI
	}
	return DefaultCLIs[entry.Type]
}
