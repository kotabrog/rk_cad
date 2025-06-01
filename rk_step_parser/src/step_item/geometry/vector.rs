//! -----------------------------------------------------------------------------
//! ISO 10303-42 ― ENTITY `VECTOR` 仕様要約
//!
//! ENTITY vector (geometric_representation_item)
//!   orientation : direction;       -- #参照
//!   magnitude   : length_measure;  -- ≥ 0.0
//! END_ENTITY;
//!
//! * orientation は **必ず DIRECTION の ID**。
//! * magnitude は INTEGER / REAL 可。0.0 も仕様上 OK。
//! -----------------------------------------------------------------------------

use super::super::common::{
    check_keyword, expect_attr_len, expect_non_negative, expect_reference, expect_single_item,
    numeric_to_f64, ConversionStepItemError, FromSimple, ValidateRefs,
};
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item_map::StepItemMap;

/// 解析直後（参照未解決）の VECTOR
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    pub orientation: EntityId,
    pub magnitude: f64,
}

impl FromSimple for Vector {
    const KEYWORD: &'static str = "VECTOR";

    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // name, orientation, magnitude
        expect_attr_len(&se, 3, Self::KEYWORD)?;

        // orientation = #id
        let orientation = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        // magnitude = REAL or INTEGER
        let magnitude = numeric_to_f64(&se.attrs[2], Self::KEYWORD)?;

        expect_non_negative(magnitude, Self::KEYWORD)?;

        Ok(Self {
            orientation,
            magnitude,
        })
    }
}

impl ValidateRefs for Vector {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        expect_single_item(arena, self.orientation, "DIRECTION")
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CartesianPoint, Direction};
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::StepItem;
    use rk_calc::Vector3;
    use std::collections::HashMap;

    #[test]
    fn vector_from_simple() {
        let se = SimpleEntity {
            keyword: "VECTOR".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::Real(2.0),
            ],
        };
        let vector = Vector::from_simple(se).unwrap();
        assert_eq!(vector.orientation, 1);
        assert_eq!(vector.magnitude, 2.0);
    }

    #[test]
    fn vector_from_simple_negative_magnitude() {
        let se = SimpleEntity {
            keyword: "VECTOR".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::Real(-2.0),
            ],
        };
        let err = Vector::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::NegativeMagnitude { keyword } if keyword == "VECTOR")
        );
    }

    #[test]
    fn vector_from_simple_non_numeric_magnitude() {
        let se = SimpleEntity {
            keyword: "VECTOR".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::String("not a number".into()),
            ],
        };
        let err = Vector::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::NonNumeric { keyword } if keyword == "VECTOR")
        );
    }

    #[test]
    fn vector_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "VECTOR".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::String("not a reference".into()),
                Parameter::Real(2.0),
            ],
        };
        let err = Vector::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::NotReference { keyword } if keyword == "VECTOR")
        );
    }

    #[test]
    fn vector_from_simple_wrong_keyword() {
        let se = SimpleEntity {
            keyword: "NOT_VECTOR".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::Real(2.0),
            ],
        };
        let err = Vector::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::Unsupported(keyword) if keyword == "NOT_VECTOR")
        );
    }

    #[test]
    fn vector_from_simple_attr_count_mismatch() {
        let se = SimpleEntity {
            keyword: "VECTOR".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::Real(2.0),
                Parameter::Real(3.0), // Extra attribute
            ],
        };
        let err = Vector::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::AttrCount { keyword, expected, found } if keyword == "VECTOR" && expected == 3 && found == 4)
        );
    }

    #[test]
    fn vector_validate_refs() {
        let vector = Vector {
            orientation: 1,
            magnitude: 2.0,
        };
        let mut arena = HashMap::new();
        arena.insert(
            1,
            vec![StepItem::Direction(Box::new(Direction {
                vec: Vector3::new(1.0, 2.0, 3.0),
            }))],
        );

        assert!(vector.validate_refs(&arena).is_ok());
    }

    #[test]
    fn vector_validate_refs_unresolved() {
        let vector = Vector {
            orientation: 999,
            magnitude: 2.0,
        };
        let arena: HashMap<EntityId, Vec<StepItem>> = HashMap::new();
        let err = vector.validate_refs(&arena).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::UnresolvedRef { id } if id == 999));
    }

    #[test]
    fn vector_validate_refs_wrong_type() {
        let vector = Vector {
            orientation: 1,
            magnitude: 2.0,
        };
        let mut arena = HashMap::new();
        arena.insert(
            1,
            vec![StepItem::CartesianPoint(Box::new(CartesianPoint {
                coords: Vector3::new(1.0, 2.0, 3.0),
            }))],
        );
        let err = vector.validate_refs(&arena).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::TypeMismatch { expected, found, id } if expected == "DIRECTION" && found == "CARTESIAN_POINT" && id == 1)
        );
    }

    #[test]
    fn vector_validate_refs_multiple_items() {
        let vector = Vector {
            orientation: 1,
            magnitude: 2.0,
        };
        let mut arena = HashMap::new();
        arena.insert(
            1,
            vec![
                StepItem::Direction(Box::new(Direction {
                    vec: Vector3::new(1.0, 2.0, 3.0),
                })),
                StepItem::Direction(Box::new(Direction {
                    vec: Vector3::new(4.0, 5.0, 6.0),
                })),
            ],
        );
        let err = vector.validate_refs(&arena).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::MultiplicityMismatch { expected, found, id } if expected == "DIRECTION" && found == 2 && id == 1)
        );
    }
}
