# gf

[![standard-readme compliant](https://img.shields.io/badge/standard--readme-OK-green.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

Universal git forge CLI

`gf` is a unified command-line wrapper for GitHub, GitLab, Gitea, and Forgejo. It auto-detects which forge your
repository is hosted on from the git remote and transparently delegates to the appropriate CLI (`gh`, `glab`, `tea`,
or `fj`), normalizing flags and subcommand names so one set of commands works everywhere.

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Install

`gf` requires at least one of the following CLIs to be installed and available on your `PATH`:

- **GitHub**: [`gh`](https://cli.github.com/)
- **GitLab**: [`glab`](https://gitlab.com/gitlab-org/cli)
- **Gitea**: [`tea`](https://gitea.com/gitea/tea)
- **Forgejo**: [`fj`](https://codeberg.org/Cyborus/forgejo-cli)

Build and install `gf` from source:

```sh
git clone https://github.com/kfkonrad/gf.git
cd gf
cargo install --path .
```

Alternatively you can download the latest version from the GitHub Releases.

## Usage

`gf` organizes commands into five groups: `pr`, `issue`, `repo`, `auth`, and `browse`. The `mr` alias for `pr` is
also supported for GitLab-centric workflows.

### Pull Requests

```sh
gf pr list --state open
gf pr create --title "My feature" --body "Description"
gf pr view 42
gf pr merge 42 --squash --delete-branch
gf pr checkout 42
gf pr review 42 --approve
gf pr comment 42 --body "Looks good"
gf pr checks 42
gf pr edit 42 --title "Updated title"
```

### Issues

```sh
gf issue list --state open
gf issue view 123
gf issue create --title "Bug report" --body "Details"
gf issue comment 123 --body "Thanks for reporting"
gf issue close 123
gf issue reopen 123
gf issue edit 123 --title "Updated title"
```

### Repository

```sh
gf repo clone owner/repo
gf repo fork
gf repo create --name my-project --description "My project"
gf repo view
```

### Authentication

```sh
gf auth login
gf auth logout
gf auth status
```

### Browse

Open the current repository, a file, a PR, or an issue in your browser. Line ranges are supported and translated to
the correct URL fragment format for each forge:

```sh
gf browse                        # repo root
gf browse src/main.rs            # a file
gf browse src/main.rs:42-55      # file with line range
gf browse --pr 42                # pull request
gf browse --issue 123            # issue
gf browse --branch develop       # specific branch
```

### Global Flags

Use `--remote` to specify a git remote other than `origin`:

```sh
gf --remote upstream pr list
```

### Shell Completions

```sh
gf completions --shell bash >> ~/.bashrc
gf completions --shell zsh  >> ~/.zshrc
gf completions --shell fish > ~/.config/fish/completions/gf.fish
```

## Configuration

`gf` reads its configuration from `~/.config/gf/config.toml`. The file is optional — `gf` automatically detects
forges hosted on `github.com`, `gitlab.com`, `gitea.io`, and `codeberg.org` without any configuration.

### Self-Hosted Forges

Add a `[[forge]]` section for each self-hosted instance:

```toml
[[forge]]
domain = "gitlab.mycompany.com"
type = "gitlab"

[[forge]]
domain = "gitea.internal"
type = "gitea"
```

Supported values for `type`: `github`, `gitlab`, `gitea`, `forgejo`.

### Merge Defaults

Set `delete_branch = true` to automatically delete the source branch after every merge:

```toml
[merge]
delete_branch = true
```

To apply this only on a specific forge, set it on the `[[forge]]` entry instead:

```toml
[[forge]]
domain = "gitlab.mycompany.com"
type = "gitlab"
delete_branch = true
```

The CLI flags `--delete-branch` and `--no-delete-branch` always take precedence over config.

### Browse Defaults

Set `detached_head_fallback` to a branch name to use when `gf browse` is run with a detached HEAD and no `--branch`
flag. Without this, `gf browse` falls back to the commit SHA:

```toml
[browse]
detached_head_fallback = "main"
```

To use a different fallback on a specific forge, set it on the `[[forge]]` entry instead:

```toml
[[forge]]
domain = "gitlab.mycompany.com"
type = "gitlab"
detached_head_fallback = "develop"
```

## Maintainers

[@kfkonrad](https://github.com/kfkonrad)

## Contributing

PRs accepted.

Small note: If editing the README, please conform to the
[standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

MIT © 2026 Kevin F. Konrad
