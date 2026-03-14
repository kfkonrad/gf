package cmd

import (
	"context"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
	"time"

	"gf/internal/config"
	"gf/internal/forge"
	"gf/internal/translate"

	"github.com/spf13/cobra"
)

const completionTimeout = 5 * time.Second

// browseCompletions returns completions for the native "gf repo browse".
//
// It handles three cases:
//   - Flag name completion when no value is being typed yet.
//   - Flag value completion for --branch/-b  (remote branch names from git).
//   - Flag value completion for --path/-p    (repo-tree paths for :/ prefix;
//     filesystem completion for plain relative paths).
//   - --commit/-c deliberately produces no completions (hashes are not guessable).
//
// Both "--flag value" and "--flag=value" styles are handled.
// Flags already present in remainingArgs are excluded from flag-name suggestions.
func browseCompletions(remainingArgs []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	// --branch=VALUE style
	if after, ok := strings.CutPrefix(toComplete, "--branch="); ok {
		branches, _ := remoteBranchCompletions(after)
		for i, b := range branches {
			branches[i] = "--branch=" + b
		}
		return branches, cobra.ShellCompDirectiveNoFileComp
	}
	// --path=:/ style — complete from the repo tree, prepend the flag prefix back
	if after, ok := strings.CutPrefix(toComplete, "--path=:/"); ok {
		results, _ := repoPathCompletions(":/" + after)
		for i, r := range results {
			results[i] = "--path=" + r
		}
		return results, cobra.ShellCompDirectiveNoFileComp
	}
	// --path=<relative> style — fall back to shell file completion
	if strings.HasPrefix(toComplete, "--path=") {
		return nil, cobra.ShellCompDirectiveDefault
	}

	// "--flag VALUE" style: look at the previous arg to determine what we're completing
	if len(remainingArgs) > 0 {
		switch remainingArgs[len(remainingArgs)-1] {
		case "--branch", "-b":
			return remoteBranchCompletions(toComplete)
		case "--path", "-p":
			if strings.HasPrefix(toComplete, ":/") {
				return repoPathCompletions(toComplete)
			}
			return nil, cobra.ShellCompDirectiveDefault
		case "--commit", "-c":
			return nil, cobra.ShellCompDirectiveNoFileComp
		}
	}

	// Build the set of flags already present in remainingArgs so we don't
	// suggest the same flag twice.
	used := make(map[string]bool)
	for _, a := range remainingArgs {
		switch a {
		case "--branch", "-b":
			used["--branch"] = true
		case "--commit", "-c":
			used["--commit"] = true
		case "--no-browser", "-n":
			used["--no-browser"] = true
		case "--path", "-p":
			used["--path"] = true
		default:
			// handle --flag=value forms
			for _, long := range []string{"--branch=", "--commit=", "--path="} {
				if strings.HasPrefix(a, long) {
					used[strings.TrimSuffix(long, "=")] = true
				}
			}
		}
	}
	// --branch and --commit are mutually exclusive.
	if used["--branch"] {
		used["--commit"] = true
	}
	if used["--commit"] {
		used["--branch"] = true
	}

	// Complete flag names, excluding already-used ones.
	flags := []string{"--branch", "--commit", "--no-browser", "--path"}
	var completions []string
	for _, f := range flags {
		if !used[f] && strings.HasPrefix(f, toComplete) {
			completions = append(completions, f)
		}
	}
	return completions, cobra.ShellCompDirectiveNoFileComp
}

// remoteBranchCompletions lists remote origin branches (without the "origin/"
// prefix) whose names start with prefix.
func remoteBranchCompletions(prefix string) ([]string, cobra.ShellCompDirective) {
	ctx, cancel := context.WithTimeout(context.Background(), completionTimeout)
	defer cancel()

	out, err := exec.CommandContext(ctx, "git", "for-each-ref",
		"--format=%(refname:short)", "refs/remotes/origin/").Output()
	if err != nil {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	var branches []string
	for _, line := range strings.Split(strings.TrimSpace(string(out)), "\n") {
		branch := strings.TrimPrefix(line, "origin/")
		if branch == "" || branch == "HEAD" {
			continue
		}
		if strings.HasPrefix(branch, prefix) {
			branches = append(branches, branch)
		}
	}
	return branches, cobra.ShellCompDirectiveNoFileComp
}

// repoPathCompletions completes ":/path" style paths against the HEAD tree.
// prefix must start with ":/". Directories are returned with a trailing "/".
//
// git ls-tree is always run from the repo root so that paths in its output are
// consistently repo-root-relative regardless of the caller's working directory.
func repoPathCompletions(prefix string) ([]string, cobra.ShellCompDirective) {
	repoPrefix := strings.TrimPrefix(prefix, ":/")

	// Split into the already-complete directory and the current file prefix.
	dir := ""
	if idx := strings.LastIndex(repoPrefix, "/"); idx >= 0 {
		dir = repoPrefix[:idx+1]
	}
	filePrefix := repoPrefix[len(dir):]

	ctx, cancel := context.WithTimeout(context.Background(), completionTimeout)
	defer cancel()

	// Resolve the repo root so git ls-tree is always run from there, ensuring
	// that path arguments and output are repo-root-relative.
	root, err := getRepoRoot()
	if err != nil {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	var args []string
	if dir != "" {
		args = []string{"ls-tree", "HEAD", "--", dir}
	} else {
		args = []string{"ls-tree", "HEAD"}
	}
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = root
	out, err := cmd.Output()
	if err != nil {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	var completions []string
	for _, line := range strings.Split(strings.TrimSpace(string(out)), "\n") {
		if line == "" {
			continue
		}
		// git ls-tree output: "<mode> <type> <hash>\t<name>"
		// When run from the repo root, <name> is always the full repo-root-relative
		// path, e.g. "cmd/browse.go" for a subdirectory query.
		tab := strings.Index(line, "\t")
		if tab < 0 {
			continue
		}
		name := line[tab+1:]
		// Filter: name must start with dir (guaranteed by git) and then filePrefix.
		if !strings.HasPrefix(name, dir+filePrefix) {
			continue
		}
		meta := strings.Fields(line[:tab])
		entry := ":/" + name
		if len(meta) >= 2 && meta[1] == "tree" {
			entry += "/"
		}
		completions = append(completions, entry)
	}
	return completions, cobra.ShellCompDirectiveNoFileComp
}

// delegateCompletion resolves the active forge and delegates completion to the
// underlying forge CLI. Any error silently returns no completions — errors must
// not disrupt the shell.
func delegateCompletion(gfSubcmd, gfVerb string, remainingArgs []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	// repo browse is implemented natively; complete its own flags and values.
	if gfSubcmd == "repo" && gfVerb == "browse" {
		return browseCompletions(remainingArgs, toComplete)
	}

	cfg, err := config.Load()
	if err != nil {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	host, err := forge.RemoteHost()
	if err != nil {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	entry, ok := cfg.Forges[host]
	if !ok {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	result, err := translate.Translate(entry.Type, gfSubcmd, gfVerb)
	if err != nil {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	translatedArgs := make([]string, len(result.Args))
	copy(translatedArgs, result.Args)
	if !result.DropArgs {
		translatedArgs = append(translatedArgs, remainingArgs...)
	}

	cliBin := config.EffectiveCLI(entry)
	cliPath, err := exec.LookPath(cliBin)
	if err != nil {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}

	switch entry.Type {
	case "github", "gitlab":
		// Cobra-based CLIs: use the __complete protocol for best performance.
		return cobraDelegate(cliPath, translatedArgs, toComplete)
	default:
		// tea (urfave/cli) and fj (clap): parse --help output.
		return helpDelegate(cliPath, translatedArgs, toComplete)
	}
}

// cobraDelegate delegates to a cobra-based forge CLI (gh, glab) via its
// __complete subcommand. toComplete is passed as the last argument so the CLI
// can pre-filter results itself.
func cobraDelegate(cliPath string, translatedArgs []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	args := make([]string, 0, len(translatedArgs)+2)
	args = append(args, "__complete")
	args = append(args, translatedArgs...)
	args = append(args, toComplete)

	ctx, cancel := context.WithTimeout(context.Background(), completionTimeout)
	defer cancel()

	// cobra exits non-zero when there are no completions but still writes
	// valid output — ignore the error and parse whatever we got.
	out, _ := exec.CommandContext(ctx, cliPath, args...).Output()
	if len(out) == 0 {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}
	return parseCobraOutput(out)
}

// helpDelegate runs "<cli> <args> --help" and extracts flag completions from
// the output. Works for both tea (urfave/cli) and fj (clap) because both
// print well-structured --help text, unlike the proprietary dynamic completion
// protocols those CLIs use which are either broken or absent at the flag level.
func helpDelegate(cliPath string, translatedArgs []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	args := make([]string, 0, len(translatedArgs)+1)
	args = append(args, translatedArgs...)
	args = append(args, "--help")

	ctx, cancel := context.WithTimeout(context.Background(), completionTimeout)
	defer cancel()

	// CombinedOutput: some CLIs print help to stderr or exit non-zero for
	// --help (e.g. fj when a required COMMAND is missing) but still produce
	// parseable usage text.
	out, _ := exec.CommandContext(ctx, cliPath, args...).CombinedOutput()
	if len(out) == 0 {
		return nil, cobra.ShellCompDirectiveNoFileComp
	}
	return parseHelpOutput(out, toComplete), cobra.ShellCompDirectiveNoFileComp
}

var (
	// longFlagRe matches --flag-name tokens.
	longFlagRe = regexp.MustCompile(`--[a-zA-Z][a-zA-Z0-9-]*`)

	// shortFlagRe matches -c tokens that are preceded by start-of-string,
	// whitespace, or a comma — avoiding false matches inside --long-flag names.
	shortFlagRe = regexp.MustCompile(`(?:^|[\s,])-([a-zA-Z])\b`)

	// descSepRe matches a run of 2+ whitespace characters that separates the
	// flag definition from its inline description on the same line.
	descSepRe = regexp.MustCompile(`\s{2,}`)
)

// parseHelpOutput extracts --flag and -c completions from CLI --help output.
//
// It handles both tea (urfave/cli) single-line format:
//
//	   --state string, -s string   Filter by state
//
// and fj (clap) multi-line format:
//
//	  -s, --state <STATE>
//	          Filter by state
//
// Only lines with ≥2 spaces of leading indentation whose trimmed content
// begins with '-' are processed (flag definition lines). Descriptions are
// stripped by splitting on the first 2+-space gap, preventing flag names
// mentioned in descriptive text from appearing as false completions.
func parseHelpOutput(out []byte, toComplete string) []string {
	var completions []string
	seen := make(map[string]bool)

	add := func(flag string) {
		if !seen[flag] && strings.HasPrefix(flag, toComplete) {
			completions = append(completions, flag)
			seen[flag] = true
		}
	}

	for _, line := range strings.Split(string(out), "\n") {
		trimmed := strings.TrimLeft(line, " \t")
		// Require ≥2 spaces of indentation and the line must start a flag ('-').
		if len(line)-len(trimmed) < 2 || !strings.HasPrefix(trimmed, "-") {
			continue
		}

		// Strip the inline description (everything after the first 2+-space gap)
		// to avoid extracting flag names that appear in description text.
		flagPart := trimmed
		if loc := descSepRe.FindStringIndex(trimmed); loc != nil {
			flagPart = trimmed[:loc[0]]
		}

		for _, m := range longFlagRe.FindAllString(flagPart, -1) {
			add(m)
		}
		for _, sub := range shortFlagRe.FindAllStringSubmatch(flagPart, -1) {
			add("-" + sub[1])
		}
	}
	return completions
}

// parseCobraOutput parses the output of a cobra __complete invocation:
//
//	completion1[\tdescription]
//	completion2[\tdescription]
//	:<directive>
//
// Completions preserve the tab-delimited description so that shell completion
// scripts can display them.
func parseCobraOutput(out []byte) ([]string, cobra.ShellCompDirective) {
	lines := strings.Split(strings.TrimRight(string(out), "\n"), "\n")

	directive := cobra.ShellCompDirectiveNoFileComp
	if len(lines) > 0 {
		last := lines[len(lines)-1]
		if strings.HasPrefix(last, ":") {
			if n, err := strconv.Atoi(last[1:]); err == nil {
				directive = cobra.ShellCompDirective(n)
				lines = lines[:len(lines)-1]
			}
		}
	}

	completions := make([]string, 0, len(lines))
	for _, line := range lines {
		if line != "" {
			completions = append(completions, line)
		}
	}
	return completions, directive
}
