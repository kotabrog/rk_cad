use thiserror::Error;

use crate::step_entity::{Parameter, SimpleEntity};

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
}

pub trait FromSimple: Sized {
    const KEYWORD: &'static str;
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError>;
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

/// Convert an aggregate of INTEGER/REAL parameters into Vec<f64>.
pub fn aggregate_to_f64(
    param: &Parameter,
    ctx: &'static str,
) -> Result<Vec<f64>, ConversionStepItemError> {
    if let Parameter::Aggregate(items) = param {
        let mut out = Vec::with_capacity(items.len());
        for p in items {
            match p {
                Parameter::Real(r) => out.push(*r),
                Parameter::Integer(i) => out.push(*i as f64),
                _ => {
                    return Err(ConversionStepItemError::NonNumeric { keyword: ctx });
                }
            }
        }
        Ok(out)
    } else {
        Err(ConversionStepItemError::NotAggregate { keyword: ctx })
    }
}
