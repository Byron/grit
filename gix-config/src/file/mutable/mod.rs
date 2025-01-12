use std::borrow::Cow;

use bstr::{BStr, BString, ByteSlice, ByteVec};

use crate::{file, parse::Event};

pub(crate) mod multi_value;
pub(crate) mod section;
pub(crate) mod value;

fn escape_value(value: &BStr) -> BString {
    let starts_with_whitespace = value.first().is_some_and(u8::is_ascii_whitespace);
    let ends_with_whitespace = value
        .get(value.len().saturating_sub(1))
        .is_some_and(u8::is_ascii_whitespace);
    let contains_comment_indicators = value.find_byteset(b";#").is_some();
    let quote = starts_with_whitespace || ends_with_whitespace || contains_comment_indicators;

    let mut buf: BString = Vec::with_capacity(value.len()).into();
    if quote {
        buf.push(b'"');
    }

    for b in value.iter().copied() {
        match b {
            b'\n' => buf.push_str("\\n"),
            b'\t' => buf.push_str("\\t"),
            b'"' => buf.push_str("\\\""),
            b'\\' => buf.push_str("\\\\"),
            _ => buf.push(b),
        }
    }

    if quote {
        buf.push(b'"');
    }
    buf
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Whitespace<'a> {
    pre_key: Option<Cow<'a, BStr>>,
    pre_sep: Option<Cow<'a, BStr>>,
    post_sep: Option<Cow<'a, BStr>>,
}

impl Default for Whitespace<'_> {
    fn default() -> Self {
        Whitespace {
            pre_key: Some(b"\t".as_bstr().into()),
            pre_sep: Some(b" ".as_bstr().into()),
            post_sep: Some(b" ".as_bstr().into()),
        }
    }
}

impl<'a> Whitespace<'a> {
    fn key_value_separators(&self) -> Vec<Event<'a>> {
        let mut out = Vec::with_capacity(3);
        if let Some(ws) = &self.pre_sep {
            out.push(Event::Whitespace(ws.clone()));
        }
        out.push(Event::KeyValueSeparator);
        if let Some(ws) = &self.post_sep {
            out.push(Event::Whitespace(ws.clone()));
        }
        out
    }

    fn from_body(s: &file::section::Body<'a>) -> Self {
        let key_pos =
            s.0.iter()
                .enumerate()
                .find_map(|(idx, e)| matches!(e, Event::SectionValueName(_)).then(|| idx));
        key_pos
            .map(|key_pos| {
                let pre_key = s.0[..key_pos].iter().next_back().and_then(|e| match e {
                    Event::Whitespace(s) => Some(s.clone()),
                    _ => None,
                });
                let from_key = &s.0[key_pos..];
                let (pre_sep, post_sep) = from_key
                    .iter()
                    .enumerate()
                    .find_map(|(idx, e)| matches!(e, Event::KeyValueSeparator).then(|| idx))
                    .map(|sep_pos| {
                        (
                            from_key.get(sep_pos - 1).and_then(|e| match e {
                                Event::Whitespace(ws) => Some(ws.clone()),
                                _ => None,
                            }),
                            from_key.get(sep_pos + 1).and_then(|e| match e {
                                Event::Whitespace(ws) => Some(ws.clone()),
                                _ => None,
                            }),
                        )
                    })
                    .unwrap_or_default();
                Whitespace {
                    pre_key,
                    pre_sep,
                    post_sep,
                }
            })
            .unwrap_or_default()
    }
}
