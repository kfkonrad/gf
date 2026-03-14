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

// delegateCompletion resolves the active forge and delegates completion to the
// underlying forge CLI. Any error silently returns no completions — errors must
// not disrupt the shell.
func delegateCompletion(gfSubcmd, gfVerb string, remainingArgs []string, toComplete string) ([]string, cobra.ShellCompDirective) {
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
