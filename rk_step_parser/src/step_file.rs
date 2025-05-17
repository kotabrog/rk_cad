//! ファイル全体を扱う層

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum StepFileParseError {
    #[error("unterminated record at line {lineno}: {line}")]
    Unterminated { lineno: usize, line: String },
}

/// STEP ファイルを 3 つのセクションに分割して保持する
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepFile {
    pub header: Vec<String>,   // ISO-10303-21 HEADER;
    pub entities: Vec<String>, // DATA; 〜 ENDSEC; までの各エンティティ行
    pub trailer: Vec<String>,  // END-ISO-10303-21 以降
}

/// いま読んでいるセクションを示す内部状態
enum Section {
    Header,
    WaitingForData,
    Data,
    Trailer,
}

/// 1 行を現在のバッファに連結し、行末が ';' なら collection へ退避してバッファを空にする。
///
/// * `start_lineno` はバッファに 1 行目を積んだ時点で更新し、
///   セミコロンを忘れたまま区切りトークンに遭遇した場合のエラー行番号に使う。
fn accumulate_record(
    line: &str,
    buf: &mut String,
    collection: &mut Vec<String>,
    start_lineno: &mut usize,
    lineno_0origin: usize,
) {
    if buf.is_empty() {
        *start_lineno = lineno_0origin + 1; // 1-origin で保持
    } else {
        buf.push(' ');
    }
    buf.push_str(line);

    if line.ends_with(';') {
        collection.push(buf.clone());
        buf.clear();
    }
}

/// 区切りトークン直前にバッファが残っていれば StepFileParseError を返す。
fn ensure_no_unterminated_record(buf: &str, lineno: usize) -> Result<(), StepFileParseError> {
    if buf.is_empty() {
        Ok(())
    } else {
        Err(StepFileParseError::Unterminated {
            lineno,
            line: buf.to_owned(),
        })
    }
}

/// STEP ファイル全文をパースしてセクションごとに分離
pub fn parse_step_file(src: &str) -> Result<StepFile, StepFileParseError> {
    let mut header = Vec::<String>::new();
    let mut entities = Vec::<String>::new();
    let mut trailer = Vec::<String>::new();

    let mut section = Section::Header;

    let mut buf = String::new(); // 多行レコードの一時保持
    let mut start_lineno = 0; // バッファ開始行（1-origin）

    for (i, raw) in src.lines().enumerate() {
        let line = raw.trim();

        match section {
            // ─────────── HEADER ───────────
            Section::Header => {
                if line.eq_ignore_ascii_case("ENDSEC;") {
                    ensure_no_unterminated_record(&buf, start_lineno)?;
                    buf.clear();
                    section = Section::WaitingForData;
                    continue;
                }
                if line.is_empty() || line.starts_with('!') {
                    continue;
                }
                accumulate_record(line, &mut buf, &mut header, &mut start_lineno, i);
            }

            // ─────────── HEADER と DATA の隙間（ANCHOR 等を含む）───────────
            Section::WaitingForData => {
                if line.eq_ignore_ascii_case("DATA;") {
                    ensure_no_unterminated_record(&buf, start_lineno)?;
                    buf.clear();
                    section = Section::Data;
                    continue;
                }
                if line.is_empty() || line.starts_with('!') {
                    continue;
                }
                accumulate_record(line, &mut buf, &mut header, &mut start_lineno, i);
            }

            // ─────────── DATA 部 ───────────
            Section::Data => {
                if line.eq_ignore_ascii_case("ENDSEC;") {
                    ensure_no_unterminated_record(&buf, start_lineno)?;
                    buf.clear();
                    section = Section::Trailer;
                    continue;
                }
                if line.is_empty() || line.starts_with('!') {
                    continue;
                }
                accumulate_record(line, &mut buf, &mut entities, &mut start_lineno, i);
            }

            // ─────────── Trailer ───────────
            // SIGNATUREでは;区切りではないので、改行で分割する
            Section::Trailer => {
                if line.is_empty() || line.eq_ignore_ascii_case("ENDSEC;") {
                    continue; // SIGNATURE 終端の ENDSEC; は無視
                }
                trailer.push(line.to_string());
            }
        }
    }

    // ファイル末尾時点の未完チェック
    ensure_no_unterminated_record(&buf, start_lineno)?;

    Ok(StepFile {
        header,
        entities,
        trailer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_step_file_normal() {
        let src = r#"
            ISO-10303-21;
            HEADER;
            FILE_DESCRIPTION(('STEP AP203'), '1');
            FILE_NAME('test.stp', '2023-10-01T12:00:00', ('Author'), ('Organization'), 'Description');
            FILE_SCHEMA(('AP203'));
            ENDSEC;

            DATA;
            #1 = PRODUCT('Product1', 'Description1');
            #2 = PRODUCT('Product2', Description2');
            ENDSEC;

            END-ISO-10303-21;
        "#;

        let step_file = parse_step_file(src).unwrap();
        println!("{:?}", step_file);
        assert_eq!(step_file.header.len(), 5);
        assert_eq!(step_file.entities.len(), 2);
        assert_eq!(step_file.trailer.len(), 1);
    }

    #[test]
    fn parse_step_file_error() {
        let src = r#"
            ISO-10303-21;
            HEADER;
            FILE_DESCRIPTION(('STEP AP203'), '1');
            FILE_NAME('test.stp', '2023-10-01T12:00:00', ('Author'), ('Organization'), 'Description');
            FILE_SCHEMA(('AP203'))
            ENDSEC;

            DATA;
            #1 = PRODUCT('Product1', 'Description1');
            #2 = PRODUCT('Product2', Description2');
            ENDSEC;

            END-ISO-10303-21;
        "#;

        let err = parse_step_file(src);
        println!("{:?}", err);
        assert_eq!(
            err,
            Err(StepFileParseError::Unterminated {
                lineno: 6,
                line: "FILE_SCHEMA(('AP203'))".to_string(),
            })
        );
    }
}
