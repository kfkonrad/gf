package cmd

import (
	"bufio"
	"errors"
	"fmt"
	"gf/internal/config"
	"gf/internal/forge"
	"io"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

var forgeCmd = &cobra.Command{
	Use:   "forge",
	Short: "Manage forge configurations",
	Long:  "Manage the hostname-to-forge mappings in config.yaml.",
}

var (
	forgeAddHostname string
	forgeAddType     string
	forgeAddCLI      string
	forgeAddYes      bool
)

var forgeAddCmd = &cobra.Command{
	Use:               "add",
	Short:             "Add a forge entry",
	RunE:              runForgeAdd,
	ValidArgsFunction: cobra.NoFileCompletions,
}

var forgeListCmd = &cobra.Command{
	Use:   "list",
	Short: "List configured forges",
	RunE:  runForgeList,
}

var forgeRemoveYes bool

var forgeRemoveCmd = &cobra.Command{
	Use:   "remove <hostname>",
	Short: "Remove a forge entry",
	Args:  cobra.ExactArgs(1),
	RunE:  runForgeRemove,
	ValidArgsFunction: func(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
		if len(args) != 0 {
			return nil, cobra.ShellCompDirectiveNoFileComp
		}
		cfg, err := config.Load()
		if err != nil {
			return nil, cobra.ShellCompDirectiveNoFileComp
		}
		var hosts []string
		for h := range cfg.Forges {
			if strings.HasPrefix(h, toComplete) {
				hosts = append(hosts, h)
			}
		}
		return hosts, cobra.ShellCompDirectiveNoFileComp
	},
}

func init() {
	forgeAddCmd.Flags().StringVar(&forgeAddHostname, "hostname", "", "Hostname to add")
	forgeAddCmd.Flags().StringVar(&forgeAddType, "type", "", "Forge type (github, gitlab, gitea, forgejo)")
	forgeAddCmd.Flags().StringVar(&forgeAddCLI, "cli", "", "CLI override (name or path)")
	forgeAddCmd.Flags().BoolVarP(&forgeAddYes, "yes", "y", false, "Skip overwrite confirmation")

	forgeRemoveCmd.Flags().BoolVarP(&forgeRemoveYes, "yes", "y", false, "Skip confirmation prompt")

	forgeCmd.AddCommand(forgeAddCmd)
	forgeCmd.AddCommand(forgeListCmd)
	forgeCmd.AddCommand(forgeRemoveCmd)
}

func runForgeAdd(cmd *cobra.Command, args []string) error {
	reader := bufio.NewReader(os.Stdin)

	// Step 1: hostname
	hostname := forgeAddHostname
	if hostname == "" {
		detected, detectedErr := forge.RemoteHost()
		if detectedErr == nil {
			fmt.Printf("Hostname [%s]: ", detected)
		} else {
			fmt.Print("Hostname: ")
		}
		input, err := reader.ReadString('\n')
		if err != nil && !errors.Is(err, io.EOF) {
			return fmt.Errorf("reading input: %w", err)
		}
		input = strings.TrimSpace(input)
		if input != "" {
			hostname = input
		} else if detectedErr == nil {
			hostname = detected
		}
	}
	if hostname == "" {
		return fmt.Errorf("hostname is required")
	}

	// Step 2: forge type
	forgeType := forgeAddType
	if forgeType == "" {
		suggested := suggestForgeType(hostname)
		if suggested != "" {
			fmt.Printf("Forge type (github/gitlab/gitea/forgejo) [%s]: ", suggested)
		} else {
			fmt.Print("Forge type (github/gitlab/gitea/forgejo): ")
		}
		input, err := reader.ReadString('\n')
		if err != nil && !errors.Is(err, io.EOF) {
			return fmt.Errorf("reading input: %w", err)
		}
		input = strings.TrimSpace(input)
		if input != "" {
			forgeType = input
		} else {
			forgeType = suggested
		}
	}
	if !isValidForgeType(forgeType) {
		return fmt.Errorf("invalid forge type %q; must be one of: github, gitlab, gitea, forgejo", forgeType)
	}

	// Step 3: optional CLI override
	cliOverride := forgeAddCLI
	if !cmd.Flags().Changed("cli") {
		defaultCLI := config.DefaultCLIs[forgeType]
		fmt.Printf("CLI override (leave blank to use default %q): ", defaultCLI)
		input, err := reader.ReadString('\n')
		if err != nil && !errors.Is(err, io.EOF) {
			return fmt.Errorf("reading input: %w", err)
		}
		cliOverride = strings.TrimSpace(input)
	}

	cfg, err := config.Load()
	if err != nil {
		return fmt.Errorf("loading config: %w", err)
	}

	if existing, ok := cfg.Forges[hostname]; ok && !forgeAddYes {
		fmt.Printf("Hostname %q is already configured (type: %s, cli: %s).\nOverwrite? [y/N] ", hostname, existing.Type, config.EffectiveCLI(existing))
		input, err := reader.ReadString('\n')
		if err != nil && !errors.Is(err, io.EOF) {
			return fmt.Errorf("reading input: %w", err)
		}
		input = strings.TrimSpace(strings.ToLower(input))
		if input != "y" && input != "yes" {
			fmt.Println("Aborted.")
			return nil
		}
	}

	entry := config.ForgeEntry{Type: forgeType}
	if cliOverride != "" {
		entry.CLI = cliOverride
	}
	cfg.Forges[hostname] = entry

	if err := config.Save(cfg); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	fmt.Printf("Added forge %q (%s) → %s\n", hostname, forgeType, config.EffectiveCLI(entry))
	return nil
}

func runForgeList(cmd *cobra.Command, args []string) error {
	cfg, err := config.Load()
	if err != nil {
		return fmt.Errorf("loading config: %w", err)
	}

	if len(cfg.Forges) == 0 {
		fmt.Println("No forges configured. Run 'gf forge add' to add one.")
		return nil
	}

	fmt.Printf("%-30s %-10s %s\n", "HOSTNAME", "TYPE", "CLI")
	fmt.Printf("%-30s %-10s %s\n", strings.Repeat("-", 29), strings.Repeat("-", 9), strings.Repeat("-", 10))
	for host, entry := range cfg.Forges {
		fmt.Printf("%-30s %-10s %s\n", host, entry.Type, config.EffectiveCLI(entry))
	}
	return nil
}

func runForgeRemove(cmd *cobra.Command, args []string) error {
	hostname := args[0]

	cfg, err := config.Load()
	if err != nil {
		return fmt.Errorf("loading config: %w", err)
	}

	if _, ok := cfg.Forges[hostname]; !ok {
		return fmt.Errorf("hostname %q not found in config", hostname)
	}

	if !forgeRemoveYes {
		fmt.Printf("Remove forge %q? [Y/n] ", hostname)
		reader := bufio.NewReader(os.Stdin)
		input, err := reader.ReadString('\n')
		if err != nil && !errors.Is(err, io.EOF) {
			return fmt.Errorf("reading input: %w", err)
		}
		input = strings.TrimSpace(strings.ToLower(input))
		if input == "n" || input == "no" {
			fmt.Println("Aborted.")
			return nil
		}
	}

	delete(cfg.Forges, hostname)

	if err := config.Save(cfg); err != nil {
		return fmt.Errorf("saving config: %w", err)
	}

	fmt.Printf("Removed forge %q\n", hostname)
	return nil
}

// suggestForgeType suggests a forge type based on well-known hostnames.
func suggestForgeType(hostname string) string {
	lower := strings.ToLower(hostname)
	switch {
	case lower == "github.com" || strings.Contains(lower, "github"):
		return "github"
	case lower == "gitlab.com" || strings.Contains(lower, "gitlab"):
		return "gitlab"
	case lower == "gitea.com" || strings.Contains(lower, "gitea"):
		return "gitea"
	case lower == "codeberg.org" || strings.Contains(lower, "forgejo"):
		return "forgejo"
	}
	return ""
}

func isValidForgeType(t string) bool {
	for _, v := range config.ValidTypes {
		if v == t {
			return true
		}
	}
	return false
}
