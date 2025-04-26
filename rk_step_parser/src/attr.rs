//! 共通データ型：Node と再帰属性 Attr

use std::cell::RefCell;
use std::rc::Weak;

#[derive(Debug)]
pub struct Node {
    pub id:   usize,
    pub kind: String,
    pub attrs: RefCell<Vec<Attr>>,
}

#[derive(Debug, Clone)]
pub enum Attr {
    Scalar(String),
    RefId(usize),
    Ref(Weak<Node>),
    List(Vec<Attr>),
}

// ────────────────────────────────────────────────
// 文字列 → Vec<Attr> への簡易パーサ
// ────────────────────────────────────────────────
impl Attr {
    /// `input` は "a,b,(c,d),#12" のようなカッコ込み引数列
    pub fn parse_list(input: &str) -> Vec<Attr> {
        let mut out   = Vec::<Attr>::new();
        let mut buf   = String::new();
        let mut depth = 0;
        let mut in_quote = false;

        for ch in input.chars() {
            match ch {
                '\'' => { in_quote = !in_quote; buf.push(ch); } // クォート保持
                '(' if !in_quote => { depth += 1; buf.push(ch); }
                ')' if !in_quote => {
                    buf.push(ch);
                    depth -= 1;
                }
                ',' if !in_quote && depth == 0 => {
                    if !buf.trim().is_empty() {
                        out.push(Attr::from_token(&buf));
                        buf.clear();
                    }
                }
                _ => buf.push(ch),
            }
        }
        if !buf.trim().is_empty() {
            out.push(Attr::from_token(&buf));
        }
        out
    }

    /// トークン 1 個 → Attr
    fn from_token(tok: &str) -> Attr {
        let t = tok.trim();
        if t.starts_with('#') {
            let id = t[1..].parse().unwrap_or(0);
            Attr::RefId(id)
        } else if t.starts_with('(') && t.ends_with(')') {
            Attr::List(Attr::parse_list(&t[1..t.len() - 1]))
        } else {
            Attr::Scalar(t.to_string())
        }
    }
}
