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

	for _, sc := range []struct {
		name    string
		short   string
		aliases []string
	}{
		{"pr", "Pull request operations", nil},
		{"mr", "Merge request operations (alias for pr)", nil},
		{"issue", "Issue operations", []string{"i"}},
		{"repo", "Repository operations", []string{"r"}},
		{"release", "Release operations", nil},
		{"pipeline", "CI/CD pipeline operations", []string{"p"}},
		{"milestone", "Milestone operations", []string{"m"}},
		{"label", "Label operations", []string{"l"}},
		{"org", "Organization operations", []string{"o"}},
	} {
		cmd := newPassthroughCmd(sc.name, sc.short)
		cmd.Aliases = sc.aliases
		rootCmd.AddCommand(cmd)
	}
}

func newPassthroughCmd(name, short string) *cobra.Command {
	return &cobra.Command{
		Use:                name + " <verb> [args...]",
		Short:              short,
		Long:               short + ".\n\nSupported verbs:\n  " + verbList(name) + "\n\nAll flags and arguments are forwarded verbatim to the forge CLI.",
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
			if len(args) == 0 || args[0] == "--help" || args[0] == "-h" {
				_ = cmd.Help()
				return
			}
			cmdExitCode = dispatch(cmd.Name(), args)
		},
	}
}
