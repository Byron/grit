use argh::FromArgs;

#[derive(FromArgs)]
#[argh(name = "cargo smart-release")]
/// Release workspace crates fearlessly.
///
/// Use --execute to actually perform the operation.
pub struct Args {
    #[argh(subcommand)]
    pub subcommands: SubCommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum SubCommands {
    SmartRelease(SmartRelease),
    Changelog(ChangeLog),
}
#[derive(FromArgs)]
#[argh(subcommand, name = "changelog")]
/// Generate changelogs from commit histories, none-destructively.
///
/// Use --write to actually write generated changelogs
pub struct ChangeLog {
    /// actually write the changelog to the respective files
    #[argh(switch, short = 'w')]
    pub write: bool,

    /// allow changelog updates to take place on a dirty working tree when --write is set as well.
    ///
    /// For now this is not recommended as changelogs might be damaged beyond repair.
    #[argh(switch)]
    pub allow_dirty: bool,

    /// if --write is not set, 'bat' will be used (if available) to print the new changelog to stdout as preview. Use this flag
    /// to disable such behaviour.
    #[argh(switch)]
    pub no_preview: bool,

    /// do not take into consideration any dependencies of the crates to generate the changelog for.
    ///
    /// This flag is useful if you plan to review and finalize changelogs before a a smart-release, where dependencies
    /// are taken into consideration by default, but would like to do so one at a time.
    #[argh(switch)]
    pub no_dependencies: bool,

    /// the name of the crates to generate a changelog for.
    ///
    /// Defaults to the top-level workspace crate if unset.
    #[argh(positional)]
    pub crates: Vec<String>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "smart-release")]
/// Release workspace crates fearlessly.
///
/// Use --execute to actually perform the operation.
pub struct SmartRelease {
    /// actually perform a release. Dry-run mode is the default
    #[argh(switch, short = 'n')]
    pub execute: bool,

    /// provide more detailed messages on the INFO log level in dry-run mode.
    ///
    /// Note --verbose is implied with --execute.
    #[argh(switch, short = 'v')]
    pub verbose: bool,

    /// as dependencies are automatically picked up for release, put all manifest changes into a single commit.
    ///
    /// If this flag is set, each dependency will yield its own commit with respective manifest changes, possibly
    /// adding a lot of additional commits even though the release could have looked like coherent.
    #[argh(switch)]
    pub no_multi_crate_release: bool,

    /// always bump versions as specified by --bump or --bump-dependencies even if this is not required.
    ///
    /// If it's required or not is determined by looking at the published versions in the crates index.
    #[argh(switch)]
    pub no_bump_on_demand: bool,

    /// additionally run 'cargo publish --dry-run' when --execute is not set. This can be useful to see which local
    /// crates do not build with the released versions of their workspace dependencies anymore.
    #[argh(switch)]
    pub dry_run_cargo_publish: bool,

    /// allow publishes to take place on a dirty working tree. Really not recommended alongside --execute.
    #[argh(switch)]
    pub allow_dirty: bool,

    /// always update the crates-index beforehand. It is used to determine if the computed version to be published was
    /// already published.
    #[argh(switch, short = 'u')]
    pub update_crates_index: bool,

    /// disallow to also publish stable crates when discoverying changed crates, bumping their version according to -d <spec>.
    #[argh(switch)]
    pub no_auto_publish_of_stable_crates: bool,

    /// do not force pre-release version requirements to be updated in dependent crates on patch update, forcing
    /// an update of the lower bound.
    ///
    /// It's best to look at this issue for a deeper understanding: https://github.com/Byron/gitoxide/issues/194
    #[argh(switch)]
    pub no_conservative_pre_release_version_handling: bool,

    /// do not bump versions of dependent crates if the crates to be published indicate breaking changes with their semantic version.
    ///
    /// For details, it's best to look at https://github.com/Byron/gitoxide/issues/192
    #[argh(switch)]
    pub no_isolate_dependencies_from_breaking_changes: bool,

    /// don't actually publish, but perform all other operations like manifest adjustments and tag creation.
    #[argh(switch)]
    pub skip_publish: bool,

    /// don't create tags indicating the version numbers of all crates that are to be published after changing
    /// their manifests.
    #[argh(switch)]
    pub skip_tag: bool,

    /// don't push tags and the HEAD branch after any successful run of `cargo publish`.
    #[argh(switch)]
    pub skip_push: bool,

    /// do not take into consideration any dependencies of the crates to publish.
    ///
    /// This flag is useful when various `--skip-X` are specified in order to bump versions only, without publishing.
    #[argh(switch)]
    pub no_dependencies: bool,

    /// pass --no-verify to 'cargo publish' which should only be a last resort when fixing up packages that
    /// otherwise wouldn't publish, but need to be publish to resolve the situation.
    #[argh(switch)]
    pub dangerously_pass_no_verify: bool,

    /// if set it will be allowed to publish crates with cycles to other workspace crates which are not published.
    ///
    /// Doing so causes repeated publishes to never stabilize, as one set of dependencies destabilizes another set.
    /// A possible fix is to bump version numbers of all crates in the set at once and publishing those who would
    /// depend on an unpublished version with "--no-validate".
    #[argh(switch)]
    pub ignore_instability: bool,

    /// specify the kind of version bump you seek for the crate and potentially it's dependencies.
    ///
    /// Can be 'major', 'minor' or 'patch', or 'keep' which doesn't alter the version.
    /// If unspecified, the current version will be kept, useful if versions are specified manually.
    #[argh(option, short = 'b')]
    pub bump: Option<String>,

    /// specify the kind of version bump to apply to dependencies only.
    ///
    /// Can be 'major', 'minor' or 'patch', or 'keep' which doesn't alter the version.
    /// If unspecified, "keep" will be used.
    #[argh(option, short = 'd')]
    pub bump_dependencies: Option<String>,

    /// the name of the crates to be released, along with all of their dependencies if needed.
    ///
    /// Defaults to the top-level workspace crate if unset.
    #[argh(positional)]
    pub crates: Vec<String>,
}
