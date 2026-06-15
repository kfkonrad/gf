# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]

### Added
* Add `-V`/`--version` flag to display the version
* Add configurable default for branch deletion on merge
* Add `detached_head_fallback` config option

### Fixed
* Fix domain parsing error where `foo@bar.com`-style remotes were not parsed correctly

## [1.2.0] - 2026-03-19

### Added
* Add `pr comment` and `issue comment` commands
* Add `pr edit` and `issue edit` commands with validate-then-build translation
* Add `pr checks` command

## [1.1.0] - 2026-03-18

### Added
* Add PR workflow commands: `pr list`, `pr checkout`, `pr merge`, `pr review`, and `pr approve`
* Add issue management commands: create, list, close, and reopen
* Add `repo clone` command with configurable `clone_host` default
* Add line-range deep-linking to `gf browse`
* Add `--pr`/`--mr` and `--issue` flags to `gf browse` for opening pull requests, merge requests, and issues in the browser
* Add automatic detection of self-hosted forges by probing the available CLIs, with results cached at `~/.cache/gf/probes.toml`
* Add `[defaults]` config section

### Changed
* Audit and normalize flag translation across all forges
* Deduplicate the known-host match table

## [1.0.0] - 2026-03-17

Initial release.

### Added
* Auto-detect the forge (GitHub, GitLab, Gitea, Forgejo) from the git remote and delegate to the matching CLI (`gh`, `glab`, `tea`, or `fj`)
* Normalize subcommand names and flags so one set of commands works across forges
* `pr`/`mr`, `repo`, and `auth` command groups with cross-forge translation
* `gf browse` to open the current repository in a web browser
* TOML configuration with host lookup for mapping remotes to forge types
