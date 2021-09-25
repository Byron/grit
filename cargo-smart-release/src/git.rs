use std::process::Command;

use anyhow::{anyhow, bail};
use cargo_metadata::Package;
use git_repository::{
    bstr::{BStr, ByteSlice},
    easy::object,
    prelude::ReferenceAccessExt,
    refs::FullNameRef,
};

use crate::utils::{component_to_bytes, tag_name};

pub fn has_changed_since_last_release(package: &Package, ctx: &crate::Context, verbose: bool) -> anyhow::Result<bool> {
    let version_tag_name = tag_name(package, &package.version.to_string(), &ctx.repo);
    let mut tag_ref = match ctx.repo.try_find_reference(&version_tag_name)? {
        None => {
            if verbose {
                log::info!(
                    "Package {} wasn't tagged with {} yet and thus needs a release",
                    package.name,
                    version_tag_name
                );
            }
            return Ok(true);
        }
        Some(r) => r,
    };
    let repo_relative_crate_dir = ctx.repo_relative_path(package);
    Ok(match ctx.repo.head()?.into_fully_peeled_id() {
        Some(c) => {
            let current_commit = c?;
            let released_target = tag_ref.peel_to_id_in_place()?;

            match repo_relative_crate_dir {
                None => current_commit != released_target,
                Some(dir) => {
                    let components = dir.components().map(component_to_bytes);
                    let current_dir_id = current_commit
                        .object()?
                        .peel_to_kind(object::Kind::Tree)?
                        .into_tree()
                        .lookup_path(components.clone())?
                        .expect("path must exist in current commit")
                        .oid;
                    let released_dir_id = released_target
                        .object()?
                        .peel_to_kind(object::Kind::Tree)?
                        .into_tree()
                        .lookup_path(components)?
                        .expect("path must exist as it was supposedly released there")
                        .oid;

                    released_dir_id != current_dir_id
                }
            }
        }
        None => true,
    })
}

pub fn assure_clean_working_tree() -> anyhow::Result<()> {
    let tracked_changed = !Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .arg("--exit-code")
        .arg("--name-only")
        .status()?
        .success();
    if tracked_changed {
        bail!("Detected working tree changes. Please commit beforehand as otherwise these would be committed as part of manifest changes, or use --allow-dirty to force it.")
    }

    let untracked = Command::new("git")
        .arg("ls-files")
        .arg("--exclude-standard")
        .arg("--others")
        .output()?
        .stdout;
    if !untracked.trim().is_empty() {
        let err = anyhow!(git_repository::bstr::BString::from(untracked));
        return Err(err.context("Found untracked files which would possibly be packaged when publishing."));
    }
    Ok(())
}

pub mod history {
    use std::{cell::RefCell, collections::BTreeMap, iter::FromIterator, path::PathBuf, time::Instant};

    use anyhow::bail;
    use cargo_metadata::Package;
    use git_repository as git;
    use git_repository::{
        bstr::ByteSlice,
        easy::head,
        prelude::{CacheAccessExt, ObjectAccessExt, ReferenceAccessExt, ReferenceExt},
    };

    use crate::{
        commit,
        git::strip_tag_path,
        utils::{component_to_bytes, is_tag_name, is_tag_version, tag_prefix},
    };

    pub fn collect(repo: &git::Easy) -> anyhow::Result<Option<commit::History>> {
        let start = Instant::now();
        let prev = repo.object_cache_size(64 * 1024)?;
        let reference = match repo.head()?.peeled()?.kind {
            head::Kind::Detached { .. } => bail!("Refusing to operate on a detached head."),
            head::Kind::Unborn { .. } => return Ok(None),
            head::Kind::Symbolic(r) => r.attach(repo),
        };

        let mut items = Vec::new();
        for commit_id in reference.id().ancestors()?.all() {
            let commit_id = commit_id?;
            let (message, tree_id) = {
                let object = commit_id.object()?;
                let commit = object.to_commit();
                (commit.message.to_vec(), commit.tree())
            };

            let message = match message.to_str() {
                Err(_) => {
                    log::warn!(
                        "Commit message of {} could not be decoded to UTF-8 - ignored",
                        commit_id.as_ref()
                    );
                    continue;
                }
                Ok(m) => m,
            };
            items.push(commit::history::Item {
                id: commit_id.detach(),
                message: commit::Message::from(message),
                tree_data: repo.find_object(tree_id)?.data.to_owned(),
            });
        }
        repo.object_cache_size(prev)?;

        let elapsed = start.elapsed();
        log::trace!(
            "Cached commit history of {} commits and trees in {}s ({:.0} items/s)",
            items.len(),
            elapsed.as_secs_f32(),
            items.len() as f32 / elapsed.as_secs_f32()
        );
        Ok(Some(commit::History {
            head: reference.detach(),
            items,
        }))
    }

    /// Return the head reference followed by all tags affecting `crate_name` as per our tag name rules, ordered by ancestry.
    pub fn crate_ref_segments<'h>(
        package: &Package,
        ctx: &crate::Context,
        history: &'h commit::History,
    ) -> anyhow::Result<Vec<commit::history::Segment<'h>>> {
        let tag_prefix = tag_prefix(package, &ctx.repo);
        let start = Instant::now();
        let mut tags_by_commit = {
            let refs = ctx.repo.references()?;
            match tag_prefix {
                Some(prefix) => BTreeMap::from_iter(
                    refs.prefixed(PathBuf::from(format!("refs/tags/{}-", prefix)))?
                        .peeled()
                        .filter_map(|r| r.ok().map(|r| r.detach()))
                        .filter(|r| is_tag_name(prefix, strip_tag_path(r.name.to_ref())))
                        .map(|r| {
                            let t = r.peeled.expect("already peeled");
                            (t, r)
                        }),
                ),
                None => BTreeMap::from_iter(
                    refs.prefixed("refs/tags")?
                        .peeled()
                        .filter_map(|r| r.ok().map(|r| r.detach()))
                        .filter(|r| is_tag_version(strip_tag_path(r.name.to_ref())))
                        .map(|r| {
                            let t = r.peeled.expect("already peeled");
                            (t, r)
                        }),
                ),
            }
        };

        let elapsed = start.elapsed();
        log::trace!(
            "{}: Mapped {} tags in {}s ({:.0} refs/s)",
            package.name,
            tags_by_commit.len(),
            elapsed.as_secs_f32(),
            tags_by_commit.len() as f32 / elapsed.as_secs_f32()
        );

        let start = Instant::now();
        let mut segments = Vec::new();
        let mut segment = commit::history::Segment {
            head: history.head.to_owned(),
            history: vec![],
        };

        let dir = ctx.repo_relative_path(package);
        enum Filter<'a> {
            None,
            Fast(&'a [u8]),
            Slow(Vec<&'a [u8]>),
        }
        let filter = dir
            .map(|dir| {
                let mut components = dir.components().collect::<Vec<_>>();
                match components.len() {
                    0 => unreachable!("BUG: it's None if empty"),
                    1 => Filter::Fast(components.pop().map(component_to_bytes).expect("exactly one")),
                    _ => Filter::Slow(components.into_iter().map(component_to_bytes).collect()),
                }
            })
            .unwrap_or(Filter::None);

        let mut items = history.items.iter().peekable();
        while let Some(item) = items.next() {
            match tags_by_commit.remove(&item.id) {
                None => match filter {
                    Filter::None => segment.history.push(item),
                    Filter::Fast(comp) => {
                        let current = git::objs::TreeRefIter::from_bytes(&item.tree_data)
                            .filter_map(Result::ok)
                            .find(|e| e.filename == comp);
                        let parent = items.peek().and_then(|parent| {
                            git::objs::TreeRefIter::from_bytes(&parent.tree_data)
                                .filter_map(Result::ok)
                                .find(|e| e.filename == comp)
                        });
                        match (current, parent) {
                            (Some(current), Some(parent)) => {
                                if current.oid != parent.oid {
                                    segment.history.push(item)
                                }
                            }
                            (Some(_), None) => segment.history.push(item),
                            (None, Some(_)) | (None, None) => {}
                        };
                    }
                    Filter::Slow(ref components) => {
                        let prev = ctx.repo.object_cache_size(1024 * 1024)?;
                        let current_data = RefCell::new(item.tree_data.clone());
                        let current = git::easy::TreeRef::from_id_and_data(
                            item.id,
                            std::cell::Ref::map(current_data.borrow(), |v| v.as_slice()),
                            &ctx.repo,
                        )
                        .lookup_path(components.iter().copied())?;
                        let parent = match items.peek() {
                            Some(parent) => {
                                let parent_data = RefCell::new(parent.tree_data.clone());
                                git::easy::TreeRef::from_id_and_data(
                                    parent.id,
                                    std::cell::Ref::map(parent_data.borrow(), |v| v.as_slice()),
                                    &ctx.repo,
                                )
                                .lookup_path(components.iter().copied())?
                            }
                            None => None,
                        };
                        match (current, parent) {
                            (Some(current), Some(parent)) => {
                                if current.oid != parent.oid {
                                    segment.history.push(item)
                                }
                            }
                            (Some(_), None) => segment.history.push(item),
                            (None, Some(_)) | (None, None) => {}
                        };
                        ctx.repo.object_cache_size(prev)?;
                    }
                },
                Some(next_ref) => segments.push(std::mem::replace(
                    &mut segment,
                    commit::history::Segment {
                        head: next_ref,
                        history: vec![item],
                    },
                )),
            }
        }
        segments.push(segment);

        if !tags_by_commit.is_empty() {
            log::warn!(
                "{}: The following tags were on branches which are ignored during traversal: {}",
                package.name,
                tags_by_commit
                    .into_values()
                    .map(|v| v.name.as_bstr().to_str_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }

        let elapsed = start.elapsed();
        let num_commits = segments.iter().map(|s| s.history.len()).sum::<usize>();
        log::trace!(
            "{}: Found {} relevant commits out of {} in {} segments {}s ({:.0} commits/s)",
            package.name,
            num_commits,
            history.items.len(),
            segments.len(),
            elapsed.as_secs_f32(),
            num_commits as f32 / elapsed.as_secs_f32()
        );

        Ok(segments)
    }
}

pub fn strip_tag_path(name: FullNameRef<'_>) -> &BStr {
    try_strip_tag_path(name).expect("prefix iteration works")
}

pub fn try_strip_tag_path(name: FullNameRef<'_>) -> Option<&BStr> {
    name.as_bstr().strip_prefix(b"refs/tags/").map(|b| b.as_bstr())
}
