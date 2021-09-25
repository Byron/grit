use cargo_smart_release::{
    changelog::{section, Section, Version},
    ChangeLog,
};

#[test]
fn all_unknown_in_section() {
    let log = ChangeLog::from_markdown(include_str!("fixtures/known-section-unknown-content.md"));
    assert_eq!(
        log.sections,
        vec![
            Section::Release {
                name: Version::Unreleased,
                date: None,
                heading_level: 3,
                thanks_clippy_count: 0,
                segments: vec![section::Segment::Unknown {
                    text: "- hello ~~this is not understood~~\n* this isn't either\n\n".into()
                }],
                unknown: String::new(),
            },
            Section::Release {
                name: Version::Semantic("1.0.0".parse().unwrap()),
                date: None,
                heading_level: 4,
                thanks_clippy_count: 0,
                segments: vec![section::Segment::Unknown {
                    text: "Some free text in a paragraph\nthat won't parse.\n".into()
                }],
                unknown: String::new(),
            }
        ]
    )
}

#[test]
fn unknown_link_and_headling() {
    let log = ChangeLog::from_markdown(include_str!("fixtures/known-section-unknown-headline-with-link.md"));
    assert_eq!(
        log.sections,
        vec![Section::Release {
            name: Version::Unreleased,
            date: None,
            heading_level: 4,
            thanks_clippy_count: 0,
            segments: vec![section::Segment::Unknown {
                text: "##### Special\n\nHello [there][194] period.\n".into()
            }],
            unknown: String::new(),
        },]
    )
}
