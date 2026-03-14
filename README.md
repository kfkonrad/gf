# gf

[![standard-readme compliant](https://img.shields.io/badge/standard--readme-OK-green.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

gf - a thin command dispatcher for git forges

`gf` sits in front of `gh`, `glab`, `tea`, and `fj` and gives them a single, consistent command interface. Run the same
command regardless of which forge your repo lives on — `gf` detects the forge from your git remote, translates the
command, and delegates to the right CLI.

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Install

```sh
go install github.com/kfkonrad/gf@latest
```

Or clone and build:

```sh
git clone https://github.com/kfkonrad/gf
cd gf
make install
```

`gf` requires the CLIs for the forges you use to be installed and in your `PATH`:

| Forge   | CLI                                                      |
| ------- | -------------------------------------------------------- |
| GitHub  | [`gh`](https://cli.github.com)                           |
| GitLab  | [`glab`](https://gitlab.com/gitlab-org/cli)              |
| Gitea   | [`tea`](https://gitea.com/gitea/tea)                     |
| Forgejo | [`fj`](https://codeberg.org/forgejo-contrib/forgejo-cli) |

## Usage

```sh
gf <subcommand> <verb> [args...]
```

`gf` auto-detects the forge from the current repo's `origin` remote. Any arguments after the verb are forwarded verbatim
to the underlying CLI.

### Subcommands and verbs

| Subcommand  | Alias   | Supported verbs                                                   |
| ----------- | ------- | ----------------------------------------------------------------- |
| `pr`        | `mr`    | `list`, `view`, `create`, `close`, `merge`, `checkout`, `comment` |
| `issue`     | `i`     | `list`, `view`, `create`, `close`, `comment`                      |
| `repo`      | `r`     | `list`, `view`, `create`, `browse`, `fork`                        |
| `release`   |         | `list`, `view`, `create`                                          |
| `pipeline`  | `p`     | `list`, `view`, `cancel`                                          |
| `milestone` | `m`     | `list`, `view`, `create`, `close`                                 |
| `label`     | `l`     | `list`, `create`                                                  |
| `org`       | `o`     | `list`, `view`                                                    |

Some verbs also have single-letter aliases: `l`=`list`, `v`=`view`, `c`=`create`, `d`=`close`, `b`=`browse`, `e`=`edit`,
`a`=`add`.

### `gf repo browse`

`repo browse` is special in that it is natively implemented and does not dispatch to the forge CLI. It opens the current
repository in a web browser.

```
gf repo browse [flags]

  -b, --branch <branch>  Browse at a specific branch
  -c, --commit <sha>     Browse at a specific commit
  -p, --path <path>      Browse a specific path.
                         Prefix with :/ for a repo-root-relative path,
                         otherwise resolved from the current directory.
                         Append :<line> to highlight a line (e.g. :/main.go:42).
  -n, --no-browser       Print the URL instead of opening a browser
```

`--branch` and `--commit` are mutually exclusive.

### Examples

```sh
# List open pull requests
gf pr list

# Create an issue (args forwarded to the underlying CLI)
gf issue create --title "Bug report"

# Browse the repo in your browser at a specific branch and file
gf repo browse --branch main --path :/README.md

# View a pipeline run
gf pipeline view 123
```

### Configuration

`gf` detects your forge from the hostname in the `origin` remote URL, but it cannot infer what software a hostname
runs — a self-hosted instance at `git.company.com` could be GitLab, Gitea, or Forgejo. The config maps hostnames to
forge types so `gf` knows which CLI to call and how to translate commands.

On first run `gf` creates a config file with the four major public forges (github.com, gitlab.com. gitea.com,
codeberg.org) pre-configured. The config lives at:

- **Linux/macOS/Unix**: `$XDG_CONFIG_HOME/gf/config.yaml` (default: `~/.config/gf/config.yaml`)
- **Windows**: `%APPDATA%/gf/config.yaml`
- **Override**: set `GF_CONFIG` to a custom path

Manage forges with the `gf forge` subcommand:

```sh
# List all configured forges
gf forge list

# Add a forge interactively
gf forge add

# Add a forge non-interactively
gf forge add --hostname gitlab.company.com --type gitlab

# Add a forge with a custom CLI binary path
gf forge add --hostname gitea.company.com --type gitea --cli /usr/local/bin/tea

# Remove a forge
gf forge remove --hostname gitlab.company.com

# Remove without confirmation prompt
gf forge remove --hostname gitlab.company.com --yes
```

Supported types are `github`, `gitlab`, `gitea`, and `forgejo`.

### Shell completion

```sh
gf completion bash   # or zsh, fish, powershell
```

## Maintainers

[@kfkonrad](https://github.com/kfkonrad)

## Contributing

PRs accepted.

Small note: If editing the README, please conform to the
[standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

MIT © 2026 Kevin F. Konrad
