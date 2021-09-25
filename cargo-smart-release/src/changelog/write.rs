use crate::{
    changelog,
    changelog::{section, Section},
    ChangeLog,
};

impl std::fmt::Display for changelog::Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            changelog::Version::Unreleased => f.write_str("Unreleased"),
            changelog::Version::Semantic(v) => write!(f, "v{}", v),
        }
    }
}

impl Section {
    pub const UNKNOWN_TAG_START: &'static str = "<csm-unknown>";
    pub const UNKNOWN_TAG_END: &'static str = "<csm-unknown/>";
    pub const THANKS_CLIPPY_TITLE: &'static str = "Thanks Clippy…";

    pub fn write_to(&self, mut out: impl std::io::Write) -> std::io::Result<()> {
        match self {
            Section::Verbatim { text, .. } => out.write_all(text.as_bytes()),
            Section::Release {
                name,
                date,
                heading_level,
                thanks_clippy_count,
                segments,
                unknown,
            } => {
                write!(out, "{} {}", heading(*heading_level), name)?;
                match date {
                    None => out.write_all(b"\n\n"),
                    Some(date) => writeln!(
                        out,
                        " ({:04}-{:02}-{:02})\n",
                        date.year(),
                        date.month() as u32,
                        date.day()
                    ),
                }?;
                let section_level = *heading_level + 1;
                for segment in segments {
                    segment.write_to(&mut out)?;
                }
                if *thanks_clippy_count > 0 {
                    writeln!(out, "{} {}\n", heading(section_level), Section::THANKS_CLIPPY_TITLE)?;
                    writeln!(
                        out,
                        "Clippy is a linter to help keeping code idiomatic. It was helpful {} {} in this release.\n",
                        thanks_clippy_count,
                        if *thanks_clippy_count > 1 { "times" } else { "time" }
                    )?;
                }
                if !unknown.is_empty() {
                    writeln!(out, "{}", Section::UNKNOWN_TAG_START)?;
                    out.write_all(unknown.as_bytes())?;
                    writeln!(out, "{}", Section::UNKNOWN_TAG_END)?;
                }
                Ok(())
            }
        }
    }
}

fn heading(level: usize) -> String {
    "#".repeat(level)
}

impl ChangeLog {
    pub fn write_to(&self, mut out: impl std::io::Write) -> std::io::Result<()> {
        for section in &self.sections {
            section.write_to(&mut out)?;
        }
        Ok(())
    }
}

impl section::Segment {
    pub fn write_to(&self, mut out: impl std::io::Write) -> std::io::Result<()> {
        match self {
            section::Segment::Unknown { text } => out.write_all(text.as_bytes()),
        }
    }
}
