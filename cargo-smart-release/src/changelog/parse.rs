use std::{convert::TryFrom, ops::Range, str::FromStr};

use git_repository::bstr::ByteSlice;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till, take_while, take_while_m_n},
    combinator::{all_consuming, map, map_res, opt},
    error::{FromExternalError, ParseError},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};
use pulldown_cmark::{Event, OffsetIter, Tag};

use crate::{
    changelog,
    changelog::{section, Section},
    ChangeLog,
};

impl ChangeLog {
    /// Obtain as much information as possible from `input` and keep everything we didn't understand in respective sections.
    pub fn from_markdown(input: &str) -> ChangeLog {
        let mut sections = Vec::new();
        let mut plain_text = String::new();
        let mut previous_headline = None;
        for line in input.as_bytes().as_bstr().lines_with_terminator() {
            let line = line.to_str().expect("valid UTF-8");
            match Headline::try_from(line) {
                Ok(headline) => {
                    match previous_headline {
                        Some(headline) => {
                            sections.push(Section::from_headline_and_body(
                                headline,
                                std::mem::take(&mut plain_text),
                            ));
                        }
                        None => {
                            if !plain_text.is_empty() {
                                sections.push(Section::Verbatim {
                                    text: std::mem::take(&mut plain_text),
                                    generated: false,
                                })
                            }
                        }
                    };
                    previous_headline = Some(headline);
                }
                Err(()) => {
                    plain_text.push_str(line);
                }
            }
        }

        match previous_headline {
            Some(headline) => {
                sections.push(Section::from_headline_and_body(
                    headline,
                    std::mem::take(&mut plain_text),
                ));
            }
            None => sections.push(Section::Verbatim {
                text: plain_text,
                generated: false,
            }),
        }
        ChangeLog { sections }
    }
}

impl Section {
    fn from_headline_and_body(Headline { level, version, date }: Headline, body: String) -> Self {
        let mut events = pulldown_cmark::Parser::new_ext(&body, pulldown_cmark::Options::all()).into_offset_iter();
        let mut unknown = String::new();
        let mut thanks_clippy_count = 0;
        let mut segments = Vec::new();

        // let mut user_authored = String::new();
        let mut unknown_range = None;
        while let Some((e, range)) = events.next() {
            // dbg!(&e, &range);
            match e {
                Event::Html(text) if text.starts_with(Section::UNKNOWN_TAG_START) => {
                    consume_unknown_range(&mut segments, unknown_range.take(), &body);
                    for (event, _range) in events.by_ref().take_while(
                        |(e, _range)| !matches!(e, Event::Html(text) if text.starts_with(Section::UNKNOWN_TAG_END)),
                    ) {
                        track_unknown_event(event, &mut unknown);
                    }
                }
                Event::Start(Tag::Heading(_indent)) => {
                    consume_unknown_range(&mut segments, unknown_range.take(), &body);
                    enum State {
                        ParseClippy,
                        ConsiderUserAuthored,
                    }
                    let state = match events.next() {
                        Some((Event::Text(title), _range)) if title.starts_with(Section::THANKS_CLIPPY_TITLE) => {
                            State::ParseClippy
                        }
                        Some((_event, next_range)) => {
                            update_unknown_range(&mut unknown_range, range);
                            update_unknown_range(&mut unknown_range, next_range);
                            State::ConsiderUserAuthored
                        }
                        None => State::ConsiderUserAuthored,
                    };

                    events
                        .by_ref()
                        .take_while(|(e, range)| {
                            if matches!(state, State::ConsiderUserAuthored) {
                                update_unknown_range(&mut unknown_range, range.clone());
                            }
                            !matches!(e, Event::End(Tag::Heading(_)))
                        })
                        .count();
                    match state {
                        State::ParseClippy => {
                            if let Some(p) = collect_paragraph(events.by_ref(), &mut unknown) {
                                thanks_clippy_count = p
                                    .split(' ')
                                    .filter_map(|num| num.parse::<usize>().ok())
                                    .next()
                                    .unwrap_or(0)
                            }
                        }
                        State::ConsiderUserAuthored => {}
                    }
                }
                _unknown_event => update_unknown_range(&mut unknown_range, range),
            };
        }
        consume_unknown_range(&mut segments, unknown_range.take(), &body);
        Section::Release {
            name: match version {
                Some(version) => changelog::Version::Semantic(version),
                None => changelog::Version::Unreleased,
            },
            date,
            heading_level: level,
            thanks_clippy_count,
            segments,
            unknown,
        }
    }
}

fn update_unknown_range(target: &mut Option<Range<usize>>, source: Range<usize>) {
    match target {
        Some(range_thus_far) => {
            if source.end > range_thus_far.end {
                range_thus_far.end = source.end;
            }
        }
        None => *target = source.into(),
    }
}

fn consume_unknown_range(out: &mut Vec<section::Segment>, range: Option<Range<usize>>, markdown: &str) {
    if let Some(range) = range {
        out.push(section::Segment::Unknown {
            text: markdown[range].to_owned(),
        })
    }
}

fn track_unknown_event(unknown_event: Event<'_>, unknown: &mut String) {
    log::trace!("Cannot handle {:?}", unknown_event);
    match unknown_event {
        Event::Html(text)
        | Event::Code(text)
        | Event::Text(text)
        | Event::FootnoteReference(text)
        | Event::Start(Tag::FootnoteDefinition(text))
        | Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(text)))
        | Event::Start(Tag::Link(_, text, _))
        | Event::Start(Tag::Image(_, text, _)) => unknown.push_str(text.as_ref()),
        _ => {}
    }
}

fn collect_paragraph(events: &mut OffsetIter<'_>, unknown: &mut String) -> Option<String> {
    match events.next() {
        Some((Event::Start(Tag::Paragraph), _range)) => {
            return events
                .take_while(|(e, _range)| !matches!(e, Event::End(Tag::Paragraph)))
                .filter_map(|(e, _range)| match e {
                    Event::Text(text) => Some(text),
                    _ => None,
                })
                .fold(String::new(), |mut acc, text| {
                    acc.push_str(&text);
                    acc
                })
                .into()
        }
        Some((event, _range)) => track_unknown_event(event, unknown),
        None => {}
    };
    None
}

struct Headline {
    level: usize,
    version: Option<semver::Version>,
    date: Option<time::OffsetDateTime>,
}

impl<'a> TryFrom<&'a str> for Headline {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        all_consuming(headline::<()>)(value).finish().map(|(_, h)| h)
    }
}

fn headline<'a, E: ParseError<&'a str> + FromExternalError<&'a str, ()>>(i: &'a str) -> IResult<&'a str, Headline, E> {
    let hashes = take_while(|c: char| c == '#');
    let greedy_whitespace = |i| take_while(|c: char| c.is_whitespace())(i);
    let take_n_digits = |n: usize| {
        map_res(take_while_m_n(n, n, |c: char| c.is_digit(10)), |num| {
            u32::from_str(num).map_err(|_| ())
        })
    };
    map(
        terminated(
            tuple((
                separated_pair(
                    hashes,
                    greedy_whitespace,
                    alt((
                        preceded(
                            tag("v"),
                            map_res(take_till(|c: char| c.is_whitespace()), |v| {
                                semver::Version::parse(v).map_err(|_| ()).map(Some)
                            }),
                        ),
                        map(tag_no_case("unreleased"), |_| None),
                    )),
                ),
                opt(preceded(
                    greedy_whitespace,
                    delimited(
                        tag("("),
                        map_res(
                            tuple((take_n_digits(4), tag("-"), take_n_digits(2), tag("-"), take_n_digits(2))),
                            |(year, _, month, _, day)| {
                                time::Month::try_from(month as u8).map_err(|_| ()).and_then(|month| {
                                    time::Date::from_calendar_date(year as i32, month, day as u8)
                                        .map_err(|_| ())
                                        .map(|d| d.midnight().assume_utc())
                                })
                            },
                        ),
                        tag(")"),
                    ),
                )),
            )),
            greedy_whitespace,
        ),
        |((hashes, version), date)| Headline {
            level: hashes.len(),
            version,
            date,
        },
    )(i)
}
