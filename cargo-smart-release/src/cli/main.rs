mod options;
use options::{Args, ChangeLog, SmartRelease, SubCommands};

use cargo_smart_release::command;

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();
    init_logging();

    match args.subcommands {
        SubCommands::Changelog(ChangeLog {
            write,
            crates,
            no_dependencies,
            no_preview,
            allow_dirty,
        }) => command::changelog(
            command::changelog::Options {
                dry_run: !write,
                allow_dirty,
                preview: !no_preview,
                dependencies: !no_dependencies,
            },
            crates,
        )?,
        SubCommands::SmartRelease(SmartRelease {
            execute,
            verbose,
            bump,
            bump_dependencies,
            crates,
            allow_dirty,
            ignore_instability,
            skip_publish,
            skip_tag,
            skip_push,
            dangerously_pass_no_verify,
            no_auto_publish_of_stable_crates,
            no_conservative_pre_release_version_handling,
            dry_run_cargo_publish,
            update_crates_index,
            no_bump_on_demand,
            no_dependencies,
            no_multi_crate_release,
            no_isolate_dependencies_from_breaking_changes,
        }) => command::release(
            command::release::Options {
                dry_run: !execute,
                verbose: execute || verbose,
                conservative_pre_release_version_handling: !no_conservative_pre_release_version_handling,
                bump_when_needed: !no_bump_on_demand,
                multi_crate_release: !no_multi_crate_release,
                isolate_dependencies_from_breaking_changes: !no_isolate_dependencies_from_breaking_changes,
                allow_dirty,
                ignore_instability,
                skip_publish,
                skip_tag,
                skip_push,
                skip_dependencies: no_dependencies,
                dry_run_cargo_publish,
                no_verify: dangerously_pass_no_verify,
                allow_auto_publish_of_stable_crates: !no_auto_publish_of_stable_crates,
                update_crates_index,
            },
            crates,
            bump.unwrap_or_else(|| "keep".into()),
            bump_dependencies.unwrap_or_else(|| "keep".into()),
        )?,
    };

    Ok(())
}

fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_module_path(false)
        .format_target(false)
        .format_timestamp(None)
        .init();
}
