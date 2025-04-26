//! STEP エンティティの引数 "(…)" を再帰構造に落とし込む

#[derive(Debug, Clone, PartialEq)]
pub enum Attr {
    Scalar(String),   // 引用符付き文字列・数値・ENUM・$ など
    Ref(usize),       // #123
    List(Vec<Attr>),  // (…) or (…) , (…) のネスト
}

impl Attr {
    /// 最上位だけをカンマ分割する簡易パーサ
    pub fn parse_list(input: &str) -> Vec<Attr> {
        let mut out = Vec::new();
        let mut buf = String::new();
        let mut depth = 0;
        let mut in_quote = false;
    
        for ch in input.chars() {
            match ch {
                '\'' => { in_quote = !in_quote; buf.push(ch); }   // クォートは常に保持
                '('  if !in_quote => {
                    depth += 1;
                    buf.push(ch);
                }
                ')'  if !in_quote => {
                    buf.push(ch);
                    depth -= 1;
                }
                ',' if !in_quote && depth == 0 => {
                    // 深度 0 のカンマでトークンを確定
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

    fn from_token(tok: &str) -> Attr {
        let t = tok.trim();
        if t.starts_with('#') {
            let id = t[1..].parse().unwrap_or(0);
            Attr::Ref(id)
        } else if t.starts_with('(') && t.ends_with(')') {
            Attr::List(Attr::parse_list(&t[1..t.len() - 1]))
        } else {
            Attr::Scalar(t.to_string())
        }
    }
}
