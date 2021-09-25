use anyhow::anyhow;
use cargo_metadata::{
    camino::{Utf8Component, Utf8Path},
    Dependency, Metadata, Package, PackageId,
};
use git_repository as git;
use git_repository::bstr::{BStr, ByteSlice};
use semver::Version;

pub fn will(not_really: bool) -> &'static str {
    if not_really {
        "WOULD"
    } else {
        "Will"
    }
}

pub fn is_pre_release_version(semver: &Version) -> bool {
    semver.major == 0
}

pub fn is_top_level_package(manifest_path: &Utf8Path, shared: &git::Easy) -> bool {
    manifest_path
        .strip_prefix(shared.repo.work_tree.as_ref().expect("repo with working tree"))
        .map_or(false, |p| p.components().count() == 1)
}

pub fn is_dependency_with_version_requirement(dep: &Dependency) -> bool {
    !dep.req.comparators.is_empty()
}

pub fn is_workspace_member(meta: &Metadata, crate_name: &str) -> bool {
    workspace_package_by_name(meta, crate_name).is_some()
}

pub fn package_eq_dependency(package: &Package, dependency: &Dependency) -> bool {
    package.name == dependency.name
}

pub fn workspace_package_by_name<'a>(meta: &'a Metadata, crate_name: &str) -> Option<&'a Package> {
    meta.packages
        .iter()
        .find(|p| p.name == crate_name)
        .filter(|p| meta.workspace_members.iter().any(|m| m == &p.id))
}

pub fn workspace_package_by_id<'a>(meta: &'a Metadata, id: &PackageId) -> Option<&'a Package> {
    meta.packages
        .iter()
        .find(|p| &p.id == id)
        .filter(|p| meta.workspace_members.iter().any(|m| m == &p.id))
}

pub fn package_by_name<'a>(meta: &'a Metadata, name: &str) -> anyhow::Result<&'a Package> {
    meta.packages
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("workspace member '{}' must be a listed package", name))
}

pub fn package_for_dependency<'a>(meta: &'a Metadata, dep: &Dependency) -> &'a Package {
    meta.packages
        .iter()
        .find(|p| package_eq_dependency(p, dep))
        .expect("dependency always available as package")
}

pub fn names_and_versions(publishees: &[(&Package, String)]) -> String {
    publishees
        .iter()
        .map(|(p, nv)| format!("{} v{}", p.name, nv))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn package_by_id<'a>(meta: &'a Metadata, id: &PackageId) -> &'a Package {
    meta.packages
        .iter()
        .find(|p| &p.id == id)
        .expect("workspace members are in packages")
}

pub fn tag_prefix<'p>(package: &'p Package, repo: &git::Easy) -> Option<&'p str> {
    if is_top_level_package(&package.manifest_path, repo) {
        None
    } else {
        Some(&package.name)
    }
}

pub fn tag_name(package: &Package, version: &str, repo: &git::Easy) -> String {
    tag_name_inner(tag_prefix(package, repo), version)
}

fn tag_name_inner(package_name: Option<&str>, version: &str) -> String {
    match package_name {
        Some(name) => format!("{}-v{}", name, version),
        None => format!("v{}", version),
    }
}

pub fn parse_possibly_prefixed_tag_version(package_name: Option<&str>, tag_name: &BStr) -> Option<Version> {
    match package_name {
        Some(name) => tag_name
            .strip_prefix(name.as_bytes())
            .and_then(|r| r.strip_prefix(b"-"))
            .and_then(|possibly_version| parse_tag_version(possibly_version.as_bstr())),
        None => parse_tag_version(tag_name),
    }
}

pub fn parse_tag_version(name: &BStr) -> Option<Version> {
    name.strip_prefix(b"v")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Version::parse(v).ok())
}

pub fn is_tag_name(package_name: &str, tag_name: &git::bstr::BStr) -> bool {
    match tag_name
        .strip_prefix(package_name.as_bytes())
        .and_then(|r| r.strip_prefix(b"-"))
    {
        None => false,
        Some(possibly_version) => parse_tag_version(possibly_version.as_bstr()).is_some(),
    }
}

pub fn is_tag_version(name: &git::bstr::BStr) -> bool {
    parse_tag_version(name).is_some()
}

pub fn component_to_bytes(c: Utf8Component<'_>) -> &[u8] {
    match c {
        Utf8Component::Normal(c) => c.as_bytes(),
        _ => unreachable!("only normal components are possible in paths here"),
    }
}

#[cfg(test)]
mod tests {
    mod parse_possibly_prefixed_tag_version {
        mod matches {
            use git_repository::bstr::ByteSlice;
            use semver::Version;

            use crate::utils::{parse_possibly_prefixed_tag_version, tag_name_inner};

            #[test]
            fn whatever_tag_name_would_return() {
                assert_eq!(
                    parse_possibly_prefixed_tag_version(
                        "git-test".into(),
                        tag_name_inner("git-test".into(), "1.0.1").as_bytes().as_bstr()
                    ),
                    Version::parse("1.0.1").expect("valid").into()
                );

                assert_eq!(
                    parse_possibly_prefixed_tag_version(
                        "single".into(),
                        tag_name_inner("single".into(), "0.0.1-beta.1").as_bytes().as_bstr()
                    ),
                    Version::parse("0.0.1-beta.1").expect("valid").into()
                );

                assert_eq!(
                    parse_possibly_prefixed_tag_version(None, tag_name_inner(None, "0.0.1+123.x").as_bytes().as_bstr()),
                    Version::parse("0.0.1+123.x").expect("valid").into()
                );
            }
        }
    }

    mod is_tag_name {
        mod no_match {
            use git_repository::bstr::ByteSlice;

            use crate::utils::{is_tag_name, tag_name_inner};

            #[test]
            fn due_to_crate_name() {
                assert!(!is_tag_name(
                    "foo",
                    tag_name_inner("bar".into(), "0.0.1-beta.1").as_bytes().as_bstr()
                ));
            }
        }
        mod matches {
            use git_repository::bstr::ByteSlice;

            use crate::utils::{is_tag_name, tag_name_inner};

            #[test]
            fn whatever_tag_name_would_return() {
                assert!(is_tag_name(
                    "git-test",
                    tag_name_inner("git-test".into(), "1.0.1").as_bytes().as_bstr()
                ));

                assert!(is_tag_name(
                    "single",
                    tag_name_inner("single".into(), "0.0.1-beta.1").as_bytes().as_bstr()
                ));
            }
        }
    }
    mod is_tag_version {
        mod no_match {
            use git_repository::bstr::ByteSlice;

            use crate::utils::is_tag_version;

            #[test]
            fn not_enough_numbers() {
                assert!(!is_tag_version(b"v0.0".as_bstr()));
            }
            #[test]
            fn invalid_prefix() {
                assert!(!is_tag_version(b"x0.0.1".as_bstr()));
            }

            #[test]
            fn funky() {
                assert!(!is_tag_version(b"vers0.0.1".as_bstr()));
                assert!(!is_tag_version(b"vHi.Ho.yada-anythingreally".as_bstr()));
            }
        }
        mod matches {
            use git_repository::bstr::ByteSlice;

            use crate::utils::is_tag_version;

            #[test]
            fn pre_release() {
                assert!(is_tag_version(b"v0.0.1".as_bstr()));
                assert!(is_tag_version(b"v0.10.0-beta.1".as_bstr()));
            }

            #[test]
            fn production() {
                assert!(is_tag_version(b"v1.0.1-alpha.1".as_bstr()));
                assert!(is_tag_version(b"v18.10.0+meta".as_bstr()));
            }
        }
    }
}
