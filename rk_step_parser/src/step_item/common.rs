use thiserror::Error;

use crate::step_entity::{EntityId, Parameter, SimpleEntity};
use crate::step_item_map::StepItemMap;

#[derive(Error, Debug)]
pub enum ConversionStepItemError {
    #[error("unsupported keyword `{0}`")]
    Unsupported(String),

    #[error("{keyword}: expected {expected} attributes, found {found}")]
    AttrCount {
        keyword: &'static str,
        expected: usize,
        found: usize,
    },

    #[error("{keyword}: attribute must be an aggregate list (LIST/SET)")]
    NotAggregate { keyword: &'static str },

    #[error("{keyword}: attribute must be a reference to an entity")]
    NotReference { keyword: &'static str },

    #[error("{keyword}: non‑numeric value in aggregate")]
    NonNumeric { keyword: &'static str },

    #[error(
        "{keyword}: item count must be between {expected_min} and {expected_max}, found {found}"
    )]
    ItemCount {
        keyword: &'static str,
        expected_min: usize,
        expected_max: usize,
        found: usize,
    },

    #[error("{keyword}: 2‑D direction is currently unsupported in this library")]
    TwoDimUnsupported { keyword: &'static str },

    #[error("{keyword}: 1‑D direction is currently unsupported in this library")]
    OneDimUnsupported { keyword: &'static str },

    #[error("{keyword}: all direction ratios are zero")]
    AllZero { keyword: &'static str },

    #[error("{keyword}: magnitude must be non‑negative")]
    NegativeMagnitude { keyword: &'static str },

    #[error("unresolved reference #{id}")]
    UnresolvedRef { id: EntityId },

    #[error("#{id}: expected exactly one {expected}, but found {found} items")]
    MultiplicityMismatch {
        expected: &'static str,
        found: usize,
        id: EntityId,
    },

    #[error("#{id}: expected {expected}, found {found}")]
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
        id: EntityId,
    },
}

pub trait FromSimple: Sized {
    const KEYWORD: &'static str;
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError>;
}

/// 参照 ID が正しい型を指しているか検証するトレイト
pub trait ValidateRefs {
    /// arena: `EntityId -> StepItem` テーブル
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError>;
}

/// Check if the keyword matches the expected one
pub fn check_keyword(
    se: &SimpleEntity,
    expected: &'static str,
) -> Result<(), ConversionStepItemError> {
    if se.keyword == expected {
        Ok(())
    } else {
        Err(ConversionStepItemError::Unsupported(se.keyword.clone()))
    }
}

/// Ensure attribute length matches expectation.
pub fn expect_attr_len(
    se: &SimpleEntity,
    expected: usize,
    ctx: &'static str,
) -> Result<(), ConversionStepItemError> {
    if se.attrs.len() == expected {
        Ok(())
    } else {
        Err(ConversionStepItemError::AttrCount {
            keyword: ctx,
            expected,
            found: se.attrs.len(),
        })
    }
}

/// Convert a REAL or INTEGER `Parameter` into `f64`.
///
/// * `ctx` … エラーメッセージに用いるキーワード（ENTITY 名など）
///
/// 成功: `Ok(f64)`  
/// 失敗: `NonNumeric { keyword: ctx }`
pub fn numeric_to_f64(
    param: &Parameter,
    ctx: &'static str,
) -> Result<f64, ConversionStepItemError> {
    match param {
        Parameter::Real(r) => Ok(*r),
        Parameter::Integer(i) => Ok(*i as f64),
        _ => Err(ConversionStepItemError::NonNumeric { keyword: ctx }),
    }
}

/// Convert an aggregate of INTEGER/REAL parameters into Vec<f64>.
pub fn aggregate_to_f64(
    param: &Parameter,
    ctx: &'static str,
) -> Result<Vec<f64>, ConversionStepItemError> {
    if let Parameter::Aggregate(items) = param {
        let mut out = Vec::with_capacity(items.len());
        for p in items {
            let value = numeric_to_f64(p, ctx)?;
            out.push(value);
        }
        Ok(out)
    } else {
        Err(ConversionStepItemError::NotAggregate { keyword: ctx })
    }
}

/// Ensure the given scalar is ≥ 0.0.
///
/// * `ctx` … ENTITY 名など、エラーに使うキーワード
pub fn expect_non_negative(value: f64, ctx: &'static str) -> Result<f64, ConversionStepItemError> {
    if value < 0.0 {
        Err(ConversionStepItemError::NegativeMagnitude { keyword: ctx })
    } else {
        Ok(value)
    }
}

/// Extract `EntityId` when the parameter is a `#<id>` reference.
///
/// * `ctx` … ENTITY 名など、エラーメッセージに使うキーワード
///
/// 成功: `Ok(EntityId)`  
/// 失敗: `NotReference { keyword: ctx }`
pub fn expect_reference(
    param: &Parameter,
    ctx: &'static str,
) -> Result<EntityId, ConversionStepItemError> {
    if let Parameter::Reference(id) = param {
        Ok(*id)
    } else {
        Err(ConversionStepItemError::NotReference { keyword: ctx })
    }
}

/// Ensure that `map[id]`
/// * 存在している
/// * 要素数が **1 つだけ**
/// * その `StepItem::keyword()` が `expected_kw`
///
/// 成功: `Ok(())`  
/// 失敗:  
///   * `UnresolvedRef { id }` — #id が登録されていない  
///   * `MultiplicityMismatch { expected, found, id }` — 数が 1 でない
///   * `TypeMismatch { expected, found, id }` — 種類が違う
pub fn expect_single_item(
    map: &StepItemMap,
    id: EntityId,
    expected_kw: &'static str,
) -> Result<(), ConversionStepItemError> {
    match map.get(&id) {
        None => Err(ConversionStepItemError::UnresolvedRef { id }),

        Some(items) if items.len() != 1 => Err(ConversionStepItemError::MultiplicityMismatch {
            expected: expected_kw,
            found: items.len(),
            id,
        }),

        Some(items) if items[0].keyword() != expected_kw => {
            Err(ConversionStepItemError::TypeMismatch {
                expected: expected_kw,
                found: items[0].keyword(),
                id,
            })
        }

        Some(_) => Ok(()), // len == 1 かつ keyword 一致
    }
}
