package forge

import (
	"fmt"
	"net/url"
	"os/exec"
	"strings"
)

// RemoteHost extracts the hostname from the current repo's origin remote URL.
func RemoteHost() (string, error) {
	out, err := exec.Command("git", "remote", "get-url", "origin").Output()
	if err != nil {
		return "", fmt.Errorf("could not get git remote origin: not in a git repository or no origin remote configured")
	}
	return ParseHost(strings.TrimSpace(string(out)))
}

// ParseHost extracts the hostname from a git remote URL.
// Supports https://, git@, and ssh:// formats.
func ParseHost(remoteURL string) (string, error) {
	remoteURL = strings.TrimSpace(remoteURL)

	// Handle SCP-style: git@host:path
	if strings.HasPrefix(remoteURL, "git@") {
		rest := strings.TrimPrefix(remoteURL, "git@")
		host, _, found := strings.Cut(rest, ":")
		if !found {
			return "", fmt.Errorf("invalid SCP-style URL: %s", remoteURL)
		}
		return host, nil
	}

	// Handle https:// and ssh://
	u, err := url.Parse(remoteURL)
	if err != nil {
		return "", fmt.Errorf("invalid remote URL %q: %w", remoteURL, err)
	}
	if u.Host == "" {
		return "", fmt.Errorf("could not extract hostname from URL: %s", remoteURL)
	}
	return u.Hostname(), nil
}
