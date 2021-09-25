# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## v0.5.0 and v0.6.0

Maintenance release without any new features.

These are created to account for breaking changes within the dependency graph of
`gitoxide` crates. Due to some blunders in the past the version on crates.io
could not be installed anymore.
This was eventually fixed with new minor releases across the ecosystem.

Finally, yet another breaking change due to the introduction of the `git-hash`
crate to break a dependency cycle between `git-object` and `git-features` caused
yet another maintenance release.

## v0.4.1

* fix installation via `cargo install`

## v0.4

* add `remote-ref-list` and `pack-receive` subcommands to **gixp**

### CLI Breaking

 * rename plumbing sub-command from `index-from-pack` to `pack-index-from-data`

## v0.3

* add `pack-explode` and `pack-index-from-data` sub-commands
* massive speed improvements for `pack-verify`

Many small and possibly breaking changes are not mentioned here.

## v0.1

* Initial release with `pack-verify`
