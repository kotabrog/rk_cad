use regex::Regex;
use std::sync::OnceLock;

// ===========================================================================
// このファイルは STEP Part 21 の 1 行（インスタンス割り当て）を "生" の状態で
// 扱うための軽量パーサを提供します。外部マッピング行（複数のエンティティ型を
// ひとつの ID に束ねる書式）にも対応するため、右辺を Record の列として保持する
// 方式を採用しています。
// ===========================================================================

/// `(KEYWORD(...))` もしくは `( ...(省略) )` の 1 かたまりを表す。
/// キーワードが書かれていないケースに備えて `keyword` は `Option`。
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// 例: `CARTESIAN_POINT`。
    pub keyword: Option<String>,
    /// 括弧内部を丸ごと保持した文字列。ネストは未展開。
    pub params: String,
}

/// STEP エンティティ 1 行を保持する最小構造。
#[derive(Debug, Clone, PartialEq)]
pub struct RawEntity {
    pub id: usize,
    pub records: Vec<Record>,
}

// ---------------------------------------------------------------------------
// 正規表現のコンパイルは高コストなので OnceLock で 1 度だけ初期化し再利用する。
// `(?s)` は dot に改行もマッチさせる DOTALL フラグ。
// ---------------------------------------------------------------------------
static SIMPLE_RE: OnceLock<Regex> = OnceLock::new();
static COMPLEX_RE: OnceLock<Regex> = OnceLock::new();

fn simple_re() -> &'static Regex {
    // 例: #10 = CARTESIAN_POINT(1.0, 2.0, 3.0);
    SIMPLE_RE.get_or_init(|| {
        Regex::new(r"(?s)^#(\d+)\s*=\s*([A-Z0-9_]+)\((.*)\);$")
            .expect("simple regex compile failed")
    })
}

fn complex_re() -> &'static Regex {
    // 例: #165 = ( ENTITY_A(...) ENTITY_B(...));
    COMPLEX_RE.get_or_init(|| {
        Regex::new(r"(?s)^#(\d+)\s*=\s*\((.*)\);$").expect("complex regex compile failed")
    })
}

/// 文字列 `buf` が 1 行分のエンティティであれば `Some(RawEntity)` を返す。
/// マッチしなければ `None`。
pub fn parse_entity(buf: &str) -> Option<RawEntity> {
    // 1) 単純行を試す
    if let Some(caps) = simple_re().captures(buf) {
        let id: usize = caps[1].parse().ok()?;
        let keyword = caps[2].to_string();
        let params = caps[3].to_string();
        return Some(RawEntity {
            id,
            records: vec![Record {
                keyword: Some(keyword),
                params,
            }],
        });
    }

    // 2) 外部マッピング行を試す
    if let Some(caps) = complex_re().captures(buf) {
        let id: usize = caps[1].parse().ok()?;
        let body = caps[2].trim(); // 最外括弧内
        let mut records = Vec::new();
        for token in split_top_level(body) {
            // token は "KEYWORD(...)" または "(...)" の形
            if let Some(pos) = token.find('(') {
                let kw = token[..pos].trim();
                let params = token[pos + 1..token.len() - 1].to_string(); // 括弧を外す
                let keyword = if kw.is_empty() {
                    None
                } else {
                    Some(kw.to_string())
                };
                records.push(Record { keyword, params });
            }
        }
        return Some(RawEntity { id, records });
    }

    // どちらにもマッチしない
    None
}

// ---------------------------------------------------------------------------
// 与えられた文字列をトップレベルの括弧単位で分割するユーティリティ。
// ネストを考慮し、"," や空白だけで区切るのではなく括弧深度が 0 でスペースが
// 出た地点をトークン境界とみなす。
// ---------------------------------------------------------------------------
fn split_top_level(s: &str) -> Vec<&str> {
    let mut depth = 0;
    let mut start = 0;
    let mut tokens = Vec::new();

    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            // 深度 0 で空白または改行が来たら区切る
            c if depth == 0 && c.is_whitespace() => {
                if start < i {
                    tokens.push(s[start..i].trim());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        tokens.push(s[start..].trim());
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_entity_simple() {
        let src = "#1 = AXIS2_PLACEMENT_3D('', (#2,#3,#4));";
        let ent = parse_entity(src).unwrap();
        assert_eq!(ent.id, 1);
        assert_eq!(ent.records.len(), 1);
        assert_eq!(
            ent.records[0].keyword.as_deref(),
            Some("AXIS2_PLACEMENT_3D")
        );
        assert_eq!(ent.records[0].params, "'', (#2,#3,#4)");
    }

    #[test]
    fn parse_entity_all_types() {
        let src = "#2 = DUMMY('', (#2,#3,#4,(#2,3.,4.111,.F.,.T.,*,$,'a'),1.1));";
        let ent = parse_entity(src).unwrap();
        assert_eq!(ent.id, 2);
        assert_eq!(ent.records.len(), 1);
        assert_eq!(ent.records[0].keyword.as_deref(), Some("DUMMY"));
        assert_eq!(
            ent.records[0].params,
            "'', (#2,#3,#4,(#2,3.,4.111,.F.,.T.,*,$,'a'),1.1)"
        );
    }

    #[test]
    fn parse_entity_complex() {
        let src = "#166 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );";
        let ent = parse_entity(src).unwrap();
        assert_eq!(ent.id, 166);
        assert_eq!(ent.records.len(), 3);
        assert_eq!(ent.records[0].keyword.as_deref(), Some("LENGTH_UNIT"));
        assert_eq!(ent.records[0].params, "");
        assert_eq!(ent.records[1].keyword.as_deref(), Some("NAMED_UNIT"));
        assert_eq!(ent.records[1].params, "*");
        assert_eq!(ent.records[2].keyword.as_deref(), Some("SI_UNIT"));
        assert_eq!(ent.records[2].params, ".MILLI.,.METRE.");
    }
}
