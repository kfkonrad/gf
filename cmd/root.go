package cmd

import (
	"fmt"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:           "gf",
	Short:         "Git forge CLI wrapper",
	Long:          "gf is a thin command dispatcher for git forges.\nIt provides a single consistent entry point across GitHub, GitLab, Gitea, and Forgejo.",
	SilenceErrors: true,
	SilenceUsage:  true,
}

// cmdExitCode stores the exit code from the last dispatched passthrough command.
var cmdExitCode int

// Execute runs the root command and returns an exit code.
func Execute() int {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, "gf:", err)
		return 1
	}
	return cmdExitCode
}

func init() {
	rootCmd.AddCommand(forgeCmd)

	for _, sc := range []struct{ name, short string }{
		{"pr", "Pull request operations"},
		{"mr", "Merge request operations (alias for pr)"},
		{"issue", "Issue operations"},
		{"repo", "Repository operations"},
		{"release", "Release operations"},
		{"pipeline", "CI/CD pipeline operations"},
		{"milestone", "Milestone operations"},
		{"label", "Label operations"},
		{"org", "Organization operations"},
	} {
		rootCmd.AddCommand(newPassthroughCmd(sc.name, sc.short))
	}
}

func newPassthroughCmd(name, short string) *cobra.Command {
	return &cobra.Command{
		Use:                name + " <verb> [args...]",
		Short:              short,
		Long:               short + ".\n\nSupported verbs: " + verbList(name) + ".\n\nAll flags and arguments are forwarded verbatim to the forge CLI.",
		DisableFlagParsing: true,
		Args:               cobra.ArbitraryArgs,
		ValidArgsFunction: func(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
			if len(args) == 0 {
				// Cobra does not filter ValidArgsFunction results — do it ourselves.
				var completions []string
				for _, v := range validVerbs[cmd.Name()] {
					if strings.HasPrefix(v, toComplete) {
						completions = append(completions, v)
					}
				}
				return completions, cobra.ShellCompDirectiveNoFileComp
			}
			verb := args[0]
			if !isValidVerb(cmd.Name(), verb) {
				return nil, cobra.ShellCompDirectiveNoFileComp
			}
			return delegateCompletion(cmd.Name(), verb, args[1:], toComplete)
		},
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				_ = cmd.Help()
				return
			}
			cmdExitCode = dispatch(cmd.Name(), args)
		},
	}
}
