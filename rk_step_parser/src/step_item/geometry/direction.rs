//! Representation of the STEP **DIRECTION** entity (ISO 10303‑42).
//!
//! EXPRESS excerpt:
//! ```text
//! ENTITY direction
//!   SUBTYPE OF (geometric_representation_item);
//!   direction_ratios : LIST [2:3] OF REAL;
//! WHERE
//!   WR1: SIZEOF(QUERY(tmp <* direction_ratios | tmp <> 0.0)) > 0;
//! END_ENTITY;
//! ```
//!
//! ### Parameter mapping (Part 21 physical file)
//! 1. `name` — OPTIONAL STRING (`''` or `$` usually)
//! 2. `direction_ratios` — LIST of 2 or 3 REALs. INTEGER literals are
//!    commonplace and are promotable to REAL during parsing.
//!    References (`#123`), enumerations, logicals, etc. are **not valid**.
//!
//! ### Library policy (current stage)
//! *Only 3‑D directions are supported.* If exactly two ratios are
//! provided (i.e. a 2‑D direction), the converter returns
//! `ConversionStepItemError::TwoDimUnsupported`.
//!
//! Rationale: The present code targets B‑rep 3‑D models exclusively.
//! When 2‑D STEP (e.g., AP 203 drawings) becomes a requirement, this
//! restriction can be lifted by storing a dynamic‑length vector or a
//! `Dim` flag.

use super::super::common::{
    aggregate_to_f64, check_keyword, expect_attr_len, ConversionStepItemError, FromSimple,
};
use crate::step_entity::SimpleEntity;
use rk_calc::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Direction {
    pub vec: Vector3,
}

impl FromSimple for Direction {
    const KEYWORD: &'static str = "DIRECTION";

    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 2 parameters (name, ratios).
        expect_attr_len(&se, 2, Self::KEYWORD)?;

        let ratios = aggregate_to_f64(&se.attrs[1], Self::KEYWORD)?;
        // Enforce 3‑D only at this stage.
        match ratios.len() {
            3 => { /* ok */ }
            2 => {
                return Err(ConversionStepItemError::TwoDimUnsupported {
                    keyword: Self::KEYWORD,
                })
            }
            len => {
                return Err(ConversionStepItemError::ItemCount {
                    keyword: Self::KEYWORD,
                    expected_min: 2,
                    expected_max: 3,
                    found: len,
                })
            }
        }
        if ratios.iter().all(|v| v.abs() < f64::EPSILON) {
            return Err(ConversionStepItemError::AllZero {
                keyword: Self::KEYWORD,
            });
        }

        Ok(Direction {
            vec: Vector3::new(ratios[0], ratios[1], ratios[2]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;

    #[test]
    fn direction_from_simple() {
        let se = SimpleEntity {
            keyword: "DIRECTION".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(1.0),
                    Parameter::Real(2.0),
                    Parameter::Real(3.0),
                ]),
            ],
        };
        let dir = Direction::from_simple(se).unwrap();
        assert_eq!(dir.vec.x, 1.0);
        assert_eq!(dir.vec.y, 2.0);
        assert_eq!(dir.vec.z, 3.0);
    }

    #[test]
    fn direction_from_simple_2d() {
        let se = SimpleEntity {
            keyword: "DIRECTION".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Aggregate(vec![Parameter::Real(1.0), Parameter::Real(2.0)]),
            ],
        };
        let err = Direction::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::TwoDimUnsupported { keyword } if keyword == "DIRECTION")
        );
    }

    #[test]
    fn direction_from_simple_all_zero() {
        let se = SimpleEntity {
            keyword: "DIRECTION".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(0.0),
                    Parameter::Real(0.0),
                    Parameter::Real(0.0),
                ]),
            ],
        };
        let err = Direction::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::AllZero { keyword } if keyword == "DIRECTION")
        );
    }

    #[test]
    fn direction_from_simple_invalid_count() {
        let se = SimpleEntity {
            keyword: "DIRECTION".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(1.0),
                    Parameter::Real(2.0),
                    Parameter::Real(3.0),
                    Parameter::Real(4.0), // Too many
                ]),
            ],
        };
        let err = Direction::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::ItemCount { keyword, expected_min, expected_max, found } if keyword == "DIRECTION" && expected_min == 2 && expected_max == 3 && found == 4)
        );
    }

    #[test]
    fn direction_from_simple_non_numeric() {
        let se = SimpleEntity {
            keyword: "DIRECTION".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(1.0),
                    Parameter::String("not_a_number".into()), // Invalid type
                ]),
            ],
        };
        let err = Direction::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::NonNumeric { keyword } if keyword == "DIRECTION")
        );
    }

    #[test]
    fn direction_from_simple_unsupported_keyword() {
        let se = SimpleEntity {
            keyword: "UNSUPPORTED".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(1.0),
                    Parameter::Real(2.0),
                    Parameter::Real(3.0),
                ]),
            ],
        };
        let err = Direction::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::Unsupported(keyword) if keyword == "UNSUPPORTED")
        );
    }

    #[test]
    fn direction_from_simple_attr_count_mismatch() {
        let se = SimpleEntity {
            keyword: "DIRECTION".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(1.0),
                    Parameter::Real(2.0),
                    Parameter::Real(3.0),
                ]),
                Parameter::Real(4.0), // Extra attribute
            ],
        };
        let err = Direction::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::AttrCount { keyword, expected, found } if keyword == "DIRECTION" && expected == 2 && found == 3)
        );
    }
}
