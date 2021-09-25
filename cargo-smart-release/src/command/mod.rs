pub mod release {
    #[derive(Debug, Clone, Copy)]
    pub struct Options {
        pub dry_run: bool,
        pub allow_dirty: bool,
        pub ignore_instability: bool,
        pub skip_publish: bool,
        pub dry_run_cargo_publish: bool,
        pub conservative_pre_release_version_handling: bool,
        /// Pass --no-verify unconditionally to cargo publish. Really just for fixing things
        pub no_verify: bool,
        pub skip_tag: bool,
        pub allow_auto_publish_of_stable_crates: bool,
        pub update_crates_index: bool,
        pub bump_when_needed: bool,
        pub verbose: bool,
        pub skip_push: bool,
        pub skip_dependencies: bool,
        pub multi_crate_release: bool,
        pub isolate_dependencies_from_breaking_changes: bool,
    }
}
#[path = "release/mod.rs"]
mod release_impl;
pub use release_impl::release;

pub mod changelog {
    #[derive(Debug, Clone, Copy)]
    pub struct Options {
        pub dry_run: bool,
        pub dependencies: bool,
        pub allow_dirty: bool,
        pub preview: bool,
    }
}
#[path = "changelog/mod.rs"]
mod changelog_impl;
pub use changelog_impl::changelog;
