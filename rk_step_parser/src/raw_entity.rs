use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;

// =============================================================================
// STEP Part 21 の 1 行 (instance assignment) を "そのまま" 保持する構造体とパーサ。
// 外部マッピング行 ("= ( A(...) B(...) )") を含めるため右辺を Record のベクタ
// として保存する。エラー発生箇所を呼び出し側で判断できるよう、Result で返す。
// =============================================================================

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

/// STEP エンティティ行のパースエラー。
/// MissingOpenParen, MissingCloseParen になるケースはないはず。（NoMatchになる）
#[derive(Debug, Error, PartialEq)]
pub enum RawEntityParseError {
    #[error("line does not match STEP entity syntax")]
    NoMatch,
    #[error("invalid ID number: {0}")]
    InvalidId(String),
    #[error("unmatched parentheses")]
    UnmatchedParenthesis,
    #[error("record is missing opening '(': {token}")]
    MissingOpenParen { token: String },
    #[error("record is missing closing ')': {token}")]
    MissingCloseParen { token: String },
}

type Result<T> = std::result::Result<T, RawEntityParseError>;

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

// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------
/// 行 `buf` を解析する。
/// * `Ok(Some(entity))` … STEP エンティティ行として成功
/// * `Err(NoMatch)`    … エンティティ形式にマッチしない行
/// * `Err(...)`        … 構文エラー
pub fn parse_raw_entity(buf: &str) -> Result<Option<RawEntity>> {
    if let Some(entity) = try_parse_simple(buf)? {
        return Ok(Some(entity));
    }
    if let Some(entity) = try_parse_complex(buf)? {
        return Ok(Some(entity));
    }
    Err(RawEntityParseError::NoMatch)
}

// -----------------------------------------------------------------------------
// 単純エンティティ行の解析 – `#id = KEYWORD(...);`
// -----------------------------------------------------------------------------
fn try_parse_simple(buf: &str) -> Result<Option<RawEntity>> {
    let caps = match simple_re().captures(buf) {
        Some(c) => c,
        None => return Ok(None),
    };
    let id: usize = caps[1]
        .parse()
        .map_err(|_| RawEntityParseError::InvalidId(caps[1].to_string()))?;
    let keyword = caps[2].to_string();
    let params = caps[3].to_string();
    Ok(Some(RawEntity {
        id,
        records: vec![Record {
            keyword: Some(keyword),
            params,
        }],
    }))
}

// -----------------------------------------------------------------------------
// 外部マッピング行の解析 – `#id = ( A(...) B(...) ... );`
// -----------------------------------------------------------------------------
fn try_parse_complex(buf: &str) -> Result<Option<RawEntity>> {
    let caps = match complex_re().captures(buf) {
        Some(c) => c,
        None => return Ok(None),
    };
    let id: usize = caps[1]
        .parse()
        .map_err(|_| RawEntityParseError::InvalidId(caps[1].to_string()))?;
    let body = caps[2].trim();
    let tokens = split_top_level(body)?;
    let mut records = Vec::with_capacity(tokens.len());
    for tok in tokens {
        records.push(token_to_record(tok)?);
    }
    Ok(Some(RawEntity { id, records }))
}

// -----------------------------------------------------------------------------
// 1 トークンを Record 型へ変換
// -----------------------------------------------------------------------------
fn token_to_record(token: &str) -> Result<Record> {
    let open = token
        .find('(')
        .ok_or_else(|| RawEntityParseError::MissingOpenParen {
            token: token.to_string(),
        })?;
    if !token.ends_with(')') {
        return Err(RawEntityParseError::MissingCloseParen {
            token: token.to_string(),
        });
    }
    let kw = token[..open].trim();
    let params = token[open + 1..token.len() - 1].trim().to_string();
    let keyword = if kw.is_empty() {
        None
    } else {
        Some(kw.to_string())
    };
    Ok(Record { keyword, params })
}

// -----------------------------------------------------------------------------
// トップレベル括弧単位で分割 – ネスト対応
// -----------------------------------------------------------------------------
fn split_top_level(s: &str) -> Result<Vec<&str>> {
    let mut depth: isize = 0;
    let mut start = 0usize;
    let mut tokens = Vec::new();
    let mut in_str = false;
    let bytes = s.as_bytes();

    let mut i = 0usize;
    while i < bytes.len() {
        let ch = bytes[i] as char;

        if in_str {
            if ch == '\'' {
                // '' はエスケープ
                if i + 1 < bytes.len() && bytes[i + 1] as char == '\'' {
                    i += 1;
                } else {
                    in_str = false;
                }
            }
        } else {
            match ch {
                '\'' => in_str = true,
                '('  => depth += 1,
                ')'  => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(RawEntityParseError::UnmatchedParenthesis);
                    }
                    if depth == 0 {
                        // トップレベルの ')' を読んだのでレコード確定
                        let token = s[start..=i].trim();
                        if !token.is_empty() {
                            tokens.push(token);
                        }
                        // 次トークン開始位置を ')' の直後から最初の非空白まで進める
                        let mut j = i + 1;
                        while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                            j += 1;
                        }
                        start = j;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }

    if depth != 0 || in_str {
        return Err(RawEntityParseError::UnmatchedParenthesis);
    }
    Ok(tokens)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_raw_entity_simple() {
        let src = "#1 = AXIS2_PLACEMENT_3D('', (#2,#3,#4));";
        let ent = parse_raw_entity(src).unwrap().unwrap();
        assert_eq!(ent.id, 1);
        assert_eq!(ent.records.len(), 1);
        assert_eq!(
            ent.records[0].keyword.as_deref(),
            Some("AXIS2_PLACEMENT_3D")
        );
        assert_eq!(ent.records[0].params, "'', (#2,#3,#4)");
    }

    #[test]
    fn parse_raw_entity_all_types() {
        let src = "#2 = DUMMY('', (#2,#3,#4,(#2,3.,4.111,.F.,.T.,*,$,'a'),1.1));";
        let ent = parse_raw_entity(src).unwrap().unwrap();
        assert_eq!(ent.id, 2);
        assert_eq!(ent.records.len(), 1);
        assert_eq!(ent.records[0].keyword.as_deref(), Some("DUMMY"));
        assert_eq!(
            ent.records[0].params,
            "'', (#2,#3,#4,(#2,3.,4.111,.F.,.T.,*,$,'a'),1.1)"
        );
    }

    #[test]
    fn parse_raw_entity_complex() {
        let src = "#166 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );";
        let ent = parse_raw_entity(src).unwrap().unwrap();
        assert_eq!(ent.id, 166);
        assert_eq!(ent.records.len(), 3);
        assert_eq!(ent.records[0].keyword.as_deref(), Some("LENGTH_UNIT"));
        assert_eq!(ent.records[0].params, "");
        assert_eq!(ent.records[1].keyword.as_deref(), Some("NAMED_UNIT"));
        assert_eq!(ent.records[1].params, "*");
        assert_eq!(ent.records[2].keyword.as_deref(), Some("SI_UNIT"));
        assert_eq!(ent.records[2].params, ".MILLI.,.METRE.");
    }

    #[test]
    fn parse_raw_entity_complex_not_keyword() {
        let src = "#166 = ( () NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );";
        let ent = parse_raw_entity(src).unwrap().unwrap();
        assert_eq!(ent.id, 166);
        assert_eq!(ent.records.len(), 3);
        assert_eq!(ent.records[0].keyword, None);
        assert_eq!(ent.records[0].params, "");
    }

    #[test]
    fn parse_raw_entity_complex2() {
        let src = "#165 = ( GEOMETRIC_REPRESENTATION_CONTEXT(3) 
GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#169)) GLOBAL_UNIT_ASSIGNED_CONTEXT
((#166,#167,#168)) REPRESENTATION_CONTEXT('Context #1',
  '3D Context with UNIT and UNCERTAINTY') );";
        let ent = parse_raw_entity(src).unwrap().unwrap();
        assert_eq!(ent.id, 165);
        assert_eq!(ent.records.len(), 4);
        assert_eq!(ent.records[0].keyword.as_deref(), Some("GEOMETRIC_REPRESENTATION_CONTEXT"));
        assert_eq!(ent.records[0].params, "3");
        assert_eq!(ent.records[1].keyword.as_deref(), Some("GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT"));
        assert_eq!(ent.records[1].params, "(#169)");
        assert_eq!(ent.records[2].keyword.as_deref(), Some("GLOBAL_UNIT_ASSIGNED_CONTEXT"));
        assert_eq!(ent.records[2].params, "(#166,#167,#168)");
        assert_eq!(ent.records[3].keyword.as_deref(), Some("REPRESENTATION_CONTEXT"));
        assert_eq!(
            ent.records[3].params,
            "'Context #1',\n  '3D Context with UNIT and UNCERTAINTY'"
        );
    }

    #[test]
    fn parse_raw_entity_not_match() {
        let src = "NOT_STEP_LINE";
        let err = parse_raw_entity(src).unwrap_err();
        assert_eq!(err, RawEntityParseError::NoMatch);
    }

    #[test]
    fn parse_raw_entity_unmatched_parenthesis() {
        let src = "#1 = (A(B(C(D(E(F(G(H(I(J(K(L(M(N(O(P(Q(R(S(T(U(V(W(X(Y(Z(0.0);";
        let err = parse_raw_entity(src).unwrap_err();
        assert_eq!(err, RawEntityParseError::UnmatchedParenthesis);
    }

    #[test]
    fn parse_raw_entity_invalid_id() {
        let src = "#11111111111111111111111111111111111 = AXIS2_PLACEMENT_3D('', (#2,#3,#4));";
        let err = parse_raw_entity(src).unwrap_err();
        assert_eq!(
            err,
            RawEntityParseError::InvalidId("11111111111111111111111111111111111".to_string())
        );
    }
}
