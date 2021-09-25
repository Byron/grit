# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### Unreleased

#### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 10 times in this release.

### v0.11.0 (2021-09-08)

- manual bump for safety as its dependencies have breaking changes



### v0.10.0 (2021-09-07)

- **renames**
   - `data::Object::into_commit_iter()` -> `data::Object::try_into_commit_iter()`
   - `data::Object::into_tree_iter()` -> `data::Object::try_into_tree_iter()`
   - `data::Object::into_tag_iter()` -> `data::Object::try_into_tag_iter()`



### v0.9.0 (2021-08-27)

- **renames / moves / visibility**
   - `find::Find`  and `find::FindExt` only in `Find` and `FindExt` (not in `find` anymore)
   - `data::output::count::Count` -> `data::output::Count`
   - `data::output::entry::Entry` -> `data::output::Entry`
   - `Find::find_existing_*` -> `Find::find_*`
   - `Find::find_existing_*` -> `Find::find_*`
   - `Find::find()-> `Find::try_find()`
   - `bundle::Bundle` -> `Bundle`
   - `bundle::Error` -> `bundle::init::Error`
   - `pub tree::` -> `pub(crate) cache::delta::`
   - `data::object::Object` -> `data::Object`
   - `data::entry::Entry` -> `data::Entry`

* **new methods**
   - `Find::find_tag_iter()`
#### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 4 times in this release.

### v0.8.2 (2021-08-17)

### v0.8.1 (2021-08-13)

### v0.8.0 (2021-08-12)

### v0.6.0 (2021-08-11)

### v0.5.0 (2021-08-11)

### v0.3.1 (2021-08-10)

### v0.3.0 (2021-08-10)

#### Thanks Clippy…

Clippy is a linter to help keeping code idiomatic. It was helpful 7 times in this release.

### v0.2.0 (2021-05-25)

### v0.1.0 (2021-05-24)

