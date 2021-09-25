# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 8 times in this release.

## v0.7.3 (2021-09-10)

- Compatibility with Rust 1.55. It informed about the incorrect usage of `std::io::ErrorKind::Other` which this crate also dependent on causing
  runtime bugs. The latter were detected by tests and this release contains the fix to not rely on certain behaviour anymore.

### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 2 times in this release.

## v0.7.2 (2021-09-10)

### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 1 time in this release.

## v0.7.1 (2021-09-08)

## v0.7.0 (2021-09-07)

### Breaking

* Replace `transaction::Create` with `transaction::PreviousValue` and remove `transaction::Create`
* Remove `file::Reference` in favor of `Reference`
* Move `file::log::Line` to `log::Line`
* `TargetRef::Symbolic(&BStr)` -> `TargetRef::Symbolic(FullNameRef)`
* replace `Transaction::namespacce()` with `file::Store::namespace`
 
## v0.6.1

### Bugfixes

* splits of edits to symbolic references will now 'move' the desired previous values down to the
  referents while resorting to not having any requirements in the symbolic ref instead.

## v0.5.4 (2021-08-17)

## v0.5.3 (2021-08-15)

## v0.5.2 (2021-08-13)

## v0.5.1 (2021-08-10)

## v0.4.1 (2020-12-19)

### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 1 time in this release.

## v0.4.0 (2020-09-12)

### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 1 time in this release.

## v0.3.0 (2020-08-12)

## v0.2.0 (2020-07-23)

## v0.1.0 (2020-07-15)

## v0.5.0 (2021-08-10)

### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 15 times in this release.

## v0.6.0 (2021-08-27)

### BREAKING

- rename `file::Store::packed()` to `file::Store::packed_buffer()`
### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 7 times in this release.
