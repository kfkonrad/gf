package translate

import "fmt"

// UnsupportedError is returned for unsupported subcommand/verb combinations.
type UnsupportedError struct {
	Forge  string
	Subcmd string
	Verb   string
}

func (e *UnsupportedError) Error() string {
	if e.Verb != "" {
		return fmt.Sprintf("forge %q does not support %q %q", e.Forge, e.Subcmd, e.Verb)
	}
	return fmt.Sprintf("forge %q does not support %q", e.Forge, e.Subcmd)
}

// Result holds the translated CLI arguments to prepend before remaining user args.
type Result struct {
	Args     []string
	DropArgs bool // if true, do not forward remaining user args
}

// subcmdTable maps gfSubcmd -> forgeType -> translated subcommand tokens.
// A missing forgeType entry means the subcommand is unsupported for that forge.
var subcmdTable = map[string]map[string][]string{
	"pr": {
		"github":  {"pr"},
		"gitlab":  {"mr"},
		"gitea":   {"pulls"},
		"forgejo": {"pr"},
	},
	"mr": {
		"github":  {"pr"},
		"gitlab":  {"mr"},
		"gitea":   {"pulls"},
		"forgejo": {"pr"},
	},
	"issue": {
		"github":  {"issue"},
		"gitlab":  {"issue"},
		"gitea":   {"issues"},
		"forgejo": {"issue"},
	},
	"repo": {
		"github":  {"repo"},
		"gitlab":  {"repo"},
		"gitea":   {"repos"},
		"forgejo": {"repo"},
	},
	"release": {
		"github":  {"release"},
		"gitlab":  {"release"},
		"gitea":   {"releases"},
		"forgejo": {"release"},
	},
	"pipeline": {
		"github": {"run"},
		"gitlab": {"ci"},
		"gitea":  {"actions", "runs"},
		// forgejo: unsupported
	},
	"milestone": {
		// github: unsupported
		"gitlab": {"milestone"},
		"gitea":  {"milestones"},
		// forgejo: unsupported
	},
	"label": {
		"github": {"label"},
		"gitlab": {"label"},
		"gitea":  {"labels"},
		// forgejo: unsupported
	},
	"org": {
		"github": {"org"},
		// gitlab: unsupported
		// gitea: unsupported
		"forgejo": {"org"},
	},
}

// Translate translates a gf subcommand + verb to forge CLI arguments.
// The returned Args should be prepended to the user's remaining args
// (unless DropArgs is true).
func Translate(forgeType, gfSubcmd, gfVerb string) (*Result, error) {
	// Structural exceptions for gitea (tea CLI)
	if forgeType == "gitea" {
		switch gfSubcmd {
		case "pr", "mr", "issue":
			if gfVerb == "comment" {
				// tea comment <ID> is a top-level command; drop pr/issue context
				return &Result{Args: []string{"comment"}}, nil
			}
		}
	}

	// Look up subcommand
	forgeMap, ok := subcmdTable[gfSubcmd]
	if !ok {
		return nil, fmt.Errorf("unknown gf subcommand %q", gfSubcmd)
	}
	subcmdTokens, ok := forgeMap[forgeType]
	if !ok {
		return nil, &UnsupportedError{Forge: forgeType, Subcmd: gfSubcmd}
	}

	// Translate the verb
	verbTokens, err := translateVerb(forgeType, gfSubcmd, gfVerb)
	if err != nil {
		return nil, err
	}

	args := make([]string, 0, len(subcmdTokens)+len(verbTokens))
	args = append(args, subcmdTokens...)
	args = append(args, verbTokens...)
	return &Result{Args: args}, nil
}

// translateVerb translates a gf verb to the forge CLI verb tokens.
// Returns the original verb as a single token if no special mapping applies.
func translateVerb(forgeType, gfSubcmd, gfVerb string) ([]string, error) {
	switch gfVerb {
	case "list":
		if forgeType == "forgejo" {
			switch gfSubcmd {
			case "pr", "mr", "issue":
				return []string{"search"}, nil
			case "repo":
				return nil, &UnsupportedError{Forge: forgeType, Subcmd: gfSubcmd, Verb: gfVerb}
			}
		}

	case "comment":
		if forgeType == "gitlab" {
			return []string{"note"}, nil
		}
		// gitea structural exception is handled before this function is called

	case "view":
		if gfSubcmd == "milestone" {
			switch forgeType {
			case "gitlab":
				return []string{"get"}, nil
			case "gitea":
				return []string{"list"}, nil
			}
		}

	case "close":
		if gfSubcmd == "milestone" && forgeType == "gitlab" {
			return nil, &UnsupportedError{Forge: forgeType, Subcmd: gfSubcmd, Verb: gfVerb}
		}
	}

	// Default: passthrough the verb as-is
	return []string{gfVerb}, nil
}
