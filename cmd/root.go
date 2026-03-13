package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:           "gf",
	Short:         "Git forge CLI wrapper",
	Long:          "gf is a thin command dispatcher for git forges.\nIt provides a single consistent entry point across GitHub, GitLab, Gitea, and Forgejo.",
	SilenceErrors: true,
	SilenceUsage:  true,
}

// Execute runs the root command and returns an exit code.
func Execute() int {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, "gf:", err)
		return 1
	}
	return 0
}

func init() {
	rootCmd.AddCommand(forgeCmd)

	for _, sc := range []struct{ name, short, verbs string }{
		{"pr", "Pull request operations", "list, view, create, close, merge, checkout, comment"},
		{"mr", "Merge request operations (alias for pr)", "list, view, create, close, merge, checkout, comment"},
		{"issue", "Issue operations", "list, view, create, close, comment"},
		{"repo", "Repository operations", "list, view, create, browse, fork"},
		{"release", "Release operations", "list, view, create"},
		{"pipeline", "CI/CD pipeline operations", "list, view, cancel"},
		{"milestone", "Milestone operations", "list, view, create, close"},
		{"label", "Label operations", "list, create"},
		{"org", "Organization operations", "list, view"},
	} {
		rootCmd.AddCommand(newPassthroughCmd(sc.name, sc.short, sc.verbs))
	}
}

func newPassthroughCmd(name, short, verbs string) *cobra.Command {
	return &cobra.Command{
		Use:                name + " <verb> [args...]",
		Short:              short,
		Long:               short + ".\n\nSupported verbs: " + verbs + ".\n\nAll flags and arguments are forwarded verbatim to the forge CLI.",
		DisableFlagParsing: true,
		Args:               cobra.ArbitraryArgs,
		Run: func(cmd *cobra.Command, args []string) {
			os.Exit(dispatch(cmd.Name(), args))
		},
	}
}
