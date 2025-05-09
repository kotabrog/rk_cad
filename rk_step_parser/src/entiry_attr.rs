//! RawEntity → Entity/Attr 変換モジュール
//!
//! * `RawEntity`（行単位の文字列）を意味のある `Entity` と `Attr` に分解する
//! * STEP ファイルの構文仕様に沿い、入れ子の括弧・文字列・参照等をすべて扱う
//! * 解析できないトークンや不整合は必ず `AttrParseError` を返す
//!
//! **依存:** `thiserror`, `crate::raw_entity::RawEntity`

use std::str::FromStr;

use thiserror::Error;

use crate::raw_entity::RawEntity;

/// 行を構造化した結果
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: usize,
    pub kind: String,
    pub attrs: Vec<Attr>,
}

/// STEP 属性
#[derive(Debug, Clone, PartialEq)]
pub enum Attr {
    Id(usize),      // #123
    Float(f64),     // 1.23E-4
    Bool(bool),     // .T. / .TRUE.
    Str(String),    // 'TEXT'
    List(Vec<Attr>),// ( ... )
    None,           // * or $
}

#[derive(Debug, Error, PartialEq)]
pub enum AttrParseError {
    #[error("unmatched parenthesis or quote")] Unmatched,
    #[error("invalid number: {0}")] InvalidNumber(String),
    #[error("invalid token: {0}")] InvalidToken(String),
}

/// ------------------------------------------------------------
/// Public API
/// ------------------------------------------------------------
impl TryFrom<&RawEntity> for Entity {
    type Error = AttrParseError;

    fn try_from(src: &RawEntity) -> Result<Self, Self::Error> {
        Ok(Self {
            id: src.id,
            kind: src.keyword.clone(),
            attrs: parse_attr_slice(&src.params)?,
        })
    }
}

/// ------------------------------------------------------------
/// 内部実装
/// ------------------------------------------------------------
fn parse_attr_slice(text: &str) -> Result<Vec<Attr>, AttrParseError> {
    split_top_level(text.trim())?
        .into_iter()
        .map(parse_token)
        .collect()
}

/// 深さ 0 のカンマで分割しトークン列を返す
fn split_top_level(s: &str) -> Result<Vec<&str>, AttrParseError> {
    let mut tokens = Vec::new();
    let mut depth = 0usize;
    let mut in_str = false;
    let mut start = 0usize;
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_str {
            if c == '\'' {
                // 連続シングルクォートはエスケープ
                if i + 1 < bytes.len() && bytes[i + 1] as char == '\'' {
                    i += 1; // skip second quote
                } else {
                    in_str = false;
                }
            }
        } else {
            match c {
                '\'' => in_str = true,
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        return Err(AttrParseError::Unmatched);
                    }
                    depth -= 1;
                }
                ',' if depth == 0 => {
                    tokens.push(s[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }
        i += 1;
    }

    if depth != 0 || in_str {
        return Err(AttrParseError::Unmatched);
    }
    tokens.push(s[start..].trim());
    Ok(tokens)
}

fn parse_token(tok: &str) -> Result<Attr, AttrParseError> {
    if tok.is_empty() {
        return Ok(Attr::None);
    }
    match tok {
        "*" | "$" => return Ok(Attr::None),
        _ => {}
    }
    if let Some(id) = tok.strip_prefix('#') {
        return usize::from_str(id)
            .map(Attr::Id)
            .map_err(|_| AttrParseError::InvalidNumber(tok.into()));
    }
    // Bool – STEP 仕様では .TRUE. / .FALSE.、実務では .T. / .F. もよく見かける
    let upper = tok.to_ascii_uppercase();
    match upper.as_str() {
        ".T." | ".TRUE." => return Ok(Attr::Bool(true)),
        ".F." | ".FALSE." => return Ok(Attr::Bool(false)),
        _ => {}
    }
    // ネスト
    if tok.starts_with('(') && tok.ends_with(')') {
        let inner = &tok[1..tok.len() - 1];
        return parse_attr_slice(inner).map(Attr::List);
    }
    // 文字列
    if tok.starts_with('\'') && tok.ends_with('\'') {
        let mut out = String::new();
        let mut chars = tok[1..tok.len() - 1].chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\'' && chars.peek() == Some(&'\'') {
                chars.next();
                out.push('\'');
            } else {
                out.push(ch);
            }
        }
        return Ok(Attr::Str(out));
    }
    // 数値
    if let Ok(v) = f64::from_str(tok) {
        return Ok(Attr::Float(v));
    }
    Err(AttrParseError::InvalidToken(tok.into()))
}

/// ------------------------------------------------------------
/// Tests
/// ------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn raw(id: usize, kw: &str, params: &str) -> RawEntity {
        RawEntity { id, keyword: kw.into(), params: params.into() }
    }

    #[test]
    fn simple_ids() {
        let e = Entity::try_from(&raw(1, "XYZ", "#1,#2,#3")) .unwrap();
        assert_eq!(e.attrs, vec![Attr::Id(1), Attr::Id(2), Attr::Id(3)]);
    }

    #[test]
    fn nested_list() {
        let e = Entity::try_from(&raw(2, "XYZ", "#1,(1.0,2.0),.F.")) .unwrap();
        assert_eq!(e.attrs.len(), 3);
        assert_eq!(e.attrs[0], Attr::Id(1));
        if let Attr::List(ref l) = e.attrs[1] {
            assert_eq!(l.len(), 2);
            assert_eq!(l[0], Attr::Float(1.0));
            assert_eq!(l[1], Attr::Float(2.0));
        }
        assert_eq!(e.attrs[2], Attr::Bool(false));
    }

    #[test]
    fn invalid_token() {
        assert!(matches!(Entity::try_from(&raw(3, "XYZ", "(???)")), Err(AttrParseError::InvalidToken(_))));
    }

    #[test]
    fn nested_nested_list() {
        let e = Entity::try_from(&raw(4, "XYZ", "#1,(#2,(#3,#4)),.T.")) .unwrap();
        assert_eq!(e.attrs.len(), 3);
        assert_eq!(e.attrs[0], Attr::Id(1));
        if let Attr::List(ref l) = e.attrs[1] {
            assert_eq!(l.len(), 2);
            assert_eq!(l[0], Attr::Id(2));
            if let Attr::List(ref l2) = l[1] {
                assert_eq!(l2.len(), 2);
                assert_eq!(l2[0], Attr::Id(3));
                assert_eq!(l2[1], Attr::Id(4));
            }
        }
        assert_eq!(e.attrs[2], Attr::Bool(true));
    }

    #[test]
    fn float() {
        let e = Entity::try_from(&raw(5, "XYZ", "1.23E-4,.5E+3,.0,1.,-1.0")) .unwrap();
        assert_eq!(e.attrs.len(), 5);
        assert_eq!(e.attrs[0], Attr::Float(1.23E-4));
        assert_eq!(e.attrs[1], Attr::Float(0.5E+3));
        assert_eq!(e.attrs[2], Attr::Float(0.0));
        assert_eq!(e.attrs[3], Attr::Float(1.0));
        assert_eq!(e.attrs[4], Attr::Float(-1.0));
    }

    #[test]
    fn bool() {
        let e = Entity::try_from(&raw(6, "XYZ", ".T.,.F.,.TRUE.,.FALSE.")) .unwrap();
        assert_eq!(e.attrs.len(), 4);
        assert_eq!(e.attrs[0], Attr::Bool(true));
        assert_eq!(e.attrs[1], Attr::Bool(false));
        assert_eq!(e.attrs[2], Attr::Bool(true));
        assert_eq!(e.attrs[3], Attr::Bool(false));
    }

    #[test]
    fn string() {
        let e = Entity::try_from(&raw(7, "XYZ", "'hello','world','foo''bar'")) .unwrap();
        assert_eq!(e.attrs.len(), 3);
        assert_eq!(e.attrs[0], Attr::Str("hello".into()));
        assert_eq!(e.attrs[1], Attr::Str("world".into()));
        assert_eq!(e.attrs[2], Attr::Str("foo'bar".into()));
    }

    #[test]
    fn none() {
        let e = Entity::try_from(&raw(8, "XYZ", "*,$")) .unwrap();
        assert_eq!(e.attrs.len(), 2);
        assert_eq!(e.attrs[0], Attr::None);
        assert_eq!(e.attrs[1], Attr::None);
    }
}
