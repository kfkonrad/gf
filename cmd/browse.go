package cmd

import (
	"fmt"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"

	"github.com/derkev/gf/internal/config"
	"github.com/derkev/gf/internal/forge"
)

func printBrowseHelp() {
	fmt.Print(`Usage: gf repo browse [flags]

Open the current repository in a web browser, optionally at a specific
branch, commit, or file path.

Flags:
  -b, --branch <branch>  Browse at a specific branch
  -c, --commit <sha>     Browse at a specific commit
  -p, --path <path>      Browse a specific path.
                         Prefix with :/ for a repo-root-relative path,
                         otherwise resolved from the current directory.
                         Append :<line> to highlight a line (e.g. :/main.go:42).
  -n, --no-browser       Print the URL instead of opening a browser
  -h, --help             Show this help

--branch and --commit are mutually exclusive.
`)
}

// runBrowse implements "gf repo browse" natively without calling the forge CLI.
func runBrowse(args []string) int {
	var (
		rawPath   string
		commit    string
		branch    string
		noBrowser bool
	)

	for i := 0; i < len(args); i++ {
		arg := args[i]
		switch {
		case arg == "--path" || arg == "-p":
			i++
			if i >= len(args) {
				fmt.Fprintln(os.Stderr, "gf: --path requires a value")
				return 2
			}
			rawPath = args[i]
		case strings.HasPrefix(arg, "--path="):
			rawPath = strings.TrimPrefix(arg, "--path=")
		case arg == "--commit" || arg == "-c":
			i++
			if i >= len(args) {
				fmt.Fprintln(os.Stderr, "gf: --commit requires a value")
				return 2
			}
			commit = args[i]
		case strings.HasPrefix(arg, "--commit="):
			commit = strings.TrimPrefix(arg, "--commit=")
		case arg == "--branch" || arg == "-b":
			i++
			if i >= len(args) {
				fmt.Fprintln(os.Stderr, "gf: --branch requires a value")
				return 2
			}
			branch = args[i]
		case strings.HasPrefix(arg, "--branch="):
			branch = strings.TrimPrefix(arg, "--branch=")
		case arg == "--no-browser" || arg == "-n":
			noBrowser = true
		case arg == "--help" || arg == "-h":
			printBrowseHelp()
			return 0
		default:
			fmt.Fprintf(os.Stderr, "gf: repo browse: unknown flag %q\n", arg)
			return 2
		}
	}

	if branch != "" && commit != "" {
		fmt.Fprintln(os.Stderr, "gf: repo browse: --branch and --commit are mutually exclusive")
		return 2
	}

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "gf: error reading config: %v\n", err)
		return 1
	}

	host, repoPath, err := forge.RemoteRepo()
	if err != nil {
		fmt.Fprintf(os.Stderr, "gf: %v\n", err)
		return 1
	}

	entry, ok := cfg.Forges[host]
	if !ok {
		fmt.Fprintf(os.Stderr, "gf: hostname %q not found in config. Run 'gf forge add' to add it.\n", host)
		return 4
	}

	// If a path is given without an explicit ref, resolve to the current branch.
	if rawPath != "" && branch == "" && commit == "" {
		branch, err = getCurrentBranch()
		if err != nil {
			fmt.Fprintf(os.Stderr, "gf: %v\n", err)
			return 1
		}
	}

	filePath, line := "", 0
	if rawPath != "" {
		filePath, line, err = parseBrowsePath(rawPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "gf: %v\n", err)
			return 2
		}
	}

	browseURL := buildBrowseURL(entry.Type, host, repoPath, branch, commit, filePath, line)

	if noBrowser {
		fmt.Println(browseURL)
		return 0
	}

	if err := openBrowser(browseURL); err != nil {
		fmt.Fprintf(os.Stderr, "gf: could not open browser: %v\n", err)
		return 1
	}
	return 0
}

// parseBrowsePath splits a PATH[:LINE] string into the resolved file path and
// an optional line number. If no valid line suffix is present, line is 0.
func parseBrowsePath(raw string) (path string, line int, err error) {
	rawPath := raw
	if idx := strings.LastIndex(raw, ":"); idx > 0 {
		if n, e := strconv.Atoi(raw[idx+1:]); e == nil && n > 0 {
			line = n
			rawPath = raw[:idx]
		}
	}
	path, err = resolveBrowsePath(rawPath)
	return
}

// resolveBrowsePath converts a user-supplied path to a repo-root-relative,
// forward-slash path.
//
// Paths starting with ":/" are taken as repo-root-relative (the prefix is
// stripped). All other paths are resolved relative to the current working
// directory and then made relative to the git repo root.
func resolveBrowsePath(rawPath string) (string, error) {
	if strings.HasPrefix(rawPath, ":/") {
		return strings.TrimPrefix(rawPath, ":/"), nil
	}

	abs, err := filepath.Abs(rawPath)
	if err != nil {
		return "", fmt.Errorf("could not resolve path %q: %w", rawPath, err)
	}

	root, err := getRepoRoot()
	if err != nil {
		return "", err
	}

	rel, err := filepath.Rel(root, abs)
	if err != nil {
		return "", fmt.Errorf("could not resolve path %q relative to repo root: %w", rawPath, err)
	}
	if strings.HasPrefix(rel, "..") {
		return "", fmt.Errorf("path %q is outside the repository", rawPath)
	}
	return filepath.ToSlash(rel), nil
}

// getRepoRoot returns the absolute path to the git repository root.
func getRepoRoot() (string, error) {
	out, err := exec.Command("git", "rev-parse", "--show-toplevel").Output()
	if err != nil {
		return "", fmt.Errorf("could not determine repository root: %w", err)
	}
	return strings.TrimSpace(string(out)), nil
}

// getCurrentBranch returns the name of the current git branch.
// Returns an error when HEAD is detached.
func getCurrentBranch() (string, error) {
	out, err := exec.Command("git", "rev-parse", "--abbrev-ref", "HEAD").Output()
	if err != nil {
		return "", fmt.Errorf("could not determine current branch: %w", err)
	}
	branch := strings.TrimSpace(string(out))
	if branch == "HEAD" {
		return "", fmt.Errorf("in detached HEAD state; use --branch or --commit to specify a reference")
	}
	return branch, nil
}

// buildBrowseURL constructs the forge web URL for the given parameters.
func buildBrowseURL(forgeType, host, repoPath, branch, commit, filePath string, line int) string {
	base := "https://" + host + "/" + repoPath
	switch forgeType {
	case "github":
		return githubBrowseURL(base, branch, commit, filePath, line)
	case "gitlab":
		return gitlabBrowseURL(base, branch, commit, filePath, line)
	default: // gitea, forgejo — identical URL structure
		return giteaBrowseURL(base, branch, commit, filePath, line)
	}
}

func githubBrowseURL(base, branch, commit, filePath string, line int) string {
	if commit != "" {
		if filePath != "" {
			u := base + "/blob/" + encodeURLPath(commit) + "/" + encodeURLPath(filePath)
			if line > 0 {
				u += "#L" + strconv.Itoa(line)
			}
			return u
		}
		return base + "/commit/" + encodeURLPath(commit)
	}
	if branch != "" {
		if filePath != "" {
			u := base + "/blob/" + encodeURLPath(branch) + "/" + encodeURLPath(filePath)
			if line > 0 {
				u += "#L" + strconv.Itoa(line)
			}
			return u
		}
		return base + "/tree/" + encodeURLPath(branch)
	}
	return base
}

func gitlabBrowseURL(base, branch, commit, filePath string, line int) string {
	if commit != "" {
		if filePath != "" {
			u := base + "/-/blob/" + encodeURLPath(commit) + "/" + encodeURLPath(filePath)
			if line > 0 {
				u += "#L" + strconv.Itoa(line)
			}
			return u
		}
		return base + "/-/commit/" + encodeURLPath(commit)
	}
	if branch != "" {
		if filePath != "" {
			u := base + "/-/blob/" + encodeURLPath(branch) + "/" + encodeURLPath(filePath)
			if line > 0 {
				u += "#L" + strconv.Itoa(line)
			}
			return u
		}
		return base + "/-/tree/" + encodeURLPath(branch)
	}
	return base
}

func giteaBrowseURL(base, branch, commit, filePath string, line int) string {
	if commit != "" {
		if filePath != "" {
			u := base + "/src/commit/" + encodeURLPath(commit) + "/" + encodeURLPath(filePath)
			if line > 0 {
				u += "#L" + strconv.Itoa(line)
			}
			return u
		}
		return base + "/src/commit/" + encodeURLPath(commit)
	}
	if branch != "" {
		if filePath != "" {
			u := base + "/src/branch/" + encodeURLPath(branch) + "/" + encodeURLPath(filePath)
			if line > 0 {
				u += "#L" + strconv.Itoa(line)
			}
			return u
		}
		return base + "/src/branch/" + encodeURLPath(branch)
	}
	return base
}

// encodeURLPath percent-encodes each slash-separated segment of a URL path,
// preserving slashes so that multi-segment paths remain valid.
func encodeURLPath(path string) string {
	if path == "" {
		return ""
	}
	parts := strings.Split(path, "/")
	for i, p := range parts {
		parts[i] = url.PathEscape(p)
	}
	return strings.Join(parts, "/")
}

// openBrowser opens the given URL in the default system browser.
//
// Resolution order:
//  1. $BROWSER env var (any platform)
//  2. Platform-native launcher (open on macOS, cmd /C start on Windows)
//  3. On Linux/WSL: wslview, powershell.exe Start-Process, xdg-open,
//     x-www-browser, www-browser, sensible-browser — first one found wins.
func openBrowser(url string) error {
	if browser := os.Getenv("BROWSER"); browser != "" {
		parts := strings.Fields(browser)
		return exec.Command(parts[0], append(parts[1:], url)...).Start()
	}

	switch runtime.GOOS {
	case "darwin":
		return exec.Command("open", url).Start()
	case "windows":
		// cmd /C start handles spaces in URLs correctly with an empty title arg.
		return exec.Command("cmd", "/C", "start", "", url).Start()
	default:
		return openBrowserLinux(url)
	}
}

// openBrowserLinux tries available launchers in order, preferring WSL-aware
// ones when running inside WSL2.
func openBrowserLinux(url string) error {
	var launchers [][]string
	if isWSL() {
		launchers = [][]string{
			{"wslview", url},
			{"powershell.exe", "-NoProfile", "-Command", "Start-Process", url},
		}
	}
	launchers = append(launchers,
		[]string{"xdg-open", url},
		[]string{"x-www-browser", url},
		[]string{"www-browser", url},
		[]string{"sensible-browser", url},
	)

	for _, args := range launchers {
		if path, err := exec.LookPath(args[0]); err == nil {
			return exec.Command(path, args[1:]...).Start()
		}
	}
	return fmt.Errorf("no browser launcher found; set the BROWSER environment variable")
}

// isWSL reports whether the process is running inside WSL by inspecting
// /proc/version, which contains "microsoft" on all WSL variants.
func isWSL() bool {
	data, err := os.ReadFile("/proc/version")
	if err != nil {
		return false
	}
	lower := strings.ToLower(string(data))
	return strings.Contains(lower, "microsoft")
}
