use crate::{ParseError, RawEntity};
use rk_calc::Vector3;

/// STEP → struct
pub trait StepParse: Sized {
    const KEYWORD: &'static str;
    fn parse(e: &RawEntity) -> Result<Self, ParseError>;
}

/// struct → STEP
pub trait StepWrite {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError>;
}

/// キーワードチェックを一行で書く
pub fn expect_keyword(e: &RawEntity, kw: &'static str) -> Result<(), ParseError> {
    if e.keyword != kw {
        return Err(ParseError::Keyword {
            expected: kw,
            got: e.keyword.clone(),
        });
    }
    Ok(())
}

/// "'' , #123 , 4.5 , .T." → vec!["#123", "4.5", ".T."]
pub fn tokenized(params: &str) -> impl Iterator<Item = &str> {
    params
        .trim_end_matches(';')
        .trim_end_matches(')')
        .split(',')
        .skip(1) // 先頭 ''
        .map(|s| s.trim())
}

pub fn expect_token_count(tok: &[&str], count: usize, params: &str) -> Result<(), ParseError> {
    if tok.len() != count {
        return Err(ParseError::Attr(format!(
            "expected {} tokens, got {} in {}",
            count,
            tok.len(),
            params
        )));
    }
    Ok(())
}

/// STEP (ISO 10303-21) 用に f64 を文字列化する.
///
/// - 常に小数点を含む
/// - 必要に応じて科学記法 (大文字 E)
/// - 末尾 0 を削除し最短化
/// - NaN / ±Inf は Err(ParseError::NonFiniteReal)
pub fn fmt_step_real(v: f64) -> Result<String, ParseError> {
    if !v.is_finite() {
        return Err(ParseError::NonFiniteReal);
    }

    // 科学記法へ切替える閾値
    let abs = v.abs();
    let use_exp = abs != 0.0 && !(1.0e-9..1.0e9).contains(&abs);

    // 大文字 E を直接得る
    let raw = if use_exp {
        format!("{:.15E}", v) // 例: "1.234567890123457E+06"
    } else {
        format!("{:.15}", v) // 例: "2.000000000000000"
    };

    // `E` があれば指数部を分離
    let (mut mant, exp_opt) = match raw.find('E') {
        Some(i) => (raw[..i].to_owned(), Some(&raw[i + 1..])),
        None => (raw, None),
    };

    // 仮数部の末尾 0 を除去
    mant.truncate(mant.trim_end_matches('0').len());

    // 末尾が '.' のケースを残しつつ、'.' が無くなったら補う
    if !mant.contains('.') {
        mant.push_str(".0");
    }

    let out = if let Some(exp) = exp_opt {
        // `E+06` / `E-04` などをそのまま残す
        format!("{mant}E{exp}")
    } else {
        mant
    };

    Ok(out)
}

/* ---------- ①  #123  → usize ---------------------------------- */
pub fn as_id(tok: &str) -> Result<usize, ParseError> {
    let rest = tok
        .strip_prefix('#')
        .ok_or_else(|| ParseError::Attr(format!("expected #id, got {tok}")))?;
    rest.parse::<usize>()
        .map_err(|_| ParseError::Attr(format!("bad id: {tok}")))
}

/* ---------- ②  (x,y,z) → Vector3 ------------------------------ */
pub fn as_vec3(tok: &str) -> Result<Vector3, ParseError> {
    let inner = tok
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| ParseError::Attr(format!("expected (x,y,z), got {tok}")))?;

    let nums: Vec<_> = inner.split(',').collect();
    if nums.len() != 3 {
        return Err(ParseError::Attr(format!("vec3 needs 3 comps, got {tok}")));
    }
    let x = nums[0]
        .parse::<f64>()
        .map_err(|_| ParseError::Attr(nums[0].into()))?;
    let y = nums[1]
        .parse::<f64>()
        .map_err(|_| ParseError::Attr(nums[1].into()))?;
    let z = nums[2]
        .parse::<f64>()
        .map_err(|_| ParseError::Attr(nums[2].into()))?;
    Ok(Vector3::new(x, y, z))
}

/* ---------- ③  数値 → f64 ------------------------------------- */
pub fn as_f64(tok: &str) -> Result<f64, ParseError> {
    tok.parse::<f64>()
        .map_err(|_| ParseError::Attr(format!("bad number: {tok}")))
}

/* ---------- ④  .T./.F. → bool --------------------------------- */
pub fn as_bool(tok: &str) -> Result<bool, ParseError> {
    match tok {
        ".T." => Ok(true),
        ".F." => Ok(false),
        _ => Err(ParseError::Attr(format!("expected .T. or .F., got {tok}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_step_real_test() {
        assert_eq!(fmt_step_real(0.0).unwrap(), "0.");
        assert_eq!(fmt_step_real(2.0).unwrap(), "2.");
        assert_eq!(fmt_step_real(0.125).unwrap(), "0.125");
        assert_eq!(fmt_step_real(1.0e9).unwrap(), "1.E9");
        assert_eq!(fmt_step_real(-3.2e-10).unwrap(), "-3.2E-10");

        assert!(fmt_step_real(f64::NAN).is_err());
        assert!(fmt_step_real(f64::INFINITY).is_err());
    }
}
