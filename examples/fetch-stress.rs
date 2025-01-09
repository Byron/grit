use gix::interrupt::IS_INTERRUPTED;
use gix::remote::Direction;

fn main() -> anyhow::Result<()> {
    let remote_dir = tempfile::tempdir()?;
    let remote_repo = gix::init_bare(remote_dir.path())?;
    create_empty_commit(&remote_repo)?;

    let local_dir = tempfile::tempdir()?;
    let (local_repo, _) = gix::prepare_clone_bare(remote_repo.path(), local_dir.path())?
        .fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

    unsafe { gix::interrupt::init_handler(0, gix::interrupt::trigger)? };

    let remote = local_repo
        .branch_remote(
            local_repo.head_ref()?.expect("branch available").name().shorten(),
            Direction::Fetch,
        )
        .expect("remote is configured after clone")?;
    for round in 1.. {
        if gix::interrupt::is_triggered() {
            eprintln!("Round {round} aborted by user - cleaning up");
            break;
        }

        eprintln!("Fetch number {round}…");
        create_empty_commit(&remote_repo)?;
        let out = remote
            .connect(Direction::Fetch)?
            .prepare_fetch(gix::progress::Discard, Default::default())?
            .receive(gix::progress::Discard, &IS_INTERRUPTED)?;
        for local_tracking_branch_name in out.ref_map.mappings.into_iter().filter_map(|m| m.local) {
            let r = local_repo.find_reference(&local_tracking_branch_name)?;
            r.id()
                .object()
                .expect("object should be present after fetching, triggering pack refreshes works");
        }
    }
    Ok(())
}

fn create_empty_commit(repo: &gix::Repository) -> anyhow::Result<()> {
    let name = repo.head_name()?.expect("no detached head");
    repo.commit(
        name.as_bstr(),
        "empty",
        gix::hash::ObjectId::empty_tree(repo.object_hash()),
        repo.try_find_reference(name.as_ref())?.map(|r| r.id()),
    )?;
    Ok(())
}
