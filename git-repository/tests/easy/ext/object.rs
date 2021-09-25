mod write_object {
    use git_repository::prelude::{ObjectAccessExt, RepositoryAccessExt};

    #[test]
    fn empty_tree() -> crate::Result {
        let tmp = tempfile::tempdir()?;
        let repo = git_repository::init_bare(&tmp)?.into_easy();
        let oid = repo.write_object(&git_repository::objs::TreeRef::empty())?;
        assert_eq!(
            oid,
            git_repository::hash::ObjectId::empty_tree(repo.hash_kind()?),
            "it produces a well-known empty tree id"
        );
        Ok(())
    }
}

mod find {
    use git_repository::prelude::{CacheAccessExt, ReferenceAccessExt};

    #[test]
    fn find_and_try_find_with_and_without_object_cache() -> crate::Result {
        let repo = crate::basic_repo()?;
        for round in 1..=2 {
            match round {
                1 => assert!(
                    repo.object_cache_size(None)?.is_none(),
                    "default is to have no object cache"
                ),
                2 => {
                    repo.object_cache_size(128 * 1024)?;
                }
                _ => unreachable!("BUG"),
            }
            for commit_id in repo.head()?.peeled()?.id().expect("born").ancestors()?.all() {
                let commit = commit_id?;
                assert_eq!(commit.object()?.kind, git_object::Kind::Commit);
                if round == 2 {
                    assert_eq!(
                        commit.object()?.kind,
                        git_object::Kind::Commit,
                        "repeated request triggers cache and doesn't fail"
                    );
                }
                assert_eq!(commit.try_object()?.expect("exists").kind, git_object::Kind::Commit,);
            }

            if round == 2 {
                assert_eq!(
                    repo.object_cache_size(None)?,
                    Some(128 * 1024),
                    "it returns the previous cache"
                );
            }
        }
        Ok(())
    }
}

mod commit {
    use git_repository as git;
    use git_repository::prelude::{ObjectAccessExt, ReferenceAccessExt};
    use git_testtools::hex_to_id;
    #[test]
    fn parent_in_initial_commit_causes_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = git::init(&tmp).unwrap().into_easy();
        let empty_tree_id = repo.write_object(&git::objs::Tree::empty()).unwrap().detach();
        let author = git::actor::Signature::empty();
        let err = repo
            .commit(
                "HEAD",
                &author.to_ref(),
                &author.to_ref(),
                "initial",
                empty_tree_id,
                [empty_tree_id],
            )
            .unwrap_err();
        assert_eq!(err.to_string(), "Reference 'refs/heads/main' was supposed to exist with value 4b825dc642cb6eb9a060e54bf8d69288fbee4904, but didn't.", "cannot provide parent id in initial commit");
    }

    #[test]
    fn single_line_initial_commit_empty_tree_ref_nonexisting() -> crate::Result {
        let tmp = tempfile::tempdir()?;
        let repo = git::init(&tmp)?.into_easy();
        let empty_tree_id = repo.write_object(&git::objs::Tree::empty())?;
        let author = git::actor::Signature::empty();
        let commit_id = repo.commit(
            "HEAD",
            &author.to_ref(),
            &author.to_ref(),
            "initial",
            empty_tree_id,
            git::commit::NO_PARENT_IDS,
        )?;
        assert_eq!(
            commit_id,
            hex_to_id("302ea5640358f98ba23cda66c1e664a6f274643f"),
            "the commit id is stable"
        );

        let head = repo.head()?.into_referent();
        assert_eq!(head.name().as_bstr(), "refs/heads/main", "'main' is the default name");
        assert_eq!(
            head.logs()?
                .iter_rev()?
                .expect("log present")
                .next()
                .expect("one line")?
                .message,
            "commit (initial): initial"
        );
        Ok(())
    }

    #[test]
    fn multi_line_commit_message_uses_first_line_in_ref_log_ref_nonexisting() -> crate::Result {
        let (repo, _keep) = crate::basic_rw_repo()?;
        let parent = repo.find_reference("HEAD")?.peel_to_id_in_place()?;
        let empty_tree_id = parent.object()?.to_commit_iter().tree_id().expect("tree to be set");
        assert_eq!(
            parent
                .try_object()?
                .expect("present")
                .to_commit_iter()
                .tree_id()
                .expect("tree to be set"),
            empty_tree_id,
            "try and non-try work the same"
        );
        let author = git::actor::Signature::empty();
        let first_commit_id = repo.commit(
            "HEAD",
            &author.to_ref(),
            &author.to_ref(),
            "hello there \r\n\nthe body",
            empty_tree_id,
            Some(parent),
        )?;
        assert_eq!(
            first_commit_id,
            hex_to_id("1ff7decccf76bfa15bfdb0b66bac0c9144b4b083"),
            "the commit id is stable"
        );

        let head_log_entries: Vec<_> = repo
            .head()?
            .logs()?
            .iter_rev()?
            .expect("log present")
            .map(Result::unwrap)
            .map(|l| l.message)
            .collect();
        assert_eq!(
            head_log_entries,
            vec!["commit: hello there", "commit: c2", "commit (initial): c1"],
            "we get the actual HEAD log, not the log of some reference"
        );
        let current_commit = repo.head()?.into_fully_peeled_id().expect("born")?;
        assert_eq!(current_commit, first_commit_id, "the commit was set");

        let second_commit_id = repo.commit(
            "refs/heads/new-branch",
            &author.to_ref(),
            &author.to_ref(),
            "committing into a new branch creates it",
            empty_tree_id,
            Some(first_commit_id),
        )?;

        assert_eq!(
            second_commit_id,
            hex_to_id("b0d041ade77e51d31c79c7147fb769336ccc77b1"),
            "the second commit id is stable"
        );

        let mut branch = repo.find_reference("new-branch")?;
        let current_commit = branch.peel_to_id_in_place()?;
        assert_eq!(current_commit, second_commit_id, "the commit was set");

        let mut log = branch.logs()?;
        let mut log_iter = log.iter_rev()?.expect("log present");
        assert_eq!(
            log_iter.next().expect("one line")?.message,
            "commit: committing into a new branch creates it"
        );
        assert!(
            log_iter.next().is_none(),
            "there is only one log line in the new branch"
        );
        Ok(())
    }
}
