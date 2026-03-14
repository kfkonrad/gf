package cmd

import (
	"fmt"
	"gf/internal/config"
	"gf/internal/forge"
	"gf/internal/translate"
	"os"
	"os/exec"
	"strings"
)

// dispatch looks up the forge for the current repo, translates the gf subcommand
// and verb, then exec's the forge CLI with the translated arguments.
// args is [verb, remaining...]. Returns an exit code.
func dispatch(gfSubcmd string, args []string) int {
	gfVerb := resolveVerb(args[0])
	remainingArgs := args[1:]

	// Reject unknown verbs immediately — before any forge or config lookup.
	if !strings.HasPrefix(gfVerb, "-") && !isValidVerb(gfSubcmd, gfVerb) {
		fmt.Fprintf(os.Stderr, "gf: %s: unknown verb %q\n", gfSubcmd, gfVerb)
		fmt.Fprintf(os.Stderr, "Supported verbs: %s\n", verbList(gfSubcmd))
		return 2
	}

	// repo browse is implemented natively — no forge CLI involved.
	if gfSubcmd == "repo" && gfVerb == "browse" {
		return runBrowse(remainingArgs)
	}

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "gf: error reading config: %v\n", err)
		return 1
	}

	host, err := forge.RemoteHost()
	if err != nil {
		fmt.Fprintf(os.Stderr, "gf: %v\n", err)
		return 1
	}

	entry, ok := cfg.Forges[host]
	if !ok {
		fmt.Fprintf(os.Stderr, "gf: hostname %q not found in config. Run 'gf forge add' to add it.\n", host)
		return 4
	}

	result, err := translate.Translate(entry.Type, gfSubcmd, gfVerb)
	if err != nil {
		if _, ok := err.(*translate.UnsupportedError); ok {
			fmt.Fprintf(os.Stderr, "gf: %v\n", err)
			return 2
		}
		fmt.Fprintf(os.Stderr, "gf: translation error: %v\n", err)
		return 1
	}

	cliArgs := result.Args
	if !result.DropArgs {
		if len(remainingArgs) == 0 {
			// Only the verb was given — show the forge CLI's own help for this command.
			cliArgs = append(cliArgs, "--help")
		} else {
			cliArgs = append(cliArgs, remainingArgs...)
		}
	}

	cliBin := config.EffectiveCLI(entry)
	if cliBin == "" {
		fmt.Fprintf(os.Stderr, "gf: no CLI configured for forge type %q\n", entry.Type)
		return 1
	}

	cliPath, err := exec.LookPath(cliBin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "gf: forge CLI %q not found in PATH: %v\n", cliBin, err)
		return 5
	}

	cmd := exec.Command(cliPath, cliArgs...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return exitErr.ExitCode()
		}
		fmt.Fprintf(os.Stderr, "gf: failed to run %s: %v\n", cliBin, err)
		return 1
	}
	return 0
}
