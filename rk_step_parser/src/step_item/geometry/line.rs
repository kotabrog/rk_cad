//! -----------------------------------------------------------------------------
//! ISO 10303-42 ― ENTITY `LINE` 仕様要約
//!
//! ENTITY line
//!   SUBTYPE OF (curve);
//!   pnt : cartesian_point;
//!   dir : vector;
//! WHERE
//!   WR1: dir.dim = pnt.dim;
//! END_ENTITY;
//!
//! * 現在はdim = 3 のみをサポート。
//! -----------------------------------------------------------------------------

use super::super::common::{
    check_keyword, expect_attr_len, expect_reference, expect_single_item, ConversionStepItemError,
    FromSimple, HasKeyword, StepItemCast,
};
use super::super::StepItem;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::StepItemMap;

#[derive(Debug, Clone)]
pub struct Line {
    pub pnt: EntityId, // CartesianPoint
    pub dir: EntityId, // Vector
}

impl HasKeyword for Line {
    const KEYWORD: &'static str = "LINE";
}

impl FromSimple for Line {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 3 parameters (name, pnt, dir).
        expect_attr_len(&se, 3, Self::KEYWORD)?;

        // pnt = #id
        let pnt = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        // dir = #id
        let dir = expect_reference(&se.attrs[2], Self::KEYWORD)?;

        Ok(Line { pnt, dir })
    }
}

impl ValidateRefs for Line {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        // pnt must be a CARTESIAN_POINT
        expect_single_item(arena, self.pnt, "CARTESIAN_POINT")?;
        // dir must be a VECTOR
        expect_single_item(arena, self.dir, "VECTOR")?;
        Ok(())
    }
}

impl StepItemCast for Line {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::Line(line) => Some(line),
            _ => None,
        }
    }
}

impl From<Line> for StepItem {
    fn from(line: Line) -> Self {
        StepItem::Line(Box::new(line))
    }
}

#[cfg(test)]
mod tests {
    use rk_calc::Vector3;

    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::{CartesianPoint, Vector};
    use crate::step_item_map::StepItems;

    #[test]
    fn test_line_from_simple() {
        let se = SimpleEntity {
            keyword: "LINE".to_string(),
            attrs: vec![
                Parameter::String("".to_string()),
                Parameter::Reference(1),
                Parameter::Reference(2),
            ],
        };

        let line = Line::from_simple(se).unwrap();
        assert_eq!(line.pnt, 1);
        assert_eq!(line.dir, 2);
    }

    #[test]
    fn test_line_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID".to_string(),
            attrs: vec![
                Parameter::String("".to_string()),
                Parameter::Reference(1),
                Parameter::Reference(2),
            ],
        };

        let err = Line::from_simple(se).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::Unsupported(_)));
    }

    #[test]
    fn test_line_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "LINE".to_string(),
            attrs: vec![Parameter::String("".to_string()), Parameter::Reference(1)],
        };

        let err = Line::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 3 && found == 2 && keyword == "LINE")
        );
    }

    #[test]
    fn test_line_from_simple_invalid_reference() {
        let se = SimpleEntity {
            keyword: "LINE".to_string(),
            attrs: vec![
                Parameter::String("".to_string()),
                Parameter::Real(1.0), // Not a reference
                Parameter::Reference(2),
            ],
        };

        let err = Line::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::NotReference { keyword } if keyword == "LINE")
        );
    }

    #[test]
    fn test_line_validate_refs() {
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
        arena.insert(
            2,
            StepItems::new_with_one_item(
                Vector {
                    orientation: 3,
                    magnitude: 4.0,
                }
                .into(),
            ),
        );

        let line = Line { pnt: 1, dir: 2 };
        assert!(line.validate_refs(&arena).is_ok());
    }

    #[test]
    fn test_line_validate_refs_missing_pnt() {
        let mut arena = StepItemMap::new();
        arena.insert(
            2,
            StepItems::new_with_one_item(
                Vector {
                    orientation: 3,
                    magnitude: 4.0,
                }
                .into(),
            ),
        );

        let line = Line { pnt: 1, dir: 2 };
        let err = line.validate_refs(&arena).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::UnresolvedRef { id } if id == 1));
    }

    #[test]
    fn test_line_validate_refs_missing_dir() {
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

        let line = Line { pnt: 1, dir: 2 };
        let err = line.validate_refs(&arena).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::UnresolvedRef { id } if id == 2));
    }

    #[test]
    fn test_line_validate_refs_wrong_type() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                Vector {
                    orientation: 3,
                    magnitude: 4.0,
                }
                .into(),
            ),
        );
        arena.insert(
            2,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );

        let line = Line { pnt: 1, dir: 2 };
        let err = line.validate_refs(&arena).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::TypeMismatch { expected, found, id } if expected == "CARTESIAN_POINT" && found == "VECTOR" && id == 1)
        );
    }
}
