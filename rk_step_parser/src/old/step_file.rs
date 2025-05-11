//! ファイル全体を扱う層

use super::raw_entity::{parse_entity, RawEntity};
use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepFile {
    pub header: Vec<String>,
    pub entities: Vec<RawEntity>,
    pub trailer: Vec<String>,
}

enum Section {
    Header,
    WaitingForData,
    Data,
    Trailer,
}

pub fn parse_step_file(src: &str) -> Result<StepFile, ParseError> {
    let mut header = Vec::new();
    let mut entities = Vec::new();
    let mut trailer = Vec::new();

    let mut section = Section::Header;
    let mut buf = String::new();
    let mut start_lineno = 0;

    for (i, raw) in src.lines().enumerate() {
        let line = raw.trim();

        match section {
            Section::Header => {
                header.push(line.into());
                if line.eq_ignore_ascii_case("ENDSEC;") {
                    section = Section::WaitingForData;
                }
            }
            Section::WaitingForData => {
                if line.eq_ignore_ascii_case("DATA;") {
                    section = Section::Data;
                } else {
                    header.push(line.into());
                }
            }
            Section::Data => {
                if line.eq_ignore_ascii_case("ENDSEC;") {
                    section = Section::Trailer;
                    continue;
                }
                if line.is_empty() || line.starts_with('!') {
                    continue;
                }

                if buf.is_empty() {
                    start_lineno = i + 1;
                } else {
                    buf.push(' ');
                }
                buf.push_str(line);

                if !line.ends_with(';') {
                    continue;
                }

                if let Some(ent) = parse_entity(&buf) {
                    entities.push(ent);
                } else {
                    return Err(ParseError::InvalidLine {
                        lineno: start_lineno,
                        line: buf.clone(),
                    });
                }
                buf.clear();
            }
            Section::Trailer => trailer.push(line.into()),
        }
    }

    Ok(StepFile {
        header,
        entities,
        trailer,
    })
}
