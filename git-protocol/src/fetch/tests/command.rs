mod v1 {
    fn capabilities(input: &str) -> git_transport::client::Capabilities {
        git_transport::client::Capabilities::from_bytes(format!("\0{}", input).as_bytes())
            .expect("valid input capabilities")
            .0
    }

    const GITHUB_CAPABILITIES: &str = "multi_ack thin-pack side-band ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag allow-tip-sha1-in-want allow-reachable-sha1-in-want no-done symref=HEAD:refs/heads/main filter agent=git/github-gdf51a71f0236";
    mod fetch {
        mod default_features {
            use crate::fetch::{
                self,
                tests::command::v1::{capabilities, GITHUB_CAPABILITIES},
                Command,
            };

            #[test]
            fn it_chooses_the_best_multi_ack_and_sideband() {
                assert_eq!(
                    Command::Fetch.default_features(
                        git_transport::Protocol::V1,
                        &capabilities("multi_ack side-band side-band-64k multi_ack_detailed")
                    ),
                    &[("side-band-64k", None), ("multi_ack_detailed", None), fetch::agent()]
                );
            }

            #[test]
            fn it_chooses_all_supported_non_stacking_capabilities_and_leaves_no_progress() {
                assert_eq!(
                    Command::Fetch.default_features(git_transport::Protocol::V1, &capabilities(GITHUB_CAPABILITIES)),
                    &[
                        ("multi_ack", None),
                        ("thin-pack", None),
                        ("side-band", None),
                        ("ofs-delta", None),
                        ("shallow", None),
                        ("deepen-since", None),
                        ("deepen-not", None),
                        ("deepen-relative", None),
                        ("allow-tip-sha1-in-want", None),
                        ("allow-reachable-sha1-in-want", None),
                        ("no-done", None),
                        ("filter", None),
                        fetch::agent()
                    ],
                    "we don't enforce include-tag or no-progress"
                );
            }
        }
    }
    mod ls_refs {
        mod validate_arguments {
            use crate::{fetch::tests::command::v1::capabilities, fetch::Command};
            use bstr::ByteSlice;

            #[test]
            #[should_panic]
            fn ref_prefixes_cannot_be_used() {
                Command::LsRefs.validate_argument_prefixes_or_panic(
                    git_transport::Protocol::V1,
                    &capabilities("do-not-matter"),
                    &[b"ref-prefix hello/".as_bstr().into()],
                    &[],
                );
            }
        }
        mod initial_arguments {
            use crate::fetch::Command;
            use bstr::ByteSlice;

            #[test]
            fn default_as_there_are_no_features_yet() {
                assert_eq!(
                    Command::LsRefs.initial_arguments(&[]),
                    &[b"symrefs".as_bstr(), b"peel".as_bstr()]
                );
            }
        }
    }
}

mod v2 {
    use git_transport::client::Capabilities;

    fn capabilities(command: &str, input: &str) -> Capabilities {
        Capabilities::from_lines(format!("version 2\n{}={}", command, input).as_bytes())
            .expect("valid input for V2 capabilities")
    }

    mod fetch {
        mod default_features {
            use crate::fetch::{self, tests::command::v2::capabilities, Command};

            #[test]
            fn all_features() {
                assert_eq!(
                    Command::Fetch.default_features(
                        git_transport::Protocol::V2,
                        &capabilities("fetch", "shallow filter ref-in-want sideband-all packfile-uris")
                    ),
                    ["shallow", "filter", "ref-in-want", "sideband-all", "packfile-uris"]
                        .iter()
                        .map(|s| (*s, None))
                        .chain(Some(fetch::agent()))
                        .collect::<Vec<_>>()
                )
            }
        }

        mod initial_arguments {
            use crate::fetch::{tests::command::v2::capabilities, Command};
            use bstr::ByteSlice;

            #[test]
            fn for_all_features() {
                assert_eq!(
                    Command::Fetch.initial_arguments(&Command::Fetch.default_features(
                        git_transport::Protocol::V2,
                        &capabilities("fetch", "shallow filter ref-in-want sideband-all packfile-uris")
                    )),
                    [
                        "thin-pack",
                        "include-tag",
                        "ofs-delta",
                        "sideband-all",
                        "ref-in-want",
                        "packfile-uris"
                    ]
                    .iter()
                    .map(|s| s.as_bytes().as_bstr().to_owned())
                    .collect::<Vec<_>>()
                )
            }
        }
    }

    mod ls_refs {
        mod default_features {
            use crate::fetch::{self, tests::command::v2::capabilities, Command};

            #[test]
            fn default_as_there_are_no_features() {
                assert_eq!(
                    Command::LsRefs.default_features(
                        git_transport::Protocol::V2,
                        &capabilities("something-else", "does not matter as there are none")
                    ),
                    &[fetch::agent()]
                );
            }
        }

        mod validate_arguments {
            use crate::fetch::{tests::command::v2::capabilities, Command};
            use bstr::ByteSlice;

            #[test]
            fn ref_prefixes_always_be_used() {
                Command::LsRefs.validate_argument_prefixes_or_panic(
                    git_transport::Protocol::V2,
                    &capabilities("something else", "do-not-matter"),
                    &[b"ref-prefix hello/".as_bstr().into()],
                    &[],
                );
            }
        }
    }
}
