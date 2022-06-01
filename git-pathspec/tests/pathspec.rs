use git_attributes::State;
use git_pathspec::parse::Error;
use git_pathspec::{MagicSignature, Pattern, SearchMode};

#[test]
fn can_parse_empty_signatures() {
    let inputs = vec![
        ("some/path", pat_with_path("some/path")),
        (":some/path", pat_with_path("some/path")),
        (":()some/path", pat_with_path("some/path")),
        ("::some/path", pat_with_path("some/path")),
        (":::some/path", pat_with_path(":some/path")),
    ];

    check_valid_inputs(inputs)
}

#[test]
fn can_parse_short_signatures() {
    let inputs = vec![
        (":/some/path", pat_with_path_and_sig("some/path", MagicSignature::TOP)),
        (
            ":^some/path",
            pat_with_path_and_sig("some/path", MagicSignature::EXCLUDE),
        ),
        (
            ":!some/path",
            pat_with_path_and_sig("some/path", MagicSignature::EXCLUDE),
        ),
        (
            ":/!some/path",
            pat_with_path_and_sig("some/path", MagicSignature::TOP | MagicSignature::EXCLUDE),
        ),
        (
            ":!/^/:",
            pat_with_path_and_sig("", MagicSignature::TOP | MagicSignature::EXCLUDE),
        ),
    ];

    check_valid_inputs(inputs)
}

#[test]
fn can_parse_signatures_and_searchmodes() {
    let inputs = vec![
        (":(top)", pat_with_path_and_sig("", MagicSignature::TOP)),
        (":(icase)", pat_with_path_and_sig("", MagicSignature::ICASE)),
        (":(attr)", pat_with_path_and_sig("", MagicSignature::ATTR)),
        (":(exclude)", pat_with_path_and_sig("", MagicSignature::EXCLUDE)),
        (
            ":(literal)",
            pat("", MagicSignature::empty(), SearchMode::Literal, vec![]),
        ),
        (":(glob)", pat("", MagicSignature::empty(), SearchMode::Glob, vec![])),
        (
            ":(top,exclude)",
            pat_with_path_and_sig("", MagicSignature::TOP | MagicSignature::EXCLUDE),
        ),
        (
            ":(icase,literal)",
            pat("", MagicSignature::ICASE, SearchMode::Literal, vec![]),
        ),
        (
            ":!(literal)some/*path",
            pat("some/*path", MagicSignature::EXCLUDE, SearchMode::Literal, vec![]),
        ),
        (
            ":(top,literal,icase,attr,exclude)some/path",
            pat("some/path", MagicSignature::all(), SearchMode::Literal, vec![]),
        ),
        (
            ":(top,glob,icase,attr,exclude)some/path",
            pat("some/path", MagicSignature::all(), SearchMode::Glob, vec![]),
        ),
    ];

    check_valid_inputs(inputs);
}

#[test]
fn can_parse_attributes_in_signature() {
    let inputs = vec![
        (
            ":(attr:someAttr)",
            pat(
                "",
                MagicSignature::ATTR,
                SearchMode::Default,
                vec![("someAttr", State::Set)],
            ),
        ),
        (
            ":(attr:!someAttr)",
            pat(
                "",
                MagicSignature::ATTR,
                SearchMode::Default,
                vec![("someAttr", State::Unspecified)],
            ),
        ),
        (
            ":(attr:-someAttr)",
            pat(
                "",
                MagicSignature::ATTR,
                SearchMode::Default,
                vec![("someAttr", State::Unset)],
            ),
        ),
        (
            ":(attr:someAttr=value)",
            pat(
                "",
                MagicSignature::ATTR,
                SearchMode::Default,
                vec![("someAttr", State::Value("value".into()))],
            ),
        ),
        (
            ":(attr:someAttr anotherAttr)",
            pat(
                "",
                MagicSignature::ATTR,
                SearchMode::Default,
                vec![("someAttr", State::Set), ("anotherAttr", State::Set)],
            ),
        ),
    ];

    check_valid_inputs(inputs)
}

#[test]
fn should_fail_on_empty_input() {
    let input = "";

    assert!(!is_valid_in_git(input), "This pathspec is valid in git: {}", input);

    let output = git_pathspec::parse(input.as_bytes());
    assert!(output.is_err());
    assert!(matches!(output.unwrap_err(), Error::EmptyString { .. }));
}

#[test]
fn should_fail_on_invalid_keywords() {
    let inputs = vec![
        ":( )some/path",
        ":(tp)some/path",
        ":(top, exclude)some/path",
        ":(top,exclude,icse)some/path",
    ];

    inputs.into_iter().for_each(|input| {
        assert!(!is_valid_in_git(input), "This pathspec is valid in git: {}", input);

        let output = git_pathspec::parse(input.as_bytes());
        assert!(output.is_err());
        assert!(matches!(output.unwrap_err(), Error::InvalidKeyword { .. }));
    });
}

#[test]
fn should_fail_on_invalid_attributes() {
    let inputs = vec![
        ":(attr:+invalidAttr)some/path",
        ":(attr:validAttr +invalidAttr)some/path",
    ];

    for input in inputs {
        assert!(!is_valid_in_git(input), "This pathspec is valid in git: {}", input);

        let output = git_pathspec::parse(input.as_bytes());
        assert!(output.is_err());
        assert!(matches!(output.unwrap_err(), Error::InvalidAttribute { .. }));
    }
}

#[test]
fn should_fail_on_missing_parentheses() {
    let input = ":(top";

    assert!(!is_valid_in_git(input), "This pathspec is valid in git: {}", input);

    let output = git_pathspec::parse(input.as_bytes());
    assert!(output.is_err());
    assert!(matches!(output.unwrap_err(), Error::MissingClosingParenthesis { .. }));
}

#[test]
fn should_fail_on_glob_and_literal_present() {
    let input = ":(glob,literal)some/path";

    assert!(!is_valid_in_git(input), "This pathspec is valid in git: {}", input);

    let output = git_pathspec::parse(input.as_bytes());
    assert!(output.is_err());
    assert!(matches!(output.unwrap_err(), Error::IncompatibleSearchmodes));
}

// --- Helpers ---

fn check_valid_inputs(inputs: Vec<(&str, Pattern)>) {
    inputs.into_iter().for_each(|(input, expected)| {
        assert!(is_valid_in_git(input), "This pathspec is invalid in git: {}", input);

        let pattern = git_pathspec::parse(input.as_bytes()).expect("parsing should not fail");
        assert_eq!(pattern, expected, "while checking input: \"{}\"", input);
    });
}

fn pat_with_path(path: &str) -> Pattern {
    pat_with_path_and_sig(path, MagicSignature::empty())
}

fn pat_with_path_and_sig(path: &str, signature: MagicSignature) -> Pattern {
    pat(path, signature, SearchMode::Default, vec![])
}

fn pat(path: &str, signature: MagicSignature, searchmode: SearchMode, attributes: Vec<(&str, State)>) -> Pattern {
    Pattern {
        path: path.into(),
        signature,
        searchmode,
        attributes: attributes
            .into_iter()
            .map(|(attr, state)| (attr.into(), state))
            .collect(),
    }
}

// TODO: Cache results instead of running them with each test run
fn is_valid_in_git(pathspec: &str) -> bool {
    use std::process::Command;

    let output = Command::new("git")
        .args(["ls-files", pathspec])
        .output()
        .expect("failed to execute process");

    output.status.success()
}
