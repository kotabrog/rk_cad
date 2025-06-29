//! -----------------------------------------------------------------------------
//! ISO 10303-42 ― ENTITY `VECTOR` 仕様要約
//!
//! ENTITY vector
//!   SUBTYPE OF (geometric_representation_item);
//!   orientation : direction;
//!   magnitude   : length_measure;
//! WHERE
//!   WR1 : magnitude >= 0.0;
//! END_ENTITY;
//!
//! * orientation は **必ず DIRECTION の ID**。
//! * magnitude は INTEGER / REAL 可。0.0 も仕様上 OK。
//! -----------------------------------------------------------------------------

use super::super::common::{
    check_keyword, expect_attr_len, expect_non_negative, expect_reference, expect_single_item,
    expect_single_item_cast, numeric_to_f64, ConversionStepItemError, FromSimple, HasKeyword,
    StepItemCast, ValidateRefs,
};
use super::super::StepItem;
use super::Direction;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item_map::{StepItemMap, StepItems};
use rk_calc::Vector3;

/// 解析直後（参照未解決）の VECTOR
#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub orientation: EntityId,
    pub magnitude: f64,
}

impl HasKeyword for Vector {
    const KEYWORD: &'static str = "VECTOR";
}

impl FromSimple for Vector {
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
        expect_single_item(arena, self.orientation, "DIRECTION")?;
        Ok(())
    }
}

impl StepItemCast for Vector {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::Vector(boxed) => Some(boxed),
            _ => None,
        }
    }
}

impl From<Vector> for StepItem {
    fn from(vec: Vector) -> Self {
        StepItem::Vector(Box::new(vec))
    }
}

impl Vector {
    /// zero vector でないことの確認
    pub fn is_non_zero_magnitude(&self) -> bool {
        self.magnitude.abs() >= f64::EPSILON
    }

    pub fn orientation_value(
        &self,
        arena: &StepItemMap,
    ) -> Result<Vector3, ConversionStepItemError> {
        let dir_item = expect_single_item_cast::<Direction>(arena, self.orientation)?;
        Ok(dir_item.vec)
    }

    /// 各値から arena に StepItem を登録する
    pub fn register_to_item_map(
        orientation_vec: Vector3,
        magnitude: f64,
        arena: &mut StepItemMap,
    ) -> EntityId {
        let direction = Direction {
            vec: orientation_vec,
        };
        let dir_id = arena.insert_default_id(StepItems::new_with_one_item(direction.into()));

        let vector = Vector {
            orientation: dir_id,
            magnitude,
        };
        arena.insert_default_id(StepItems::new_with_one_item(vector.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CartesianPoint, Direction};
    use super::*;
    use crate::step_entity::Parameter;
    use rk_calc::Vector3;

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
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                Direction {
                    vec: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );

        assert!(vector.validate_refs(&arena).is_ok());
    }

    #[test]
    fn vector_validate_refs_unresolved() {
        let vector = Vector {
            orientation: 999,
            magnitude: 2.0,
        };
        let arena = StepItemMap::new();
        let err = vector.validate_refs(&arena).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::UnresolvedRef { id } if id == 999));
    }

    #[test]
    fn vector_validate_refs_wrong_type() {
        let vector = Vector {
            orientation: 1,
            magnitude: 2.0,
        };
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
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
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems {
                items: vec![
                    Direction {
                        vec: Vector3::new(1.0, 2.0, 3.0),
                    }
                    .into(),
                    Direction {
                        vec: Vector3::new(4.0, 5.0, 6.0),
                    }
                    .into(),
                ],
            },
        );
        let err = vector.validate_refs(&arena).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::MultiplicityMismatch { expected, found, id } if expected == "DIRECTION" && found == 2 && id == 1)
        );
    }
}
