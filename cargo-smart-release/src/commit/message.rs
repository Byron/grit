use git_conventional::Type;
use git_repository as git;
use git_repository::bstr::{BStr, ByteSlice};

use crate::commit::Message;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Addition {
    /// The plain issue ID, like "123".
    IssueId(String),
}

mod additions {
    use std::{borrow::Cow, ops::Range};

    use crate::commit::message::Addition;

    fn cut(mut s: String, Range { start, end }: Range<usize>) -> String {
        let part_to_left = &s[..start];
        let new_start = part_to_left
            .rfind(|c: char| !c.is_whitespace())
            .and_then(|p| {
                part_to_left
                    .is_char_boundary(p + 1)
                    .then(|| p + 1)
                    .or_else(|| part_to_left[p..].chars().next().map(|c| p + c.len_utf8()))
            })
            .unwrap_or(start);
        let new_end = s[end..]
            .find(|c: char| !c.is_whitespace())
            .map(|p| p + end)
            .unwrap_or(end);
        s.replace_range(
            new_start..new_end,
            if new_end != end && new_start != start { " " } else { "" },
        );
        s
    }

    pub fn strip(mut title: Cow<'_, str>) -> (Cow<'_, str>, Vec<Addition>) {
        let mut additions = Vec::new();
        loop {
            let previous_len = title.len();
            let issue_sep = "(#";
            if let Some((pos, end_pos)) = title.find(issue_sep).and_then(|mut pos| {
                pos += issue_sep.len();
                title[pos..].find(')').map(|ep| (pos, ep))
            }) {
                additions.push(Addition::IssueId(title[pos..][..end_pos].to_owned()));
                title = cut(title.into_owned(), (pos - issue_sep.len())..(pos + end_pos + 1)).into();
            };
            if title.len() == previous_len {
                break;
            }
        }
        (title, additions)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn no_addition() {
            let (nt, a) = strip("hello there [abc] (abc)".into());
            assert_eq!(nt, "hello there [abc] (abc)");
            assert_eq!(a, vec![]);
        }

        #[test]
        fn strip_multiple_issue_numbers() {
            let (nt, a) = strip("(#other) foo (#123) hello (#42)".into());
            assert_eq!(nt, "foo hello");
            assert_eq!(
                a,
                vec![
                    Addition::IssueId("other".into()),
                    Addition::IssueId("123".into()),
                    Addition::IssueId("42".into())
                ]
            );
        }
    }
}

impl From<&'_ str> for Message {
    fn from(m: &str) -> Self {
        let (title, kind, body, breaking, breaking_description) = git_conventional::Commit::parse(m)
            .map(|c: git_conventional::Commit<'_>| {
                (
                    c.description().into(),
                    Some(c.type_()),
                    c.body().map(Into::into),
                    c.breaking(),
                    c.breaking_description()
                        .and_then(|d| if d == c.description() { None } else { Some(d) }),
                )
            })
            .unwrap_or_else(|_| {
                let m = git::objs::commit::MessageRef::from_bytes(m.as_bytes());
                (
                    m.summary().as_ref().to_string().into(),
                    None,
                    m.body().map(|b| b.without_trailer().to_str_lossy()),
                    false,
                    None,
                )
            });
        let (title, additions) = additions::strip(title);
        Message {
            title: title.into_owned(),
            kind: to_static(kind),
            body: body.map(|b| b.into_owned()),
            breaking,
            breaking_description: breaking_description.map(ToOwned::to_owned),
            additions,
        }
    }
}

fn to_static(kind: Option<git_conventional::Type<'_>>) -> Option<&'static str> {
    kind.map(|kind| match kind.as_str() {
        "feat" => "feat",
        "fix" => "fix",
        "revert" => "revert",
        "docs" => "docs",
        "style" => "style",
        "refactor" => "refactor",
        "perf" => "perf",
        "test" => "test",
        "chore" => "chore",
        _ => "other",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_conventional_no_additions_no_body() {
        assert_eq!(
            Message::from("hi"),
            Message {
                title: "hi".into(),
                body: None,
                kind: None,
                breaking: false,
                breaking_description: None,
                additions: vec![]
            }
        )
    }

    #[test]
    fn no_conventional_uses_summary() {
        assert_eq!(
            Message::from("hi\nho\nfoo\n\nbody"),
            Message {
                title: "hi ho foo".into(),
                body: Some("body".into()),
                kind: None,
                breaking: false,
                breaking_description: None,
                additions: vec![]
            }
        )
    }

    #[test]
    fn no_conventional_additions() {
        assert_eq!(
            Message::from("hi (#14123)\n\nbody\nother\n\nSigned: bar"),
            Message {
                title: "hi".into(),
                body: Some("body\nother".into()),
                kind: None,
                breaking: false,
                breaking_description: None,
                additions: vec![Addition::IssueId("14123".into())]
            }
        )
    }

    #[test]
    fn conventional_with_additions() {
        assert_eq!(
            Message::from("feat!: hi (#123)\n\nthe body\nBREAKING-CHANGE: breaks\n\nSigned: foobar"),
            Message {
                title: "hi".into(),
                body: Some("the body".into()),
                kind: Some("feat"),
                breaking: true,
                breaking_description: Some("breaks".into()),
                additions: vec![Addition::IssueId("123".into())]
            }
        )
    }
}
